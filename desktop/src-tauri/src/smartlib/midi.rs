//! Standard MIDI File reader and analysis for the library: tracks, notes, range, intervals,
//! tempo, duration, GM instruments, key estimate, polyphony, rhythm and pattern type.
use crate::music::theory;
use serde::Serialize;
use serde_json::{json, Value};

const GM: [&str; 128] = [
    "Acoustic Grand Piano", "Bright Acoustic Piano", "Electric Grand Piano", "Honky-tonk Piano", "Electric Piano 1", "Electric Piano 2", "Harpsichord", "Clavinet",
    "Celesta", "Glockenspiel", "Music Box", "Vibraphone", "Marimba", "Xylophone", "Tubular Bells", "Dulcimer",
    "Drawbar Organ", "Percussive Organ", "Rock Organ", "Church Organ", "Reed Organ", "Accordion", "Harmonica", "Tango Accordion",
    "Acoustic Guitar (nylon)", "Acoustic Guitar (steel)", "Electric Guitar (jazz)", "Electric Guitar (clean)", "Electric Guitar (muted)", "Overdriven Guitar", "Distortion Guitar", "Guitar Harmonics",
    "Acoustic Bass", "Electric Bass (finger)", "Electric Bass (pick)", "Fretless Bass", "Slap Bass 1", "Slap Bass 2", "Synth Bass 1", "Synth Bass 2",
    "Violin", "Viola", "Cello", "Contrabass", "Tremolo Strings", "Pizzicato Strings", "Orchestral Harp", "Timpani",
    "String Ensemble 1", "String Ensemble 2", "Synth Strings 1", "Synth Strings 2", "Choir Aahs", "Voice Oohs", "Synth Voice", "Orchestra Hit",
    "Trumpet", "Trombone", "Tuba", "Muted Trumpet", "French Horn", "Brass Section", "Synth Brass 1", "Synth Brass 2",
    "Soprano Sax", "Alto Sax", "Tenor Sax", "Baritone Sax", "Oboe", "English Horn", "Bassoon", "Clarinet",
    "Piccolo", "Flute", "Recorder", "Pan Flute", "Blown Bottle", "Shakuhachi", "Whistle", "Ocarina",
    "Lead 1 (square)", "Lead 2 (sawtooth)", "Lead 3 (calliope)", "Lead 4 (chiff)", "Lead 5 (charang)", "Lead 6 (voice)", "Lead 7 (fifths)", "Lead 8 (bass + lead)",
    "Pad 1 (new age)", "Pad 2 (warm)", "Pad 3 (polysynth)", "Pad 4 (choir)", "Pad 5 (bowed)", "Pad 6 (metallic)", "Pad 7 (halo)", "Pad 8 (sweep)",
    "FX 1 (rain)", "FX 2 (soundtrack)", "FX 3 (crystal)", "FX 4 (atmosphere)", "FX 5 (brightness)", "FX 6 (goblins)", "FX 7 (echoes)", "FX 8 (sci-fi)",
    "Sitar", "Banjo", "Shamisen", "Koto", "Kalimba", "Bagpipe", "Fiddle", "Shanai",
    "Tinkle Bell", "Agogo", "Steel Drums", "Woodblock", "Taiko Drum", "Melodic Tom", "Synth Drum", "Reverse Cymbal",
    "Guitar Fret Noise", "Breath Noise", "Seashore", "Bird Tweet", "Telephone Ring", "Helicopter", "Applause", "Gunshot",
];
const NOTES: [&str; 12] = ["C", "C#", "D", "D#", "E", "F", "F#", "G", "G#", "A", "A#", "B"];

#[derive(Clone, Debug, Serialize)]
pub struct Note {
    pub track: usize,
    pub channel: u8,
    pub pitch: u8,
    pub start: u64,
    pub dur: u64,
    pub vel: u8,
}

#[derive(Clone, Debug, Default, Serialize)]
pub struct Smf {
    pub format: u16,
    pub tracks: usize,
    pub ppq: u16,
    pub track_names: Vec<String>,
    /// (tick, microseconds per quarter note)
    pub tempos: Vec<(u64, u32)>,
    pub time_signature: Option<(u8, u8)>,
    pub key_signature: Option<(i8, bool)>,
    pub programs: Vec<(u8, u8)>,
    pub notes: Vec<Note>,
    pub end_tick: u64,
}

fn vlq(b: &[u8], i: &mut usize) -> Result<u32, String> {
    let mut v = 0u32;
    for _ in 0..4 {
        let c = *b.get(*i).ok_or("truncated variable-length value")?;
        *i += 1;
        v = (v << 7) | (c & 0x7F) as u32;
        if c & 0x80 == 0 {
            return Ok(v);
        }
    }
    Err("bad variable-length value".into())
}

pub fn parse(bytes: &[u8]) -> Result<Smf, String> {
    let be16 = |b: &[u8]| u16::from_be_bytes([b[0], b[1]]);
    let be32 = |b: &[u8]| u32::from_be_bytes([b[0], b[1], b[2], b[3]]);
    if bytes.len() < 14 || &bytes[0..4] != b"MThd" {
        return Err("not a Standard MIDI File".into());
    }
    let hl = be32(&bytes[4..8]) as usize;
    let mut s = Smf { format: be16(&bytes[8..10]), ppq: be16(&bytes[12..14]), ..Default::default() };
    if s.ppq & 0x8000 != 0 || s.ppq == 0 {
        return Err("SMPTE time division is not supported".into());
    }
    let ntrks = be16(&bytes[10..12]) as usize;
    let mut pos = 8 + hl;
    for t in 0..ntrks {
        if pos + 8 > bytes.len() {
            break;
        }
        if &bytes[pos..pos + 4] != b"MTrk" {
            let skip = be32(&bytes[pos + 4..pos + 8]) as usize;
            pos += 8 + skip;
            continue;
        }
        let len = be32(&bytes[pos + 4..pos + 8]) as usize;
        let end = (pos + 8 + len).min(bytes.len());
        let b = &bytes[pos + 8..end];
        pos = end;
        s.tracks += 1;
        let (mut i, mut tick, mut status) = (0usize, 0u64, 0u8);
        let mut open: std::collections::HashMap<(u8, u8), (u64, u8)> = Default::default();
        while i < b.len() {
            tick += vlq(b, &mut i)? as u64;
            let mut st = *b.get(i).ok_or("truncated event")?;
            if st & 0x80 != 0 {
                i += 1;
                if st < 0xF0 {
                    status = st;
                }
            } else {
                st = status; // running status
                if st == 0 {
                    return Err("running status without a previous status".into());
                }
            }
            match st {
                0xFF => {
                    let kind = *b.get(i).ok_or("truncated meta")?;
                    i += 1;
                    let l = vlq(b, &mut i)? as usize;
                    let d = b.get(i..i + l).ok_or("truncated meta data")?;
                    i += l;
                    match kind {
                        0x03 => s.track_names.push(String::from_utf8_lossy(d).trim().to_string()),
                        0x51 if l == 3 => s.tempos.push((tick, u32::from_be_bytes([0, d[0], d[1], d[2]]))),
                        0x58 if l >= 2 => s.time_signature = s.time_signature.or(Some((d[0], 1u8 << d[1].min(6)))),
                        0x59 if l == 2 => s.key_signature = s.key_signature.or(Some((d[0] as i8, d[1] == 1))),
                        0x2F => break,
                        _ => {}
                    }
                }
                0xF0 | 0xF7 => {
                    let l = vlq(b, &mut i)? as usize;
                    i += l;
                }
                _ => {
                    let ch = st & 0x0F;
                    let n = match st & 0xF0 {
                        0xC0 | 0xD0 => 1,
                        0x80 | 0x90 | 0xA0 | 0xB0 | 0xE0 => 2,
                        _ => return Err(format!("unknown status {st:#x}")),
                    };
                    let d = b.get(i..i + n).ok_or("truncated event data")?;
                    i += n;
                    match st & 0xF0 {
                        0x90 if d[1] > 0 => {
                            if let Some((on, vel)) = open.insert((ch, d[0]), (tick, d[1])) {
                                s.notes.push(Note { track: t, channel: ch, pitch: d[0], start: on, dur: tick - on, vel });
                            }
                        }
                        0x80 | 0x90 => {
                            if let Some((on, vel)) = open.remove(&(ch, d[0])) {
                                s.notes.push(Note { track: t, channel: ch, pitch: d[0], start: on, dur: (tick - on).max(1), vel });
                            }
                        }
                        0xC0 => s.programs.push((ch, d[0])),
                        _ => {}
                    }
                }
            }
        }
        s.end_tick = s.end_tick.max(tick);
    }
    s.notes.sort_by_key(|n| (n.start, n.pitch));
    Ok(s)
}

/// Seconds at `tick` with the tempo map.
fn seconds(s: &Smf, tick: u64) -> f64 {
    let mut tempos = s.tempos.clone();
    tempos.sort_by_key(|t| t.0);
    let (mut last_tick, mut us, mut secs) = (0u64, 500_000u32, 0.0f64);
    for (t, u) in tempos {
        if t >= tick {
            break;
        }
        secs += (t - last_tick) as f64 * us as f64 / 1e6 / s.ppq as f64;
        last_tick = t;
        us = u;
    }
    secs + (tick - last_tick) as f64 * us as f64 / 1e6 / s.ppq as f64
}

/// Musical analysis of a parsed file. Key and pattern type are estimates.
pub fn analyze(s: &Smf) -> Value {
    let notes = &s.notes;
    let drums: Vec<&Note> = notes.iter().filter(|n| n.channel == 9).collect();
    let pitched: Vec<&Note> = notes.iter().filter(|n| n.channel != 9).collect();
    let bpm = s.tempos.first().map(|t| ((60_000_000.0 / t.1 as f64) * 100.0).round() / 100.0);
    let (num, den) = s.time_signature.map_or((4, 4), |t| (t.0 as u64, t.1 as u64));
    let bar = s.ppq as u64 * 4 * num / den.max(1);
    let len_ticks = notes.iter().map(|n| n.start + n.dur).max().unwrap_or(0).max(s.end_tick.min(notes.iter().map(|n| n.start + n.dur).max().unwrap_or(0) + bar));
    let exact = if bar > 0 { len_ticks as f64 / bar as f64 } else { 0.0 };
    // whole bars as a DAW shows them (a last note released just before the bar line still fills it)
    let bars = (exact - 0.02).ceil().max(if exact > 0.0 { 1.0 } else { 0.0 });
    let mut w = [0.0f64; 12];
    for n in &pitched {
        w[(n.pitch % 12) as usize] += n.dur as f64;
    }
    let key = if pitched.len() >= 6 {
        let r = theory::detect_key(&w);
        let conf = ((r[0].1 - 0.5) / 0.4).clamp(0.0, 1.0);
        json!({ "value": if r[0].1 >= 0.55 { Some(r[0].0.name()) } else { None }, "confidence": (conf * 1000.0).round() / 1000.0, "source": "analysis",
                "candidates": r.iter().take(3).map(|(k, c)| json!([k.name(), (c * 1000.0).round() / 1000.0])).collect::<Vec<_>>() })
    } else {
        json!({ "value": null, "confidence": 0.0, "source": "unknown", "note": "too few pitched notes" })
    };
    // polyphony: average number of notes sounding at note starts
    let mut poly_sum = 0usize;
    for n in &pitched {
        poly_sum += pitched.iter().filter(|m| m.start <= n.start && m.start + m.dur > n.start).count();
    }
    let poly = if pitched.is_empty() { 0.0 } else { poly_sum as f64 / pitched.len() as f64 };
    // melodic intervals between successive notes of the top voice
    let mut intervals = [0u32; 13];
    let mut mono: Vec<&Note> = Vec::new();
    for n in &pitched {
        match mono.last() {
            Some(l) if l.start == n.start => {
                if n.pitch > l.pitch {
                    mono.pop();
                    mono.push(n);
                }
            }
            _ => mono.push(n),
        }
    }
    for p in mono.windows(2) {
        intervals[(p[1].pitch as i32 - p[0].pitch as i32).unsigned_abs().min(12) as usize] += 1;
    }
    let (lo, hi) = (pitched.iter().map(|n| n.pitch).min(), pitched.iter().map(|n| n.pitch).max());
    let avg_pitch = if pitched.is_empty() { 0.0 } else { pitched.iter().map(|n| n.pitch as f64).sum::<f64>() / pitched.len() as f64 };
    let sixteenth = (s.ppq as u64 / 4).max(1);
    let offbeat = notes.iter().filter(|n| (n.start / sixteenth) % 2 == 1 || n.start % (s.ppq as u64) == s.ppq as u64 / 2).count();
    let short = pitched.iter().filter(|n| n.dur <= sixteenth * 2).count();
    let steps = mono.windows(2).filter(|p| (p[1].pitch as i32 - p[0].pitch as i32).abs() <= 2).count();
    let pattern = if !drums.is_empty() && drums.len() * 2 >= notes.len() {
        "drums"
    } else if poly >= 2.5 {
        "chords"
    } else if avg_pitch < 52.0 {
        "bass"
    } else if poly < 1.3 && short * 10 >= pitched.len() * 7 && mono.len() >= 8 && steps * 2 < mono.len() {
        "arpeggio"
    } else {
        "melody"
    };
    let instruments: Vec<String> = s.programs.iter().map(|(ch, p)| if *ch == 9 { "Drum kit (channel 10)".to_string() } else { GM[*p as usize % 128].to_string() }).collect();
    json!({
        "format": s.format, "tracks": s.tracks, "track_names": s.track_names, "ppq": s.ppq,
        "notes": notes.len(), "drum_notes": drums.len(),
        "bpm": bpm.map(|b| json!({ "value": b, "confidence": 1.0, "source": "embedded" })).unwrap_or(json!({ "value": null, "confidence": 0.0, "source": "unknown", "note": "no tempo event" })),
        "time_signature": s.time_signature.map(|(n, d)| format!("{n}/{d}")),
        "key_signature": s.key_signature.map(|(sf, minor)| json!({ "sharps_flats": sf, "minor": minor })),
        "duration_s": ((seconds(s, len_ticks)) * 100.0).round() / 100.0, "bars": bars, "bars_exact": (exact * 100.0).round() / 100.0,
        "range": lo.zip(hi).map(|(l, h)| format!("{}{}–{}{}", NOTES[(l % 12) as usize], l as i32 / 12 - 1, NOTES[(h % 12) as usize], h as i32 / 12 - 1)),
        "average_pitch": (avg_pitch * 10.0).round() / 10.0, "polyphony": (poly * 100.0).round() / 100.0,
        "intervals": (0..13).map(|i| json!([theory::interval_name(i as u8 % 12).replace("P1", if i == 12 { "P8" } else { "P1" }), intervals[i]])).collect::<Vec<_>>(),
        "pitch_classes": w.iter().enumerate().filter(|(_, v)| **v > 0.0).map(|(i, _)| NOTES[i]).collect::<Vec<_>>(),
        "key": key,
        "pattern": { "value": pattern, "source": "analysis", "confidence": 0.6 },
        "rhythm": { "offbeat_share": if notes.is_empty() { 0.0 } else { ((offbeat as f64 / notes.len() as f64) * 100.0).round() / 100.0 }, "notes_per_bar": if bars > 0.0 { ((notes.len() as f64 / bars) * 10.0).round() / 10.0 } else { 0.0 } },
        "instruments": instruments,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::music::{caps, smf, theory::{progression, Key}, generate::*};

    fn file(parts: &[Part], bpm: f64) -> Vec<u8> {
        smf::write(parts, bpm, Key::parse("A minor").unwrap(), "test")
    }

    #[test]
    fn analyzes_generated_parts() {
        let key = Key::parse("A minor").unwrap();
        let ch = progression(key, "i-VI-III-VII", false).unwrap();
        let spec = Spec { key, chords: &ch, bars: 4, seed: 2, swing: 0 };
        for (part, expect) in [(bass(&spec, "rolling").unwrap(), "bass"), (chords(&spec, "sustain").unwrap(), "chords"), (drums(&spec, "techno", false).unwrap(), "drums"), (melody(&spec), "melody")] {
            let s = parse(&file(&[part], 124.0)).unwrap();
            let a = analyze(&s);
            assert_eq!(a["pattern"]["value"], expect, "{a}");
            assert_eq!(a["bpm"]["value"], 124.0);
            assert_eq!(a["bars"], 4.0);
            if expect != "drums" {
                let k = a["key"]["value"].as_str().unwrap_or("");
                assert!(k == "A minor" || k == "C major", "{expect}: {}", a["key"]);
            }
        }
        // an arpeggio: 16ths cycling chord tones
        let mut notes = Vec::new();
        for i in 0..64u32 {
            notes.push(MidiNote { pitch: [57, 60, 64, 69][i as usize % 4], start: i * 120, dur: 100, vel: 100 });
        }
        let a = analyze(&parse(&file(&[Part { id: "melody", name: "Arp", channel: 0, program: 81, notes }], 124.0)).unwrap());
        assert_eq!(a["pattern"]["value"], "arpeggio", "{a}");
        assert_eq!(a["instruments"][0], "Lead 2 (sawtooth)");
        let _ = caps::base64(b"");
    }

    #[test]
    fn rejects_bad_files() {
        assert!(parse(b"RIFF....").is_err());
        let mut b = file(&[], 120.0);
        b.truncate(20);
        assert!(parse(&b).is_ok(), "truncated track chunk is tolerated");
    }
}
