//! Music Theory Engine – notes, octaves, MIDI numbers, frequencies and intervals.
//! Conventions: scientific pitch notation (C4 = MIDI 60, A4 = MIDI 69 = 440 Hz, 12-tone equal temperament).
//! Italian solfège (Do Re Mi Fa Sol La Si) and English letters (C D E F G A B) are both accepted and returned.
//! Source: docs/knowledge/music-theory/notes-intervals.md.
use super::theory::{Note, LETTERS};
use serde_json::{json, Value};

pub const SOLFEGE: [&str; 7] = ["Do", "Re", "Mi", "Fa", "Sol", "La", "Si"];
const NATURAL: [i32; 7] = [0, 2, 4, 5, 7, 9, 11];

/// Parse a note name in English ("F#", "Bb") or Italian ("Fa#", "Fa diesis", "Si bemolle", "Sib", "Re♭").
pub fn parse_note(s: &str) -> Result<Note, String> {
    let t = s.trim();
    let lower = t.to_lowercase();
    for (i, syl) in SOLFEGE.iter().enumerate() {
        let sl = syl.to_lowercase();
        if let Some(rest) = lower.strip_prefix(&sl) {
            // "sol" must not be read as "so"+"l", and "si" must not eat a following word
            let rest = rest.trim();
            let acc: i8 = match rest {
                "" => 0,
                "#" | "♯" | "diesis" => 1,
                "b" | "♭" | "bemolle" | "bem" => -1,
                "##" | "x" | "doppio diesis" | "doppiodiesis" => 2,
                "bb" | "doppio bemolle" | "doppiobemolle" => -2,
                _ => continue,
            };
            return Ok(Note { letter: i as u8, acc });
        }
    }
    Note::parse(t)
}

pub fn italian(n: Note) -> String {
    let acc = match n.acc {
        -2 => "♭♭",
        -1 => "♭",
        1 => "♯",
        2 => "♯♯",
        _ => "",
    };
    format!("{}{acc}", SOLFEGE[n.letter as usize])
}

/// A note at an octave (scientific pitch notation).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Pitch {
    pub note: Note,
    pub octave: i32,
}

impl Pitch {
    /// MIDI note number; the octave counts by letter, so B#3 = C4 = 60 and Cb4 = B3 = 59.
    pub fn midi(self) -> i32 {
        (self.octave + 1) * 12 + NATURAL[self.note.letter as usize] + self.note.acc as i32
    }

    /// "C4", "F#3", "Do4", "Sib2" (octave digits at the end, negative allowed: "C-1").
    pub fn parse(s: &str) -> Result<Pitch, String> {
        let t = s.trim();
        let cut = t.find(|c: char| c.is_ascii_digit() || c == '-').ok_or_else(|| format!("missing octave in {s:?} (e.g. C4)"))?;
        let note = parse_note(&t[..cut])?;
        let octave: i32 = t[cut..].parse().map_err(|_| format!("invalid octave in {s:?}"))?;
        let p = Pitch { note, octave };
        if !(0..=127).contains(&p.midi()) {
            return Err(format!("{s} is outside the MIDI range 0–127"));
        }
        Ok(p)
    }

    /// Spelling of a MIDI number with sharps or flats.
    pub fn from_midi(m: i32, flats: bool) -> Pitch {
        let note = Note::simple(m.rem_euclid(12) as u8, flats);
        Pitch { note, octave: (m - NATURAL[note.letter as usize] - note.acc as i32) / 12 - 1 }
    }

    pub fn name(self) -> String {
        format!("{}{}", self.note.name(), self.octave)
    }

    pub fn italian(self) -> String {
        format!("{}{}", italian(self.note), self.octave)
    }

    pub fn frequency(self) -> f64 {
        frequency(self.midi() as f64)
    }
}

/// Frequency of a (possibly fractional) MIDI number, A4 = 440 Hz.
pub fn frequency(midi: f64) -> f64 {
    440.0 * 2f64.powf((midi - 69.0) / 12.0)
}

/// Fractional MIDI number of a frequency, the nearest MIDI note and the deviation in cents.
pub fn from_frequency(hz: f64) -> Option<(f64, i32, f64)> {
    if !(hz > 0.0 && hz.is_finite()) {
        return None;
    }
    let m = 69.0 + 12.0 * (hz / 440.0).log2();
    let nearest = m.round() as i32;
    Some((m, nearest, ((m - nearest as f64) * 100.0 * 10.0).round() / 10.0))
}

/// Interval between two spelled pitches: number by letters, quality by semitones.
#[derive(Clone, Debug, PartialEq)]
pub struct Interval {
    /// 1 = unison, 2 = second … 8 = octave, 9 = ninth …
    pub number: i32,
    /// P M m A d (AA / dd for doubly altered)
    pub quality: String,
    pub semitones: i32,
}

const IT_NUM: [&str; 15] = ["unisono", "seconda", "terza", "quarta", "quinta", "sesta", "settima", "ottava", "nona", "decima", "undicesima", "dodicesima", "tredicesima", "quattordicesima", "quindicesima"];
const EN_NUM: [&str; 15] = ["unison", "second", "third", "fourth", "fifth", "sixth", "seventh", "octave", "ninth", "tenth", "eleventh", "twelfth", "thirteenth", "fourteenth", "fifteenth"];

fn perfect_type(simple: i32) -> bool {
    matches!(simple, 1 | 4 | 5)
}

/// Semitones of the major/perfect interval for a number (1 → 0, 3 → 4, 8 → 12, 10 → 16).
fn reference(number: i32) -> i32 {
    let simple = (number - 1) % 7;
    [0, 2, 4, 5, 7, 9, 11][simple as usize] + 12 * ((number - 1) / 7)
}

impl Interval {
    pub fn between(a: Pitch, b: Pitch) -> Interval {
        let (lo, hi) = if b.midi() >= a.midi() { (a, b) } else { (b, a) };
        let steps = (hi.octave * 7 + hi.note.letter as i32) - (lo.octave * 7 + lo.note.letter as i32);
        let number = steps + 1;
        let semitones = hi.midi() - lo.midi();
        let diff = semitones - reference(number);
        let simple = (number - 1) % 7 + 1;
        let quality = if perfect_type(simple) {
            match diff {
                0 => "P".into(),
                d if d > 0 => "A".repeat(d as usize),
                d => "d".repeat((-d) as usize),
            }
        } else {
            match diff {
                0 => "M".into(),
                -1 => "m".into(),
                d if d > 0 => "A".repeat(d as usize),
                d => "d".repeat((-d - 1) as usize),
            }
        };
        Interval { number, quality, semitones }
    }

    /// "M3", "P5", "m7", "A4", "d5", "M9", "P8".
    pub fn parse(s: &str) -> Result<Interval, String> {
        let t = s.trim();
        let cut = t.find(|c: char| c.is_ascii_digit()).ok_or_else(|| format!("invalid interval {s:?} (e.g. M3, P5, m7, A4)"))?;
        let (q, n) = t.split_at(cut);
        let number: i32 = n.parse().map_err(|_| format!("invalid interval {s:?}"))?;
        if !(1..=15).contains(&number) {
            return Err(format!("interval number must be 1–15: {s:?}"));
        }
        let simple = (number - 1) % 7 + 1;
        let base = reference(number);
        let semitones = match q {
            "P" if perfect_type(simple) => base,
            "M" if !perfect_type(simple) => base,
            "m" if !perfect_type(simple) => base - 1,
            "A" => base + 1,
            "AA" => base + 2,
            "d" if perfect_type(simple) => base - 1,
            "d" => base - 2,
            _ => return Err(format!("invalid quality in {s:?}")),
        };
        if semitones < 0 {
            return Err(format!("invalid interval {s:?}"));
        }
        Ok(Interval { number, quality: q.to_string(), semitones })
    }

    pub fn short(&self) -> String {
        format!("{}{}", self.quality, self.number)
    }

    pub fn compound(&self) -> bool {
        self.number > 8
    }

    pub fn names(&self) -> (String, String) {
        let i = (self.number - 1).clamp(0, 14) as usize;
        let (qe, qi) = match self.quality.as_str() {
            "P" => ("perfect", if self.number == 1 { "" } else { "giusta" }),
            "M" => ("major", "maggiore"),
            "m" => ("minor", "minore"),
            "A" => ("augmented", if self.number == 1 { "aumentato" } else { "aumentata" }),
            "d" => ("diminished", if self.number == 1 { "diminuito" } else { "diminuita" }),
            q if q.starts_with('A') => ("doubly augmented", "più che aumentata"),
            _ => ("doubly diminished", "più che diminuita"),
        };
        let en = if self.number == 1 && self.quality == "P" { "unison".to_string() } else { format!("{qe} {}", EN_NUM[i]) };
        let it = if self.number == 1 && self.quality == "P" { "unisono".to_string() } else { format!("{} {qi}", IT_NUM[i]).trim().to_string() };
        (en, it)
    }

    pub fn view(&self) -> Value {
        let (en, it) = self.names();
        let tritone = self.semitones % 12 == 6 && !self.compound();
        json!({
            "short": self.short(), "name": en, "nome": it, "number": self.number, "quality": self.quality,
            "semitones": self.semitones, "tones": self.semitones as f64 / 2.0, "compound": self.compound(),
            "simple": if self.compound() { json!(format!("{}{}", self.quality, (self.number - 1) % 7 + 1)) } else { Value::Null },
            "tritone": tritone,
            "consonance": consonance(self.semitones),
        })
    }
}

/// Spell the note a given interval above `n` (letters first, then semitones).
pub fn above(n: Note, iv: &Interval) -> Note {
    n.up(((iv.number - 1) % 7) as u8, (iv.semitones % 12) as u8)
}

/// Traditional classification by semitones (simple interval).
pub fn consonance(semitones: i32) -> &'static str {
    match semitones.rem_euclid(12) {
        0 | 7 => "perfect_consonance",
        3 | 4 | 8 | 9 => "imperfect_consonance",
        5 => "perfect_fourth (consonant melodically, dissonant over the bass in common practice)",
        _ => "dissonance",
    }
}

/// Everything about a note or pitch, for the API.
pub fn note_info(name: &str, flats: Option<bool>) -> Result<Value, String> {
    let t = name.trim();
    if let Ok(m) = t.parse::<i32>() {
        if !(0..=127).contains(&m) {
            return Err("MIDI note numbers are 0–127".into());
        }
        let p = Pitch::from_midi(m, flats.unwrap_or(false));
        let alt = Pitch::from_midi(m, !flats.unwrap_or(false));
        return Ok(pitch_view(p, Some(alt)));
    }
    if t.chars().any(|c| c.is_ascii_digit()) {
        let p = Pitch::parse(t)?;
        return Ok(pitch_view(p, None));
    }
    let n = parse_note(t)?;
    Ok(json!({
        "note": n.name(), "italian": italian(n), "pitch_class": n.pc(), "letter": LETTERS[n.letter as usize].to_string(),
        "accidentals": n.acc, "enharmonics": enharmonics(n).iter().map(|e| json!({ "note": e.name(), "italian": italian(*e) })).collect::<Vec<_>>(),
        "octaves": (0..=8).map(|o| { let p = Pitch { note: n, octave: o }; json!({ "pitch": p.name(), "midi": p.midi(), "hz": (p.frequency() * 100.0).round() / 100.0 }) }).filter(|v| (0..=127).contains(&v["midi"].as_i64().unwrap_or(-1))).collect::<Vec<_>>(),
    }))
}

fn pitch_view(p: Pitch, alt: Option<Pitch>) -> Value {
    let mut enh: Vec<Value> = enharmonics(p.note)
        .iter()
        .map(|e| {
            // the octave that gives the same MIDI number with this spelling (B#3 = C4)
            let e = Pitch { note: *e, octave: (p.midi() - NATURAL[e.letter as usize] - e.acc as i32).div_euclid(12) - 1 };
            json!({ "pitch": e.name(), "italian": e.italian() })
        })
        .collect();
    if let Some(a) = alt {
        if a != p && !enh.iter().any(|x| x["pitch"] == a.name()) {
            enh.push(json!({ "pitch": a.name(), "italian": a.italian() }));
        }
    }
    json!({ "pitch": p.name(), "italian": p.italian(), "midi": p.midi(), "pitch_class": p.note.pc(), "octave": p.octave,
            "hz": (p.frequency() * 100.0).round() / 100.0, "enharmonics": enh })
}

/// Other spellings of the same pitch class with at most two accidentals.
pub fn enharmonics(n: Note) -> Vec<Note> {
    (0..7u8).filter(|&l| l != n.letter).filter_map(|l| Note::spell(l, n.pc())).collect()
}

/// Interval request: two notes ("C4", "E4") or a note and an interval ("D", "m7").
pub fn interval_info(a: &str, b: &str) -> Result<Value, String> {
    if let Ok(iv) = Interval::parse(b) {
        let root = parse_note(a.trim_end_matches(|c: char| c.is_ascii_digit() || c == '-'))?;
        let top = above(root, &iv);
        let mut v = iv.view();
        v["from"] = json!(root.name());
        v["to"] = json!(top.name());
        v["to_italian"] = json!(italian(top));
        return Ok(v);
    }
    let pa = Pitch::parse(a).or_else(|_| parse_note(a).map(|n| Pitch { note: n, octave: 4 }))?;
    let mut pb = Pitch::parse(b).or_else(|_| parse_note(b).map(|n| Pitch { note: n, octave: 4 }))?;
    if !b.chars().any(|c| c.is_ascii_digit()) && pb.midi() < pa.midi() {
        pb.octave += 1; // two plain note names: the second is taken above the first
    }
    let iv = Interval::between(pa, pb);
    let mut v = iv.view();
    v["from"] = json!(pa.name());
    v["to"] = json!(pb.name());
    Ok(v)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn italian_and_english_names_octaves_midi_frequency() {
        assert_eq!(parse_note("Fa diesis").unwrap().name(), "F#");
        assert_eq!(parse_note("Sib").unwrap().name(), "Bb");
        assert_eq!(parse_note("sol").unwrap().name(), "G");
        assert_eq!(parse_note("La").unwrap().name(), "A");
        assert_eq!(parse_note("Re bemolle").unwrap().name(), "Db");
        assert_eq!(italian(Note::parse("C#").unwrap()), "Do♯");
        assert_eq!(Pitch::parse("C4").unwrap().midi(), 60);
        assert_eq!(Pitch::parse("A4").unwrap().midi(), 69);
        assert_eq!(Pitch::parse("B#3").unwrap().midi(), 60, "octave counts by letter");
        assert_eq!(Pitch::parse("Cb4").unwrap().midi(), 59);
        assert_eq!(Pitch::parse("La3").unwrap().midi(), 57);
        assert_eq!(Pitch::from_midi(61, true).name(), "Db4");
        assert_eq!(Pitch::from_midi(0, false).name(), "C-1");
        assert!((frequency(69.0) - 440.0).abs() < 1e-9);
        assert!((Pitch::parse("C4").unwrap().frequency() - 261.6256).abs() < 1e-3);
        let (_, n, cents) = from_frequency(446.0).unwrap();
        assert_eq!((n, cents), (69, 23.4));
        assert!(Pitch::parse("C10").is_err() && Pitch::parse("H4").is_err());
        let e: Vec<String> = enharmonics(Note::parse("C#").unwrap()).iter().map(|n| n.name()).collect();
        assert_eq!(e, ["Db", "B##"]);
    }

    #[test]
    fn intervals_by_letter_and_semitones() {
        let iv = |a: &str, b: &str| Interval::between(Pitch::parse(a).unwrap(), Pitch::parse(b).unwrap()).short();
        assert_eq!(iv("C4", "C4"), "P1");
        assert_eq!(iv("C4", "Db4"), "m2");
        assert_eq!(iv("C4", "C#4"), "A1");
        assert_eq!(iv("C4", "E4"), "M3");
        assert_eq!(iv("C4", "Eb4"), "m3");
        assert_eq!(iv("C4", "F#4"), "A4");
        assert_eq!(iv("C4", "Gb4"), "d5");
        assert_eq!(iv("C4", "G4"), "P5");
        assert_eq!(iv("C4", "Ab4"), "m6");
        assert_eq!(iv("C4", "Bb4"), "m7");
        assert_eq!(iv("C4", "B4"), "M7");
        assert_eq!(iv("C4", "C5"), "P8");
        assert_eq!(iv("C4", "D5"), "M9");
        assert_eq!(iv("C4", "F5"), "P11");
        assert_eq!(iv("C4", "A5"), "M13");
        assert_eq!(iv("E4", "C4"), "M3", "order does not matter: the lower note is C4");
        assert_eq!(iv("E4", "C5"), "m6");
        assert_eq!(iv("B3", "F4"), "d5");
        let m9 = Interval::parse("M9").unwrap();
        assert_eq!((m9.semitones, m9.compound()), (14, true));
        assert_eq!(Interval::parse("m3").unwrap().names().1, "terza minore");
        assert_eq!(Interval::parse("P5").unwrap().names().1, "quinta giusta");
        assert_eq!(Interval::parse("A4").unwrap().view()["tritone"], true);
        assert_eq!(above(Note::parse("D").unwrap(), &Interval::parse("m7").unwrap()).name(), "C");
        assert_eq!(above(Note::parse("F#").unwrap(), &Interval::parse("M3").unwrap()).name(), "A#");
        assert!(Interval::parse("P3").is_err() && Interval::parse("M5").is_err());
        for s in ["P1", "m2", "M2", "m3", "M3", "P4", "A4", "d5", "P5", "m6", "M6", "m7", "M7", "P8", "m9", "M9", "P11", "A11", "M13"] {
            let i = Interval::parse(s).unwrap();
            let top = Pitch { note: above(Note::parse("C").unwrap(), &i), octave: 4 + (i.number - 1) / 7 };
            let top = if top.midi() - 60 < i.semitones { Pitch { octave: top.octave + 1, ..top } } else { top };
            assert_eq!(Interval::between(Pitch::parse("C4").unwrap(), top).short(), s, "round trip {s}");
        }
    }
}
