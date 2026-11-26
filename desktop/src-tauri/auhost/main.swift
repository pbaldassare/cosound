// cosound-auhost – COSOUND's isolated Audio Unit host (macOS).
//
// Runs as a separate process so that a plug-in that crashes or hangs never takes COSOUND down.
// Uses only Apple's public APIs (AVFoundation / AudioToolbox):
//   list      every Audio Unit registered with macOS (AVAudioUnitComponentManager), without loading any
//   describe  load one unit and report its parameter tree and factory presets
//   render    offline render: optional instrument + effect chain, parameter values and ramps, MIDI notes or an
//             input audio file; writes a WAV file and, on request, each unit's state as an .aupreset file
// Request: JSON on stdin. Response: one JSON object on stdout ({"ok":true,...} or {"ok":false,"error":...}).
// It never writes anywhere except the paths given in the request, and never modifies plug-in files.
import AVFoundation
import AudioToolbox
import Foundation

func fourcc(_ v: OSType) -> String {
    let b = [UInt8((v >> 24) & 0xff), UInt8((v >> 16) & 0xff), UInt8((v >> 8) & 0xff), UInt8(v & 0xff)]
    return String(bytes: b, encoding: .macOSRoman) ?? String(v)
}

func ostype(_ s: String) -> OSType {
    var v: OSType = 0
    for c in s.padding(toLength: 4, withPad: " ", startingAt: 0).utf8.prefix(4) { v = (v << 8) | OSType(c) }
    return v
}

func reply(_ obj: [String: Any]) -> Never {
    let data = (try? JSONSerialization.data(withJSONObject: obj, options: [.sortedKeys])) ?? Data("{\"ok\":false,\"error\":\"encoding\"}".utf8)
    FileHandle.standardOutput.write(data)
    FileHandle.standardOutput.write(Data("\n".utf8))
    exit(obj["ok"] as? Bool == true ? 0 : 1)
}

func fail(_ msg: String) -> Never { reply(["ok": false, "error": msg]) }

func versionString(_ v: Int) -> String { "\(v >> 16).\((v >> 8) & 0xff).\(v & 0xff)" }

func describeComponent(_ c: AVAudioUnitComponent) -> [String: Any] {
    let d = c.audioComponentDescription
    var o: [String: Any] = [
        "name": c.name, "manufacturer": c.manufacturerName, "type_name": c.typeName,
        "type": fourcc(d.componentType), "subtype": fourcc(d.componentSubType), "manufacturer_code": fourcc(d.componentManufacturer),
        "version": versionString(c.version), "tags": c.allTagNames, "has_midi_input": c.hasMIDIInput,
        "has_midi_output": c.hasMIDIOutput, "sandbox_safe": c.isSandboxSafe, "has_custom_view": c.hasCustomView,
        "flags": d.componentFlags,
        "v3": (d.componentFlags & AudioComponentFlags.isV3AudioUnit.rawValue) != 0,
    ]
    if let u = c.componentURL { o["path"] = u.path }
    return o
}

func desc(_ j: [String: Any]) -> AudioComponentDescription? {
    guard let t = j["type"] as? String, let s = j["subtype"] as? String, let m = j["manufacturer"] as? String else { return nil }
    return AudioComponentDescription(componentType: ostype(t), componentSubType: ostype(s), componentManufacturer: ostype(m), componentFlags: 0, componentFlagsMask: 0)
}

func instantiate(_ j: [String: Any]) async throws -> AVAudioUnit {
    guard let d = desc(j) else { throw NSError(domain: "auhost", code: 1, userInfo: [NSLocalizedDescriptionKey: "component needs type, subtype and manufacturer"]) }
    return try await AVAudioUnit.instantiate(with: d, options: [])
}

func unitName(_ u: AudioUnitParameterUnit) -> String {
    switch u {
    case .generic: return "generic"; case .indexed: return "indexed"; case .boolean: return "boolean"
    case .percent: return "percent"; case .seconds: return "seconds"; case .sampleFrames: return "samples"
    case .phase: return "phase"; case .rate: return "rate"; case .hertz: return "hertz"; case .cents: return "cents"
    case .relativeSemiTones: return "semitones"; case .midiNoteNumber: return "midi_note"; case .midiController: return "midi_cc"
    case .decibels: return "decibels"; case .linearGain: return "linear_gain"; case .degrees: return "degrees"
    case .equalPowerCrossfade: return "crossfade"; case .mixerFaderCurve1: return "fader"; case .pan: return "pan"
    case .meters: return "meters"; case .absoluteCents: return "absolute_cents"; case .octaves: return "octaves"
    case .BPM: return "bpm"; case .beats: return "beats"; case .milliseconds: return "milliseconds"; case .ratio: return "ratio"
    case .customUnit: return "custom"
    @unknown default: return "other"
    }
}

func parameters(_ au: AUAudioUnit) -> [[String: Any]] {
    (au.parameterTree?.allParameters ?? []).map { p in
        var o: [String: Any] = [
            "address": p.address, "identifier": p.identifier, "key_path": p.keyPath, "name": p.displayName,
            "unit": unitName(p.unit), "min": p.minValue, "max": p.maxValue, "value": p.value,
            "writable": p.flags.contains(.flag_IsWritable), "readable": p.flags.contains(.flag_IsReadable),
        ]
        if let u = p.unitName { o["unit_name"] = u }
        if let vs = p.valueStrings { o["value_strings"] = vs }
        o["display"] = p.string(fromValue: nil)
        return o
    }
}

func setParams(_ au: AUAudioUnit, _ list: [[String: Any]]) -> [String] {
    var errors: [String] = []
    for p in list {
        guard let addr = (p["address"] as? NSNumber)?.uint64Value, let v = (p["value"] as? NSNumber)?.floatValue else { continue }
        guard let param = au.parameterTree?.parameter(withAddress: addr) else { errors.append("no parameter at address \(addr)"); continue }
        guard param.flags.contains(.flag_IsWritable) else { errors.append("\(param.displayName) is read-only"); continue }
        param.value = min(max(v, param.minValue), param.maxValue)
    }
    return errors
}

func loadState(_ au: AUAudioUnit, _ path: String) throws {
    let data = try Data(contentsOf: URL(fileURLWithPath: path))
    guard let dict = try PropertyListSerialization.propertyList(from: data, format: nil) as? [String: Any] else {
        throw NSError(domain: "auhost", code: 2, userInfo: [NSLocalizedDescriptionKey: "not a property-list preset"])
    }
    au.fullState = dict
}

func saveState(_ au: AUAudioUnit, _ path: String, _ name: String?) throws {
    guard var state = au.fullState else { throw NSError(domain: "auhost", code: 3, userInfo: [NSLocalizedDescriptionKey: "the unit has no saveable state"]) }
    if let n = name { state["name"] = n }
    let url = URL(fileURLWithPath: path)
    if FileManager.default.fileExists(atPath: path) { throw NSError(domain: "auhost", code: 4, userInfo: [NSLocalizedDescriptionKey: "refusing to overwrite \(path)"]) }
    try FileManager.default.createDirectory(at: url.deletingLastPathComponent(), withIntermediateDirectories: true)
    let data = try PropertyListSerialization.data(fromPropertyList: state, format: .xml, options: 0)
    try data.write(to: url, options: .withoutOverwriting)
}

// The output file is closed when this function returns, before the reply (exit() would skip that).
func render(_ req: [String: Any]) async throws -> [String: Any] {
    let sr = (req["sample_rate"] as? NSNumber)?.doubleValue ?? 48000
    let seconds = min((req["seconds"] as? NSNumber)?.doubleValue ?? 4, 120)
    guard let out = req["out"] as? String else { fail("render needs \"out\"") }
    let fmt = AVAudioFormat(standardFormatWithSampleRate: sr, channels: 2)!
    let engine = AVAudioEngine()
    try engine.enableManualRenderingMode(.offline, format: fmt, maximumFrameCount: 4096)

    struct Slot { let unit: AVAudioUnit; let spec: [String: Any] }
    var slots: [Slot] = []
    var warnings: [String] = []
    if let i = req["instrument"] as? [String: Any] { slots.append(Slot(unit: try await instantiate(i), spec: i)) }
    for e in (req["chain"] as? [[String: Any]] ?? []) { slots.append(Slot(unit: try await instantiate(e), spec: e)) }
    let hasInstrument = req["instrument"] is [String: Any]
    if hasInstrument, !(slots[0].unit is AVAudioUnitMIDIInstrument) { fail("the instrument is not a music device (aumu)") }

    // state first (preset file or factory preset), then explicit values
    for s in slots {
        let au = s.unit.auAudioUnit
        if let p = s.spec["state_file"] as? String { try loadState(au, p) }
        if let n = (s.spec["factory_preset"] as? NSNumber)?.intValue, let fp = au.factoryPresets?.first(where: { $0.number == n }) { au.currentPreset = fp }
        warnings += setParams(au, s.spec["params"] as? [[String: Any]] ?? [])
    }

    var player: AVAudioPlayerNode? = nil
    var file: AVAudioFile? = nil
    if let inPath = req["input_file"] as? String {
        file = try AVAudioFile(forReading: URL(fileURLWithPath: inPath))
        player = AVAudioPlayerNode()
        engine.attach(player!)
    }
    for s in slots { engine.attach(s.unit) }
    var prev: AVAudioNode? = player
    for s in slots {
        if let p = prev { engine.connect(p, to: s.unit, format: p === player ? file!.processingFormat : fmt) }
        prev = s.unit
    }
    guard let last = prev else { fail("nothing to render: give an instrument, a chain or an input file") }
    engine.connect(last, to: engine.mainMixerNode, format: last === player ? file!.processingFormat : fmt)
    try engine.start()
    if let p = player, let f = file { p.scheduleFile(f, at: nil, completionHandler: nil); p.play() }

    if hasInstrument, let prog = (req["program"] as? NSNumber)?.uint8Value, let inst = slots[0].unit as? AVAudioUnitMIDIInstrument {
        inst.sendProgramChange(prog, onChannel: 0)
    }
    // note events and parameter ramps, applied at block boundaries (block = 256 frames ≈ 5 ms at 48 kHz)
    struct Ev { let frame: AVAudioFramePosition; let on: Bool; let pitch: UInt8; let vel: UInt8 }
    var events: [Ev] = []
    for n in (req["notes"] as? [[String: Any]] ?? []) {
        let p = (n["pitch"] as? NSNumber)?.uint8Value ?? 60, v = (n["vel"] as? NSNumber)?.uint8Value ?? 100
        let st = (n["start_s"] as? NSNumber)?.doubleValue ?? 0, du = (n["dur_s"] as? NSNumber)?.doubleValue ?? 0.5
        events.append(Ev(frame: AVAudioFramePosition(st * sr), on: true, pitch: p, vel: v))
        events.append(Ev(frame: AVAudioFramePosition((st + du) * sr), on: false, pitch: p, vel: 0))
    }
    events.sort { $0.frame < $1.frame || ($0.frame == $1.frame && !$0.on && $1.on) }
    struct Ramp { let param: AUParameter; let from: Float; let to: Float; let a: Double; let b: Double }
    var ramps: [Ramp] = []
    for s in slots {
        for r in (s.spec["automation"] as? [[String: Any]] ?? []) {
            guard let addr = (r["address"] as? NSNumber)?.uint64Value, let prm = s.unit.auAudioUnit.parameterTree?.parameter(withAddress: addr) else { continue }
            ramps.append(Ramp(param: prm, from: (r["from"] as? NSNumber)?.floatValue ?? prm.value, to: (r["to"] as? NSNumber)?.floatValue ?? prm.value,
                              a: (r["start_s"] as? NSNumber)?.doubleValue ?? 0, b: (r["end_s"] as? NSNumber)?.doubleValue ?? seconds))
        }
    }

    let outFile = try AVAudioFile(forWriting: URL(fileURLWithPath: out), settings: [
        AVFormatIDKey: kAudioFormatLinearPCM, AVSampleRateKey: sr, AVNumberOfChannelsKey: 2,
        AVLinearPCMBitDepthKey: 24, AVLinearPCMIsFloatKey: false, AVLinearPCMIsBigEndianKey: false,
    ], commonFormat: .pcmFormatFloat32, interleaved: false)
    let total = AVAudioFramePosition(seconds * sr)
    let block: AVAudioFrameCount = 256
    let buf = AVAudioPCMBuffer(pcmFormat: engine.manualRenderingFormat, frameCapacity: block)!
    var ei = 0
    var pos: AVAudioFramePosition = 0
    while pos < total {
        if let inst = hasInstrument ? slots[0].unit as? AVAudioUnitMIDIInstrument : nil {
            while ei < events.count && events[ei].frame <= pos {
                let e = events[ei]
                if e.on { inst.startNote(e.pitch, withVelocity: e.vel, onChannel: 0) } else { inst.stopNote(e.pitch, onChannel: 0) }
                ei += 1
            }
        }
        let t = Double(pos) / sr
        for r in ramps where t >= r.a && t <= r.b + 0.01 {
            let k = r.b > r.a ? Float(min(max((t - r.a) / (r.b - r.a), 0), 1)) : 1
            r.param.value = r.from + (r.to - r.from) * k
        }
        let n = AVAudioFrameCount(min(AVAudioFramePosition(block), total - pos))
        let st = try engine.renderOffline(n, to: buf)
        if st == .error { fail("render error at \(pos)") }
        try outFile.write(from: buf)
        pos += AVAudioFramePosition(n)
    }
    engine.stop()

    var saved: [[String: Any]] = []
    for s in (req["save_states"] as? [[String: Any]] ?? []) {
        guard let i = (s["slot"] as? NSNumber)?.intValue, i >= 0, i < slots.count, let p = s["path"] as? String else { continue }
        try saveState(slots[i].unit.auAudioUnit, p, s["name"] as? String)
        saved.append(["slot": i, "path": p])
    }
    // values actually in effect after rendering (to confirm that a change was applied)
    let after = slots.map { s in parameters(s.unit.auAudioUnit).map { ["address": $0["address"]!, "value": $0["value"]!, "display": $0["display"] ?? ""] } }
    return ["ok": true, "out": out, "frames": total, "sample_rate": sr, "saved": saved, "warnings": warnings, "parameters_after": after]
}

let args = CommandLine.arguments
guard args.count >= 2 else { fail("usage: cosound-auhost list|describe|render < request.json") }
let input = FileHandle.standardInput.readDataToEndOfFile()
let req: [String: Any] = input.isEmpty ? [:] : ((try? JSONSerialization.jsonObject(with: input)) as? [String: Any] ?? [:])

switch args[1] {
case "list":
    let all = AVAudioUnitComponentManager.shared().components(passingTest: { _, _ in true })
    reply(["ok": true, "components": all.map(describeComponent)])

case "describe":
    do {
        let unit = try await instantiate(req)
        let au = unit.auAudioUnit
        var o: [String: Any] = ["ok": true, "parameters": parameters(au)]
        o["factory_presets"] = (au.factoryPresets ?? []).map { ["number": $0.number, "name": $0.name] }
        o["supports_user_presets"] = au.supportsUserPresets
        o["latency_s"] = au.latency
        o["tail_s"] = au.tailTime
        o["midi_output_names"] = au.midiOutputNames
        o["provides_user_interface"] = au.providesUserInterface
        o["instrument"] = unit is AVAudioUnitMIDIInstrument
        if let p = req["state_file"] as? String { try loadState(au, p); o["parameters_after_state"] = parameters(au) }
        if let n = (req["factory_preset"] as? NSNumber)?.intValue, let fp = au.factoryPresets?.first(where: { $0.number == n }) {
            au.currentPreset = fp
            o["parameters_after_state"] = parameters(au)
        }
        reply(o)
    } catch {
        fail("could not load the unit: \(error.localizedDescription)")
    }

case "render":
    do {
        reply(try await render(req))
    } catch {
        fail("render failed: \(error.localizedDescription)")
    }

default:
    fail("unknown command \(args[1])")
}
