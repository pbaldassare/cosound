//! Chord Engine – chord types (triads, sevenths, extended, added-note, suspended), chord symbols, building from a
//! root / scale degree / symbol, inversions and voicings, and chord recognition from MIDI notes.
//! Formulas: docs/knowledge/music-theory/harmony.md §1 (exact); voicing rules: midi-intelligence/composition-guide.md §4.
use super::notes::{italian, parse_note};
use super::theory::Note;
use serde_json::{json, Value};

#[derive(Debug)]
pub struct ChordType {
    pub id: &'static str,
    /// Canonical suffix after the root ("" = major triad).
    pub symbol: &'static str,
    pub aliases: &'static [&'static str],
    /// Semitones above the root (may exceed 12 for extensions).
    pub intervals: &'static [u8],
    /// Chord-tone degree of each interval (1, 3, 5, 7, 9, 11, 13; 2/4/6 for sus/add/6), used for spelling.
    pub degrees: &'static [u8],
    pub family: &'static str,
}

macro_rules! c {
    ($id:expr, $sym:expr, $al:expr, $iv:expr, $dg:expr, $f:expr) => {
        ChordType { id: $id, symbol: $sym, aliases: $al, intervals: $iv, degrees: $dg, family: $f }
    };
}

pub const TYPES: [ChordType; 36] = [
    c!("major", "", &["maj", "M", "major"], &[0, 4, 7], &[1, 3, 5], "triad"),
    c!("minor", "m", &["min", "-", "minor", "mi"], &[0, 3, 7], &[1, 3, 5], "triad"),
    c!("diminished", "dim", &["°", "o", "dim"], &[0, 3, 6], &[1, 3, 5], "triad"),
    c!("augmented", "aug", &["+", "#5"], &[0, 4, 8], &[1, 3, 5], "triad"),
    c!("sus2", "sus2", &["sus9"], &[0, 2, 7], &[1, 2, 5], "suspended"),
    c!("sus4", "sus4", &["sus"], &[0, 5, 7], &[1, 4, 5], "suspended"),
    c!("power", "5", &["power"], &[0, 7], &[1, 5], "dyad"),
    c!("major6", "6", &["maj6", "M6", "add6"], &[0, 4, 7, 9], &[1, 3, 5, 6], "added"),
    c!("minor6", "m6", &["min6", "-6"], &[0, 3, 7, 9], &[1, 3, 5, 6], "added"),
    c!("six_nine", "6/9", &["69", "6add9"], &[0, 4, 7, 9, 14], &[1, 3, 5, 6, 9], "added"),
    c!("add9", "add9", &["add2", "(add9)"], &[0, 4, 7, 14], &[1, 3, 5, 9], "added"),
    c!("minor_add9", "m(add9)", &["madd9", "m(add2)", "madd2", "-add9"], &[0, 3, 7, 14], &[1, 3, 5, 9], "added"),
    c!("add11", "add11", &["add4"], &[0, 4, 7, 17], &[1, 3, 5, 11], "added"),
    c!("major7", "maj7", &["M7", "Δ", "Δ7", "ma7", "j7", "maj"], &[0, 4, 7, 11], &[1, 3, 5, 7], "seventh"),
    c!("dominant7", "7", &["dom7", "dom"], &[0, 4, 7, 10], &[1, 3, 5, 7], "seventh"),
    c!("minor7", "m7", &["min7", "-7", "mi7"], &[0, 3, 7, 10], &[1, 3, 5, 7], "seventh"),
    c!("minor_major7", "m(maj7)", &["mmaj7", "mM7", "-maj7", "m(M7)"], &[0, 3, 7, 11], &[1, 3, 5, 7], "seventh"),
    c!("half_diminished7", "m7b5", &["ø", "ø7", "-7b5", "min7b5"], &[0, 3, 6, 10], &[1, 3, 5, 7], "seventh"),
    c!("diminished7", "dim7", &["°7", "o7"], &[0, 3, 6, 9], &[1, 3, 5, 7], "seventh"),
    c!("augmented7", "7#5", &["aug7", "+7", "7+5"], &[0, 4, 8, 10], &[1, 3, 5, 7], "seventh"),
    c!("augmented_major7", "maj7#5", &["+maj7", "augmaj7", "maj7+5"], &[0, 4, 8, 11], &[1, 3, 5, 7], "seventh"),
    c!("dominant7sus4", "7sus4", &["7sus"], &[0, 5, 7, 10], &[1, 4, 5, 7], "suspended"),
    c!("major9", "maj9", &["M9", "Δ9"], &[0, 4, 7, 11, 14], &[1, 3, 5, 7, 9], "extended"),
    c!("dominant9", "9", &["dom9"], &[0, 4, 7, 10, 14], &[1, 3, 5, 7, 9], "extended"),
    c!("minor9", "m9", &["min9", "-9"], &[0, 3, 7, 10, 14], &[1, 3, 5, 7, 9], "extended"),
    c!("dominant7b9", "7b9", &["7(b9)"], &[0, 4, 7, 10, 13], &[1, 3, 5, 7, 9], "altered"),
    c!("dominant7s9", "7#9", &["7(#9)"], &[0, 4, 7, 10, 15], &[1, 3, 5, 7, 9], "altered"),
    c!("dominant11", "11", &["dom11"], &[0, 4, 7, 10, 14, 17], &[1, 3, 5, 7, 9, 11], "extended"),
    c!("minor11", "m11", &["min11", "-11"], &[0, 3, 7, 10, 14, 17], &[1, 3, 5, 7, 9, 11], "extended"),
    c!("major7s11", "maj7#11", &["M7#11", "Δ#11"], &[0, 4, 7, 11, 18], &[1, 3, 5, 7, 11], "extended"),
    c!("dominant7s11", "7#11", &["7(#11)"], &[0, 4, 7, 10, 18], &[1, 3, 5, 7, 11], "altered"),
    c!("dominant13", "13", &["dom13"], &[0, 4, 7, 10, 14, 17, 21], &[1, 3, 5, 7, 9, 11, 13], "extended"),
    c!("major13", "maj13", &["M13", "Δ13"], &[0, 4, 7, 11, 14, 21], &[1, 3, 5, 7, 9, 13], "extended"),
    c!("minor13", "m13", &["min13", "-13"], &[0, 3, 7, 10, 14, 17, 21], &[1, 3, 5, 7, 9, 11, 13], "extended"),
    c!("dominant7b13", "7b13", &["7(b13)"], &[0, 4, 7, 10, 20], &[1, 3, 5, 7, 13], "altered"),
    c!("minor7b9", "m7b9", &["m7(b9)"], &[0, 3, 7, 10, 13], &[1, 3, 5, 7, 9], "altered"),
];

pub fn find_type(suffix: &str) -> Option<&'static ChordType> {
    let s = suffix.trim();
    TYPES.iter().find(|t| t.symbol == s).or_else(|| TYPES.iter().find(|t| t.aliases.contains(&s))).or_else(|| {
        let l = s.to_lowercase();
        TYPES.iter().find(|t| t.symbol.to_lowercase() == l && !t.symbol.is_empty())
    })
}

/// A chord: root, type and optional bass note (slash chord / inversion).
#[derive(Clone, Debug)]
pub struct ChordSym {
    pub root: Note,
    pub kind: &'static ChordType,
    pub bass: Option<Note>,
}

impl ChordSym {
    /// "Am7", "F#m7b5", "Bb13", "Dsus4", "C/E", "Cmaj7/B", "Lam" , "Sol7", "Fa#m".
    pub fn parse(s: &str) -> Result<ChordSym, String> {
        let t = s.trim().replace(' ', "");
        let (main, bass) = match t.rsplit_once('/') {
            Some((m, b)) if !b.is_empty() && !b.chars().next().unwrap().is_ascii_digit() => (m.to_string(), Some(parse_note(b)?)),
            _ => (t.clone(), None),
        };
        // longest root prefix (Italian syllable or letter + accidentals) whose remainder is a known chord type:
        // "Fadd9" = F + add9 (not Fa + "dd9"), "Fa#m" = Fa# + m, "Ab7" = Ab + 7
        let mut found = None;
        let mut any_root = false;
        for cut in (1..=main.len().min(4)).rev() {
            if !main.is_char_boundary(cut) {
                continue;
            }
            if let Ok(r) = parse_note(&main[..cut]) {
                any_root = true;
                if let Some(k) = find_type(&main[cut..]) {
                    found = Some((r, k));
                    break;
                }
            }
        }
        let (root, kind) = found.ok_or_else(|| if any_root { format!("unknown chord type in {s:?}") } else { format!("invalid chord symbol {s:?}") })?;
        Ok(ChordSym { root, kind, bass })
    }

    pub fn notes(&self) -> Vec<Note> {
        self.kind.intervals.iter().zip(self.kind.degrees).map(|(&i, &d)| self.root.up((d - 1) % 7, i % 12)).collect()
    }

    pub fn pcs(&self) -> Vec<u8> {
        self.notes().iter().map(|n| n.pc()).collect()
    }

    pub fn symbol(&self) -> String {
        let mut s = format!("{}{}", self.root.name(), self.kind.symbol);
        if let Some(b) = self.bass {
            s.push('/');
            s.push_str(&b.name());
        }
        s
    }

    /// Close-position voicing in a register: root (or bass) lowest, inversion = which chord tone is in the bass.
    pub fn voicing(&self, inversion: usize, low: u8) -> Vec<u8> {
        let base: Vec<u8> = self.kind.intervals.to_vec();
        let n = base.len();
        let inv = inversion % n;
        let mut iv: Vec<i32> = base.iter().map(|&x| x as i32).collect();
        for k in 0..inv {
            iv[k] += 12 * (1 + iv[n - 1] / 12 - iv[k] / 12).max(1);
        }
        iv.sort();
        let lo = iv[0];
        let root_pc = self.root.pc() as i32;
        let mut start = low as i32 + ((root_pc + lo - low as i32).rem_euclid(12));
        if let Some(b) = self.bass {
            start = low as i32 + ((b.pc() as i32 - low as i32).rem_euclid(12));
            let mut v: Vec<i32> = vec![start];
            for x in iv.iter().map(|x| (root_pc + x).rem_euclid(12)) {
                let mut p = start + ((x - start).rem_euclid(12));
                if p == start {
                    continue;
                }
                while v.contains(&p) {
                    p += 12;
                }
                v.push(p);
            }
            v.sort();
            return v.into_iter().map(|p| p.clamp(0, 127) as u8).collect();
        }
        iv.iter().map(|x| (start + x - lo).clamp(0, 127) as u8).collect()
    }

    pub fn view(&self) -> Value {
        let notes = self.notes();
        let names: Vec<String> = notes.iter().map(|n| n.name()).collect();
        let inversions: Vec<Value> = (0..self.kind.intervals.len().min(4))
            .map(|i| {
                let v = self.voicing(i, 48);
                const INV: [&str; 4] = ["root position", "1st inversion", "2nd inversion", "3rd inversion"];
                json!({ "inversion": i, "name": INV[i], "midi": v, "bass": names[i] })
            })
            .collect();
        json!({
            "symbol": self.symbol(), "root": self.root.name(), "root_italian": italian(self.root), "type": self.kind.id, "family": self.kind.family,
            "notes": names, "note_italiane": notes.iter().map(|n| italian(*n)).collect::<Vec<_>>(),
            "intervals": self.kind.intervals, "degrees": self.kind.degrees, "pitch_classes": self.pcs(),
            "bass": self.bass.map(|b| b.name()),
            "voicings": { "close": self.voicing(0, 48), "inversions": inversions, "drop2": drop2(&self.voicing(0, 55)), "spread": spread(&self.voicing(0, 48)) },
            "source": "docs/knowledge/music-theory/harmony.md §1–2",
        })
    }
}

/// Drop-2: the second voice from the top goes down an octave.
pub fn drop2(close: &[u8]) -> Vec<u8> {
    if close.len() < 4 {
        return close.to_vec();
    }
    let mut v = close.to_vec();
    let i = v.len() - 2;
    v[i] = v[i].saturating_sub(12);
    v.sort();
    v
}

/// Spread voicing: root low, upper tones an octave up (low-interval rule: only octaves/fifths below MIDI 48).
pub fn spread(close: &[u8]) -> Vec<u8> {
    let mut v = vec![close[0].saturating_sub(12)];
    v.extend(close.iter().skip(1).map(|p| if *p < 60 { p + 12 } else { *p }));
    v.sort();
    v.dedup();
    v
}

/// Voice leading: the inversion of `next` closest to `prev` (minimum total movement), in the 48–79 register.
pub fn lead(prev: &[u8], next: &ChordSym) -> Vec<u8> {
    let n = next.kind.intervals.len().min(4);
    (0..n)
        .flat_map(|inv| [48u8, 55, 60].map(move |low| (inv, low)))
        .map(|(inv, low)| next.voicing(inv, low))
        .filter(|v| v.iter().all(|p| (48..=79).contains(p)))
        .min_by_key(|v| {
            let m: i32 = v.iter().map(|p| prev.iter().map(|q| (*p as i32 - *q as i32).abs()).min().unwrap_or(0)).sum();
            (m, v[0])
        })
        .unwrap_or_else(|| next.voicing(0, 48))
}

/// Name a pitch-class set with a known root (used for diatonic chords of any scale).
pub fn name_pcs(pcs: &[u8], root: Note) -> String {
    let r = root.pc();
    let mut iv: Vec<u8> = pcs.iter().map(|p| (p + 12 - r) % 12).collect();
    iv.sort();
    iv.dedup();
    TYPES
        .iter()
        .find(|t| {
            let mut ti: Vec<u8> = t.intervals.iter().map(|x| x % 12).collect();
            ti.sort();
            ti.dedup();
            ti == iv
        })
        .map(|t| format!("{}{}", root.name(), t.symbol))
        .unwrap_or_else(|| format!("{}({})", root.name(), iv.iter().map(|x| x.to_string()).collect::<Vec<_>>().join(",")))
}

/// Chord recognition from MIDI pitches (several notes sounding together). Candidates with a confidence:
/// exact pitch-class match first; a missing fifth is tolerated; the lowest note decides inversions/slash chords.
pub fn recognize(pitches: &[u8]) -> Vec<Value> {
    if pitches.is_empty() {
        return vec![];
    }
    let set: std::collections::BTreeSet<u8> = pitches.iter().map(|p| p % 12).collect();
    let bass = pitches.iter().min().unwrap() % 12;
    if set.len() == 1 {
        return vec![json!({ "symbol": Note::simple(bass, false).name(), "type": "single_note", "confidence": 0.0, "note": "one pitch class: not a chord" })];
    }
    let mut out: Vec<(f64, Value)> = vec![];
    for t in TYPES.iter() {
        let tset: std::collections::BTreeSet<u8> = t.intervals.iter().map(|x| x % 12).collect();
        for root in 0..12u8 {
            let cs: std::collections::BTreeSet<u8> = tset.iter().map(|x| (root + x) % 12).collect();
            let missing: Vec<u8> = cs.difference(&set).copied().collect();
            let extra = set.difference(&cs).count();
            // tolerate an omitted fifth (and 11th in 13th chords), never a missing third or root
            let fifth = (root + 7) % 12;
            let tolerable = missing.iter().all(|m| *m == fifth || (t.intervals.len() >= 6 && *m == (root + 5) % 12));
            if extra > 0 || !tolerable || !set.contains(&root) {
                continue;
            }
            let mut score = 1.0 - 0.15 * missing.len() as f64 - 0.02 * (t.intervals.len() as f64 - 3.0).max(0.0);
            if bass == root {
                score += 0.1;
            }
            let root_n = Note::simple(root, false);
            let sym = ChordSym { root: root_n, kind: t, bass: (bass != root).then(|| Note::simple(bass, false)) };
            let inversion = sym.kind.intervals.iter().position(|x| (root + x % 12) % 12 == bass).unwrap_or(0);
            out.push((score, json!({ "symbol": sym.symbol(), "root": root_n.name(), "type": t.id, "inversion": inversion, "missing": missing.iter().map(|m| Note::simple(*m, false).name()).collect::<Vec<_>>() })));
        }
    }
    out.sort_by(|a, b| b.0.total_cmp(&a.0));
    let top = out.first().map(|x| x.0).unwrap_or(1.0);
    out.into_iter()
        .take(4)
        .map(|(s, mut v)| {
            v["confidence"] = json!(((s / top.max(1e-9)) * if s >= 1.0 { 0.95 } else { 0.75 } * 100.0).round() / 100.0);
            v
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn n(s: &str) -> String {
        ChordSym::parse(s).unwrap().notes().iter().map(|x| x.name()).collect::<Vec<_>>().join(" ")
    }

    #[test]
    fn symbols_spelling_and_types() {
        assert_eq!(n("C"), "C E G");
        assert_eq!(n("Am"), "A C E");
        assert_eq!(n("F#m7b5"), "F# A C E");
        assert_eq!(n("Bb13"), "Bb D F Ab C Eb G");
        assert_eq!(n("Dsus4"), "D G A");
        assert_eq!(n("Ebmaj9"), "Eb G Bb D F");
        assert_eq!(n("G7b9"), "G B D F Ab");
        assert_eq!(n("C6/9"), "C E G A D");
        assert_eq!(n("Am(add9)"), "A C E B");
        assert_eq!(n("Bdim7"), "B D F Ab");
        assert_eq!(n("C#dim"), "C# E G");
        assert_eq!(n("Lam7"), "A C E G", "Italian root");
        assert_eq!(n("Sol7"), "G B D F");
        assert_eq!(n("Fa#m"), "F# A C#");
        assert_eq!(ChordSym::parse("C/E").unwrap().symbol(), "C/E");
        assert_eq!(ChordSym::parse("Cmaj7/B").unwrap().voicing(0, 48)[0], 59, "slash bass lowest");
        assert!(ChordSym::parse("Cxyz").is_err());
        assert_eq!(TYPES.len(), 36);
    }

    #[test]
    fn inversions_voicings_and_leading() {
        let c = ChordSym::parse("C").unwrap();
        assert_eq!(c.voicing(0, 48), vec![48, 52, 55]);
        assert_eq!(c.voicing(1, 48), vec![52, 55, 60]);
        assert_eq!(c.voicing(2, 48), vec![55, 60, 64]);
        let cm7 = ChordSym::parse("Cmaj7").unwrap();
        assert_eq!(drop2(&cm7.voicing(0, 60)), vec![55, 60, 64, 71]);
        let prev = c.voicing(0, 60);
        let f = lead(&prev, &ChordSym::parse("F").unwrap());
        let moved: i32 = f.iter().map(|p| prev.iter().map(|q| (*p as i32 - *q as i32).abs()).min().unwrap()).sum();
        assert!(moved <= 3, "C → F with common tone C kept: {f:?}");
    }

    #[test]
    fn chords_recognized_from_notes() {
        assert_eq!(recognize(&[57, 60, 64])[0]["symbol"], "Am");
        assert_eq!(recognize(&[64, 67, 72])[0]["symbol"], "C/E", "first inversion");
        assert_eq!(recognize(&[55, 59, 62, 65])[0]["symbol"], "G7");
        assert_eq!(recognize(&[48, 52, 58])[0]["symbol"], "C7", "fifth omitted");
        assert_eq!(recognize(&[50, 53, 57, 60, 64])[0]["symbol"], "Dm9");
        assert_eq!(recognize(&[60])[0]["type"], "single_note");
        assert!(recognize(&[60, 61, 62])[0..].iter().all(|c| c["symbol"] != "C"), "a cluster is not called a C chord");
    }
}
