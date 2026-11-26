//! Arpeggio Engine – arpeggios from the chord of each moment: up, down, up-down, down-up, alternate (converging),
//! broken (1-3-2-4), random (seeded), pedal (root alternating), chord (polyphonic hits); 1–3 octaves; rate from
//! 1/4 to 1/32 and triplets; gate; accents and velocity curves. Pitches change at chord changes, the rhythm stays.
//! Source: docs/knowledge/music-theory/melody.md §3, midi-intelligence/composition-guide.md §5, §7.
use super::generate::{MidiNote, Rng, BAR};
use super::harmony::{chord_at, Timeline};
use super::rhythm::grid_ticks;
use serde_json::{json, Value};

pub const PATTERNS: [&str; 9] = ["up", "down", "updown", "downup", "alternate", "broken", "random", "pedal", "chord"];
pub const CURVES: [&str; 4] = ["flat", "rise", "fall", "wave"];

pub struct Opts {
    pub pattern: String,
    pub rate: String,
    pub octaves: u32,
    pub gate: f64,
    pub low: u8,
    pub accent_every: u32,
    pub curve: String,
    pub seed: u64,
}

impl Default for Opts {
    fn default() -> Self {
        Opts { pattern: "up".into(), rate: "1/16".into(), octaves: 2, gate: 0.6, low: 57, accent_every: 4, curve: "flat".into(), seed: 1 }
    }
}

fn sequence(tones: &[u8], pattern: &str, rng: &mut Rng) -> Vec<Vec<u8>> {
    let up: Vec<u8> = tones.to_vec();
    let down: Vec<u8> = tones.iter().rev().copied().collect();
    let one = |v: Vec<u8>| v.into_iter().map(|p| vec![p]).collect::<Vec<_>>();
    match pattern {
        "down" => one(down),
        "updown" => one(up.iter().chain(down.iter().skip(1).take(down.len().saturating_sub(2))).copied().collect()),
        "downup" => one(down.iter().chain(up.iter().skip(1).take(up.len().saturating_sub(2))).copied().collect()),
        "alternate" => {
            let (mut i, mut j, mut v) = (0usize, up.len() - 1, vec![]);
            while i <= j {
                v.push(up[i]);
                if i != j {
                    v.push(up[j]);
                }
                i += 1;
                j = j.saturating_sub(1);
                if j == 0 && i > 0 {
                    break;
                }
            }
            one(v)
        }
        "broken" => one((0..up.len()).flat_map(|i| [up[i], up[(i + 2).min(up.len() - 1)]]).take(up.len() * 2).collect()),
        "random" => {
            let mut v = up.clone();
            for i in (1..v.len()).rev() {
                v.swap(i, rng.below(i as u32 + 1) as usize);
            }
            one(v)
        }
        "pedal" => one(up.iter().skip(1).flat_map(|p| [up[0], *p]).collect()),
        "chord" => vec![up.iter().take(4).copied().collect()],
        _ => one(up),
    }
}

pub fn compose(tl: &Timeline, bars: u32, o: &Opts) -> Result<(Vec<MidiNote>, Value), String> {
    if !PATTERNS.contains(&o.pattern.as_str()) {
        return Err(format!("unknown arpeggio pattern {:?} (use {})", o.pattern, PATTERNS.join(", ")));
    }
    let step = grid_ticks(&o.rate)?;
    let octaves = o.octaves.clamp(1, 3);
    let gate = o.gate.clamp(0.1, 1.0);
    let mut rng = Rng::new(o.seed.wrapping_add(505));
    let total = bars * BAR;
    let mut out = vec![];
    let mut t = 0u32;
    let mut i = 0usize;
    let mut last_slot = u32::MAX;
    let mut seq: Vec<Vec<u8>> = vec![];
    while t < total {
        let slot = chord_at(tl, t);
        if slot.start != last_slot || seq.is_empty() {
            // chord tones from the low register upward over the octaves
            let pcs = slot.chord.pcs();
            let mut tones: Vec<u8> = vec![];
            for oct in 0..octaves {
                for &pc in &pcs {
                    let base = o.low as i32 + ((pc as i32 - o.low as i32).rem_euclid(12)) + 12 * oct as i32;
                    if base <= 108 {
                        tones.push(base as u8);
                    }
                }
            }
            tones.sort();
            tones.dedup();
            seq = sequence(&tones, &o.pattern, &mut rng);
            last_slot = slot.start;
            i = 0;
        }
        let hit = &seq[i % seq.len()];
        let pos = (t / step) as usize;
        let mut vel: i32 = match o.curve.as_str() {
            "rise" => 70 + (40 * (t % BAR) / BAR) as i32,
            "fall" => 110 - (40 * (t % BAR) / BAR) as i32,
            "wave" => 90 + (((t % BAR) as f64 / BAR as f64 * std::f64::consts::TAU).sin() * 20.0) as i32,
            _ => 90,
        };
        if o.accent_every > 0 && pos % o.accent_every as usize == 0 {
            vel += 18;
        }
        let dur = ((step as f64 * gate) as u32).max(10).min(total - t);
        for &p in hit {
            out.push(MidiNote { pitch: p, start: t, dur, vel: vel.clamp(1, 127) as u8 });
        }
        t += step;
        i += 1;
    }
    Ok((out, json!({ "pattern": o.pattern, "rate": o.rate, "step_ticks": step, "octaves": octaves, "gate": gate, "curve": o.curve, "accent_every": o.accent_every, "source": "docs/knowledge/music-theory/melody.md §3" })))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::music::harmony::timeline;
    use crate::music::keys::parse_key;

    #[test]
    fn arpeggios_follow_the_chords() {
        let k = parse_key("A minor").unwrap();
        let tl = timeline(k, &["Am".into(), "F".into()], 1.0, 4).unwrap();
        for p in PATTERNS {
            for rate in ["1/8", "1/16", "1/16T", "1/32"] {
                let (n, _) = compose(&tl, 4, &Opts { pattern: p.into(), rate: rate.into(), ..Default::default() }).unwrap();
                assert!(!n.is_empty());
                assert!(n.iter().all(|x| chord_at(&tl, x.start).chord.pcs().contains(&(x.pitch % 12))), "{p} {rate}: chord tones only");
            }
        }
        let (up, _) = compose(&tl, 1, &Opts { pattern: "up".into(), rate: "1/16".into(), octaves: 1, ..Default::default() }).unwrap();
        assert_eq!(up.iter().take(4).map(|n| n.pitch).collect::<Vec<_>>(), [57, 60, 64, 57], "A3 C4 E4, again");
        let (dn, _) = compose(&tl, 1, &Opts { pattern: "down".into(), rate: "1/16".into(), octaves: 1, ..Default::default() }).unwrap();
        assert_eq!(dn[0].pitch, 64);
        let (tr, _) = compose(&tl, 1, &Opts { pattern: "up".into(), rate: "1/8T".into(), ..Default::default() }).unwrap();
        assert_eq!(tr.len(), 12, "eighth-note triplets: 12 per bar");
        let (ch, _) = compose(&tl, 1, &Opts { pattern: "chord".into(), rate: "1/4".into(), ..Default::default() }).unwrap();
        assert_eq!(ch.iter().filter(|n| n.start == 0).count(), 4, "polyphonic hits");
        let (ac, _) = compose(&tl, 1, &Opts { accent_every: 4, ..Default::default() }).unwrap();
        assert!(ac[0].vel > ac[1].vel, "accents");
        assert!(compose(&tl, 1, &Opts { rate: "1/7".into(), ..Default::default() }).is_err());
    }
}
