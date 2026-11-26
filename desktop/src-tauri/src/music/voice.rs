//! Voice-to-Music bridge – turns notes ESTIMATED from singing, whistling or humming (frequency or fractional MIDI,
//! onset and length in seconds) into music: intervals, tempo and rhythm on a grid, key candidates, a MIDI part and
//! compatible chords. The user's idea is kept: pitches are only rounded to the nearest semitone (with the deviation
//! in cents reported) and are moved into the key ONLY when correction is requested. Pitch tracking itself belongs to
//! the Voice-to-Music engine (milestone M5); this module works on its output, or on notes given by the Agent.
use super::chords::ChordSym;
use super::generate::{MidiNote, BAR, PPQ};
use super::keys;
use super::notes::{from_frequency, Pitch};
use super::rhythm;
use serde_json::{json, Value};

#[derive(Clone, Debug, serde::Deserialize)]
pub struct Sung {
    #[serde(default)]
    pub hz: Option<f64>,
    #[serde(default)]
    pub midi: Option<f64>,
    pub onset_s: f64,
    pub dur_s: f64,
    #[serde(default)]
    pub confidence: Option<f64>,
}

/// Tempo that best puts the onsets on a 16th grid (60–180 BPM), unless the caller knows it.
pub fn estimate_bpm(onsets: &[f64]) -> (f64, f64) {
    if onsets.len() < 3 {
        return (120.0, 0.0);
    }
    let t0 = onsets[0];
    let mut best = (120.0, f64::MAX);
    let mut b = 60.0;
    while b <= 180.0 {
        let step = 60.0 / b / 4.0;
        let err: f64 = onsets.iter().map(|o| { let x = (o - t0) / step; (x - x.round()).abs() }).sum::<f64>() / onsets.len() as f64;
        // tiny bias to the 90–140 range where sung material usually sits
        let err = err + if (90.0..=140.0).contains(&b) { 0.0 } else { 0.02 };
        if err < best.1 - 1e-9 {
            best = (b, err);
        }
        b += 1.0;
    }
    (best.0, ((0.5 - best.1) / 0.5).clamp(0.0, 1.0))
}

/// Chords for a melody: per bar, the diatonic triad of the key that covers most of the melody's duration
/// (tonic preferred at the start and end).
pub fn harmonize(notes: &[MidiNote], key: super::theory::Key) -> Vec<String> {
    let bars = notes.iter().map(|n| n.start + n.dur).max().unwrap_or(BAR).div_ceil(BAR);
    let triads: Vec<super::theory::Chord> = (0..7).map(|d| super::theory::diatonic(key, d, false)).collect();
    (0..bars)
        .map(|b| {
            let in_bar: Vec<&MidiNote> = notes.iter().filter(|n| n.start / BAR == b).collect();
            let score = |c: &super::theory::Chord, d: usize| -> f64 {
                let cov: f64 = in_bar.iter().filter(|n| c.pcs().contains(&(n.pitch % 12))).map(|n| n.dur as f64).sum();
                let first_last = (b == 0 || b + 1 == bars) && d == 0;
                cov + if first_last { BAR as f64 * 0.3 } else { 0.0 } + if matches!(d, 0 | 3 | 4 | 5) { 10.0 } else { 0.0 }
            };
            let best = triads.iter().enumerate().max_by(|a, b| score(a.1, a.0).total_cmp(&score(b.1, b.0))).unwrap();
            best.1.symbol()
        })
        .collect()
}

pub fn to_music(sung: &[Sung], bpm: Option<f64>, grid: &str, correct: bool) -> Result<Value, String> {
    if sung.is_empty() {
        return Err("no notes received".into());
    }
    let mut pitches = vec![];
    for s in sung {
        let m = match (s.midi, s.hz) {
            (Some(m), _) => m,
            (None, Some(hz)) => from_frequency(hz).ok_or("invalid frequency")?.0,
            _ => return Err("each note needs hz or midi".into()),
        };
        pitches.push(m);
    }
    let onsets: Vec<f64> = sung.iter().map(|s| s.onset_s).collect();
    let (tempo, tempo_conf) = match bpm {
        Some(b) => (b, 1.0),
        None => estimate_bpm(&onsets),
    };
    let tick_per_s = PPQ as f64 * tempo / 60.0;
    let t0 = onsets[0];
    let g = rhythm::grid_ticks(grid)?;
    let mut notes = vec![];
    let mut deviations = vec![];
    for (s, m) in sung.iter().zip(&pitches) {
        let raw_start = (s.onset_s - t0) * tick_per_s;
        let start = ((raw_start / g as f64).round() as u32) * g;
        let dur = (((s.dur_s * tick_per_s) / g as f64).round().max(1.0) as u32) * g;
        let nearest = m.round().clamp(0.0, 127.0) as u8;
        deviations.push(json!({ "pitch": Pitch::from_midi(nearest as i32, false).name(), "cents": ((m - nearest as f64) * 100.0).round(), "timing_ticks": (start as f64 - raw_start).round(), "confidence": s.confidence }));
        notes.push(MidiNote { pitch: nearest, start, dur: dur.saturating_sub(10).max(10), vel: 96 });
    }
    // overlapping (legato singing) notes are cut at the next onset
    for i in 1..notes.len() {
        let next = notes[i].start;
        if notes[i - 1].start + notes[i - 1].dur > next {
            notes[i - 1].dur = next.saturating_sub(notes[i - 1].start).saturating_sub(10).max(10);
        }
    }
    let est = keys::estimate(&notes.iter().map(|n| (n.pitch, n.dur as f64)).collect::<Vec<_>>());
    let key = est["candidates"][0]["key"].as_str().and_then(|k| keys::parse_key(k).ok());
    let mut corrections = vec![];
    if correct {
        if let Some(k) = key {
            for n in notes.iter_mut() {
                if !k.pcs().contains(&(n.pitch % 12)) {
                    let to = [n.pitch as i32 - 1, n.pitch as i32 + 1].into_iter().find(|p| k.pcs().contains(&((*p as u8) % 12))).unwrap_or(n.pitch as i32) as u8;
                    corrections.push(json!({ "from": Pitch::from_midi(n.pitch as i32, false).name(), "to": Pitch::from_midi(to as i32, false).name() }));
                    n.pitch = to;
                }
            }
        }
    }
    let intervals: Vec<Value> = notes.windows(2).map(|w| { let i = w[1].pitch as i32 - w[0].pitch as i32; json!({ "semitones": i, "name": super::theory::interval_name(i.unsigned_abs() as u8 % 12) }) }).collect();
    let chords = key.map(|k| harmonize(&notes, k)).unwrap_or_default();
    let track = super::smf::Track { name: "Voice idea".into(), channel: 0, program: Some(0), notes: notes.clone(), cc: vec![] };
    let k = key.unwrap_or(super::theory::Key::parse("C major").unwrap());
    let bytes = super::smf::write_tracks(&[track], tempo, k, "COSOUND voice idea", (4, 4));
    Ok(json!({
        "bpm": tempo, "bpm_source": if bpm.is_some() { "given" } else { "estimated from the onsets" }, "bpm_confidence": (tempo_conf * 100.0).round() / 100.0,
        "grid": grid, "notes": notes.iter().map(|n| json!({ "pitch": n.pitch, "name": Pitch::from_midi(n.pitch as i32, false).name(), "start": n.start, "dur": n.dur, "position": rhythm::position(n.start) })).collect::<Vec<_>>(),
        "deviations": deviations, "intervals": intervals, "key": est,
        "corrections": corrections, "corrected": correct,
        "suggested_chords": chords, "suggested_chords_check": chords.iter().all(|c| ChordSym::parse(c).is_ok()),
        "midi_base64": super::caps::base64(&bytes),
        "kept": "pitches rounded to the nearest semitone only; nothing moved into the key unless correction was requested",
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn unb64(s: &str) -> Vec<u8> {
        const A: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
        let mut out = vec![];
        let v: Vec<u32> = s.bytes().filter(|b| *b != b'=').map(|b| A.iter().position(|x| *x == b).unwrap() as u32).collect();
        for c in v.chunks(4) {
            let n = c.iter().enumerate().fold(0u32, |acc, (i, x)| acc | x << (18 - 6 * i));
            out.push((n >> 16) as u8);
            if c.len() > 2 {
                out.push((n >> 8) as u8);
            }
            if c.len() > 3 {
                out.push(n as u8);
            }
        }
        out
    }

    #[test]
    fn hummed_idea_becomes_midi_and_is_not_altered() {
        // A3 C4 E4 D4 C4 B3 A3 hummed slightly off pitch and off time at 100 BPM (a 16th = 0.15 s)
        let freqs = [221.5, 260.0, 331.0, 292.0, 262.5, 246.0, 220.0];
        let on = [0.0, 0.31, 0.6, 0.9, 1.21, 1.5, 1.79];
        let sung: Vec<Sung> = freqs.iter().zip(on).map(|(f, o)| Sung { hz: Some(*f), midi: None, onset_s: o, dur_s: 0.28, confidence: Some(0.9) }).collect();
        let v = to_music(&sung, None, "1/16", false).unwrap();
        let names: Vec<&str> = v["notes"].as_array().unwrap().iter().map(|n| n["name"].as_str().unwrap()).collect();
        assert_eq!(names, ["A3", "C4", "E4", "D4", "C4", "B3", "A3"]);
        assert!((v["bpm"].as_f64().unwrap() - 100.0).abs() <= 2.0, "{}", v["bpm"]);
        assert!(v["key"]["candidates"].as_array().unwrap().iter().take(2).any(|k| k["key"] == "A minor"), "{}", v["key"]);
        assert!(v["corrections"].as_array().unwrap().is_empty(), "no correction unless asked");
        assert!(!v["suggested_chords"].as_array().unwrap().is_empty() && v["suggested_chords_check"] == true);
        // a chromatic note stays unless correction is requested
        let mut s2 = sung.clone();
        s2[3].hz = Some(311.1); // D#4
        assert!(to_music(&s2, Some(100.0), "1/16", false).unwrap()["notes"][3]["name"] == "D#4");
        let fixed = to_music(&s2, Some(100.0), "1/16", true).unwrap();
        assert_eq!(fixed["corrections"].as_array().unwrap().len(), 1);
        assert!(crate::smartlib::midi::parse(&unb64(fixed["midi_base64"].as_str().unwrap())).is_ok(), "the MIDI file reads back");
    }
}
