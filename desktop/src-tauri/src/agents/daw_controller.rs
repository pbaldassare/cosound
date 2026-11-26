//! L. DAW Controller: the only component that will operate DAWs. Today it detects DAWs and prepares
//! plans (operations, verified route per DAW, MIDI files ready to import). It executes nothing in a
//! DAW: connectors are milestone M4 (Logic Pro first).
use super::{s, unknown, Agent, AgentError, Ctx, OpResult, Outcome};
use crate::daw;
use serde_json::{json, Value};

pub struct DawController;

/// Verified route per DAW (docs/knowledge/daw-integration and the DAW Controller spec) and phase.
pub const ROUTES: [(&str, &str, u8, &str); 6] = [
    ("logic", "Logic Pro", 1, "MIDI file import; control surface / OSC for transport and mixer; macOS Accessibility (\"human hand\") for the rest"),
    ("ableton", "Ableton Live", 2, "Live Object Model through a Max for Live device"),
    ("fl", "FL Studio", 2, "Python MIDI scripting inside FL Studio; MIDI file import for content"),
    ("protools", "Pro Tools", 2, "Pro Tools Scripting SDK (PTSL, gRPC on localhost)"),
    ("cubase", "Cubase", 3, "MIDI Remote script through virtual MIDI ports; DAWproject import"),
    ("reaper", "REAPER", 3, "ReaScript (Lua/Python)"),
];

fn b64decode(s: &str) -> Option<Vec<u8>> {
    let val = |c: u8| match c {
        b'A'..=b'Z' => Some(c - b'A'),
        b'a'..=b'z' => Some(c - b'a' + 26),
        b'0'..=b'9' => Some(c - b'0' + 52),
        b'+' => Some(62),
        b'/' => Some(63),
        _ => None,
    };
    let clean: Vec<u8> = s.bytes().filter(|c| !c.is_ascii_whitespace() && *c != b'=').collect();
    let mut out = Vec::with_capacity(clean.len() * 3 / 4);
    for ch in clean.chunks(4) {
        let v: Vec<u8> = ch.iter().map(|&c| val(c)).collect::<Option<_>>()?;
        let n = v.iter().enumerate().fold(0u32, |acc, (i, &x)| acc | (x as u32) << (18 - 6 * i));
        out.extend_from_slice(&n.to_be_bytes()[1..ch.len().max(2)]);
    }
    Some(out)
}

impl Agent for DawController {
    fn id(&self) -> &'static str {
        "daw_controller"
    }
    fn operations(&self) -> &'static [&'static str] {
        &["detect", "plan_operations", "plugin_setup_plan"]
    }
    fn run(&self, op: &str, input: &Value, cx: &mut Ctx) -> OpResult {
        cx.tool("daw.detector");
        let found = daw::detect();
        match op {
            "detect" => {
                if !found.iter().any(|d| d.detected) {
                    cx.limit("no supported DAW was found in the standard install locations");
                }
                Ok(Outcome::new(json!({ "daws": found, "connected": [] }), "measured"))
            }
            "plan_operations" => {
                cx.tool("daw.routes");
                let daw_id = s(input, "daw").map(String::from).unwrap_or_else(|| found.iter().find(|d| d.detected).map_or("logic", |d| d.id).to_string());
                let (_, label, phase, route) = *ROUTES.iter().find(|r| r.0 == daw_id).ok_or_else(|| AgentError::input(format!("unknown DAW {daw_id}")))?;
                let detected = found.iter().find(|d| d.id == daw_id).filter(|d| d.detected);
                if detected.is_none() {
                    cx.limit(format!("{label} was not detected on this computer"));
                }
                let sub = format!("daw-plan-{}", &crate::db::new_id()[..8]);
                let mut files = Vec::new();
                for (i, p) in input["parts"].as_array().into_iter().flatten().enumerate() {
                    let name = s(p, "name").unwrap_or("Part");
                    let path = if let Some(fid) = p["library_file_id"].as_i64() {
                        cx.tool("library.index");
                        let r = crate::smartlib::api::resolve(&cx.db.lock().unwrap(), &[fid]).remove(0);
                        let Some(raw) = r["path"].as_str().map(String::from) else {
                            return Err(AgentError::new("FILE_NOT_AVAILABLE", format!("part {name}: library file {fid} is not available ({})", if r["error"].is_string() { "not in the library".to_string() } else { format!("drive {} not connected or file {}", r["device"], r["status"]) })));
                        };
                        cx.authorized_file(&raw)?.to_string_lossy().to_string()
                    } else if let Some(b) = s(p, "midi_base64") {
                        let bytes = b64decode(b).filter(|b| b.starts_with(b"MThd")).ok_or_else(|| AgentError::input(format!("part {name}: midi_base64 is not a MIDI file")))?;
                        cx.tool("files.writer");
                        cx.write_file(&sub, &format!("{:02}-{name}.mid", i + 1), &bytes, "midi")?.path
                    } else if let Some(raw) = s(p, "path") {
                        let pb = cx.authorized_file(raw)?;
                        let ext = pb.extension().map(|e| e.to_string_lossy().to_lowercase()).unwrap_or_default();
                        if !(ext == "mid" || ext == "midi" || crate::smartlib::decode::AUDIO_EXTS.contains(&ext.as_str())) {
                            return Err(AgentError::input(format!("part {name}: only MIDI or audio files can be imported")));
                        }
                        pb.to_string_lossy().to_string()
                    } else {
                        return Err(AgentError::input(format!("part {name}: give library_file_id, midi_base64 or path")));
                    };
                    files.push((name.to_string(), s(p, "role").unwrap_or("").to_string(), path));
                }
                let mut ops = Vec::new();
                let mut push = |op: &str, args: Value| {
                    let step = ops.len() + 1;
                    ops.push(json!({ "step": step, "op": op, "args": args, "risk": "low", "requires_approval": false, "route": route, "executable_now": false }));
                };
                if let Some(b) = input["bpm"].as_f64() {
                    push("set_tempo", json!({ "bpm": b }));
                }
                if let Some(k) = s(input, "key") {
                    push("set_key_signature", json!({ "key": k }));
                }
                for sec in input["sections"].as_array().into_iter().flatten() {
                    push("create_marker", json!({ "name": sec["name"], "bar": sec["start_bar"] }));
                }
                for (name, role, path) in &files {
                    let midi = path.to_lowercase().ends_with(".mid") || path.to_lowercase().ends_with(".midi");
                    push("create_track", json!({ "name": name, "type": if midi { "instrument" } else { "audio" }, "role": role }));
                    push(if midi { "import_midi" } else { "import_audio" }, json!({ "file": path, "track": name, "at_bar": 1 }));
                }
                let folder = files.first().and_then(|f| std::path::Path::new(&f.2).parent().map(|p| p.to_string_lossy().to_string()));
                let mut manual = vec![format!("Open {label} with a new empty project")];
                if let Some(b) = input["bpm"].as_f64() {
                    manual.push(format!("Set the project tempo to {b} BPM"));
                }
                if !files.is_empty() {
                    manual.push(format!("Import the files from {}: dragging a .mid file into the tracks area creates a track with its region", folder.clone().unwrap_or_default()));
                }
                cx.limit("nothing was executed in the DAW: DAW connectors are milestone M4 (Logic Pro first)");
                let mut out = Outcome::new(
                    json!({ "daw": daw_id, "label": label, "phase": phase, "project_id": cx.project_id, "detected": detected, "route": route, "operations": ops, "manual_steps": manual, "files_folder": folder, "executed": false }),
                    "rule_based",
                );
                out.proposed_daw_ops = ops;
                Ok(out)
            }
            "plugin_setup_plan" => {
                cx.tool("plugins.daw_adapter");
                cx.tool("daw.routes");
                let id = s(input, "design_id").ok_or_else(|| AgentError::input("design_id is required"))?;
                let daw_id = s(input, "daw").unwrap_or("logic");
                let d = crate::plugins::design::get(&cx.db.lock().unwrap(), id).ok_or_else(|| AgentError::new("NOT_FOUND", "unknown design"))?;
                if !found.iter().any(|x| x.id == daw_id && x.detected) {
                    cx.limit(format!("{daw_id} was not detected on this computer: the plan cannot be executed here"));
                }
                let plan = crate::plugins::daw::setup_plan(&d, daw_id).map_err(AgentError::input)?;
                cx.limit("plan only: no DAW connector exists yet (M4); steps marked planned are done by the user");
                Ok(Outcome::new(plan, "convention"))
            }
            _ => Err(unknown(self.id(), op)),
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn base64_round_trip() {
        for t in ["", "M", "MT", "MThd", "MThd\x00\x00\x00\x06"] {
            assert_eq!(super::b64decode(&crate::music::caps::base64(t.as_bytes())).unwrap(), t.as_bytes(), "{t:?}");
        }
        assert!(super::b64decode("!!!!").is_none());
    }
}
