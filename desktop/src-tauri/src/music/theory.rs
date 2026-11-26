//! Music theory (Prompt 05 Part B, milestone M2). Pure functions, no I/O, no AI.
//! Spelling follows letter names: every scale degree takes the next letter, so F# major is
//! spelled with E# and Eb minor with Cb, as in any theory textbook.
use serde_json::{json, Value};

pub const LETTERS: [char; 7] = ['C', 'D', 'E', 'F', 'G', 'A', 'B'];
const NATURAL: [i32; 7] = [0, 2, 4, 5, 7, 9, 11];
const ROMAN: [&str; 7] = ["I", "II", "III", "IV", "V", "VI", "VII"];

/// A spelled note: letter (0 = C … 6 = B) plus accidentals (-2 = double flat … +2 = double sharp).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Note {
    pub letter: u8,
    pub acc: i8,
}

impl Note {
    pub fn parse(s: &str) -> Result<Note, String> {
        let s = s.trim();
        let bad = || format!("invalid note name: {s:?}");
        let mut ch = s.chars();
        let l = ch.next().ok_or_else(bad)?.to_ascii_uppercase();
        let letter = LETTERS.iter().position(|&c| c == l).ok_or_else(bad)? as u8;
        let mut acc = 0i8;
        for c in ch {
            match c {
                '#' | '♯' => acc += 1,
                'b' | '♭' => acc -= 1,
                'x' | '𝄪' => acc += 2,
                _ => return Err(bad()),
            }
        }
        if acc.abs() > 2 {
            return Err(bad());
        }
        Ok(Note { letter, acc })
    }

    /// Pitch class 0 (C) … 11 (B).
    pub fn pc(self) -> u8 {
        (NATURAL[self.letter as usize] + self.acc as i32).rem_euclid(12) as u8
    }

    pub fn name(self) -> String {
        let a = match self.acc {
            -2 => "bb",
            -1 => "b",
            0 => "",
            1 => "#",
            _ => "##",
        };
        format!("{}{a}", LETTERS[self.letter as usize])
    }

    /// Spell pitch class `pc` on `letter`; None when that would need more than two accidentals.
    pub fn spell(letter: u8, pc: u8) -> Option<Note> {
        let d = (pc as i32 - NATURAL[letter as usize]).rem_euclid(12);
        let acc = if d > 6 { d - 12 } else { d };
        (acc.abs() <= 2).then_some(Note { letter, acc: acc as i8 })
    }

    /// Plain spelling of a pitch class with sharps or flats.
    pub fn simple(pc: u8, flats: bool) -> Note {
        const SH: [(u8, i8); 12] = [(0, 0), (0, 1), (1, 0), (1, 1), (2, 0), (3, 0), (3, 1), (4, 0), (4, 1), (5, 0), (5, 1), (6, 0)];
        const FL: [(u8, i8); 12] = [(0, 0), (1, -1), (1, 0), (2, -1), (2, 0), (3, 0), (4, -1), (4, 0), (5, -1), (5, 0), (6, -1), (6, 0)];
        let (letter, acc) = if flats { FL[pc as usize % 12] } else { SH[pc as usize % 12] };
        Note { letter, acc }
    }

    /// The note `letters` letter-steps and `semis` semitones above this one.
    pub fn up(self, letters: u8, semis: u8) -> Note {
        let pc = (self.pc() + semis) % 12;
        Note::spell((self.letter + letters) % 7, pc).unwrap_or_else(|| Note::simple(pc, self.acc < 0))
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Mode {
    Major,
    NaturalMinor,
    HarmonicMinor,
    MelodicMinor,
    Dorian,
    Phrygian,
    Lydian,
    Mixolydian,
    Locrian,
    MajorPentatonic,
    MinorPentatonic,
    Blues,
}

use Mode::*;

impl Mode {
    pub const ALL: [Mode; 12] = [Major, NaturalMinor, HarmonicMinor, MelodicMinor, Dorian, Phrygian, Lydian, Mixolydian, Locrian, MajorPentatonic, MinorPentatonic, Blues];

    pub fn id(self) -> &'static str {
        match self {
            Major => "major",
            NaturalMinor => "natural_minor",
            HarmonicMinor => "harmonic_minor",
            MelodicMinor => "melodic_minor",
            Dorian => "dorian",
            Phrygian => "phrygian",
            Lydian => "lydian",
            Mixolydian => "mixolydian",
            Locrian => "locrian",
            MajorPentatonic => "major_pentatonic",
            MinorPentatonic => "minor_pentatonic",
            Blues => "blues",
        }
    }

    pub fn parse(s: &str) -> Result<Mode, String> {
        let k = s.trim().to_lowercase().replace([' ', '-'], "_");
        Ok(match k.as_str() {
            "" | "major" | "maj" | "ionian" => Major,
            "minor" | "min" | "m" | "natural_minor" | "aeolian" => NaturalMinor,
            "harmonic_minor" => HarmonicMinor,
            "melodic_minor" => MelodicMinor,
            "dorian" => Dorian,
            "phrygian" => Phrygian,
            "lydian" => Lydian,
            "mixolydian" => Mixolydian,
            "locrian" => Locrian,
            "major_pentatonic" | "pentatonic" => MajorPentatonic,
            "minor_pentatonic" => MinorPentatonic,
            "blues" | "minor_blues" => Blues,
            _ => return Err(format!("unknown mode: {s:?}")),
        })
    }

    /// Semitones above the tonic.
    pub fn intervals(self) -> &'static [u8] {
        match self {
            Major => &[0, 2, 4, 5, 7, 9, 11],
            NaturalMinor => &[0, 2, 3, 5, 7, 8, 10],
            HarmonicMinor => &[0, 2, 3, 5, 7, 8, 11],
            MelodicMinor => &[0, 2, 3, 5, 7, 9, 11],
            Dorian => &[0, 2, 3, 5, 7, 9, 10],
            Phrygian => &[0, 1, 3, 5, 7, 8, 10],
            Lydian => &[0, 2, 4, 6, 7, 9, 11],
            Mixolydian => &[0, 2, 4, 5, 7, 9, 10],
            Locrian => &[0, 1, 3, 5, 6, 8, 10],
            MajorPentatonic => &[0, 2, 4, 7, 9],
            MinorPentatonic => &[0, 3, 5, 7, 10],
            Blues => &[0, 3, 5, 6, 7, 10],
        }
    }

    /// Letter steps above the tonic used to spell each degree (blues b5 shares the 5th's letter).
    fn letter_offsets(self) -> &'static [u8] {
        match self {
            MajorPentatonic => &[0, 1, 2, 4, 5],
            MinorPentatonic => &[0, 2, 3, 4, 6],
            Blues => &[0, 2, 3, 4, 4, 6],
            _ => &[0, 1, 2, 3, 4, 5, 6],
        }
    }

    /// The 7-note scale chords are built from (pentatonic and blues use their parent scale).
    pub fn harmonic_base(self) -> Mode {
        match self {
            MajorPentatonic => Major,
            MinorPentatonic | Blues => NaturalMinor,
            m => m,
        }
    }

    /// True when the third above the tonic is minor.
    pub fn minor_third(self) -> bool {
        self.harmonic_base().intervals()[2] == 3
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Key {
    pub tonic: Note,
    pub mode: Mode,
}

impl Key {
    pub fn new(tonic: &str, mode: &str) -> Result<Key, String> {
        Ok(Key { tonic: Note::parse(tonic)?, mode: Mode::parse(mode)? })
    }

    /// "A minor", "F# major", "D dorian", "Am", "C#m", "Eb".
    pub fn parse(s: &str) -> Result<Key, String> {
        let s = s.trim();
        let (first, rest) = s.split_once(char::is_whitespace).map(|(a, b)| (a, b.trim())).unwrap_or((s, ""));
        if rest.is_empty() && first.len() > 1 && first.ends_with('m') {
            return Ok(Key { tonic: Note::parse(&first[..first.len() - 1])?, mode: NaturalMinor });
        }
        Key::new(first, rest)
    }

    pub fn scale(self) -> Vec<Note> {
        self.mode.intervals().iter().zip(self.mode.letter_offsets()).map(|(&i, &l)| self.tonic.up(l, i)).collect()
    }

    pub fn pcs(self) -> Vec<u8> {
        self.scale().iter().map(|n| n.pc()).collect()
    }

    pub fn name(self) -> String {
        let m = match self.mode {
            NaturalMinor => "minor".to_string(),
            m => m.id().replace('_', " "),
        };
        format!("{} {m}", self.tonic.name())
    }

    fn with(self, mode: Mode) -> Key {
        Key { tonic: self.tonic, mode }
    }

    /// Sharps (> 0) or flats (< 0) in the key signature, counted on the spelled scale.
    /// Harmonic/melodic minor take the natural minor signature, pentatonic/blues their parent's.
    pub fn signature(self) -> i32 {
        let base = match self.mode {
            HarmonicMinor | MelodicMinor | MinorPentatonic | Blues => NaturalMinor,
            MajorPentatonic => Major,
            m => m,
        };
        self.with(base).scale().iter().map(|n| n.acc as i32).sum()
    }

    pub fn relative(self) -> Option<Key> {
        let k = |tonic, mode| Some(Key { tonic, mode });
        match self.mode {
            Major => k(self.scale()[5], NaturalMinor),
            NaturalMinor | HarmonicMinor | MelodicMinor => k(self.with(NaturalMinor).scale()[2], Major),
            MajorPentatonic => k(self.with(Major).scale()[5], MinorPentatonic),
            MinorPentatonic | Blues => k(self.with(NaturalMinor).scale()[2], MajorPentatonic),
            _ => None,
        }
    }

    pub fn parallel(self) -> Option<Key> {
        match self.mode {
            Major => Some(self.with(NaturalMinor)),
            NaturalMinor | HarmonicMinor | MelodicMinor => Some(self.with(Major)),
            MajorPentatonic => Some(self.with(MinorPentatonic)),
            MinorPentatonic | Blues => Some(self.with(MajorPentatonic)),
            _ => None,
        }
    }
}

pub fn interval_name(semis: u8) -> &'static str {
    ["P1", "m2", "M2", "m3", "M3", "P4", "TT", "P5", "m6", "M6", "m7", "M7"][(semis % 12) as usize]
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Quality {
    Major,
    Minor,
    Diminished,
    Augmented,
    Sus2,
    Sus4,
    Major7,
    Dominant7,
    Minor7,
    HalfDiminished7,
    Diminished7,
    MinorMajor7,
    AugmentedMajor7,
}

impl Quality {
    pub const ALL: [Quality; 13] = [
        Quality::Major,
        Quality::Minor,
        Quality::Diminished,
        Quality::Augmented,
        Quality::Sus2,
        Quality::Sus4,
        Quality::Major7,
        Quality::Dominant7,
        Quality::Minor7,
        Quality::HalfDiminished7,
        Quality::Diminished7,
        Quality::MinorMajor7,
        Quality::AugmentedMajor7,
    ];

    pub fn intervals(self) -> &'static [u8] {
        use Quality as Q;
        match self {
            Q::Major => &[0, 4, 7],
            Q::Minor => &[0, 3, 7],
            Q::Diminished => &[0, 3, 6],
            Q::Augmented => &[0, 4, 8],
            Q::Sus2 => &[0, 2, 7],
            Q::Sus4 => &[0, 5, 7],
            Q::Major7 => &[0, 4, 7, 11],
            Q::Dominant7 => &[0, 4, 7, 10],
            Q::Minor7 => &[0, 3, 7, 10],
            Q::HalfDiminished7 => &[0, 3, 6, 10],
            Q::Diminished7 => &[0, 3, 6, 9],
            Q::MinorMajor7 => &[0, 3, 7, 11],
            Q::AugmentedMajor7 => &[0, 4, 8, 11],
        }
    }

    fn letters(self) -> &'static [u8] {
        match self {
            Quality::Sus2 => &[0, 1, 4],
            Quality::Sus4 => &[0, 3, 4],
            q if q.intervals().len() == 4 => &[0, 2, 4, 6],
            _ => &[0, 2, 4],
        }
    }

    pub fn symbol(self) -> &'static str {
        use Quality as Q;
        match self {
            Q::Major => "",
            Q::Minor => "m",
            Q::Diminished => "dim",
            Q::Augmented => "aug",
            Q::Sus2 => "sus2",
            Q::Sus4 => "sus4",
            Q::Major7 => "maj7",
            Q::Dominant7 => "7",
            Q::Minor7 => "m7",
            Q::HalfDiminished7 => "m7b5",
            Q::Diminished7 => "dim7",
            Q::MinorMajor7 => "m(maj7)",
            Q::AugmentedMajor7 => "maj7#5",
        }
    }

    pub fn id(self) -> &'static str {
        use Quality as Q;
        match self {
            Q::Major => "major",
            Q::Minor => "minor",
            Q::Diminished => "diminished",
            Q::Augmented => "augmented",
            Q::Sus2 => "sus2",
            Q::Sus4 => "sus4",
            Q::Major7 => "major7",
            Q::Dominant7 => "dominant7",
            Q::Minor7 => "minor7",
            Q::HalfDiminished7 => "half_diminished7",
            Q::Diminished7 => "diminished7",
            Q::MinorMajor7 => "minor_major7",
            Q::AugmentedMajor7 => "augmented_major7",
        }
    }

    pub fn from_intervals(iv: &[u8]) -> Option<Quality> {
        Quality::ALL.iter().copied().find(|q| q.intervals() == iv)
    }

    /// Minor third above the root (written lower-case in roman numerals).
    pub fn minorish(self) -> bool {
        self.intervals()[1] == 3
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Chord {
    pub root: Note,
    pub quality: Quality,
    /// Scale degree 0..6 the root sits on (after any b/# alteration).
    pub degree: u8,
    /// Root altered from the scale (bVII in major, bII …): a chord borrowed from another mode.
    pub borrowed: bool,
    pub roman: String,
}

impl Chord {
    pub fn notes(&self) -> Vec<Note> {
        self.quality.intervals().iter().zip(self.quality.letters()).map(|(&i, &l)| self.root.up(l, i)).collect()
    }

    pub fn pcs(&self) -> Vec<u8> {
        self.notes().iter().map(|n| n.pc()).collect()
    }

    pub fn symbol(&self) -> String {
        format!("{}{}", self.root.name(), self.quality.symbol())
    }

    /// Harmonic function: tonic (I iii vi), predominant (ii IV), dominant (V vii), borrowed.
    pub fn function(&self) -> &'static str {
        if self.borrowed {
            return "borrowed";
        }
        match self.degree {
            0 | 2 | 5 => "tonic",
            1 | 3 => "predominant",
            _ => "dominant",
        }
    }

    /// 0 = rest (tonic) … 3 = strongest pull towards resolution.
    pub fn tension(&self) -> u8 {
        let base = match self.function() {
            "tonic" => 0,
            "predominant" | "borrowed" => 1,
            _ => 2,
        };
        base + matches!(self.quality, Quality::Dominant7 | Quality::Diminished7 | Quality::HalfDiminished7) as u8
    }

    pub fn view(&self) -> Value {
        json!({
            "symbol": self.symbol(),
            "roman": self.roman,
            "root": self.root.name(),
            "quality": self.quality.id(),
            "notes": self.notes().iter().map(|n| n.name()).collect::<Vec<_>>(),
            "pitch_classes": self.pcs(),
            "degree": self.degree + 1,
            "function": self.function(),
            "tension": self.tension(),
        })
    }
}

fn roman(q: Quality, degree: u8, prefix: &str) -> String {
    use Quality as Q;
    let mut s = ROMAN[degree as usize].to_string();
    if q.minorish() {
        s = s.to_lowercase();
    }
    let suffix = match q {
        Q::Major | Q::Minor => "",
        Q::Diminished => "°",
        Q::Augmented => "+",
        Q::Sus2 => "sus2",
        Q::Sus4 => "sus4",
        Q::Major7 => "maj7",
        Q::Dominant7 | Q::Minor7 => "7",
        Q::HalfDiminished7 => "ø7",
        Q::Diminished7 => "°7",
        Q::MinorMajor7 => "(maj7)",
        Q::AugmentedMajor7 => "+maj7",
    };
    format!("{prefix}{s}{suffix}")
}

/// Chord built by stacking thirds on scale degree `degree` (0-based) of the key.
pub fn diatonic(key: Key, degree: u8, seventh: bool) -> Chord {
    let base = key.with(key.mode.harmonic_base());
    let (sc, pcs) = (base.scale(), base.pcs());
    let d = degree as usize % 7;
    let n = if seventh { 4 } else { 3 };
    let iv: Vec<u8> = (0..n).map(|k| (pcs[(d + 2 * k) % 7] + 12 - pcs[d]) % 12).collect();
    let quality = Quality::from_intervals(&iv).expect("thirds stacked on a 7-note scale form a known chord");
    Chord { root: sc[d], quality, degree: d as u8, borrowed: false, roman: roman(quality, d as u8, "") }
}

/// Parse one roman numeral (case = major/minor, optional b/# prefix and quality suffix).
pub fn numeral(key: Key, tok: &str, sevenths: bool) -> Result<Chord, String> {
    use Quality as Q;
    let bad = || format!("invalid roman numeral: {tok:?}");
    let (shift, s) = if let Some(r) = tok.strip_prefix('b').or_else(|| tok.strip_prefix('♭')) {
        (-1i8, r)
    } else if let Some(r) = tok.strip_prefix('#').or_else(|| tok.strip_prefix('♯')) {
        (1, r)
    } else {
        (0, tok)
    };
    let n = s.chars().take_while(|c| matches!(c, 'I' | 'V' | 'i' | 'v')).count();
    let (num, suffix) = s.split_at(n);
    let degree = ROMAN.iter().position(|r| r.eq_ignore_ascii_case(num)).ok_or_else(bad)? as u8;
    let upper = num.chars().all(|c| c.is_ascii_uppercase());
    if !upper && !num.chars().all(|c| c.is_ascii_lowercase()) {
        return Err(bad());
    }
    let (dia, dia7) = (diatonic(key, degree, false), diatonic(key, degree, true));
    let case_fits = |q: Quality| shift == 0 && q.minorish() != upper;
    let triad = if case_fits(dia.quality) { dia.quality } else if upper { Q::Major } else { Q::Minor };
    let seventh_of = |t: Quality| match t {
        _ if case_fits(dia7.quality) && t == dia.quality => dia7.quality,
        Q::Major if degree == 4 => Q::Dominant7,
        Q::Major => Q::Major7,
        Q::Minor => Q::Minor7,
        Q::Diminished => Q::HalfDiminished7,
        Q::Augmented => Q::AugmentedMajor7,
        other => other,
    };
    let quality = match suffix.trim() {
        "" if sevenths => seventh_of(triad),
        "" => triad,
        "7" if case_fits(dia7.quality) => dia7.quality,
        "7" if upper => Q::Dominant7,
        "7" => Q::Minor7,
        "maj7" | "M7" | "Δ" | "Δ7" if upper => Q::Major7,
        "maj7" | "M7" | "Δ" | "Δ7" | "(maj7)" => Q::MinorMajor7,
        "°" | "o" | "dim" => Q::Diminished,
        "°7" | "o7" | "dim7" => Q::Diminished7,
        "ø" | "ø7" | "m7b5" => Q::HalfDiminished7,
        "+" | "aug" => Q::Augmented,
        "+maj7" => Q::AugmentedMajor7,
        "sus2" => Q::Sus2,
        "sus4" | "sus" => Q::Sus4,
        _ => return Err(bad()),
    };
    let acc = dia.root.acc + shift;
    if acc.abs() > 2 {
        return Err(bad());
    }
    let prefix = match shift {
        -1 => "b",
        1 => "#",
        _ => "",
    };
    Ok(Chord { root: Note { letter: dia.root.letter, acc }, quality, degree, borrowed: shift != 0, roman: roman(quality, degree, prefix) })
}

/// Parse a progression such as "i-VI-III-VII", "ii7 V7 Imaj7" or "I, bVII, IV".
pub fn progression(key: Key, text: &str, sevenths: bool) -> Result<Vec<Chord>, String> {
    let toks: Vec<&str> = text.split(|c: char| c == '-' || c == ',' || c == '|' || c.is_whitespace()).filter(|t| !t.is_empty()).collect();
    if toks.is_empty() || toks.len() > 32 {
        return Err("progression must have 1 to 32 chords".into());
    }
    toks.iter().map(|t| numeral(key, t, sevenths)).collect()
}

/// Name of the cadence formed by `a` → `b`.
pub fn cadence(a: &Chord, b: &Chord) -> &'static str {
    match (a.borrowed, a.degree, b.borrowed, b.degree) {
        (false, 4, false, 0) => "authentic",
        (false, 6, false, 0) if matches!(a.quality, Quality::Diminished | Quality::Diminished7 | Quality::HalfDiminished7) => "leading_tone",
        (_, 6, false, 0) => "subtonic",
        (false, 1 | 3, false, 0) => "plagal",
        (false, 4, false, 5) => "deceptive",
        (_, _, false, 4) => "half",
        (_, _, false, 0) => "to_tonic",
        _ => "open",
    }
}

// Krumhansl–Kessler key profiles (probe-tone ratings), C major / C minor.
const KK_MAJOR: [f64; 12] = [6.35, 2.23, 3.48, 2.33, 4.38, 4.09, 2.52, 5.19, 2.39, 3.66, 2.29, 2.88];
const KK_MINOR: [f64; 12] = [6.33, 2.68, 3.52, 5.38, 2.60, 3.53, 2.54, 4.75, 3.98, 2.69, 3.34, 3.17];

fn pearson(a: &[f64; 12], b: &[f64; 12]) -> f64 {
    let (ma, mb) = (a.iter().sum::<f64>() / 12.0, b.iter().sum::<f64>() / 12.0);
    let (mut num, mut da, mut db) = (0.0, 0.0, 0.0);
    for i in 0..12 {
        num += (a[i] - ma) * (b[i] - mb);
        da += (a[i] - ma).powi(2);
        db += (b[i] - mb).powi(2);
    }
    if da == 0.0 || db == 0.0 { 0.0 } else { num / (da * db).sqrt() }
}

/// Tonic spelled with the fewer accidentals in its key signature (Bb major, not A# major).
fn tonic_for(pc: u8, mode: Mode) -> Key {
    let [a, b] = [Key { tonic: Note::simple(pc, false), mode }, Key { tonic: Note::simple(pc, true), mode }];
    if b.signature().abs() < a.signature().abs() { b } else { a }
}

/// Krumhansl–Schmuckler key finding on pitch-class weights (e.g. total note durations).
/// Returns the 24 major/minor keys sorted by correlation. An estimate, never a measurement.
pub fn detect_key(weights: &[f64; 12]) -> Vec<(Key, f64)> {
    let mut out = Vec::with_capacity(24);
    for pc in 0..12u8 {
        for (mode, profile) in [(Major, &KK_MAJOR), (NaturalMinor, &KK_MINOR)] {
            let rotated: [f64; 12] = std::array::from_fn(|i| profile[(i + 12 - pc as usize) % 12]);
            out.push((tonic_for(pc, mode), pearson(weights, &rotated)));
        }
    }
    out.sort_by(|a, b| b.1.total_cmp(&a.1));
    out
}

/// Progression used when none is given, chosen to suit the mode.
pub fn default_progression(mode: Mode) -> &'static str {
    match mode.harmonic_base() {
        Major => "I-V-vi-IV",
        Dorian => "i-IV-i-IV",
        Phrygian => "i-II-i-VII",
        Lydian => "I-II-I-II",
        Mixolydian => "I-VII-IV-I",
        Locrian => "i-II-iv-II",
        _ => "i-VI-III-VII",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn names(v: &[Note]) -> String {
        v.iter().map(|n| n.name()).collect::<Vec<_>>().join(" ")
    }
    fn key(s: &str) -> Key {
        Key::parse(s).unwrap()
    }

    #[test]
    fn notes_and_pitch_classes() {
        assert_eq!(Note::parse("C#").unwrap().pc(), 1);
        assert_eq!(Note::parse("Db").unwrap().pc(), 1);
        assert_eq!(Note::parse("cb").unwrap().pc(), 11);
        assert_eq!(Note::parse("E#").unwrap().pc(), 5);
        assert_eq!(Note::parse("Fx").unwrap().name(), "F##");
        assert!(Note::parse("H").is_err());
        assert!(Note::parse("Cbbb").is_err());
        assert!(Note::parse("C$").is_err());
        assert!(Note::parse("").is_err());
    }

    #[test]
    fn scales_are_spelled_by_letter() {
        assert_eq!(names(&key("C major").scale()), "C D E F G A B");
        assert_eq!(names(&key("A minor").scale()), "A B C D E F G");
        assert_eq!(names(&key("F# major").scale()), "F# G# A# B C# D# E#");
        assert_eq!(names(&key("Eb minor").scale()), "Eb F Gb Ab Bb Cb Db");
        assert_eq!(names(&key("A harmonic minor").scale()), "A B C D E F G#");
        assert_eq!(names(&key("A melodic minor").scale()), "A B C D E F# G#");
        assert_eq!(names(&key("D dorian").scale()), "D E F G A B C");
        assert_eq!(names(&key("E phrygian").scale()), "E F G A B C D");
        assert_eq!(names(&key("F lydian").scale()), "F G A B C D E");
        assert_eq!(names(&key("G mixolydian").scale()), "G A B C D E F");
        assert_eq!(names(&key("B locrian").scale()), "B C D E F G A");
        assert_eq!(names(&key("C major pentatonic").scale()), "C D E G A");
        assert_eq!(names(&key("A minor pentatonic").scale()), "A C D E G");
        assert_eq!(names(&key("A blues").scale()), "A C D Eb E G");
        assert_eq!(names(&key("Bbm").scale()), "Bb C Db Eb F Gb Ab");
        // every heptatonic scale uses each letter exactly once
        for m in Mode::ALL.iter().filter(|m| m.intervals().len() == 7) {
            for t in ["C", "C#", "Db", "D", "Eb", "E", "F", "F#", "Gb", "G", "Ab", "A", "Bb", "B"] {
                let mut l: Vec<u8> = Key { tonic: Note::parse(t).unwrap(), mode: *m }.scale().iter().map(|n| n.letter).collect();
                l.sort();
                assert_eq!(l, vec![0, 1, 2, 3, 4, 5, 6], "{t} {}", m.id());
            }
        }
    }

    #[test]
    fn key_signatures_relative_and_parallel() {
        for (k, sig) in [("C major", 0), ("G major", 1), ("D major", 2), ("F# major", 6), ("C# major", 7), ("F major", -1), ("Bb major", -2), ("Gb major", -6), ("Cb major", -7), ("A minor", 0), ("E minor", 1), ("Eb minor", -6), ("D minor", -1), ("A harmonic minor", 0), ("D dorian", 0), ("A blues", 0)] {
            assert_eq!(key(k).signature(), sig, "{k}");
        }
        assert_eq!(key("C major").relative().unwrap().name(), "A minor");
        assert_eq!(key("F# minor").relative().unwrap().name(), "A major");
        assert_eq!(key("Eb major").relative().unwrap().name(), "C minor");
        assert_eq!(key("A minor").parallel().unwrap().name(), "A major");
        assert!(key("D dorian").relative().is_none());
    }

    #[test]
    fn intervals_are_named() {
        let iv: Vec<&str> = key("C major").mode.intervals().iter().map(|&i| interval_name(i)).collect();
        assert_eq!(iv, ["P1", "M2", "M3", "P4", "P5", "M6", "M7"]);
        assert_eq!(interval_name(6), "TT");
    }

    #[test]
    fn diatonic_chords() {
        let c = key("C major");
        let q: Vec<&str> = (0..7).map(|d| diatonic(c, d, false).quality.id()).collect();
        assert_eq!(q, ["major", "minor", "minor", "major", "major", "minor", "diminished"]);
        let r: Vec<String> = (0..7).map(|d| diatonic(c, d, true).roman).collect();
        assert_eq!(r, ["Imaj7", "ii7", "iii7", "IVmaj7", "V7", "vi7", "viiø7"]);
        let s: Vec<String> = (0..7).map(|d| diatonic(c, d, true).symbol()).collect();
        assert_eq!(s, ["Cmaj7", "Dm7", "Em7", "Fmaj7", "G7", "Am7", "Bm7b5"]);
        let h = key("A harmonic minor");
        assert_eq!(diatonic(h, 4, false).symbol(), "E");
        assert_eq!(diatonic(h, 2, false).roman, "III+");
        assert_eq!(diatonic(h, 6, true).symbol(), "G#dim7");
        assert_eq!(names(&diatonic(h, 6, true).notes()), "G# B D F");
        assert_eq!(names(&diatonic(key("F# major"), 6, false).notes()), "E# G# B");
        // every mode, every degree, triads and sevenths: notes stay inside the scale
        for m in Mode::ALL {
            let k = Key { tonic: Note::parse("D").unwrap(), mode: m };
            let sc = k.with(m.harmonic_base()).pcs();
            for d in 0..7 {
                for sev in [false, true] {
                    let ch = diatonic(k, d, sev);
                    assert!(ch.pcs().iter().all(|p| sc.contains(p)), "{} {}", m.id(), ch.symbol());
                }
            }
        }
    }

    #[test]
    fn key_detection_finds_the_scale() {
        for k in ["A minor", "Eb major", "F# minor", "C major", "Bb minor"] {
            let key = Key::parse(k).unwrap();
            let mut w = [0.0; 12];
            // tonic triad weighted more, as in real melodies
            for (i, pc) in key.pcs().iter().enumerate() {
                w[*pc as usize] += if [0, 2, 4].contains(&i) { 3.0 } else { 1.0 };
            }
            assert_eq!(detect_key(&w)[0].0.name(), key.name(), "{k}");
        }
        assert_eq!(detect_key(&[0.0; 12]).len(), 24);
    }

    #[test]
    fn progressions_from_roman_numerals() {
        let am = key("A minor");
        let p = progression(am, "i-VI-III-VII", false).unwrap();
        assert_eq!(p.iter().map(|c| c.symbol()).collect::<Vec<_>>(), ["Am", "F", "C", "G"]);
        assert_eq!(p.iter().map(|c| c.function()).collect::<Vec<_>>(), ["tonic", "tonic", "tonic", "dominant"]);
        // V in a minor key is major (harmonic minor dominant), V7 is dominant seventh
        let v = progression(am, "iv V7 i", false).unwrap();
        assert_eq!(v.iter().map(|c| c.symbol()).collect::<Vec<_>>(), ["Dm", "E7", "Am"]);
        assert_eq!(cadence(&v[1], &v[2]), "authentic");
        let c = key("C major");
        let b = progression(c, "I, bVII, IV, I", false).unwrap();
        assert_eq!(b[1].symbol(), "Bb");
        assert!(b[1].borrowed && b[1].function() == "borrowed");
        assert_eq!(cadence(&b[2], &b[3]), "plagal");
        let s = progression(c, "ii V I", true).unwrap();
        assert_eq!(s.iter().map(|c| c.symbol()).collect::<Vec<_>>(), ["Dm7", "G7", "Cmaj7"]);
        assert_eq!(progression(c, "vii°7", false).unwrap()[0].symbol(), "Bdim7");
        assert_eq!(progression(c, "Vsus4 V", false).unwrap()[0].symbol(), "Gsus4");
        assert_eq!(cadence(&progression(c, "V", false).unwrap()[0], &progression(c, "vi", false).unwrap()[0]), "deceptive");
        assert!(progression(c, "VIII", false).is_err());
        assert!(progression(c, "Iv", false).is_err());
        assert!(progression(c, "", false).is_err());
        assert!(progression(c, "I7x", false).is_err());
        for m in Mode::ALL {
            assert!(progression(Key { tonic: Note::parse("E").unwrap(), mode: m }, default_progression(m), false).is_ok(), "{}", m.id());
        }
    }
}
