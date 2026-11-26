//! Bassline Engine – bass lines anchored to the chord roots, in 11 styles, aware of the kick (onsets on kick hits are
//! avoided where the style wants a clean low end, and every collision is reported). Each note is labelled (root,
//! fifth, octave, third, approach, passing). Register E1–D#3 (MIDI 28–51).
//! Sources: docs/knowledge/midi-intelligence/composition-guide.md §1, §3; music-production/genres.md §1–2.
use super::generate::{MidiNote, Rng, BAR, PPQ};
use super::harmony::{chord_at, Timeline};
use super::theory::Key;
use serde_json::{json, Value};

const S16: u32 = PPQ / 4;
pub const STYLES: [&str; 11] = ["sub", "melodic", "rhythmic", "syncopated", "sequential", "hypnotic", "progressive", "offbeat", "rolling", "house", "pulse"];

/// (16th step, length in 16ths, tone): R root, 5 fifth, 8 octave, 3 third, A approach to the next chord root.
fn pattern(style: &str, bar: u32, bars: u32, rng: &mut Rng) -> Vec<(u32, u32, char)> {
    match style {
        "sub" => vec![(0, 16, 'R')],
        "offbeat" => (0..4).map(|b| (b * 4 + 2, 2, 'R')).collect(),
        "rolling" => (0..4).flat_map(|b| (1..4).map(move |k| (b * 4 + k, 1, 'R'))).map(|(p, l, t)| (p, l, if p == 7 && rng.chance(30) { '8' } else if p == 11 && rng.chance(25) { '5' } else { t })).collect(),
        "house" => vec![(2, 2, 'R'), (6, 1, '8'), (7, 1, 'R'), (10, 2, 'R'), (14, 2, '5')],
        "syncopated" => vec![(0, 2, 'R'), (3, 2, 'R'), (6, 2, '8'), (8, 2, 'R'), (11, 2, '5'), (14, 2, 'R')],
        "pulse" => (0..8).map(|e| (e * 2, 2, if e == 7 && rng.chance(40) { '5' } else { 'R' })).collect(),
        "melodic" => vec![(0, 4, 'R'), (4, 2, '3'), (6, 2, '5'), (8, 4, 'R'), (12, 2, '8'), (15, 1, 'A')],
        "rhythmic" => vec![(0, 1, 'R'), (2, 1, 'R'), (3, 1, '8'), (6, 2, 'R'), (8, 1, 'R'), (10, 1, '5'), (11, 1, 'R'), (14, 2, 'R')],
        "sequential" => vec![(0, 2, 'R'), (2, 2, '5'), (4, 2, '8'), (6, 2, '5'), (8, 2, 'R'), (10, 2, '5'), (12, 2, '8'), (14, 2, '5')],
        "hypnotic" => (0..16).filter(|s| s % 4 != 0).map(|s| (s, 1, if s == 14 && rng.chance(35) { '8' } else { 'R' })).collect(),
        "progressive" => {
            // evolves: off-beat sub at the start, 16ths later, octave jumps in the last quarter
            let stage = bar * 4 / bars.max(1);
            match stage {
                0 => (0..4).map(|b| (b * 4 + 2, 2, 'R')).collect(),
                1 => (0..4).flat_map(|b| [(b * 4 + 2, 1, 'R'), (b * 4 + 3, 1, 'R')]).collect(),
                2 => (0..4).flat_map(|b| (1..4).map(move |k| (b * 4 + k, 1, 'R'))).collect(),
                _ => (0..4).flat_map(|b| (1..4).map(move |k| (b * 4 + k, 1, if k == 3 { '8' } else { 'R' }))).collect(),
            }
        }
        _ => vec![(0, 16, 'R')],
    }
}

pub struct Opts {
    pub style: String,
    pub kick: Vec<u32>,
    pub avoid_kick: Option<bool>,
    pub seed: u64,
}

/// Four-on-the-floor kick positions for `bars`.
pub fn four_on_floor(bars: u32) -> Vec<u32> {
    (0..bars * 4).map(|b| b * PPQ).collect()
}

pub fn compose(key: Key, tl: &Timeline, bars: u32, o: &Opts) -> Result<(Vec<MidiNote>, Value), String> {
    if !STYLES.contains(&o.style.as_str()) {
        return Err(format!("unknown bass style {:?} (use {})", o.style, STYLES.join(", ")));
    }
    let mut rng = Rng::new(o.seed.wrapping_add(404));
    // styles that leave the kick alone by design (genres.md §1: rolling line with the first 16th of each beat rested)
    let avoid = o.avoid_kick.unwrap_or(matches!(o.style.as_str(), "offbeat" | "rolling" | "hypnotic" | "house" | "progressive"));
    let mut notes: Vec<(MidiNote, &'static str)> = vec![];
    for bar in 0..bars {
        for (p16, l16, tone) in pattern(&o.style, bar, bars, &mut rng) {
            let mut start = bar * BAR + p16 * S16;
            if avoid && o.kick.contains(&start) {
                start += S16; // move off the kick
                if start >= (bar + 1) * BAR {
                    continue;
                }
            }
            let slot = chord_at(tl, start);
            let pcs = slot.chord.pcs();
            let root = 28 + ((pcs[0] as i32 - 4).rem_euclid(12)) as u8; // E1..D#2
            let third_iv = if slot.chord.kind.intervals.contains(&3) && !slot.chord.kind.intervals.contains(&4) { 3 } else { 4 };
            let fifth_iv = slot.chord.kind.intervals.iter().find(|x| matches!(**x, 6..=8)).copied().unwrap_or(7);
            let (pitch, role) = match tone {
                '5' => (root + fifth_iv, "fifth"),
                '8' => (root + 12, "octave"),
                '3' => (root + third_iv, "third"),
                'A' => {
                    // approach the next chord's root by a scale step from below/above
                    let next = chord_at(tl, (bar + 1) * BAR);
                    let nr = 28 + ((next.chord.pcs()[0] as i32 - 4).rem_euclid(12)) as u8;
                    let scale = key.pcs();
                    let below = (1..=2).map(|d| nr.saturating_sub(d)).find(|p| scale.contains(&(p % 12))).unwrap_or(nr.saturating_sub(1));
                    (below, "approach")
                }
                _ => (root, "root"),
            };
            let end_of_bar = (bar + 1) * BAR;
            let dur = (l16 * S16).saturating_sub(if l16 >= 4 { 10 } else { 20 }).max(30).min(end_of_bar - start).min(slot.start + slot.len - start);
            let vel = if p16 % 4 == 2 { 108 } else if p16 % 4 == 0 { 104 } else { 92 };
            notes.push((MidiNote { pitch, start, dur, vel: rng.jitter(vel, 5) }, role));
        }
    }
    notes.sort_by_key(|n| n.0.start);
    for i in 1..notes.len() {
        let next = notes[i].0.start;
        let p = &mut notes[i - 1].0;
        if p.start + p.dur > next {
            p.dur = (next - p.start).saturating_sub(10).max(10);
        }
    }
    let collisions = notes.iter().filter(|n| o.kick.contains(&n.0.start)).count();
    let downbeats: Vec<&(MidiNote, &str)> = notes.iter().filter(|n| n.0.start % BAR == 0 || chord_at(tl, n.0.start).start == n.0.start).collect();
    let on_root = downbeats.iter().filter(|n| n.1 == "root").count();
    let mut roles = serde_json::Map::new();
    for (_, r) in &notes {
        let c = roles.get(*r).and_then(|x| x.as_u64()).unwrap_or(0) + 1;
        roles.insert((*r).into(), json!(c));
    }
    let chord_tone_share = notes.iter().filter(|n| chord_at(tl, n.0.start).chord.pcs().contains(&(n.0.pitch % 12))).count() as f64 / notes.len().max(1) as f64;
    let out: Vec<MidiNote> = notes.iter().map(|n| n.0).collect();
    Ok((out, json!({
        "style": o.style, "avoid_kick": avoid, "kick_collisions": collisions, "roles": roles,
        "roots_on_chord_changes": if downbeats.is_empty() { 1.0 } else { (on_root as f64 / downbeats.len() as f64 * 100.0).round() / 100.0 },
        "chord_tone_share": (chord_tone_share * 100.0).round() / 100.0,
        "source": "docs/knowledge/midi-intelligence/composition-guide.md §3",
    })))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::music::harmony::timeline;
    use crate::music::keys::parse_key;

    #[test]
    fn every_style_follows_the_harmony_and_respects_the_kick() {
        let k = parse_key("A minor").unwrap();
        let tl = timeline(k, &["Am".into(), "F".into(), "C".into(), "G".into()], 1.0, 8).unwrap();
        let kick = four_on_floor(8);
        for style in STYLES {
            for seed in 0..6 {
                let (n, a) = compose(k, &tl, 8, &Opts { style: style.into(), kick: kick.clone(), avoid_kick: None, seed }).unwrap();
                assert!(!n.is_empty(), "{style}");
                assert!(n.iter().all(|x| (28..=51).contains(&x.pitch)), "{style} register");
                assert!(n.windows(2).all(|w| w[0].start + w[0].dur <= w[1].start), "{style} monophonic");
                assert!(a["chord_tone_share"].as_f64().unwrap() >= 0.85, "{style} {a}");
                if a["avoid_kick"] == true {
                    assert_eq!(a["kick_collisions"], 0, "{style} leaves the kick alone");
                }
            }
        }
        // melodic bass approaches the next root
        let (_, a) = compose(k, &tl, 4, &Opts { style: "melodic".into(), kick: kick.clone(), avoid_kick: Some(false), seed: 1 }).unwrap();
        assert!(a["roles"]["approach"].as_u64().unwrap() >= 3);
        // progressive gets denser over time
        let (p, _) = compose(k, &tl, 8, &Opts { style: "progressive".into(), kick, avoid_kick: None, seed: 1 }).unwrap();
        let count = |b: u32| p.iter().filter(|n| n.start / BAR == b).count();
        assert!(count(7) > count(0));
        assert!(compose(k, &tl, 4, &Opts { style: "polka".into(), kick: vec![], avoid_kick: None, seed: 1 }).is_err());
    }
}
