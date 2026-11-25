// Audio preview of a sketch with simple Web Audio sounds (no samples, no plug-ins).
// It only lets the user hear the idea; the real sound comes from the DAW once the MIDI is exported.
import type { Part, PartId, Sketch } from './api'

const hz = (p: number) => 440 * 2 ** ((p - 69) / 12)

function noise(ctx: AudioContext) {
  const b = ctx.createBuffer(1, ctx.sampleRate, ctx.sampleRate)
  const d = b.getChannelData(0)
  for (let i = 0; i < d.length; i++) d[i] = Math.random() * 2 - 1
  return b
}

function env(ctx: AudioContext, out: AudioNode, at: number, len: number, peak: number) {
  const g = ctx.createGain()
  g.gain.setValueAtTime(0.0001, at)
  g.gain.exponentialRampToValueAtTime(peak, at + 0.005)
  g.gain.exponentialRampToValueAtTime(0.0001, at + len)
  g.connect(out)
  return g
}

function tone(ctx: AudioContext, out: AudioNode, part: PartId, pitch: number, at: number, len: number, vel: number) {
  const o = ctx.createOscillator()
  const f = ctx.createBiquadFilter()
  const v = vel / 127
  o.type = part === 'bass' ? 'square' : part === 'chords' ? 'triangle' : 'sawtooth'
  o.frequency.value = hz(pitch)
  f.type = 'lowpass'
  f.frequency.value = part === 'bass' ? 700 : part === 'chords' ? 1800 : 2600
  const g = env(ctx, out, at, Math.max(len, 0.06) + 0.05, (part === 'bass' ? 0.22 : part === 'chords' ? 0.07 : 0.11) * v)
  o.connect(f).connect(g)
  o.start(at)
  o.stop(at + len + 0.1)
}

function drum(ctx: AudioContext, out: AudioNode, buf: AudioBuffer, pitch: number, at: number, vel: number) {
  const v = vel / 127
  if (pitch === 36) {
    const o = ctx.createOscillator()
    o.frequency.setValueAtTime(150, at)
    o.frequency.exponentialRampToValueAtTime(45, at + 0.12)
    o.connect(env(ctx, out, at, 0.3, 0.9 * v))
    o.start(at)
    o.stop(at + 0.32)
    return
  }
  if (pitch === 37) {
    const o = ctx.createOscillator()
    o.type = 'square'
    o.frequency.value = 1700
    o.connect(env(ctx, out, at, 0.03, 0.08 * v))
    o.start(at)
    o.stop(at + 0.05)
    return
  }
  // noise-based: clap, snare, hats, shaker, cymbals
  const [type, freq, len, gain] =
    pitch === 39 ? ['bandpass', 1500, 0.18, 0.5] :
    pitch === 38 ? ['highpass', 1000, 0.16, 0.4] :
    pitch === 42 ? ['highpass', 7500, 0.04, 0.18] :
    pitch === 46 ? ['highpass', 7000, 0.22, 0.16] :
    pitch === 70 ? ['highpass', 9000, 0.03, 0.08] :
    pitch === 49 ? ['highpass', 5000, 1.2, 0.2] : ['highpass', 6000, 0.4, 0.12]
  const s = ctx.createBufferSource()
  s.buffer = buf
  const f = ctx.createBiquadFilter()
  f.type = type as BiquadFilterType
  f.frequency.value = freq as number
  s.connect(f).connect(env(ctx, out, at, len as number, (gain as number) * v))
  s.start(at)
  s.stop(at + (len as number) + 0.05)
}

/** Plays the given parts once. Returns the start time and a stop function. */
export function play(sketch: Sketch, parts: Part[]) {
  const ctx = new AudioContext()
  const master = ctx.createGain()
  master.gain.value = 0.6
  const comp = ctx.createDynamicsCompressor()
  master.connect(comp).connect(ctx.destination)
  const buf = noise(ctx)
  const sec = 60 / sketch.bpm / sketch.ppq
  const t0 = ctx.currentTime + 0.12
  for (const p of parts) {
    for (const n of p.notes) {
      const at = t0 + n.start * sec
      if (p.id === 'drums') drum(ctx, master, buf, n.pitch, at, n.vel)
      else tone(ctx, master, p.id, n.pitch, at, n.dur * sec, n.vel)
    }
  }
  return {
    ctx,
    tickAt: () => (ctx.currentTime - t0) / sec,
    stop: () => { void ctx.close() },
  }
}
