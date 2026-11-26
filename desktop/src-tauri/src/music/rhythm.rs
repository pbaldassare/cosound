//! Rhythm & Groove Engine. Keeps three things apart: the metric GRID (subdivision of the beat), the POSITION of an
//! event (ticks, and bar.beat.sixteenth), and its DURATION (ticks and note value). 4/4, 480 PPQ.
//! Patterns: step strings ("x..x" per grid step), Euclidean rhythms, polyrhythms, swing, accents, syncopation.
//! Sources: docs/knowledge/music-production/README.md §2 (16-step grid), midi-intelligence/composition-guide.md §6–10.
use super::generate::{MidiNote, Rng, BAR, PPQ};
use serde_json::{json, Value};

/// Ticks per step for a subdivision name.
pub fn grid_ticks(sub: &str) -> Result<u32, String> {
    Ok(match sub {
        "1/1" => BAR,
        "1/2" => PPQ * 2,
        "1/4" => PPQ,
        "1/8" => PPQ / 2,
        "1/16" => PPQ / 4,
        "1/32" => PPQ / 8,
        "1/4T" => PPQ * 2 / 3,
        "1/8T" => PPQ / 3,
        "1/16T" => PPQ / 6,
        "1/32T" => PPQ / 12,
        "1/8." => PPQ * 3 / 4,
        "1/16." => PPQ * 3 / 8,
        _ => return Err(format!("unknown subdivision {sub:?} (use 1/4, 1/8, 1/16, 1/32, 1/8T, 1/16T, 1/8. …)")),
    })
}

/// Note-value name of a duration, when it is a standard one (dotted and triplets included).
pub fn value_name(ticks: u32) -> Option<&'static str> {
    [("1/1", BAR), ("1/2", PPQ * 2), ("1/4.", PPQ * 3 / 2), ("1/4", PPQ), ("1/8.", PPQ * 3 / 4), ("1/4T", PPQ * 2 / 3), ("1/8", PPQ / 2), ("1/16.", PPQ * 3 / 8), ("1/8T", PPQ / 3), ("1/16", PPQ / 4), ("1/16T", PPQ / 6), ("1/32", PPQ / 8)]
        .iter()
        .find(|(_, t)| *t == ticks)
        .map(|(n, _)| *n)
}

/// "bar.beat.sixteenth" (1-based) plus the remaining ticks, e.g. 1.2.3 +0.
pub fn position(tick: u32) -> String {
    let bar = tick / BAR + 1;
    let beat = (tick % BAR) / PPQ + 1;
    let six = (tick % PPQ) / (PPQ / 4) + 1;
    let rest = tick % (PPQ / 4);
    if rest == 0 { format!("{bar}.{beat}.{six}") } else { format!("{bar}.{beat}.{six}+{rest}") }
}

/// Parse a step pattern: x = hit, X = accent, . or - = rest, | ignored.
pub fn steps(pattern: &str) -> Vec<Option<u8>> {
    pattern.chars().filter(|c| !c.is_whitespace() && *c != '|').map(|c| match c {
        'X' => Some(118),
        'x' | 'o' => Some(96),
        _ => None,
    }).collect()
}

/// Euclidean rhythm E(k, n): k hits spread as evenly as possible over n steps (Bjorklund), rotated.
pub fn euclid(k: usize, n: usize, rotate: usize) -> Vec<bool> {
    if n == 0 {
        return vec![];
    }
    let k = k.min(n);
    // hit when (i·k) mod n < k: the same pattern as Bjorklund's algorithm, starting on a hit (E(3,8) = x..x..x.)
    let mut v: Vec<bool> = (0..n).map(|i| (i * k) % n < k).collect();
    let r = rotate % n;
    v.rotate_left(r);
    v
}

/// Notes from a step pattern on a grid, repeated for `bars`; velocity from the pattern (accents), duration = gate.
pub fn from_steps(pattern: &[Option<u8>], pitch: u8, grid: u32, gate: f64, bars: u32) -> Vec<MidiNote> {
    let len = bars * BAR;
    let mut out = vec![];
    let mut i = 0usize;
    let mut t = 0u32;
    while t < len && !pattern.is_empty() {
        if let Some(v) = pattern[i % pattern.len()] {
            out.push(MidiNote { pitch, start: t, dur: ((grid as f64 * gate) as u32).max(10).min(len - t), vel: v });
        }
        t += grid;
        i += 1;
    }
    out
}

/// Polyrhythm: `a` against `b` evenly over one bar, repeated (e.g. 3 against 4).
pub fn polyrhythm(a: u32, b: u32, bars: u32, pitch_a: u8, pitch_b: u8) -> (Vec<MidiNote>, Vec<MidiNote>) {
    let line = |n: u32, p: u8| -> Vec<MidiNote> {
        (0..bars).flat_map(|bar| (0..n).map(move |i| MidiNote { pitch: p, start: bar * BAR + i * BAR / n, dur: (BAR / n / 2).max(10), vel: if i == 0 { 112 } else { 92 } })).collect()
    };
    (line(a, pitch_a), line(b, pitch_b))
}

/// Swing: off-beat positions of the grid are delayed toward the triplet position (0–100 %).
pub fn swing(notes: &mut [MidiNote], grid: u32, amount: u32) {
    let shift = grid / 3 * amount.min(100) / 100;
    for n in notes.iter_mut() {
        if n.start % grid == 0 && (n.start / grid) % 2 == 1 {
            n.start += shift;
        }
    }
}

/// Syncopation: share of onsets on weak positions (off the beat) and of notes held across a beat.
pub fn syncopation(notes: &[MidiNote]) -> f64 {
    if notes.is_empty() {
        return 0.0;
    }
    let weak = notes.iter().filter(|n| n.start % PPQ != 0).count() as f64;
    let held = notes.iter().filter(|n| n.start % PPQ != 0 && (n.start % PPQ) + n.dur > PPQ).count() as f64;
    ((weak + held) / (2.0 * notes.len() as f64) * 100.0).round() / 100.0
}

/// Quantize starts (and optionally ends) to a grid with a strength (100 = hard).
pub fn quantize(notes: &mut [MidiNote], grid: u32, strength: u32, ends: bool) {
    let s = strength.min(100) as i64;
    for n in notes.iter_mut() {
        let snap = |t: u32| ((t + grid / 2) / grid) * grid;
        let target = snap(n.start) as i64;
        let new_start = n.start as i64 + (target - n.start as i64) * s / 100;
        let end = (n.start + n.dur) as i64;
        n.start = new_start.max(0) as u32;
        let new_end = if ends { end + (snap(end as u32) as i64 - end) * s / 100 } else { end };
        n.dur = (new_end - n.start as i64).max(10) as u32;
    }
}

/// Humanize: small seeded timing (ticks) and velocity deviations. The same seed gives the same result.
pub fn humanize(notes: &mut [MidiNote], timing: u32, velocity: u8, seed: u64) {
    let mut rng = Rng::new(seed.wrapping_add(55));
    for n in notes.iter_mut() {
        let dt = rng.below(2 * timing + 1) as i64 - timing as i64;
        n.start = (n.start as i64 + dt).max(0) as u32;
        n.vel = rng.jitter(n.vel, velocity);
    }
}

/// Genre groove patterns (16 steps; docs/knowledge/music-production/genres.md grids).
pub const GROOVES: [(&str, &str, &str, &str); 6] = [
    ("techno", "X...X...X...X...", ".xXx.xXx.xXx.xXx", "..O...O...O...O."),
    ("melodic_techno", "X...X...X...X...", "XXXXXXXXXXXXXXXX", "...xxx....x.xxx."),
    ("house", "X...X...X...X...", "..x...x...x...x.", "....X.......X..."),
    ("deep_house", "X...X...X...X...", "..x...x...x...x.", "....X.......X..."),
    ("tech_house", "X...X...X...X...", "..x...x...x...x.", "x.x.x.x.x.x.x.x."),
    ("trance", "X...X...X...X...", "..x...x...x...x.", "x.xxx.xxx.xxx.xx"),
];

/// A rhythm pattern for the API: from steps, Euclid or a polyrhythm, with grid/position/duration made explicit.
pub fn pattern(input: &Value) -> Result<Value, String> {
    let grid_name = input["grid"].as_str().unwrap_or("1/16");
    let grid = grid_ticks(grid_name)?;
    let bars = input["bars"].as_u64().unwrap_or(1).clamp(1, 16) as u32;
    let pitch = input["pitch"].as_u64().unwrap_or(36).min(127) as u8;
    let gate = input["gate"].as_f64().unwrap_or(0.5).clamp(0.05, 1.0);
    let (mut notes, kind) = if let (Some(a), Some(b)) = (input["poly"][0].as_u64(), input["poly"][1].as_u64()) {
        let (x, y) = polyrhythm(a as u32, b as u32, bars, pitch, pitch + 2);
        (x.into_iter().chain(y).collect::<Vec<_>>(), format!("polyrhythm {a}:{b}"))
    } else if let Some(k) = input["euclid"][0].as_u64() {
        let n = input["euclid"][1].as_u64().unwrap_or(16) as usize;
        let rot = input["euclid"][2].as_u64().unwrap_or(0) as usize;
        let p: Vec<Option<u8>> = euclid(k as usize, n, rot).into_iter().enumerate().map(|(i, h)| h.then_some(if i == 0 { 118 } else { 96 })).collect();
        (from_steps(&p, pitch, grid, gate, bars), format!("euclid E({k},{n}) rot {rot}"))
    } else {
        let s = input["steps"].as_str().unwrap_or("x.x.x.x.x.x.x.x.");
        (from_steps(&steps(s), pitch, grid, gate, bars), "steps".to_string())
    };
    if let Some(sw) = input["swing"].as_u64() {
        swing(&mut notes, grid, sw as u32);
    }
    notes.sort_by_key(|n| (n.start, n.pitch));
    Ok(json!({
        "kind": kind, "grid": { "subdivision": grid_name, "ticks": grid, "steps_per_bar": BAR / grid, "triplet": grid_name.ends_with('T') },
        "events": notes.iter().map(|n| json!({ "pitch": n.pitch, "position": position(n.start), "start_ticks": n.start, "duration_ticks": n.dur, "duration_value": value_name(n.dur), "velocity": n.vel })).collect::<Vec<_>>(),
        "syncopation": syncopation(&notes), "bars": bars, "ppq": PPQ,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn grid_position_duration_are_distinct() {
        assert_eq!(grid_ticks("1/16").unwrap(), 120);
        assert_eq!(grid_ticks("1/8T").unwrap(), 160);
        assert_eq!(grid_ticks("1/32").unwrap(), 60);
        assert!(grid_ticks("1/7").is_err());
        assert_eq!(position(0), "1.1.1");
        assert_eq!(position(PPQ + PPQ / 2), "1.2.3");
        assert_eq!(position(BAR + 10), "2.1.1+10");
        assert_eq!(value_name(PPQ * 3 / 2), Some("1/4."));
        assert_eq!(value_name(PPQ / 3), Some("1/8T"));
    }

    #[test]
    fn euclid_polyrhythm_swing_quantize_humanize() {
        let e: String = euclid(3, 8, 0).iter().map(|h| if *h { 'x' } else { '.' }).collect();
        assert_eq!(e, "x..x..x.");
        assert_eq!(euclid(4, 16, 0).iter().filter(|h| **h).count(), 4);
        let five: Vec<usize> = euclid(5, 16, 0).iter().enumerate().filter(|x| *x.1).map(|x| x.0).collect();
        assert!(five.windows(2).all(|w| (3..=4).contains(&(w[1] - w[0]))), "evenly spread: {five:?}");
        let (a, b) = polyrhythm(3, 4, 1, 60, 62);
        assert_eq!((a.len(), b.len()), (3, 4));
        assert_eq!(a[1].start, 640);
        let mut n = from_steps(&steps("xxxxxxxx"), 60, PPQ / 2, 0.5, 1);
        swing(&mut n, PPQ / 2, 100);
        assert_eq!((n[0].start, n[1].start), (0, 240 + 80), "off-8ths delayed to the triplet position");
        let mut q = vec![MidiNote { pitch: 60, start: 130, dur: 100, vel: 90 }];
        quantize(&mut q, 120, 100, false);
        assert_eq!(q[0].start, 120);
        let mut h = from_steps(&steps("x.x.x.x."), 60, 120, 0.5, 1);
        let mut h2 = h.clone();
        humanize(&mut h, 10, 8, 3);
        humanize(&mut h2, 10, 8, 3);
        assert_eq!(h, h2, "reproducible with a seed");
        assert!(syncopation(&from_steps(&steps(".x.x.x.x"), 60, PPQ / 2, 0.5, 1)) > syncopation(&from_steps(&steps("x.x.x.x."), 60, PPQ / 2, 0.5, 1)));
        let p = pattern(&json!({ "euclid": [3, 8], "grid": "1/8", "bars": 2 })).unwrap();
        assert_eq!(p["events"].as_array().unwrap().len(), 6);
        assert_eq!(p["events"][0]["position"], "1.1.1");
    }
}
