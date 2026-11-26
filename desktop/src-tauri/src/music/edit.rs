//! MIDI editing – operations on real MIDI notes, from words (IT/EN) or structured ops: transpose (chromatic or
//! within the key), octave, quantize, velocity, duration, legato/staccato, shift, humanize, syncopate, make
//! hypnotic, change mode (major ↔ minor), invert, reverse, simplify. Every edit returns the new notes and a diff
//! against the original; versions and undo live in the clip store (music::api).
use super::generate::{MidiNote, Rng, BAR, PPQ};
use super::rhythm;
use super::theory::{Key, Mode};
use serde_json::{json, Value};

const S16: u32 = PPQ / 4;

#[derive(Clone, Debug, PartialEq, serde::Serialize)]
#[serde(tag = "op", rename_all = "snake_case")]
pub enum Op {
    Transpose { semitones: i32 },
    Diatonic { steps: i32 },
    Octave { by: i32 },
    Quantize { grid: String, strength: u32 },
    Velocity { scale: f64, add: i32 },
    Duration { factor: f64 },
    Legato,
    Staccato,
    Shift { ticks: i32 },
    Humanize { timing: u32, velocity: u8 },
    Syncopate { amount: u32 },
    Hypnotic,
    ToMode { mode: String },
    Invert,
    Reverse,
    Simplify,
}

fn number_word(w: &str) -> Option<i32> {
    w.parse().ok().or(match w {
        "un" | "uno" | "una" | "one" | "a" | "an" => Some(1),
        "due" | "two" => Some(2),
        "tre" | "three" => Some(3),
        "quattro" | "four" => Some(4),
        "cinque" | "five" => Some(5),
        "sei" | "six" => Some(6),
        "sette" | "seven" => Some(7),
        _ => None,
    })
}

/// Words → operations. Returns the operations and the words that were not understood.
pub fn parse(text: &str) -> (Vec<Op>, Vec<String>) {
    let t = format!(" {} ", text.to_lowercase().replace(['\'', '’', ',', '.', '!'], " "));
    let has = |w: &str| t.contains(&format!(" {w} ")) || (w.contains(' ') && t.contains(w));
    let words: Vec<&str> = t.split_whitespace().collect();
    let down = ["abbassa", "giù", "giu", "down", "lower", "scendi", "sotto"].iter().any(|w| has(w));
    let up = ["alza", "su", "up", "raise", "higher", "sali", "sopra"].iter().any(|w| has(w));
    let dir = if down { -1 } else { 1 };
    let count_before = |keys: &[&str]| -> Option<i32> {
        words.iter().position(|w| keys.contains(w)).and_then(|i| (i > 0).then(|| number_word(words[i - 1])).flatten())
    };
    let mut ops = vec![];
    if has("ottava") || has("ottave") || has("octave") || has("octaves") {
        ops.push(Op::Octave { by: dir * count_before(&["ottava", "ottave", "octave", "octaves"]).unwrap_or(1) });
    } else if has("semitono") || has("semitoni") || has("semitone") || has("semitones") {
        ops.push(Op::Transpose { semitones: dir * count_before(&["semitono", "semitoni", "semitone", "semitones"]).unwrap_or(1) });
    } else if (has("tono") || has("toni") || has("tone") || has("whole step")) && (up || down) {
        ops.push(Op::Transpose { semitones: dir * 2 * count_before(&["tono", "toni", "tone"]).unwrap_or(1) });
    } else if (has("grado") || has("gradi") || has("step") || has("steps")) && (up || down) {
        ops.push(Op::Diatonic { steps: dir * count_before(&["grado", "gradi", "step", "steps"]).unwrap_or(1) });
    } else if has("trasponi") || has("transpose") {
        if let Some(n) = words.iter().find_map(|w| w.trim_start_matches('+').parse::<i32>().ok()) {
            ops.push(Op::Transpose { semitones: if down { -n.abs() } else { n } });
        }
    }
    if has("minore") || has("minor") {
        if has("trasforma") || has("rendi") || has("make") || has("turn") || has("tonalità") || has("convert") {
            ops.push(Op::ToMode { mode: "minor".into() });
        }
    } else if (has("maggiore") || has("major")) && (has("trasforma") || has("rendi") || has("make") || has("turn") || has("convert")) {
        ops.push(Op::ToMode { mode: "major".into() });
    }
    if has("quantizza") || has("quantize") || has("a tempo") || has("in griglia") {
        let grid = ["1/32", "1/16", "1/8", "1/4", "1/8t", "1/16t"].iter().find(|g| t.contains(*g)).map(|g| g.to_uppercase().replace('T', "T")).unwrap_or("1/16".into());
        ops.push(Op::Quantize { grid: grid.replace("1/8T", "1/8T"), strength: if has("leggermente") || has("slightly") { 50 } else { 100 } });
    }
    if has("umanizza") || has("humanize") || has("più umano") || has("piu umano") || has("meno robotico") {
        ops.push(Op::Humanize { timing: 12, velocity: 8 });
    }
    if has("sincopato") || has("sincopata") || has("sincopi") || has("syncopated") || has("syncopation") {
        ops.push(Op::Syncopate { amount: if has("molto") || has("very") { 80 } else { 50 } });
    }
    if has("ipnotico") || has("ipnotica") || has("hypnotic") {
        ops.push(Op::Hypnotic);
    }
    if has("allunga") || has("lengthen") || has("longer") || has("più lunghe") || has("piu lunghe") || has("sostenute") {
        ops.push(Op::Duration { factor: 1.5 });
    }
    if has("accorcia") || has("shorter") || has("più corte") || has("piu corte") {
        ops.push(Op::Duration { factor: 0.6 });
    }
    if has("legato") {
        ops.push(Op::Legato);
    }
    if has("staccato") || has("staccate") {
        ops.push(Op::Staccato);
    }
    if has("più forte") || has("piu forte") || has("louder") || has("più intensa") || has("piu intensa") {
        ops.push(Op::Velocity { scale: 1.0, add: 15 });
    }
    if has("più piano") || has("piu piano") || has("più morbida") || has("piu morbida") || has("softer") || has("quieter") {
        ops.push(Op::Velocity { scale: 1.0, add: -15 });
    }
    if (has("sposta") || has("shift") || has("move")) && (has("avanti") || has("indietro") || has("later") || has("earlier")) {
        let unit = if has("ottavo") || has("eighth") { PPQ / 2 } else if has("battito") || has("beat") || has("quarto") { PPQ } else { S16 };
        let n = count_before(&["sedicesimo", "sedicesimi", "ottavo", "ottavi", "sixteenth", "eighth", "beat", "battito", "quarto"]).unwrap_or(1);
        let sign = if has("indietro") || has("earlier") { -1 } else { 1 };
        ops.push(Op::Shift { ticks: sign * n * unit as i32 });
    }
    if has("inverti") || has("invert") || has("inversione") || has("capovolgi") {
        ops.push(Op::Invert);
    }
    if has("al contrario") || has("retrograda") || has("reverse") || has("backwards") {
        ops.push(Op::Reverse);
    }
    if has("semplifica") || has("simplify") || has("meno note") || has("fewer notes") {
        ops.push(Op::Simplify);
    }
    let known = ["abbassa", "alza", "di", "un", "una", "uno", "ottava", "ottave", "questa", "questo", "questi", "melodia", "basso", "accordi", "arpeggio", "rendi", "rendila", "rendilo", "rendili", "rendile", "fai", "fallo", "falla", "ancora", "po", "poco", "it", "più", "piu", "trasforma", "progressione", "in", "tonalità", "minore", "maggiore", "le", "note", "degli", "delle", "il", "la", "lo", "gli", "i", "allunga", "accorcia", "sincopato", "sincopata", "ipnotico", "ipnotica", "quantizza", "a", "umanizza", "the", "this", "make", "more", "an", "by", "octave", "lower", "raise", "up", "down", "bass", "melody", "chords", "notes", "arp", "semitoni", "semitono", "tono", "toni", "e", "and", "trasponi", "sposta", "avanti", "indietro", "sedicesimo", "ottavo", "inverti", "al", "contrario", "semplifica", "forte", "piano", "legato", "staccato", "leggermente", "molto", "minor", "major", "turn", "into", "key", "transpose", "humanize", "quantize", "syncopated", "hypnotic", "lengthen", "longer", "shorter", "su", "giù", "giu", "grado", "gradi", "griglia"];
    let unknown = words.iter().filter(|w| !known.contains(w) && number_word(w).is_none() && !w.contains('/')).map(|w| w.to_string()).collect();
    (ops, unknown)
}

fn scale_index(scale: &[u8], p: u8) -> (usize, i32) {
    // nearest scale pitch at or below, plus the chromatic offset from it
    let i = scale.iter().rposition(|s| *s <= p).unwrap_or(0);
    (i, p as i32 - scale[i] as i32)
}

/// Apply operations. Returns the new notes and the key after the edit (changes with ToMode).
pub fn apply(notes: &[MidiNote], ops: &[Op], key: Key, seed: u64) -> Result<(Vec<MidiNote>, Key), String> {
    let mut v = notes.to_vec();
    let mut key = key;
    let len = v.iter().map(|n| n.start + n.dur).max().unwrap_or(BAR).div_ceil(BAR) * BAR;
    for op in ops {
        match op {
            Op::Transpose { semitones } => {
                if v.iter().any(|n| n.pitch as i32 + semitones < 0 || n.pitch as i32 + semitones > 127) {
                    return Err("the transposition would leave the MIDI range".into());
                }
                v.iter_mut().for_each(|n| n.pitch = (n.pitch as i32 + semitones) as u8);
            }
            Op::Octave { by } => {
                if v.iter().any(|n| n.pitch as i32 + 12 * by < 0 || n.pitch as i32 + 12 * by > 127) {
                    return Err("the octave shift would leave the MIDI range".into());
                }
                v.iter_mut().for_each(|n| n.pitch = (n.pitch as i32 + 12 * by) as u8);
            }
            Op::Diatonic { steps } => {
                let scale: Vec<u8> = (0..=127u8).filter(|p| key.pcs().contains(&(p % 12))).collect();
                for n in v.iter_mut() {
                    let (i, off) = scale_index(&scale, n.pitch);
                    let j = (i as i32 + steps).clamp(0, scale.len() as i32 - 1) as usize;
                    n.pitch = (scale[j] as i32 + off).clamp(0, 127) as u8;
                }
            }
            Op::ToMode { mode } => {
                let target = Key { tonic: key.tonic, mode: if mode == "minor" { Mode::NaturalMinor } else { Mode::Major } };
                let (from, to) = (key.pcs(), target.pcs());
                if from.len() == to.len() {
                    for n in v.iter_mut() {
                        let pc = n.pitch % 12;
                        if let Some(d) = from.iter().position(|p| *p == pc) {
                            n.pitch = (n.pitch as i32 + (to[d] as i32 - from[d] as i32 + 6).rem_euclid(12) - 6) as u8;
                        }
                    }
                }
                key = target;
            }
            Op::Quantize { grid, strength } => rhythm::quantize(&mut v, rhythm::grid_ticks(grid)?, *strength, false),
            Op::Velocity { scale, add } => v.iter_mut().for_each(|n| n.vel = ((n.vel as f64 * scale) as i32 + add).clamp(1, 127) as u8),
            Op::Duration { factor } => v.iter_mut().for_each(|n| n.dur = ((n.dur as f64 * factor) as u32).max(10)),
            Op::Legato => {
                let mut starts: Vec<u32> = v.iter().map(|n| n.start).collect();
                starts.sort();
                starts.dedup();
                for n in v.iter_mut() {
                    let next = starts.iter().find(|s| **s > n.start).copied().unwrap_or(len);
                    n.dur = (next - n.start).saturating_sub(5).max(10);
                }
            }
            Op::Staccato => v.iter_mut().for_each(|n| n.dur = (n.dur / 2).clamp(30, PPQ / 4)),
            Op::Shift { ticks } => {
                for n in v.iter_mut() {
                    n.start = (n.start as i32 + ticks).max(0) as u32;
                }
            }
            Op::Humanize { timing, velocity } => rhythm::humanize(&mut v, *timing, *velocity, seed),
            Op::Syncopate { amount } => {
                // anticipate some on-beat notes by a 16th (or push to the "a" of the beat), staying in the bar
                let mut rng = Rng::new(seed.wrapping_add(31));
                for n in v.iter_mut() {
                    if n.start % PPQ == 0 && n.start % BAR != 0 && rng.chance(*amount) {
                        if rng.chance(50) && n.start >= S16 {
                            n.start -= S16;
                            n.dur += S16 / 2;
                        } else {
                            n.start += 3 * S16;
                            n.dur = n.dur.min(2 * S16);
                        }
                    }
                }
            }
            Op::Hypnotic => {
                // per bar: keep the three most used pitches, regular 16th grid, steady gate, accent every 4
                let bars = len / BAR;
                let mut out = vec![];
                for b in 0..bars.max(1) {
                    let in_bar: Vec<MidiNote> = v.iter().filter(|n| n.start / BAR == b).copied().collect();
                    if in_bar.is_empty() {
                        continue;
                    }
                    let mut count: Vec<(u8, usize)> = vec![];
                    for n in &in_bar {
                        match count.iter_mut().find(|c| c.0 == n.pitch) {
                            Some(c) => c.1 += 1,
                            None => count.push((n.pitch, 1)),
                        }
                    }
                    count.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(&b.0)));
                    let keep: Vec<u8> = count.iter().take(3).map(|c| c.0).collect();
                    let mut cell: Vec<u8> = keep.clone();
                    cell.sort();
                    for s in 0..16u32 {
                        let p = cell[s as usize % cell.len()];
                        out.push(MidiNote { pitch: p, start: b * BAR + s * S16, dur: S16 * 3 / 4, vel: if s % 4 == 0 { 108 } else { 88 } });
                    }
                }
                v = out;
            }
            Op::Invert => {
                if let Some(first) = v.iter().min_by_key(|n| n.start).map(|n| n.pitch) {
                    let scale: Vec<u8> = (0..=127u8).filter(|p| key.pcs().contains(&(p % 12))).collect();
                    let (fi, _) = scale_index(&scale, first);
                    for n in v.iter_mut() {
                        let (i, off) = scale_index(&scale, n.pitch);
                        let j = (2 * fi as i32 - i as i32).clamp(0, scale.len() as i32 - 1) as usize;
                        n.pitch = (scale[j] as i32 - off).clamp(0, 127) as u8;
                    }
                }
            }
            Op::Reverse => {
                let end = v.iter().map(|n| n.start + n.dur).max().unwrap_or(0);
                for n in v.iter_mut() {
                    n.start = end - (n.start + n.dur);
                }
            }
            Op::Simplify => {
                let keep_short = v.len() <= 4;
                v.retain(|n| keep_short || n.dur >= S16 || n.start % PPQ == 0);
            }
        }
    }
    v.sort_by_key(|n| (n.start, n.pitch));
    Ok((v, key))
}

/// Differences between two versions of a part.
pub fn diff(a: &[MidiNote], b: &[MidiNote]) -> Value {
    let key = |n: &MidiNote| (n.start, n.pitch);
    let sa: std::collections::BTreeMap<(u32, u8), &MidiNote> = a.iter().map(|n| (key(n), n)).collect();
    let sb: std::collections::BTreeMap<(u32, u8), &MidiNote> = b.iter().map(|n| (key(n), n)).collect();
    let same = sa.keys().filter(|k| sb.contains_key(k)).count();
    let changed = sa.iter().filter(|(k, n)| sb.get(k).is_some_and(|m| m.dur != n.dur || m.vel != n.vel)).count();
    let avg = |v: &[MidiNote]| if v.is_empty() { 0.0 } else { v.iter().map(|n| n.pitch as f64).sum::<f64>() / v.len() as f64 };
    json!({
        "notes_before": a.len(), "notes_after": b.len(),
        "unchanged_position_and_pitch": same, "changed_length_or_velocity": changed,
        "removed": a.len() - same, "added": b.len() - same,
        "average_pitch_change": ((avg(b) - avg(a)) * 10.0).round() / 10.0,
        "syncopation_before": rhythm::syncopation(a), "syncopation_after": rhythm::syncopation(b),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::music::keys::parse_key;

    fn n(p: u8, s: u32, d: u32) -> MidiNote {
        MidiNote { pitch: p, start: s, dur: d, vel: 100 }
    }

    #[test]
    fn the_prompt_examples_in_words() {
        assert_eq!(parse("Abbassa questa melodia di un'ottava.").0, vec![Op::Octave { by: -1 }]);
        assert_eq!(parse("Rendi il basso più sincopato.").0, vec![Op::Syncopate { amount: 50 }]);
        assert_eq!(parse("Allunga le note degli accordi.").0, vec![Op::Duration { factor: 1.5 }]);
        assert_eq!(parse("Rendi questo arpeggio più ipnotico.").0, vec![Op::Hypnotic]);
        let (ops, unknown) = parse("Abbassa questa melodia di un'ottava e rendila più sincopata");
        assert_eq!(ops, vec![Op::Octave { by: -1 }, Op::Syncopate { amount: 50 }]);
        assert!(unknown.is_empty(), "{unknown:?}");
        assert_eq!(parse("Trasforma questa progressione in tonalità minore.").0, vec![Op::ToMode { mode: "minor".into() }]);
        assert_eq!(parse("alza di due semitoni").0, vec![Op::Transpose { semitones: 2 }]);
        assert_eq!(parse("quantizza a 1/16 e umanizza").0, vec![Op::Quantize { grid: "1/16".into(), strength: 100 }, Op::Humanize { timing: 12, velocity: 8 }]);
        assert_eq!(parse("make it two octaves lower").0, vec![Op::Octave { by: -2 }]);
        let (ops, unknown) = parse("fallo suonare come un violino");
        assert!(ops.is_empty() && unknown.contains(&"violino".to_string()));
    }

    #[test]
    fn edits_change_the_real_notes() {
        let c = parse_key("C major").unwrap();
        let mel = vec![n(60, 0, 240), n(64, 480, 240), n(67, 960, 240), n(65, 1440, 240)];
        let (o, _) = apply(&mel, &[Op::Octave { by: -1 }], c, 1).unwrap();
        assert_eq!(o.iter().map(|x| x.pitch).collect::<Vec<_>>(), [48, 52, 55, 53]);
        let (m, k) = apply(&mel, &[Op::ToMode { mode: "minor".into() }], c, 1).unwrap();
        assert_eq!((m[1].pitch, k.name()), (63, "C minor".to_string()), "E → Eb");
        let (d, _) = apply(&mel, &[Op::Diatonic { steps: 1 }], c, 1).unwrap();
        assert_eq!(d.iter().map(|x| x.pitch).collect::<Vec<_>>(), [62, 65, 69, 67], "one scale step up, in key");
        let bass: Vec<MidiNote> = (0..8).map(|b| n(36, b * PPQ, 200)).collect();
        let (s, _) = apply(&bass, &[Op::Syncopate { amount: 100 }], c, 1).unwrap();
        assert!(rhythm::syncopation(&s) > rhythm::syncopation(&bass));
        let (l, _) = apply(&mel, &[Op::Duration { factor: 1.5 }], c, 1).unwrap();
        assert_eq!(l[0].dur, 360);
        let arp: Vec<MidiNote> = (0..16).map(|i| n([57, 60, 64, 69, 72][i % 5], i as u32 * 120, 100)).collect();
        let (h, _) = apply(&arp, &[Op::Hypnotic], c, 1).unwrap();
        assert!(h.iter().map(|x| x.pitch).collect::<std::collections::BTreeSet<_>>().len() <= 3, "fewer pitches, repeated");
        let dd = diff(&mel, &o);
        assert_eq!((dd["notes_before"].as_u64(), dd["average_pitch_change"].as_f64()), (Some(4), Some(-12.0)));
        assert!(apply(&[n(120, 0, 10)], &[Op::Octave { by: 1 }], c, 1).is_err(), "never outside MIDI range");
        let (r, _) = apply(&mel, &[Op::Reverse], c, 1).unwrap();
        assert_eq!(r[0].pitch, 65);
    }
}
