//! Melody Engine – motif-based composition: a 1-bar motif (rhythm cell + contour), developed into phrases
//! (repetition, sequence, fragmentation, question/answer cadences), realized on the harmony: chord tones on strong
//! positions, passing / neighbour / approach / anticipation / suspension tones on weak ones, each labelled.
//! Styles: simple, progressive, hypnotic, emotional, minimal, cinematic, rhythmic. Variations keep the motif's
//! identity (measured). Deterministic for a seed.
//! Sources: docs/knowledge/music-theory/melody.md §1–6, midi-intelligence/composition-guide.md §1–2.
use super::generate::{MidiNote, Rng, BAR, PPQ};
use super::harmony::{chord_at, Timeline};
use super::theory::Key;
use serde_json::{json, Value};

const S16: u32 = PPQ / 4;
pub const STYLES: [&str; 7] = ["simple", "progressive", "hypnotic", "emotional", "minimal", "cinematic", "rhythmic"];

/// Rhythm cells (start, length) in 16ths within one bar, per style.
fn cells(style: &str) -> &'static [&'static [(u32, u32)]] {
    match style {
        "hypnotic" => &[&[(0, 1), (2, 1), (3, 1), (6, 1), (8, 1), (10, 1), (11, 1), (14, 1)], &[(0, 2), (3, 1), (6, 2), (10, 1), (12, 2)], &[(0, 1), (3, 1), (6, 1), (8, 1), (11, 1), (14, 1)]],
        "emotional" => &[&[(0, 6), (6, 2), (8, 8)], &[(0, 4), (4, 4), (8, 6), (14, 2)], &[(2, 2), (4, 4), (8, 8)], &[(0, 3), (3, 3), (6, 2), (8, 8)]],
        "minimal" => &[&[(0, 2), (6, 2), (12, 2)], &[(0, 1), (8, 1)], &[(3, 2), (10, 2)]],
        "cinematic" => &[&[(0, 8), (8, 8)], &[(0, 12), (12, 4)], &[(0, 6), (6, 2), (8, 8)]],
        "rhythmic" => &[&[(0, 1), (2, 1), (3, 2), (6, 1), (8, 1), (10, 2), (13, 1), (14, 2)], &[(0, 2), (3, 2), (6, 2), (9, 1), (10, 2), (14, 2)]],
        "progressive" => &[&[(0, 2), (2, 2), (4, 2), (6, 2), (8, 4), (12, 4)], &[(0, 3), (3, 3), (6, 2), (8, 2), (10, 2), (12, 4)]],
        _ => &[&[(0, 4), (4, 2), (6, 2), (8, 4), (12, 4)], &[(0, 2), (2, 2), (4, 4), (8, 2), (10, 2), (12, 4)], &[(0, 4), (4, 4), (8, 4), (12, 4)]],
    }
}

/// Contour: scale-step movement between motif notes (first note = 0).
fn contour(style: &str, n: usize, rng: &mut Rng) -> Vec<i32> {
    let mut c = vec![0];
    for i in 1..n {
        let step = match style {
            "hypnotic" | "minimal" => [0, 1, -1, 2, -2, 0][rng.below(6) as usize],
            "cinematic" => if i == 1 { [3, 4, 2][rng.below(3) as usize] } else { [-1, -1, 1, -2][rng.below(4) as usize] },
            "emotional" => if i == n / 2 { [2, 3][rng.below(2) as usize] } else { [1, -1, 1, -1, 2][rng.below(5) as usize] },
            "rhythmic" => [0, 1, -1, 0, 2, -2][rng.below(6) as usize],
            _ => [1, -1, 1, 2, -1, -2][rng.below(6) as usize],
        };
        c.push(step);
    }
    c
}

pub struct Opts {
    pub style: String,
    pub register: (u8, u8),
    pub phrase_bars: u32,
    pub seed: u64,
}

#[derive(Clone, Debug)]
pub struct Labeled {
    pub note: MidiNote,
    pub role: &'static str,
}

/// Scale for melody over a chord: key scale plus the chord's own tones (e.g. G# over E in A minor), dropping the
/// scale note a semitone away from such a tone.
pub fn local_scale(key: Key, chord_pcs: &[u8]) -> Vec<u8> {
    let mut s = key.pcs();
    for &c in chord_pcs {
        if !s.contains(&c) {
            s.retain(|p| (*p as i32 - c as i32).rem_euclid(12) != 1 && (c as i32 - *p as i32).rem_euclid(12) != 1);
            s.push(c);
        }
    }
    s.sort();
    s
}

fn pitches(pcs: &[u8], lo: u8, hi: u8) -> Vec<u8> {
    (lo..=hi).filter(|p| pcs.contains(&(p % 12))).collect()
}

fn nearest(cands: &[u8], target: i32) -> u8 {
    *cands.iter().min_by_key(|p| ((**p as i32 - target).abs(), **p)).unwrap()
}

/// Compose a melody over the timeline.
pub fn compose(key: Key, tl: &Timeline, bars: u32, o: &Opts) -> (Vec<Labeled>, Value) {
    let mut rng = Rng::new(o.seed.wrapping_add(303));
    let style = if STYLES.contains(&o.style.as_str()) { o.style.as_str() } else { "simple" };
    let bank = cells(style);
    let cell_a = bank[rng.below(bank.len() as u32) as usize];
    let cell_b = bank[rng.below(bank.len() as u32) as usize];
    let motif_c = contour(style, cell_a.len(), &mut rng);
    let phrase = o.phrase_bars.clamp(1, 8);
    let (lo, hi) = o.register;
    let mid = (lo as i32 + hi as i32) / 2;
    // phrase plan: which material each bar uses
    let plan: Vec<&str> = (0..bars)
        .map(|b| {
            let pos = b % phrase;
            let last = pos + 1 == phrase;
            let question = (b / phrase) % 2 == 0;
            match style {
                "hypnotic" | "minimal" => if last && b % (phrase * 2) == phrase * 2 - 1 { "variation" } else { "repeat" },
                _ if last => if question { "cadence_open" } else { "cadence_closed" },
                _ if pos == 0 => "motif",
                _ if pos == 1 => "sequence",
                _ => "development",
            }
        })
        .collect();
    let mut out: Vec<Labeled> = vec![];
    let mut prev: i32 = mid;
    let tonic = key.tonic.pc();
    for bar in 0..bars {
        let kind = plan[bar as usize];
        let cell: &[(u32, u32)] = match kind {
            "development" => cell_b,
            "cadence_open" | "cadence_closed" => &cell_a[..cell_a.len().min(3).max(1)],
            _ => cell_a,
        };
        let mut steps = motif_c.clone();
        match kind {
            "sequence" => steps[0] += 1,
            "development" => {
                // fragmentation: the motif's first half, twice
                let h = (steps.len() / 2).max(1);
                steps = steps[..h].iter().chain(steps[..h].iter()).copied().collect();
            }
            "variation" => {
                if let Some(l) = steps.last_mut() {
                    *l += if rng.chance(50) { 1 } else { -1 };
                }
            }
            _ => {}
        }
        let cell: Vec<(u32, u32)> = if kind == "development" { cell.to_vec() } else { cell.iter().take(steps.len()).copied().collect() };
        for (i, &(p16, l16)) in cell.iter().enumerate() {
            let start = bar * BAR + p16 * S16;
            let slot = chord_at(tl, start);
            let ct = slot.chord.pcs();
            let scale = pitches(&local_scale(key, &ct), lo, hi);
            let chord_p = pitches(&ct, lo, hi);
            let strong = p16 % 4 == 0;
            let last_note = i + 1 == cell.len();
            let step = *steps.get(i).unwrap_or(&0);
            // move by scale steps from the previous pitch
            let idx = scale.iter().position(|p| *p as i32 >= prev).unwrap_or(scale.len() - 1) as i32;
            let mut target_idx = (idx + if i == 0 && bar > 0 && kind != "sequence" { 0 } else { step }).clamp(0, scale.len() as i32 - 1) as usize;
            // keep the line near the middle of the register
            if (scale[target_idx] as i32 - mid).abs() > 9 {
                target_idx = if scale[target_idx] as i32 > mid { target_idx.saturating_sub(2) } else { (target_idx + 2).min(scale.len() - 1) };
            }
            let mut pitch = scale[target_idx];
            let mut role = "scale_tone";
            if last_note && kind.starts_with("cadence") {
                // closed: tonic (or nearest stable tone of the chord); open: 2nd/5th/7th degree or chord 3rd
                let goal: Vec<u8> = if kind == "cadence_closed" {
                    if ct.contains(&tonic) { vec![tonic] } else { ct.clone() }
                } else {
                    let opens: Vec<u8> = [2u8, 4, 6].iter().map(|d| key.pcs()[*d as usize % key.pcs().len()]).filter(|p| ct.contains(p)).collect();
                    if opens.is_empty() { ct.iter().skip(1).copied().collect() } else { opens }
                };
                pitch = nearest(&pitches(&goal, lo, hi), prev);
                role = "cadence";
            } else if strong || chord_p.contains(&pitch) {
                if !chord_p.contains(&pitch) {
                    let n = nearest(&chord_p, pitch as i32);
                    // emotional/cinematic keep a held non-chord tone over a chord change as a suspension
                    let held = out.last().is_some_and(|l| l.note.pitch == pitch && l.note.start + l.note.dur >= start.saturating_sub(S16));
                    if held && matches!(style, "emotional" | "cinematic") && (pitch as i32 - n as i32) <= 2 && pitch > n {
                        role = "suspension";
                    } else {
                        pitch = n;
                        role = "chord_tone";
                    }
                } else {
                    role = "chord_tone";
                }
            }
            // gap fill: after a leap larger than a 4th, move back by step
            if let Some(l) = out.last() {
                let leap = l.note.pitch as i32 - out.get(out.len().wrapping_sub(2)).map(|x| x.note.pitch as i32).unwrap_or(l.note.pitch as i32);
                if leap.abs() > 5 && (pitch as i32 - l.note.pitch as i32).signum() == leap.signum() {
                    let back: Vec<u8> = scale.iter().copied().filter(|p| (*p as i32 - l.note.pitch as i32).signum() == -leap.signum() && (*p as i32 - l.note.pitch as i32).abs() <= 2).collect();
                    if let Some(b) = back.first() {
                        if role != "cadence" {
                            pitch = *b;
                            role = if chord_p.contains(b) { "chord_tone" } else { "scale_tone" };
                        }
                    }
                }
                if (pitch as i32 - l.note.pitch as i32).abs() > 12 {
                    pitch = nearest(&pitches(&[pitch % 12], lo, hi), l.note.pitch as i32);
                }
            }
            let end_of_bar = (bar + 1) * BAR;
            let dur = if last_note && kind.starts_with("cadence") { end_of_bar - start } else { (l16 * S16).saturating_sub(15).max(30).min(end_of_bar - start) };
            let vel = if strong { 100 } else { 86 } + if style == "rhythmic" && p16 % 2 == 1 { 6 } else { 0 };
            out.push(Labeled { note: MidiNote { pitch, start, dur, vel: rng.jitter(vel, 5) }, role });
            prev = pitch as i32;
        }
    }
    // climax in the second half of each phrase for arching styles
    if matches!(style, "emotional" | "cinematic" | "progressive") && out.len() > 4 {
        let plen = phrase * BAR;
        let phrases = (bars * BAR).div_ceil(plen);
        for ph in 0..phrases {
            let (a, b) = (ph * plen, (ph + 1) * plen);
            let idx: Vec<usize> = (0..out.len()).filter(|i| out[*i].note.start >= a && out[*i].note.start < b).collect();
            if idx.len() < 3 {
                continue;
            }
            let max = idx.iter().map(|i| out[*i].note.pitch).max().unwrap();
            let target = idx[idx.len() * 2 / 3];
            if out[target].role == "cadence" {
                continue;
            }
            let first_max = *idx.iter().find(|i| out[**i].note.pitch == max).unwrap();
            if first_max < idx[idx.len() / 2] {
                let ct = chord_at(tl, out[target].note.start).chord.pcs();
                if let Some(p) = pitches(&ct, max + 1, hi).first() {
                    if (*p as i32 - out[target - 1].note.pitch as i32).abs() <= 12 {
                        out[target].note.pitch = *p;
                        out[target].role = "climax";
                    }
                }
            }
        }
    }
    label_weak(&mut out, tl, key);
    trim(&mut out);
    let analysis = describe(&out, key, tl, &plan, cell_a, &motif_c, style);
    (out, analysis)
}

/// Name the non-chord tones on weak positions: passing, neighbour, approach, anticipation.
fn label_weak(v: &mut [Labeled], tl: &Timeline, _key: Key) {
    for i in 0..v.len() {
        if v[i].role != "scale_tone" {
            continue;
        }
        let p = v[i].note.pitch as i32;
        let ct = chord_at(tl, v[i].note.start).chord.pcs();
        if ct.contains(&(v[i].note.pitch % 12)) {
            v[i].role = "chord_tone";
            continue;
        }
        let prev = (i > 0).then(|| v[i - 1].note.pitch as i32);
        let next = v.get(i + 1).map(|n| n.note.pitch as i32);
        let next_is_ct = v.get(i + 1).is_some_and(|n| chord_at(tl, n.note.start).chord.pcs().contains(&(n.note.pitch % 12)));
        v[i].role = match (prev, next) {
            (Some(a), Some(b)) if (p - a).abs() <= 2 && (b - p).abs() <= 2 && (p - a).signum() == (b - p).signum() && (p - a) != 0 => "passing",
            (Some(a), Some(b)) if a == b && (p - a).abs() <= 2 => "neighbor",
            (_, Some(b)) if next_is_ct && b == p => "anticipation",
            (_, Some(b)) if next_is_ct && (b - p).abs() <= 2 => "approach",
            _ => "scale_tone",
        };
    }
}

fn trim(v: &mut [Labeled]) {
    for i in 1..v.len() {
        let (a, b) = v.split_at_mut(i);
        let prev = &mut a[i - 1].note;
        if prev.start + prev.dur > b[0].note.start {
            prev.dur = (b[0].note.start - prev.start).saturating_sub(10).max(10);
        }
    }
}

fn describe(v: &[Labeled], key: Key, tl: &Timeline, plan: &[&str], cell: &[(u32, u32)], motif: &[i32], style: &str) -> Value {
    let mut roles = serde_json::Map::new();
    for l in v {
        let n = roles.get(l.role).and_then(|x| x.as_u64()).unwrap_or(0) + 1;
        roles.insert(l.role.into(), json!(n));
    }
    let (lo, hi) = (v.iter().map(|l| l.note.pitch).min().unwrap_or(0), v.iter().map(|l| l.note.pitch).max().unwrap_or(0));
    let peak = v.iter().position(|l| l.note.pitch == hi).unwrap_or(0);
    let strong: Vec<&Labeled> = v.iter().filter(|l| l.note.start % PPQ == 0).collect();
    let on_ct = strong.iter().filter(|l| chord_at(tl, l.note.start).chord.pcs().contains(&(l.note.pitch % 12)) || l.role == "suspension").count();
    json!({
        "style": style, "key": key.name(),
        "motif": { "rhythm_16ths": cell, "contour_steps": motif, "notes": cell.len() },
        "phrase_plan": plan, "roles": roles,
        "range": { "low": super::notes::Pitch::from_midi(lo as i32, key.signature() < 0).name(), "high": super::notes::Pitch::from_midi(hi as i32, key.signature() < 0).name(), "semitones": hi as i32 - lo as i32 },
        "climax_position": if v.is_empty() { 0.0 } else { (peak as f64 / v.len() as f64 * 100.0).round() / 100.0 },
        "strong_beats_on_chord_tones": if strong.is_empty() { 1.0 } else { (on_ct as f64 / strong.len() as f64 * 100.0).round() / 100.0 },
        "source": "docs/knowledge/music-theory/melody.md",
    })
}

/// Similarity of two melodies (0–1): interval contour (direction and size) and onset rhythm. Used to keep a
/// variation's identity and to check that a melody does not copy a reference.
pub fn similarity(a: &[MidiNote], b: &[MidiNote]) -> f64 {
    if a.len() < 2 || b.len() < 2 {
        return 0.0;
    }
    let iv = |v: &[MidiNote]| v.windows(2).map(|w| w[1].pitch as i32 - w[0].pitch as i32).collect::<Vec<_>>();
    let (x, y) = (iv(a), iv(b));
    let n = x.len().min(y.len());
    let same_dir = (0..n).filter(|i| x[*i].signum() == y[*i].signum()).count() as f64;
    let same_iv = (0..n).filter(|i| x[*i] == y[*i]).count() as f64;
    let on = |v: &[MidiNote]| v.iter().map(|m| m.start % BAR).collect::<std::collections::BTreeSet<_>>();
    let (ra, rb) = (on(a), on(b));
    let rhythm = ra.intersection(&rb).count() as f64 / ra.union(&rb).count().max(1) as f64;
    let len = n as f64 / x.len().max(y.len()) as f64;
    ((0.35 * same_dir / n as f64 + 0.35 * same_iv / n as f64 + 0.3 * rhythm) * len * 100.0).round() / 100.0
}

pub const VARIATIONS: [&str; 7] = ["rhythmic", "ornamental", "sequence", "inversion", "ending", "octave", "fragment"];

/// A variation of a melody that keeps its identity (the similarity to the original is reported).
pub fn vary(notes: &[MidiNote], key: Key, tl: &Timeline, kind: &str, seed: u64) -> Result<(Vec<MidiNote>, Value), String> {
    if notes.is_empty() {
        return Err("no notes to vary".into());
    }
    let mut rng = Rng::new(seed.wrapping_add(707));
    let scale_pcs = key.pcs();
    let lo = notes.iter().map(|n| n.pitch).min().unwrap().saturating_sub(5);
    let hi = notes.iter().map(|n| n.pitch).max().unwrap().saturating_add(5).min(108);
    let scale = pitches(&scale_pcs, lo.min(48), hi.max(84));
    let step_by = |p: u8, k: i32| -> u8 {
        let i = scale.iter().position(|x| *x >= p).unwrap_or(0) as i32;
        scale[(i + k).clamp(0, scale.len() as i32 - 1) as usize]
    };
    let mut v = notes.to_vec();
    match kind {
        "rhythmic" => {
            for n in v.iter_mut().filter(|n| n.start % PPQ != 0) {
                if rng.chance(50) {
                    n.start += S16;
                }
            }
        }
        "ornamental" => {
            let mut add = vec![];
            for w in notes.windows(2) {
                let gap = w[1].start.saturating_sub(w[0].start + w[0].dur);
                let iv = w[1].pitch as i32 - w[0].pitch as i32;
                if (3..=4).contains(&iv.abs()) && w[0].dur >= 2 * S16 {
                    let pass = step_by(w[0].pitch, iv.signum());
                    let start = w[0].start + w[0].dur / 2;
                    add.push(MidiNote { pitch: pass, start, dur: (w[0].dur / 2).saturating_sub(10).max(30) + gap.min(0), vel: w[0].vel.saturating_sub(10) });
                }
            }
            for n in v.iter_mut() {
                if add.iter().any(|a| a.start > n.start && a.start < n.start + n.dur) {
                    n.dur /= 2;
                }
            }
            v.extend(add);
        }
        "sequence" => {
            for n in v.iter_mut().filter(|n| (n.start / BAR) % 2 == 1) {
                n.pitch = step_by(n.pitch, 1);
            }
        }
        "inversion" => {
            let first = scale.iter().position(|x| *x >= notes[0].pitch).unwrap_or(0) as i32;
            for n in v.iter_mut() {
                let i = scale.iter().position(|x| *x >= n.pitch).unwrap_or(0) as i32;
                n.pitch = scale[(2 * first - i).clamp(0, scale.len() as i32 - 1) as usize];
            }
        }
        "ending" => {
            let last_bar = notes.iter().map(|n| n.start / BAR).max().unwrap();
            if let Some(n) = v.iter_mut().filter(|n| n.start / BAR == last_bar).last() {
                let ct = chord_at(tl, n.start).chord.pcs();
                let tonic = key.tonic.pc();
                let closed = n.pitch % 12 == tonic;
                let goal: Vec<u8> = if closed { ct.iter().copied().filter(|p| *p != tonic).collect() } else { vec![tonic] };
                n.pitch = nearest(&pitches(&goal, n.pitch.saturating_sub(7), n.pitch.saturating_add(7)), n.pitch as i32);
            }
        }
        "octave" => {
            let up = v.iter().map(|n| n.pitch).max().unwrap() + 12 <= 96;
            for n in v.iter_mut() {
                n.pitch = if up { n.pitch + 12 } else { n.pitch.saturating_sub(12) };
            }
        }
        "fragment" => {
            v.retain(|n| (n.start % (2 * BAR)) < BAR + BAR / 2);
            let first_half: Vec<MidiNote> = notes.iter().filter(|n| n.start % (2 * BAR) < BAR / 2).cloned().collect();
            for n in first_half {
                v.push(MidiNote { start: n.start + BAR + BAR / 2, ..n });
            }
        }
        _ => return Err(format!("unknown variation {kind:?} (use {})", VARIATIONS.join(", "))),
    }
    v.sort_by_key(|n| (n.start, n.pitch));
    let mut l: Vec<Labeled> = v.into_iter().map(|note| Labeled { note, role: "" }).collect();
    trim(&mut l);
    let v: Vec<MidiNote> = l.into_iter().map(|x| x.note).collect();
    let sim = similarity(notes, &v);
    Ok((v, json!({ "variation": kind, "identity_kept": sim, "note": "similarity of contour and rhythm to the original (1 = identical)" })))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::music::harmony::timeline;
    use crate::music::keys::parse_key;

    fn setup() -> (Key, Timeline) {
        let k = parse_key("A minor").unwrap();
        let tl = timeline(k, &["Am".into(), "F".into(), "C".into(), "G".into()], 1.0, 8).unwrap();
        (k, tl)
    }

    #[test]
    fn every_style_is_musical() {
        let (k, tl) = setup();
        for style in STYLES {
            for seed in 0..12 {
                let (m, a) = compose(k, &tl, 8, &Opts { style: style.into(), register: (57, 84), phrase_bars: 4, seed });
                assert!(!m.is_empty(), "{style}");
                let notes: Vec<&MidiNote> = m.iter().map(|l| &l.note).collect();
                assert!(notes.iter().all(|n| (57..=84).contains(&n.pitch)), "{style} {seed} register");
                assert!(notes.windows(2).all(|w| w[0].start + w[0].dur <= w[1].start), "{style} monophonic");
                assert!(notes.windows(2).all(|w| (w[0].pitch as i32 - w[1].pitch as i32).abs() <= 12), "{style} {seed} leaps within an octave");
                // in key, or a tone of the current chord (e.g. harmonic-minor colour), or a labelled suspension
                for l in &m {
                    let ct = chord_at(&tl, l.note.start).chord.pcs();
                    assert!(k.pcs().contains(&(l.note.pitch % 12)) || ct.contains(&(l.note.pitch % 12)), "{style} {seed} {:?}", l);
                }
                assert!(a["strong_beats_on_chord_tones"].as_f64().unwrap() >= 0.75, "{style} {seed} {}", a["strong_beats_on_chord_tones"]);
            }
        }
    }

    #[test]
    fn phrases_answer_questions_and_motifs_repeat() {
        let (k, tl) = setup();
        // a progression that returns to the tonic at the end of each phrase: the answer can close on A
        let home = timeline(k, &["Am".into(), "F".into(), "G".into(), "Am".into()], 1.0, 8).unwrap();
        let (m, a) = compose(k, &home, 8, &Opts { style: "simple".into(), register: (57, 84), phrase_bars: 4, seed: 3 });
        assert_eq!(a["phrase_plan"], json!(["motif", "sequence", "development", "cadence_open", "motif", "sequence", "development", "cadence_closed"]));
        let last = m.last().unwrap();
        assert_eq!(last.note.pitch % 12, 9, "the answer ends on the tonic A");
        assert_eq!(last.role, "cadence");
        // bar 1 and bar 5 state the same motif rhythm
        let rh = |bar: u32| m.iter().filter(|l| l.note.start / BAR == bar).map(|l| l.note.start % BAR).collect::<Vec<_>>();
        assert_eq!(rh(0), rh(4));
        // hypnotic: the same bar repeated
        let (h, _) = compose(k, &tl, 4, &Opts { style: "hypnotic".into(), register: (57, 79), phrase_bars: 4, seed: 1 });
        let rhh = |bar: u32| h.iter().filter(|l| l.note.start / BAR == bar).map(|l| l.note.start % BAR).collect::<Vec<_>>();
        assert_eq!(rhh(0), rhh(1));
        // non-chord tones are named
        let roles: std::collections::BTreeSet<&str> = (0..20).flat_map(|s| compose(k, &tl, 8, &Opts { style: "progressive".into(), register: (57, 84), phrase_bars: 4, seed: s }).0.into_iter().map(|l| l.role)).collect();
        assert!(roles.contains("passing") || roles.contains("neighbor") || roles.contains("approach"), "{roles:?}");
        // reproducible
        assert_eq!(compose(k, &tl, 8, &Opts { style: "emotional".into(), register: (57, 84), phrase_bars: 4, seed: 9 }).1, compose(k, &tl, 8, &Opts { style: "emotional".into(), register: (57, 84), phrase_bars: 4, seed: 9 }).1);
    }

    #[test]
    fn variations_keep_identity() {
        let (k, tl) = setup();
        let (m, _) = compose(k, &tl, 8, &Opts { style: "simple".into(), register: (57, 84), phrase_bars: 4, seed: 5 });
        let orig: Vec<MidiNote> = m.iter().map(|l| l.note).collect();
        for kind in VARIATIONS {
            let (v, info) = vary(&orig, k, &tl, kind, 1).unwrap();
            assert!(!v.is_empty(), "{kind}");
            assert!(v.windows(2).all(|w| w[0].start + w[0].dur <= w[1].start), "{kind} monophonic");
            let id = info["identity_kept"].as_f64().unwrap();
            assert!(id > 0.3 && (id < 1.0 || kind == "octave"), "{kind}: identity {id}");
        }
        assert_eq!(similarity(&orig, &orig), 1.0);
        assert!(vary(&orig, k, &tl, "nope", 1).is_err());
    }
}
