//! Deterministic MIDI generation (chords, bass, melody, drums) on top of `theory`.
//! No AI and no randomness outside the seed: the same input always gives the same notes.
//! Every generator has a matching check in `check()`, run in the tests and reported to the caller.
use super::theory::{Chord, Key};
use serde::Serialize;
use serde_json::{json, Map, Value};

pub const PPQ: u32 = 480;
pub const BAR: u32 = PPQ * 4; // 4/4 only in this version
const S16: u32 = PPQ / 4;

// General MIDI drum map (channel 10).
pub const KICK: u8 = 36;
pub const RIM: u8 = 37;
pub const SNARE: u8 = 38;
pub const CLAP: u8 = 39;
pub const CLOSED_HAT: u8 = 42;
pub const OPEN_HAT: u8 = 46;
pub const CRASH: u8 = 49;
pub const RIDE: u8 = 51;
pub const SHAKER: u8 = 70;

pub struct Genre {
    pub id: &'static str,
    pub label: &'static str,
    pub bpm: f64,
    pub key: &'static str,
    pub progression: &'static str,
    pub bass: &'static str,
    pub drums: &'static str,
    pub chord_rhythm: &'static str,
    pub swing: u32,
}

/// Starting points per genre. Everything can be overridden by the caller.
pub const GENRES: [Genre; 5] = [
    Genre { id: "techno", label: "Techno", bpm: 130.0, key: "F minor", progression: "i-i-VII-VI", bass: "offbeat", drums: "techno", chord_rhythm: "stabs", swing: 0 },
    Genre { id: "melodic_techno", label: "Melodic Techno", bpm: 124.0, key: "A minor", progression: "i-VI-III-VII", bass: "rolling", drums: "melodic_techno", chord_rhythm: "sustain", swing: 0 },
    Genre { id: "progressive_house", label: "Progressive House", bpm: 126.0, key: "G major", progression: "vi-IV-I-V", bass: "pulse", drums: "progressive_house", chord_rhythm: "sustain", swing: 0 },
    Genre { id: "house", label: "House", bpm: 124.0, key: "C major", progression: "Imaj7-vi7-ii7-V7", bass: "syncopated", drums: "house", chord_rhythm: "stabs", swing: 25 },
    Genre { id: "edm", label: "EDM", bpm: 128.0, key: "D major", progression: "I-V-vi-IV", bass: "pulse", drums: "edm", chord_rhythm: "sustain", swing: 0 },
];
pub const BASS_STYLES: [&str; 5] = ["offbeat", "rolling", "pulse", "syncopated", "sustain"];
pub const DRUM_STYLES: [&str; 5] = ["techno", "melodic_techno", "progressive_house", "house", "edm"];
pub const CHORD_RHYTHMS: [&str; 3] = ["sustain", "stabs", "pulse"];

pub fn genre(id: &str) -> Option<&'static Genre> {
    GENRES.iter().find(|g| g.id == id)
}

/// SplitMix64: tiny, fast, good enough for musical choices, identical on every OS.
pub struct Rng(u64);
impl Rng {
    pub fn new(seed: u64) -> Rng {
        Rng(seed ^ 0x9E37_79B9_7F4A_7C15)
    }
    pub fn next(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }
    pub fn below(&mut self, n: u32) -> u32 {
        (self.next() % n.max(1) as u64) as u32
    }
    pub fn chance(&mut self, pct: u32) -> bool {
        self.below(100) < pct
    }
    pub fn jitter(&mut self, v: u8, amt: u8) -> u8 {
        let d = self.below(2 * amt as u32 + 1) as i32 - amt as i32;
        (v as i32 + d).clamp(1, 127) as u8
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub struct MidiNote {
    pub pitch: u8,
    pub start: u32,
    pub dur: u32,
    pub vel: u8,
}

#[derive(Clone, Debug, Serialize)]
pub struct Part {
    pub id: &'static str,
    pub name: &'static str,
    /// 0-based MIDI channel (9 = drums).
    pub channel: u8,
    /// General MIDI program, 0-based.
    pub program: u8,
    pub notes: Vec<MidiNote>,
}

pub struct Spec<'a> {
    pub key: Key,
    pub chords: &'a [Chord],
    pub bars: u32,
    pub seed: u64,
    /// 0 = straight, 100 = full triplet feel on the off-16ths.
    pub swing: u32,
}

impl Spec<'_> {
    pub fn chord_at(&self, tick: u32) -> &Chord {
        &self.chords[(tick / BAR) as usize % self.chords.len()]
    }
    fn len(&self) -> u32 {
        self.bars * BAR
    }
}

fn swing(pos16: u32, amount: u32) -> u32 {
    if pos16 % 2 == 1 {
        amount.min(100) * (S16 / 3) / 100
    } else {
        0
    }
}

/// Monophonic parts: a note never runs into the next one.
fn trim_overlaps(notes: &mut [MidiNote]) {
    notes.sort_by_key(|n| (n.start, n.pitch));
    for i in 1..notes.len() {
        let (a, b) = notes.split_at_mut(i);
        let prev = &mut a[i - 1];
        if prev.start + prev.dur > b[0].start {
            prev.dur = (b[0].start - prev.start).saturating_sub(10).max(10);
        }
    }
}

fn pitches_in(pcs: &[u8], lo: u8, hi: u8) -> Vec<u8> {
    (lo..=hi).filter(|p| pcs.contains(&(p % 12))).collect()
}

fn nearest(pcs: &[u8], target: u8, lo: u8, hi: u8) -> u8 {
    *pitches_in(pcs, lo, hi).iter().min_by_key(|&&p| (p as i32 - target as i32).abs()).expect("pitch range spans an octave")
}

/// Scale for the melody over `chord`: key scale plus chord tones outside it (e.g. G# over E in
/// A minor), dropping the scale note a semitone away so no clash is left.
pub fn local_scale(key: Key, chord: &Chord) -> Vec<u8> {
    let tones = chord.pcs();
    let mut s = key.pcs();
    let added: Vec<u8> = tones.iter().copied().filter(|c| !s.contains(c)).collect();
    s.retain(|&x| tones.contains(&x) || !added.iter().any(|&c| (x + 1) % 12 == c || (c + 1) % 12 == x));
    s.extend(added);
    s.sort();
    s
}

fn movement(a: &[u8], b: &[u8]) -> i32 {
    if a.len() == b.len() {
        a.iter().zip(b).map(|(x, y)| (*x as i32 - *y as i32).abs()).sum()
    } else {
        b.iter().map(|y| a.iter().map(|x| (*x as i32 - *y as i32).abs()).min().unwrap_or(0)).sum()
    }
}

/// Close voicing of a chord (lowest note F3..E4) with the smallest movement from `prev`.
pub fn voice(pcs: &[u8], prev: Option<&[u8]>) -> Vec<u8> {
    let mut best: Option<(i32, Vec<u8>)> = None;
    for inv in 0..pcs.len() {
        let order: Vec<u8> = pcs[inv..].iter().chain(&pcs[..inv]).copied().collect();
        let mut v = vec![53 + ((order[0] as i32 - 53).rem_euclid(12)) as u8];
        for &pc in &order[1..] {
            let mut q = v[v.len() - 1] + 1;
            while q % 12 != pc {
                q += 1;
            }
            v.push(q);
        }
        let cost = match prev {
            Some(p) => movement(p, &v),
            None => inv as i32,
        };
        if best.as_ref().is_none_or(|b| cost < b.0) {
            best = Some((cost, v));
        }
    }
    best.map(|b| b.1).unwrap_or_default()
}

pub fn chords(spec: &Spec, rhythm: &str) -> Result<Part, String> {
    if !CHORD_RHYTHMS.contains(&rhythm) {
        return Err(format!("unknown chord rhythm {rhythm:?} (use {})", CHORD_RHYTHMS.join(", ")));
    }
    let mut rng = Rng::new(spec.seed.wrapping_add(1));
    let mut notes = Vec::new();
    let mut prev: Option<Vec<u8>> = None;
    for bar in 0..spec.bars {
        let v = voice(&spec.chord_at(bar * BAR).pcs(), prev.as_deref());
        let hits: Vec<(u32, u32, u8)> = match rhythm {
            "sustain" => vec![(0, BAR, 78)],
            "stabs" => (0..4).map(|b| (b * PPQ + PPQ / 2, S16, 100)).collect(),
            _ => (0..8).map(|e| (e * PPQ / 2, PPQ / 2 - 30, if e % 2 == 0 { 96 } else { 84 })).collect(),
        };
        for (s, d, vel) in hits {
            for &p in &v {
                notes.push(MidiNote { pitch: p, start: bar * BAR + s, dur: d, vel: rng.jitter(vel, 4) });
            }
        }
        prev = Some(v);
    }
    Ok(Part { id: "chords", name: "Chords", channel: 0, program: 89, notes })
}

pub fn bass(spec: &Spec, style: &str) -> Result<Part, String> {
    if !BASS_STYLES.contains(&style) {
        return Err(format!("unknown bass style {style:?} (use {})", BASS_STYLES.join(", ")));
    }
    let mut rng = Rng::new(spec.seed.wrapping_add(2));
    let mut notes = Vec::new();
    for bar in 0..spec.bars {
        let pcs = spec.chord_at(bar * BAR).pcs();
        let root = 28 + ((pcs[0] as i32 - 4).rem_euclid(12)) as u8; // E1..D#2
        let fifth = root + ((pcs[2] as i32 - pcs[0] as i32).rem_euclid(12)) as u8;
        let octave = root + 12;
        let steps: Vec<(u32, u32, u8)> = match style {
            "offbeat" => (0..4).map(|b| (b * 4 + 2, 2 * S16 - 20, root)).collect(),
            "rolling" => (0..4)
                .flat_map(|b| (1..4).map(move |k| (b * 4 + k, k)))
                .map(|(p, k)| {
                    let pitch = match k {
                        2 if rng.chance(25) => octave,
                        3 if rng.chance(20) => fifth,
                        _ => root,
                    };
                    (p, S16 - 10, pitch)
                })
                .collect(),
            "pulse" => (0..8).map(|e| (e * 2, 2 * S16 - 30, if e == 7 && rng.chance(40) { fifth } else { root })).collect(),
            "syncopated" => [(0, root), (3, root), (6, octave), (8, root), (11, fifth), (14, root)].iter().map(|&(p, n)| (p, 2 * S16 - 20, n)).collect(),
            _ => vec![(0, BAR - 10, root)],
        };
        for (p16, dur, pitch) in steps {
            let vel = if p16 % 4 == 0 || p16 % 4 == 2 { 104 } else { 94 };
            let start = bar * BAR + p16 * S16 + swing(p16, spec.swing);
            // a swung last 16th must not ring past its bar (or past the end of the sketch)
            notes.push(MidiNote { pitch, start, dur: dur.min((bar + 1) * BAR - start), vel: rng.jitter(vel, 5) });
        }
    }
    trim_overlaps(&mut notes);
    Ok(Part { id: "bass", name: "Bass", channel: 2, program: 38, notes })
}

/// One-bar rhythms for the melody: (start, length) in 16ths.
const MELODY_RHYTHMS: [&[(u32, u32)]; 5] = [
    &[(0, 2), (2, 2), (4, 4), (8, 2), (10, 2), (12, 4)],
    &[(0, 3), (3, 3), (6, 2), (8, 4), (12, 2), (14, 2)],
    &[(2, 2), (4, 2), (6, 2), (8, 6), (14, 2)],
    &[(0, 4), (4, 2), (6, 2), (8, 4), (12, 4)],
    &[(0, 2), (2, 2), (4, 2), (6, 2), (8, 2), (10, 2), (12, 2), (14, 2)],
];
const MEL_LO: u8 = 60;
const MEL_HI: u8 = 84;

pub fn melody(spec: &Spec) -> Part {
    let mut rng = Rng::new(spec.seed.wrapping_add(3));
    let tonic = spec.key.tonic.pc();
    let mut prev = nearest(&spec.chords[0].pcs(), 67, MEL_LO, MEL_HI);
    let mut notes = Vec::new();
    for bar in 0..spec.bars {
        let chord = spec.chord_at(bar * BAR);
        let tones = chord.pcs();
        let scale = pitches_in(&local_scale(spec.key, chord), MEL_LO, MEL_HI);
        let rhythm = MELODY_RHYTHMS[rng.below(MELODY_RHYTHMS.len() as u32) as usize];
        for (i, &(p16, l16)) in rhythm.iter().enumerate() {
            let last = bar + 1 == spec.bars && i + 1 == rhythm.len();
            let pitch = if last {
                // phrase end: tonic when it belongs to the chord, otherwise the nearest chord tone
                let goal = if tones.contains(&tonic) { vec![tonic] } else { tones.clone() };
                nearest(&goal, prev, MEL_LO, MEL_HI)
            } else if p16 % 4 == 0 {
                // strong beat: one of the two chord tones closest to the previous note
                let mut c = pitches_in(&tones, MEL_LO, MEL_HI);
                c.sort_by_key(|&p| ((p as i32 - prev as i32).abs(), p));
                c[rng.below(2) as usize]
            } else {
                // weak position: step (or skip) through the scale, drifting back to the middle
                let idx = (0..scale.len()).min_by_key(|&k| (scale[k] as i32 - prev as i32).abs()).unwrap_or(0) as i32;
                let size = if rng.chance(70) { 1 } else { 2 };
                let dir = if prev > 76 { -1 } else if prev < 64 { 1 } else if rng.chance(50) { 1 } else { -1 };
                scale[(idx + dir * size).clamp(0, scale.len() as i32 - 1) as usize]
            };
            let start = bar * BAR + p16 * S16;
            let dur = if last { (bar + 1) * BAR - start } else { l16 * S16 - 15 };
            notes.push(MidiNote { pitch, start, dur, vel: rng.jitter(if p16 % 4 == 0 { 100 } else { 88 }, 6) });
            prev = pitch;
        }
    }
    trim_overlaps(&mut notes);
    Part { id: "melody", name: "Melody", channel: 1, program: 81, notes }
}

pub fn drums(spec: &Spec, style: &str, fill: bool) -> Result<Part, String> {
    if !DRUM_STYLES.contains(&style) {
        return Err(format!("unknown drum style {style:?} (use {})", DRUM_STYLES.join(", ")));
    }
    let mut rng = Rng::new(spec.seed.wrapping_add(4));
    let mut hits: Vec<(u32, u8, u8)> = Vec::new(); // (16th from start, drum, velocity)
    for bar in 0..spec.bars {
        let in_fill_bar = fill && spec.bars > 1 && bar + 1 == spec.bars;
        for s in 0..16u32 {
            let at = bar * 16 + s;
            let (beat, off8) = (s % 4 == 0, s % 4 == 2);
            let fill_now = in_fill_bar && s >= 12;
            if beat {
                hits.push((at, KICK, 118));
            }
            if (s == 4 || s == 12) && !fill_now {
                hits.push((at, CLAP, 108));
                if style == "edm" {
                    hits.push((at, SNARE, 96));
                }
            }
            match style {
                "techno" => {
                    if off8 {
                        hits.push((at, OPEN_HAT, 92));
                    } else if !beat && rng.chance(85) {
                        hits.push((at, CLOSED_HAT, 66));
                    }
                    if s % 4 == 3 && rng.chance(35) {
                        hits.push((at, RIM, 78));
                    }
                }
                "melodic_techno" => {
                    if off8 {
                        hits.push((at, OPEN_HAT, 84));
                    } else {
                        hits.push((at, SHAKER, if beat { 64 } else { 52 }));
                    }
                    if (s == 7 || s == 15) && rng.chance(50) {
                        hits.push((at, RIM, 72));
                    }
                }
                "house" => {
                    if off8 {
                        hits.push((at, OPEN_HAT, 96));
                    }
                    if s % 2 == 1 {
                        hits.push((at, CLOSED_HAT, 62));
                    }
                    hits.push((at, SHAKER, 44));
                }
                "progressive_house" => {
                    if off8 {
                        hits.push((at, OPEN_HAT, 90));
                    } else if !beat {
                        hits.push((at, CLOSED_HAT, 64));
                    }
                    if beat && bar % 2 == 1 {
                        hits.push((at, RIDE, 66));
                    }
                }
                _ => {
                    if off8 {
                        hits.push((at, OPEN_HAT, 90));
                    } else if s % 2 == 1 {
                        hits.push((at, CLOSED_HAT, 60));
                    }
                }
            }
            if fill_now {
                hits.push((at, SNARE, 70 + (s - 12) as u8 * 14));
            }
        }
        if bar == 0 && matches!(style, "edm" | "progressive_house") {
            hits.push((0, CRASH, 100));
        }
    }
    let mut notes: Vec<MidiNote> = hits
        .into_iter()
        .map(|(at, drum, vel)| {
            let sw = if drum == KICK { 0 } else { swing(at % 16, spec.swing) };
            MidiNote { pitch: drum, start: at * S16 + sw, dur: S16 / 2, vel: rng.jitter(vel, if drum == KICK { 3 } else { 7 }) }
        })
        .collect();
    notes.sort_by_key(|n| (n.start, n.pitch));
    notes.dedup_by_key(|n| (n.start, n.pitch));
    Ok(Part { id: "drums", name: "Drums", channel: 9, program: 0, notes })
}

/// Musical and technical checks for a generated part. The result goes back to the caller as is:
/// a part is never reported as valid unless these checks actually pass.
pub fn check(part: &Part, spec: &Spec) -> Map<String, Value> {
    let mut m = Map::new();
    let n = &part.notes;
    m.insert("has_notes".into(), json!(!n.is_empty()));
    m.insert("within_length".into(), json!(n.iter().all(|x| x.start + x.dur <= spec.len() && x.dur > 0)));
    let mut sorted = n.clone();
    sorted.sort_by_key(|x| (x.pitch, x.start));
    m.insert("no_same_pitch_overlaps".into(), json!(sorted.windows(2).all(|w| w[0].pitch != w[1].pitch || w[0].start + w[0].dur <= w[1].start)));
    let tone = |x: &MidiNote| spec.chord_at(x.start).pcs().contains(&(x.pitch % 12));
    match part.id {
        "chords" => {
            m.insert("notes_are_chord_tones".into(), json!(n.iter().all(tone)));
            let complete = (0..spec.bars).all(|b| {
                let pcs: Vec<u8> = n.iter().filter(|x| x.start / BAR == b).map(|x| x.pitch % 12).collect();
                spec.chord_at(b * BAR).pcs().iter().all(|p| pcs.contains(p))
            });
            m.insert("every_bar_has_full_chord".into(), json!(complete));
        }
        "bass" => {
            m.insert("notes_are_chord_tones".into(), json!(n.iter().all(tone)));
            m.insert("monophonic".into(), json!(n.windows(2).all(|w| w[0].start + w[0].dur <= w[1].start)));
        }
        "melody" => {
            m.insert("notes_in_key_or_chord".into(), json!(n.iter().all(|x| local_scale(spec.key, spec.chord_at(x.start)).contains(&(x.pitch % 12)))));
            m.insert("strong_beats_on_chord_tones".into(), json!(n.iter().filter(|x| x.start % PPQ == 0).all(tone)));
            let leap = n.windows(2).map(|w| (w[0].pitch as i32 - w[1].pitch as i32).unsigned_abs()).max().unwrap_or(0);
            m.insert("leaps_within_octave".into(), json!(leap <= 12));
            m.insert("monophonic".into(), json!(n.windows(2).all(|w| w[0].start + w[0].dur <= w[1].start)));
            m.insert("in_range_c4_c6".into(), json!(n.iter().all(|x| (MEL_LO..=MEL_HI).contains(&x.pitch))));
        }
        "drums" => {
            let kicks: Vec<u32> = n.iter().filter(|x| x.pitch == KICK).map(|x| x.start).collect();
            m.insert("kick_on_every_beat".into(), json!((0..spec.bars * 4).all(|b| kicks.contains(&(b * PPQ)))));
            m.insert("general_midi_channel_10".into(), json!(part.channel == 9));
        }
        _ => {}
    }
    m
}

pub fn all_pass(c: &Map<String, Value>) -> bool {
    c.values().all(|v| v.as_bool() == Some(true))
}

#[cfg(test)]
mod tests {
    use super::super::theory::{progression, Key, Mode, Note};
    use super::*;

    fn all_parts(spec: &Spec, g: &Genre) -> Vec<Part> {
        vec![chords(spec, g.chord_rhythm).unwrap(), bass(spec, g.bass).unwrap(), melody(spec), drums(spec, g.drums, true).unwrap()]
    }

    #[test]
    fn every_genre_seed_and_key_passes_the_checks() {
        for g in &GENRES {
            for tonic in ["C", "F#", "Bb", "E", "Ab"] {
                let key = Key { tonic: Note::parse(tonic).unwrap(), mode: Key::parse(g.key).unwrap().mode };
                let ch = progression(key, g.progression, false).unwrap();
                for seed in 0..25 {
                    let spec = Spec { key, chords: &ch, bars: 8, seed, swing: g.swing };
                    for p in all_parts(&spec, g) {
                        let c = check(&p, &spec);
                        assert!(all_pass(&c), "{} {tonic} seed {seed} {}: {c:?}", g.id, p.id);
                    }
                }
            }
        }
    }

    #[test]
    fn every_mode_and_style_passes_the_checks() {
        for m in Mode::ALL {
            let key = Key { tonic: Note::parse("D").unwrap(), mode: m };
            let ch = progression(key, super::super::theory::default_progression(m), true).unwrap();
            for (i, seed) in (100..110).enumerate() {
                let spec = Spec { key, chords: &ch, bars: 4, seed, swing: (i * 11) as u32 };
                let parts = [
                    chords(&spec, CHORD_RHYTHMS[i % 3]).unwrap(),
                    bass(&spec, BASS_STYLES[i % 5]).unwrap(),
                    melody(&spec),
                    drums(&spec, DRUM_STYLES[i % 5], i % 2 == 0).unwrap(),
                ];
                for p in parts {
                    let c = check(&p, &spec);
                    assert!(all_pass(&c), "{} seed {seed} {}: {c:?}", m.id(), p.id);
                }
            }
        }
    }

    #[test]
    fn deterministic_per_seed_and_varied_across_seeds() {
        let key = Key::parse("A minor").unwrap();
        let ch = progression(key, "i-VI-III-VII", false).unwrap();
        let a = Spec { key, chords: &ch, bars: 8, seed: 7, swing: 0 };
        let b = Spec { key, chords: &ch, bars: 8, seed: 8, swing: 0 };
        assert_eq!(melody(&a).notes, melody(&a).notes);
        assert_ne!(melody(&a).notes, melody(&b).notes);
        assert_eq!(drums(&a, "techno", true).unwrap().notes, drums(&a, "techno", true).unwrap().notes);
    }

    #[test]
    fn voice_leading_is_smooth() {
        let key = Key::parse("A minor").unwrap();
        let ch = progression(key, "i-VI-III-VII-iv-V-i", false).unwrap();
        let mut prev: Option<Vec<u8>> = None;
        for c in &ch {
            let v = voice(&c.pcs(), prev.as_deref());
            assert_eq!(v.len(), 3);
            assert!(v.windows(2).all(|w| w[0] < w[1] && w[1] - w[0] <= 9), "close voicing {v:?}");
            if let Some(p) = &prev {
                let moves: Vec<i32> = p.iter().zip(&v).map(|(a, b)| (*a as i32 - *b as i32).abs()).collect();
                assert!(moves.iter().all(|&m| m <= 5), "{} moves {moves:?}", c.symbol());
            }
            prev = Some(v);
        }
        assert_eq!(voice(&ch[0].pcs(), None), vec![57, 60, 64]); // Am root position A3 C4 E4
    }

    #[test]
    fn styles_have_their_groove() {
        let key = Key::parse("F minor").unwrap();
        let ch = progression(key, "i", false).unwrap();
        let spec = Spec { key, chords: &ch, bars: 2, seed: 1, swing: 0 };
        let b = bass(&spec, "offbeat").unwrap();
        assert!(b.notes.iter().all(|n| n.start % PPQ == PPQ / 2), "techno bass sits on the off-beats");
        let d = drums(&spec, "house", false).unwrap();
        let claps: Vec<u32> = d.notes.iter().filter(|n| n.pitch == CLAP).map(|n| n.start).collect();
        assert_eq!(claps, vec![PPQ, 3 * PPQ, BAR + PPQ, BAR + 3 * PPQ]);
        let swung = Spec { swing: 100, ..spec };
        let hats: Vec<u32> = drums(&swung, "house", false).unwrap().notes.iter().filter(|n| n.pitch == CLOSED_HAT).map(|n| n.start % PPQ).collect();
        assert!(hats.iter().all(|&t| t == S16 + S16 / 3 || t == 3 * S16 + S16 / 3), "full swing = triplet feel {hats:?}");
        assert!(bass(&spec, "dubstep").is_err() && drums(&spec, "polka", false).is_err() && chords(&spec, "arp").is_err());
    }
}
