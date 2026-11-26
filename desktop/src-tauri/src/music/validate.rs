//! Musical Validation Engine – checks a composition before it is returned, and tells apart:
//! - technical_error: must be fixed (notes outside the length, zero durations, same-pitch overlaps, MIDI range);
//! - stylistic_choice: a warning the user may accept (weak harmony on strong beats, clashes, register crossing);
//! - compositional_freedom: noted, never "corrected" (chromatic passing tones, borrowed notes, intentional tension).
//! Out-of-key notes are classified, not deleted. Sources: music-theory/melody.md §5, midi-intelligence/composition-guide.md §2.
use super::generate::{MidiNote, BAR, PPQ};
use super::harmony::{chord_at, Timeline};
use super::smf::Track;
use super::theory::Key;
use serde_json::{json, Value};

/// Why a note outside the key is there (in a monophonic line).
pub fn classify_out_of_key(line: &[MidiNote], i: usize, key: Key, tl: Option<&Timeline>) -> &'static str {
    let n = line[i];
    let pc = n.pitch % 12;
    if let Some(tl) = tl {
        if chord_at(tl, n.start).chord.pcs().contains(&pc) {
            return "chord_tone (harmonic/borrowed colour of the current chord)";
        }
    }
    let prev = (i > 0).then(|| line[i - 1].pitch as i32);
    let next = line.get(i + 1).map(|x| x.pitch as i32);
    let p = n.pitch as i32;
    match (prev, next) {
        (Some(a), Some(b)) if (p - a).abs() <= 2 && (b - p).abs() <= 2 && (p - a).signum() == (b - p).signum() && p != a => "chromatic_passing_tone",
        (Some(a), Some(b)) if a == b && (p - a).abs() <= 2 => "chromatic_neighbor_tone",
        (_, Some(b)) if (b - p).abs() == 1 && key.pcs().contains(&((b % 12) as u8)) => "chromatic_approach_tone",
        _ => {
            let blue = [(key.tonic.pc() + 3) % 12, (key.tonic.pc() + 6) % 12, (key.tonic.pc() + 10) % 12];
            if !key.mode.minor_third() && blue.contains(&pc) {
                "blue_note"
            } else if key.parallel().is_some_and(|par| par.pcs().contains(&pc)) {
                "borrowed_from_parallel_key"
            } else {
                "outside_note (intentional or to review)"
            }
        }
    }
}

fn finding(part: &str, check: &str, severity: &str, kind: &str, detail: String, at: Option<u32>) -> Value {
    json!({ "part": part, "check": check, "severity": severity, "kind": kind, "detail": detail, "at": at.map(super::rhythm::position) })
}

pub struct Context<'a> {
    pub key: Key,
    pub tl: Option<&'a Timeline>,
    pub length: u32,
    /// The request asks for tension/dissonance (dark, tense, aggressive): clashes are expected.
    pub dissonance_ok: bool,
}

pub fn check(tracks: &[Track], cx: &Context) -> Value {
    let mut out: Vec<Value> = vec![];
    let key_pcs = cx.key.pcs();
    let find = |id: &str| tracks.iter().find(|t| t.name.to_lowercase().contains(id));
    for t in tracks {
        let name = t.name.as_str();
        let drums = t.channel == 9;
        // technical
        for n in &t.notes {
            if n.dur == 0 {
                out.push(finding(name, "zero_duration", "error", "technical_error", format!("note {} has no length", n.pitch), Some(n.start)));
            }
            if n.start + n.dur > cx.length {
                out.push(finding(name, "beyond_length", "error", "technical_error", format!("note {} ends after the end ({} ticks)", n.pitch, cx.length), Some(n.start)));
            }
            if n.pitch > 127 || n.vel == 0 {
                out.push(finding(name, "midi_range", "error", "technical_error", "pitch or velocity outside MIDI range".into(), Some(n.start)));
            }
        }
        let mut sorted = t.notes.clone();
        sorted.sort_by_key(|n| (n.pitch, n.start));
        for w in sorted.windows(2) {
            if w[0].pitch == w[1].pitch && w[0].start + w[0].dur > w[1].start {
                out.push(finding(name, "same_pitch_overlap", "error", "technical_error", format!("two {} overlap (DAWs merge or cut them)", w[0].pitch), Some(w[1].start)));
            }
        }
        if drums {
            continue;
        }
        // key compatibility (classified, never removed)
        let mut line = t.notes.clone();
        line.sort_by_key(|n| (n.start, n.pitch));
        for i in 0..line.len() {
            if !key_pcs.contains(&(line[i].pitch % 12)) {
                let why = classify_out_of_key(&line, i, cx.key, cx.tl);
                let kind = if why.starts_with("outside") { "stylistic_choice" } else { "compositional_freedom" };
                out.push(finding(name, "outside_key", if kind == "stylistic_choice" { "warning" } else { "info" }, kind, format!("{} is not in {}: {why}", super::notes::Pitch::from_midi(line[i].pitch as i32, cx.key.signature() < 0).name(), cx.key.name()), Some(line[i].start)));
            }
        }
    }
    // melody vs harmony
    if let (Some(tl), Some(mel)) = (cx.tl, find("melod")) {
        let strong: Vec<&MidiNote> = mel.notes.iter().filter(|n| n.start % PPQ == 0).collect();
        let on = strong.iter().filter(|n| chord_at(tl, n.start).chord.pcs().contains(&(n.pitch % 12))).count();
        let share = if strong.is_empty() { 1.0 } else { on as f64 / strong.len() as f64 };
        out.push(finding("melody", "strong_beats_on_chord_tones", if share >= 0.6 { "info" } else { "warning" }, if share >= 0.6 { "compositional_freedom" } else { "stylistic_choice" }, format!("{:.0}% of the notes on beats are chord tones", share * 100.0), None));
        if let Some(ch) = find("chord") {
            for m in mel.notes.iter().filter(|n| n.start % (PPQ * 2) == 0 && n.dur >= PPQ / 2) {
                let clash = ch.notes.iter().any(|c| c.start <= m.start && c.start + c.dur > m.start && ((m.pitch as i32 - c.pitch as i32).rem_euclid(12) == 1));
                if clash && !cx.dissonance_ok {
                    out.push(finding("melody", "minor_second_clash", "warning", "stylistic_choice", format!("{} sits a semitone above a sustained chord tone", super::notes::Pitch::from_midi(m.pitch as i32, false).name()), Some(m.start)));
                }
            }
        }
        let gaps: Vec<u32> = mel.notes.windows(2).map(|w| w[1].start.saturating_sub(w[0].start + w[0].dur)).collect();
        if gaps.iter().any(|g| *g > 2 * BAR) {
            out.push(finding("melody", "phrase_continuity", "warning", "stylistic_choice", "silences longer than two bars inside the melody".into(), None));
        }
    }
    // bass vs harmony and vs melody
    if let (Some(tl), Some(bass)) = (cx.tl, find("bass")) {
        let changes: Vec<&MidiNote> = bass.notes.iter().filter(|n| { let s = chord_at(tl, n.start); n.start >= s.start && n.start < s.start + PPQ }).collect();
        let fit = changes.iter().filter(|n| chord_at(tl, n.start).chord.pcs().contains(&(n.pitch % 12))).count();
        let share = if changes.is_empty() { 1.0 } else { fit as f64 / changes.len() as f64 };
        out.push(finding("bass", "bass_on_chord_tones_at_changes", if share >= 0.8 { "info" } else { "warning" }, if share >= 0.8 { "compositional_freedom" } else { "stylistic_choice" }, format!("{:.0}% of the bass notes near chord changes are chord tones", share * 100.0), None));
        if let Some(mel) = find("melod") {
            let bass_top = bass.notes.iter().map(|n| n.pitch).max().unwrap_or(0);
            let crossing = mel.notes.iter().filter(|n| n.pitch <= bass_top).count();
            if crossing > 0 {
                out.push(finding("melody", "register_crossing", "warning", "stylistic_choice", format!("{crossing} melody notes at or below the bass register"), None));
            }
            // vertical dissonance melody–bass on beats
            let mut diss = 0;
            let mut beats = 0;
            for m in mel.notes.iter().filter(|n| n.start % PPQ == 0) {
                if let Some(b) = bass.notes.iter().find(|b| b.start <= m.start && b.start + b.dur > m.start) {
                    beats += 1;
                    if matches!((m.pitch as i32 - b.pitch as i32).rem_euclid(12), 1 | 6 | 11) {
                        diss += 1;
                    }
                }
            }
            if beats > 0 && diss * 5 > beats && !cx.dissonance_ok {
                out.push(finding("melody", "melody_bass_dissonance", "warning", "stylistic_choice", format!("{diss} of {beats} beats have a semitone, tritone or major-seventh against the bass"), None));
            }
        }
    }
    let errors = out.iter().filter(|f| f["kind"] == "technical_error").count();
    let warnings = out.iter().filter(|f| f["severity"] == "warning").count();
    json!({ "passed": errors == 0, "technical_errors": errors, "warnings": warnings, "findings": out,
            "rule": "technical errors are fixed; stylistic warnings are reported; tensions and chromatic notes are kept" })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::music::harmony::timeline;
    use crate::music::keys::parse_key;

    fn n(pitch: u8, start: u32, dur: u32) -> MidiNote {
        MidiNote { pitch, start, dur, vel: 100 }
    }

    #[test]
    fn errors_choices_and_freedom_are_told_apart() {
        let k = parse_key("C major").unwrap();
        let tl = timeline(k, &["C".into(), "G".into()], 1.0, 2).unwrap();
        // C D D# E: D# is a chromatic passing tone, kept
        let line = vec![n(60, 0, 100), n(62, 120, 100), n(63, 240, 100), n(64, 360, 100)];
        assert_eq!(classify_out_of_key(&line, 2, k, Some(&tl)), "chromatic_passing_tone");
        let nb = vec![n(64, 0, 100), n(65, 120, 100), n(66, 240, 100), n(65, 360, 100)];
        assert_eq!(classify_out_of_key(&nb, 2, k, None), "chromatic_neighbor_tone", "F → F# → F leaves and returns");
        let nbr = vec![n(67, 0, 100), n(68, 120, 100), n(67, 240, 100)];
        assert_eq!(classify_out_of_key(&nbr, 1, k, None), "chromatic_neighbor_tone");
        let blue = vec![n(60, 0, 100), n(63, 240, 200), n(67, 480, 100)];
        assert_eq!(classify_out_of_key(&blue, 1, k, None), "blue_note");
        let tracks = vec![
            Track { name: "Melody".into(), channel: 1, program: None, notes: line.clone(), cc: vec![] },
            Track { name: "Bass".into(), channel: 2, program: None, notes: vec![n(36, 0, 1900), n(36, 100, 100), n(43, 1920, 4000)], cc: vec![] },
        ];
        let v = check(&tracks, &Context { key: k, tl: Some(&tl), length: 2 * BAR, dissonance_ok: false });
        let kinds: Vec<(&str, &str)> = v["findings"].as_array().unwrap().iter().map(|f| (f["check"].as_str().unwrap(), f["kind"].as_str().unwrap())).collect();
        assert!(kinds.contains(&("same_pitch_overlap", "technical_error")));
        assert!(kinds.contains(&("beyond_length", "technical_error")));
        assert!(kinds.contains(&("outside_key", "compositional_freedom")), "the D# is kept as a chromatic passing tone: {kinds:?}");
        assert_eq!(v["passed"], false);
    }
}
