//! Harmony Engine – progression analysis (roman numerals, functions, secondary dominants, borrowed chords,
//! cadences), substitutions, and generation of NEW progressions from a functional grammar weighted by style and
//! mood (not only a list of presets: the common progressions of the knowledge base are candidates among others).
//! Sources: docs/knowledge/music-theory/harmony.md §4–8.
use super::chords::ChordSym;
use super::generate::{Rng, BAR};
use super::theory::{self as t, Key, Mode};
use serde_json::{json, Value};

/// One chord on the timeline.
#[derive(Clone, Debug)]
pub struct Slot {
    pub start: u32,
    pub len: u32,
    pub chord: ChordSym,
    pub roman: String,
}

/// Chords over time: one entry per chord change.
pub type Timeline = Vec<Slot>;

pub fn chord_at(tl: &Timeline, tick: u32) -> &Slot {
    let total = tl.last().map(|s| s.start + s.len).unwrap_or(BAR).max(1);
    let t = tick % total;
    tl.iter().find(|s| t >= s.start && t < s.start + s.len).unwrap_or(&tl[0])
}

/// Build a timeline from chord symbols, `bars_per_chord` each (0.5 = two chords per bar), repeated to `bars`.
pub fn timeline(key: Key, symbols: &[String], bars_per_chord: f64, bars: u32) -> Result<Timeline, String> {
    let chords: Vec<ChordSym> = symbols.iter().map(|s| ChordSym::parse(s)).collect::<Result<_, _>>()?;
    if chords.is_empty() {
        return Err("no chords".into());
    }
    let len = ((BAR as f64) * bars_per_chord).round().max(1.0) as u32;
    let total = bars * BAR;
    let mut tl = vec![];
    let mut at = 0;
    let mut i = 0;
    while at < total {
        let c = chords[i % chords.len()].clone();
        let roman = roman_of(key, &c).0;
        tl.push(Slot { start: at, len: len.min(total - at), chord: c, roman });
        at += len;
        i += 1;
    }
    Ok(tl)
}

const ROMAN: [&str; 7] = ["I", "II", "III", "IV", "V", "VI", "VII"];

/// Roman numeral and function of a chord in a key: diatonic degree, altered root (b/#), secondary dominant, borrowed.
pub fn roman_of(key: Key, c: &ChordSym) -> (String, &'static str) {
    let scale = key.with_base().scale();
    let pcs = key.with_base().pcs();
    let root = c.root.pc();
    let minorish = c.kind.intervals.contains(&3) && !c.kind.intervals.contains(&4);
    let dim = c.kind.id.contains("diminished");
    let aug = c.kind.id.starts_with("augmented");
    let suffix = match c.kind.id {
        "major" | "minor" => "",
        "diminished" => "°",
        "augmented" => "+",
        "half_diminished7" => "ø7",
        "diminished7" => "°7",
        "major7" => "maj7",
        "dominant7" | "minor7" => "7",
        _ => c.kind.symbol.trim_start_matches('m'),
    };
    let numeral = |d: usize, prefix: &str| {
        let r = if minorish || dim { ROMAN[d].to_lowercase() } else { ROMAN[d].to_string() };
        format!("{prefix}{r}{}{}", if aug && suffix.is_empty() { "+" } else { "" }, suffix)
    };
    if let Some(d) = pcs.iter().position(|p| *p == root) {
        // secondary dominant: a major/dominant chord on a degree where the scale has a minor/diminished triad,
        // whose root is a fifth above another diatonic root
        let dia = t::diatonic(key, d as u8, false);
        let dominantish = matches!(c.kind.id, "major" | "dominant7" | "dominant9" | "dominant7b9");
        if dominantish && dia.quality.minorish() && d != 4 {
            let target = (root + 5) % 12;
            if let Some(td) = pcs.iter().position(|p| *p == target) {
                let tr = t::diatonic(key, td as u8, false).roman;
                return (format!("V{}/{tr}", if c.kind.id == "major" { "" } else { "7" }), "secondary_dominant");
            }
        }
        let f = match d {
            0 | 2 | 5 => "tonic",
            1 | 3 => "predominant",
            _ => "dominant",
        };
        let fits = c.pcs().iter().all(|p| pcs.contains(p));
        if !fits {
            let par = key.parallel().map(|p| p.pcs()).unwrap_or_default();
            if c.pcs().iter().all(|p| par.contains(p)) {
                return (numeral(d, ""), "borrowed");
            }
            // V in minor (harmonic minor), raised notes: still the dominant function
            if d == 4 || d == 6 {
                return (numeral(d, ""), "dominant");
            }
            return (numeral(d, ""), "chromatic");
        }
        let _ = scale;
        return (numeral(d, ""), f);
    }
    // altered root: bII, bIII, bVI, bVII, #IV …
    // flats first (♭VII, ♭VI, ♭III, ♭II are the usual names), then sharps
    for (d, p) in pcs.iter().enumerate() {
        if (p + 11) % 12 == root {
            let par = key.parallel().map(|p| p.pcs()).unwrap_or_default();
            let f = if c.pcs().iter().all(|x| par.contains(x)) { "borrowed" } else { "chromatic" };
            return (numeral(d, "b"), if d == 1 { "neapolitan_or_phrygian" } else { f });
        }
    }
    for (d, p) in pcs.iter().enumerate() {
        if (p + 1) % 12 == root {
            return (numeral(d, "#"), "chromatic");
        }
    }
    ("?".into(), "chromatic")
}

trait Base {
    fn with_base(self) -> Key;
}
impl Base for Key {
    fn with_base(self) -> Key {
        Key { tonic: self.tonic, mode: self.mode.harmonic_base() }
    }
}

/// Analyse a list of chord symbols in a key (or the key estimated from the chords when none is given).
pub fn analyze(key: Option<Key>, symbols: &[String]) -> Result<Value, String> {
    let chords: Vec<ChordSym> = symbols.iter().map(|s| ChordSym::parse(s)).collect::<Result<_, _>>()?;
    let key = match key {
        Some(k) => k,
        None => key_from_chords(&chords).0,
    };
    let rows: Vec<(String, &str)> = chords.iter().map(|c| roman_of(key, c)).collect();
    let cadences: Vec<Value> = rows
        .windows(2)
        .enumerate()
        .map(|(i, w)| json!({ "at": i + 1, "from": w[0].0, "to": w[1].0, "type": cadence_name(&w[0].0, &w[1].0) }))
        .filter(|c| c["type"] != "none")
        .collect();
    let loop_c = if rows.len() > 1 { cadence_name(&rows[rows.len() - 1].0, &rows[0].0) } else { "none" };
    Ok(json!({
        "key": key.name(), "nome": super::keys::key_name_it(key),
        "chords": chords.iter().zip(&rows).map(|(c, (r, f))| json!({ "symbol": c.symbol(), "roman": r, "function": f, "notes": c.notes().iter().map(|n| n.name()).collect::<Vec<_>>() })).collect::<Vec<_>>(),
        "cadences": cadences, "loop_cadence": loop_c,
        "source": "docs/knowledge/music-theory/harmony.md §4–5, §8",
    }))
}

/// Key of a chord progression: chords that belong to the key, the tonic at the start/end, and dominant→tonic
/// resolutions decide (a pitch-class profile alone is misled by chords). Returns the key and a 0–1 score.
pub fn key_from_chords(chords: &[ChordSym]) -> (Key, f64) {
    let mut best = (Key { tonic: t::Note::simple(0, false), mode: Mode::Major }, f64::MIN);
    for pc in 0..12u8 {
        for mode in [Mode::Major, Mode::NaturalMinor] {
            for flats in [false, true] {
                let key = Key { tonic: t::Note::simple(pc, flats), mode };
                let mut scale = key.pcs();
                if mode == Mode::NaturalMinor {
                    scale.push((pc + 11) % 12); // raised 7th of harmonic minor (V, vii°)
                }
                let inside = chords.iter().filter(|c| c.pcs().iter().all(|p| scale.contains(p))).count() as f64 / chords.len().max(1) as f64;
                let tonic_q = |c: &ChordSym| c.root.pc() == pc && (c.kind.intervals.contains(&3) && !c.kind.intervals.contains(&4)) == (mode == Mode::NaturalMinor);
                let mut sc = inside * 2.0;
                if chords.last().is_some_and(tonic_q) {
                    sc += 0.6;
                }
                if chords.first().is_some_and(tonic_q) {
                    sc += 0.4;
                }
                for w in chords.windows(2) {
                    if w[0].root.pc() == (pc + 7) % 12 && w[0].kind.intervals.contains(&4) && tonic_q(&w[1]) {
                        sc += 0.5;
                    }
                }
                sc -= key.signature().abs() as f64 * 0.001; // simplest spelling on ties
                if sc > best.1 {
                    best = (key, sc);
                }
            }
        }
    }
    (best.0, (best.1 / 3.5).clamp(0.0, 1.0))
}

fn base_numeral(r: &str) -> String {
    r.trim_start_matches(['b', '#']).chars().take_while(|c| "IViv".contains(*c)).collect::<String>().to_uppercase()
}

/// Cadence between two roman numerals (common-practice names).
pub fn cadence_name(a: &str, b: &str) -> &'static str {
    let (x, y) = (base_numeral(a), base_numeral(b));
    let flat_a = a.starts_with('b');
    match (x.as_str(), y.as_str()) {
        ("V", "I") if !flat_a => "authentic",
        ("VII", "I") if !flat_a && (a.contains('°') || a.contains('ø')) => "leading_tone",
        ("VII", "I") => "subtonic (Aeolian ♭VII→i)",
        ("IV", "I") => "plagal",
        ("V", "VI") => "deceptive",
        ("VI", "VII") if b.starts_with('b') || a.starts_with('b') => "none",
        (_, "V") => "half",
        _ => "none",
    }
}

/// Substitutions for a chord in a key, each with its explanation.
pub fn substitutions(key: Key, symbol: &str) -> Result<Value, String> {
    let c = ChordSym::parse(symbol)?;
    let (roman, func) = roman_of(key, &c);
    let mut out = vec![];
    let root = c.root;
    if c.kind.id == "dominant7" || (c.kind.id == "major" && base_numeral(&roman) == "V") {
        let tri = root.up(4, 6); // a diminished fifth away: Db for G
        let tri = if tri.acc.abs() > 1 { t::Note::simple(tri.pc(), true) } else { tri };
        out.push(json!({ "chord": format!("{}7", tri.name()), "kind": "tritone substitution", "why": "shares the tritone (3rd and 7th) of the dominant; bass moves by semitone to the tonic" }));
        out.push(json!({ "chord": format!("{}dim7", root.up(2, 4).name()), "kind": "leading-tone diminished", "why": "vii°7 has the same leading tone and 7th" }));
        out.push(json!({ "chord": format!("{}7sus4", root.name()), "kind": "suspended dominant", "why": "softer dominant; the 4th resolves to the 3rd" }));
    }
    let dia: Vec<t::Chord> = (0..7).map(|d| t::diatonic(key, d, false)).collect();
    if let Some(d) = dia.iter().position(|x| x.root.pc() == root.pc()) {
        // relative / mediant substitutes share two notes
        for o in [2usize, 5] {
            let alt = &dia[(d + o) % 7];
            let shared = alt.pcs().iter().filter(|p| c.pcs().contains(p)).count();
            if shared >= 2 && alt.root.pc() != root.pc() {
                out.push(json!({ "chord": alt.symbol(), "kind": "diatonic substitute (shares two notes)", "why": format!("{} and {} share {shared} notes", c.symbol(), alt.symbol()) }));
            }
        }
        if let Some(par) = key.parallel() {
            let b = t::diatonic(par, d as u8, false);
            if b.symbol() != dia[d].symbol() {
                out.push(json!({ "chord": b.symbol(), "kind": "modal interchange", "why": format!("same degree in {}", par.name()) }));
            }
        }
        // secondary dominant leading to this chord
        let v = root.up(4, 7);
        out.push(json!({ "chord": format!("{}7", v.name()), "kind": "secondary dominant (before this chord)", "why": format!("V7/{} prepares {}", roman, c.symbol()) }));
    }
    if c.kind.id == "major" || c.kind.id == "minor" {
        out.push(json!({ "chord": format!("{}sus2", root.name()), "kind": "suspension", "why": "open colour, no third" }));
        out.push(json!({ "chord": format!("{}{}", root.name(), if c.kind.id == "minor" { "m7" } else { "maj7" }), "kind": "extension", "why": "adds the seventh" }));
    }
    Ok(json!({ "chord": c.symbol(), "roman": roman, "function": func, "substitutions": out, "source": "docs/knowledge/music-theory/harmony.md §6–8" }))
}

// ---------- generation ----------

/// Progressions from the knowledge base (harmony.md §6), used as candidates among generated ones.
pub const COMMON_MINOR: [&str; 9] = ["i-VI-III-VII", "i-VII-VI-VII", "i-VI-VII-i", "i-iv-VII-III", "i-iv-v-i", "i-VI-iv-V", "i-iv", "i-VII", "i-bII"];
pub const COMMON_MAJOR: [&str; 5] = ["I-V-vi-IV", "vi-IV-I-V", "ii7-V7-Imaj7", "I-IV", "I-bVII-IV-I"];

/// Chord vocabulary and transition weights per key type. (from, to, weight)
fn grammar(minor: bool) -> (Vec<&'static str>, Vec<(&'static str, &'static str, f64)>) {
    if minor {
        let v = vec!["i", "ii°", "III", "iv", "v", "V", "VI", "VII", "bII"];
        let e = vec![
            ("i", "VI", 3.0), ("i", "iv", 3.0), ("i", "VII", 3.0), ("i", "III", 1.5), ("i", "v", 1.0), ("i", "bII", 0.5), ("i", "ii°", 0.5),
            ("VI", "III", 2.5), ("VI", "VII", 3.0), ("VI", "iv", 1.5), ("VI", "V", 1.0), ("VI", "i", 1.0),
            ("III", "VII", 3.0), ("III", "VI", 1.5), ("III", "iv", 1.5),
            ("VII", "i", 3.0), ("VII", "VI", 2.0), ("VII", "III", 2.0), ("VII", "iv", 1.0),
            ("iv", "VII", 2.5), ("iv", "v", 1.5), ("iv", "V", 2.0), ("iv", "i", 2.0), ("iv", "VI", 1.5),
            ("v", "i", 2.0), ("v", "VI", 1.5), ("V", "i", 3.0), ("V", "VI", 1.0),
            ("ii°", "V", 3.0), ("ii°", "v", 1.0), ("bII", "i", 3.0), ("bII", "VII", 1.0),
        ];
        (v, e)
    } else {
        let v = vec!["I", "ii", "iii", "IV", "V", "vi", "vii°", "bVII", "iv", "bVI"];
        let e = vec![
            ("I", "IV", 3.0), ("I", "V", 3.0), ("I", "vi", 3.0), ("I", "ii", 1.5), ("I", "iii", 1.0), ("I", "bVII", 1.0), ("I", "bVI", 0.5),
            ("ii", "V", 3.5), ("ii", "IV", 1.0), ("ii", "vii°", 0.5),
            ("iii", "vi", 2.5), ("iii", "IV", 2.0),
            ("IV", "V", 3.0), ("IV", "I", 2.5), ("IV", "ii", 1.0), ("IV", "vi", 1.0), ("IV", "iv", 1.0),
            ("V", "I", 3.5), ("V", "vi", 2.0), ("V", "IV", 1.0),
            ("vi", "IV", 3.0), ("vi", "ii", 2.0), ("vi", "V", 1.5), ("vi", "iii", 1.0),
            ("vii°", "I", 3.0), ("bVII", "IV", 2.5), ("bVII", "I", 2.0), ("iv", "I", 2.5), ("bVI", "bVII", 3.0), ("bVI", "IV", 1.0),
        ];
        (v, e)
    }
}

/// Preferences of a style/mood for chords (multipliers on transitions into them) and loop length.
pub struct Flavor {
    pub favour: Vec<(&'static str, f64)>,
    pub length: usize,
    pub sevenths: f64,
    pub ninths: f64,
    pub sus: f64,
    pub repeat_ok: bool,
    pub note: &'static str,
}

pub fn flavor(style: &str, moods: &[String]) -> Flavor {
    let mut f = match style {
        "techno" | "minimal" => Flavor { favour: vec![("i", 2.0), ("VII", 1.5), ("bII", 1.5), ("iv", 1.3)], length: 2, sevenths: 0.1, ninths: 0.0, sus: 0.1, repeat_ok: true, note: "sparse, single tonal centre (genres.md §1)" },
        "melodic_techno" => Flavor { favour: vec![("VI", 1.8), ("III", 1.5), ("VII", 1.5), ("iv", 1.3)], length: 4, sevenths: 0.15, ninths: 0.05, sus: 0.15, repeat_ok: false, note: "minor, cinematic, about every 2 bars (genres.md §2)" },
        "progressive_house" | "trance" => Flavor { favour: vec![("VI", 1.8), ("III", 1.5), ("VII", 1.8), ("vi", 1.8), ("IV", 1.5)], length: 4, sevenths: 0.1, ninths: 0.0, sus: 0.25, repeat_ok: false, note: "epic minor loops, sus colours (harmony.md §6)" },
        "deep_house" => Flavor { favour: vec![("iv", 1.8), ("ii", 1.8), ("v", 1.5), ("vi", 1.5)], length: 4, sevenths: 0.8, ninths: 0.5, sus: 0.05, repeat_ok: true, note: "7th/9th colour (harmony.md §6)" },
        "house" | "tech_house" => Flavor { favour: vec![("ii", 1.8), ("vi", 1.5), ("IV", 1.5), ("iv", 1.3)], length: 4, sevenths: 0.6, ninths: 0.2, sus: 0.05, repeat_ok: false, note: "7ths, stabs (composition-guide §4)" },
        "edm" => Flavor { favour: vec![("V", 1.5), ("vi", 1.8), ("IV", 1.8), ("VI", 1.5), ("VII", 1.5)], length: 4, sevenths: 0.05, ninths: 0.0, sus: 0.2, repeat_ok: false, note: "pop loops (harmony.md §6)" },
        "cinematic" => Flavor { favour: vec![("bVI", 2.0), ("bVII", 1.8), ("iv", 1.8), ("VI", 1.8), ("III", 1.5)], length: 4, sevenths: 0.1, ninths: 0.1, sus: 0.3, repeat_ok: false, note: "modal interchange, plagal motion (harmony.md §8)" },
        "ambient" => Flavor { favour: vec![("IV", 1.8), ("vi", 1.5), ("VI", 1.5), ("III", 1.3)], length: 2, sevenths: 0.3, ninths: 0.4, sus: 0.5, repeat_ok: true, note: "slow, suspended and added-9 colours" },
        _ => Flavor { favour: vec![], length: 4, sevenths: 0.2, ninths: 0.05, sus: 0.1, repeat_ok: false, note: "neutral" },
    };
    for m in moods {
        match m.as_str() {
            "dark" => f.favour.extend([("bII", 2.0), ("v", 1.5), ("iv", 1.5)]),
            "emotional" | "melancholic" => f.favour.extend([("VI", 1.6), ("III", 1.4), ("iv", 1.4), ("vi", 1.6)]),
            "hypnotic" => {
                f.length = f.length.min(2);
                f.repeat_ok = true;
            }
            "uplifting" | "happy" => f.favour.extend([("VII", 1.4), ("III", 1.4), ("IV", 1.5), ("V", 1.5)]),
            "tense" => f.favour.extend([("V", 1.8), ("ii°", 1.8), ("vii°", 1.8)]),
            "cinematic" => f.favour.extend([("bVI", 1.8), ("iv", 1.6), ("VI", 1.6)]),
            "minimal" => f.length = f.length.min(2),
            _ => {}
        }
    }
    f
}

/// Generate `count` different progressions in a key for a style and moods. Deterministic for a seed.
pub fn generate(key: Key, style: &str, moods: &[String], length: Option<usize>, count: usize, seed: u64) -> Vec<Value> {
    let minor = key.mode.minor_third();
    let (vocab, edges) = grammar(minor);
    let fl = flavor(style, moods);
    let len = length.unwrap_or(fl.length).clamp(1, 8);
    let tonic = if minor { "i" } else { "I" };
    let fav = |c: &str| fl.favour.iter().filter(|(x, _)| *x == c).map(|(_, w)| *w).product::<f64>();
    let weight = |a: &str, b: &str| edges.iter().find(|(x, y, _)| *x == a && *y == b).map(|e| e.2).unwrap_or(0.0);
    let mut rng = Rng::new(seed.wrapping_mul(7919).wrapping_add(17));
    let mut cands: Vec<(f64, Vec<&str>, &str)> = vec![];
    // generated by walking the grammar
    for _ in 0..400 {
        let mut p = vec![tonic];
        while p.len() < len {
            let last = *p.last().unwrap();
            let opts: Vec<(&str, f64)> = vocab.iter().map(|b| (*b, weight(last, b) * fav(b))).filter(|x| x.1 > 0.0 && (fl.repeat_ok || x.0 != last)).collect();
            if opts.is_empty() {
                break;
            }
            let total: f64 = opts.iter().map(|x| x.1).sum();
            let mut r = (rng.below(10_000) as f64 / 10_000.0) * total;
            let mut pick = opts[0].0;
            for (c, w) in &opts {
                if r < *w {
                    pick = c;
                    break;
                }
                r -= w;
            }
            p.push(pick);
        }
        if p.len() == len {
            cands.push((0.0, p, "generated"));
        }
    }
    // common progressions of the knowledge base, as candidates (not the only source)
    for c in if minor { &COMMON_MINOR[..] } else { &COMMON_MAJOR[..] } {
        let toks: Vec<&str> = c.split('-').collect();
        if toks.len() == len {
            cands.push((0.0, toks, "common (harmony.md §6)"));
        }
    }
    for c in cands.iter_mut() {
        let p = &c.1;
        let mut s: f64 = p.windows(2).map(|w| (weight(w[0], w[1]).max(0.2) * fav(w[1])).ln()).sum();
        if len > 1 {
            s += (weight(p[len - 1], p[0]).max(0.1)).ln() * 1.5; // the loop must lead back to the start
        }
        let distinct: std::collections::BTreeSet<&&str> = p.iter().collect();
        s += distinct.len() as f64 * 0.4;
        if c.2 != "generated" {
            s -= 0.3; // prefer fresh material at equal quality
        }
        c.0 = s;
    }
    cands.sort_by(|a, b| b.0.total_cmp(&a.0));
    let mut seen = std::collections::HashSet::new();
    let mut out = vec![];
    let mut erng = Rng::new(seed.wrapping_add(99));
    for (score, p, origin) in cands {
        let k = p.join("-");
        if !seen.insert(k.clone()) {
            continue;
        }
        let chords: Vec<t::Chord> = match t::progression(key, &k, false) {
            Ok(c) => c,
            Err(_) => continue,
        };
        // colours: sevenths/ninths/sus as the style asks
        let symbols: Vec<String> = chords
            .iter()
            .map(|c| {
                let base = c.symbol();
                let r = c.root.name();
                let roll = erng.below(100) as f64 / 100.0;
                match c.quality {
                    t::Quality::Minor if roll < fl.ninths => format!("{r}m9"),
                    t::Quality::Minor if roll < fl.sevenths => format!("{r}m7"),
                    t::Quality::Major if c.degree == 4 && roll < fl.sevenths => format!("{r}7"),
                    t::Quality::Major if roll < fl.ninths => format!("{r}add9"),
                    t::Quality::Major if roll < fl.sevenths => format!("{r}maj7"),
                    t::Quality::Major | t::Quality::Minor if roll > 1.0 - fl.sus => format!("{r}sus2"),
                    _ => base,
                }
            })
            .collect();
        let rows: Vec<(String, &str)> = symbols.iter().map(|s| roman_of(key, &ChordSym::parse(s).unwrap())).collect();
        out.push(json!({
            "roman": k, "chords": symbols, "functions": rows.iter().map(|r| r.1).collect::<Vec<_>>(),
            "loop_cadence": if len > 1 { cadence_name(&rows[len - 1].0, &rows[0].0) } else { "none" },
            "origin": origin, "score": (score * 100.0).round() / 100.0, "style_note": fl.note,
        }));
        if out.len() >= count {
            break;
        }
    }
    out
}

/// Mode-specific default progressions (characteristic chord of the mode), for modal requests.
pub fn modal(key: Key) -> Option<&'static str> {
    match key.mode {
        Mode::Dorian => Some("i-IV-i-IV"),
        Mode::Phrygian => Some("i-II-i-VII"),
        Mode::Lydian => Some("I-II-I-II"),
        Mode::Mixolydian => Some("I-VII-IV-I"),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::music::keys::parse_key;

    fn s(v: &[&str]) -> Vec<String> {
        v.iter().map(|x| x.to_string()).collect()
    }

    #[test]
    fn analysis_roman_functions_secondary_and_borrowed() {
        let a = analyze(Some(parse_key("A minor").unwrap()), &s(&["Am", "F", "C", "G"])).unwrap();
        let r: Vec<&str> = a["chords"].as_array().unwrap().iter().map(|c| c["roman"].as_str().unwrap()).collect();
        assert_eq!(r, ["i", "VI", "III", "VII"]);
        assert_eq!(a["loop_cadence"], "subtonic (Aeolian ♭VII→i)");
        let b = analyze(Some(parse_key("C major").unwrap()), &s(&["C", "A7", "Dm7", "G7", "Cmaj7", "Fm", "Bb"])).unwrap();
        let rows = b["chords"].as_array().unwrap();
        assert_eq!(rows[1]["roman"], "V7/ii");
        assert_eq!(rows[1]["function"], "secondary_dominant");
        assert_eq!(rows[3]["roman"], "V7");
        assert_eq!(rows[5]["function"], "borrowed", "iv from the parallel minor");
        assert_eq!(rows[6]["roman"], "bVII");
        assert!(b["cadences"].as_array().unwrap().iter().any(|c| c["type"] == "authentic"));
        // key estimated from the chords when none is given
        assert_eq!(analyze(None, &s(&["Dm7", "G7", "Cmaj7"])).unwrap()["key"], "C major");
        assert_eq!(analyze(None, &s(&["Am", "F", "C", "G"])).unwrap()["key"], "A minor", "starts on Am: A minor over C major");
        assert_eq!(analyze(None, &s(&["Dm", "E7", "Am"])).unwrap()["key"], "A minor", "harmonic-minor dominant resolving");
    }

    #[test]
    fn substitutions_explained() {
        let x = substitutions(parse_key("C major").unwrap(), "G7").unwrap();
        let subs: Vec<&str> = x["substitutions"].as_array().unwrap().iter().map(|s| s["chord"].as_str().unwrap()).collect();
        assert!(subs.contains(&"Db7"), "tritone substitution {subs:?}");
        let y = substitutions(parse_key("C major").unwrap(), "F").unwrap();
        assert!(y["substitutions"].as_array().unwrap().iter().any(|s| s["chord"] == "Fm" && s["kind"] == "modal interchange"));
        assert!(y["substitutions"].as_array().unwrap().iter().any(|s| s["chord"] == "Dm"));
    }

    #[test]
    fn generated_progressions_are_coherent_varied_and_styled() {
        let am = parse_key("A minor").unwrap();
        let g = generate(am, "melodic_techno", &["emotional".into()], None, 6, 1);
        assert_eq!(g.len(), 6, "six different alternatives");
        let romans: std::collections::HashSet<&str> = g.iter().map(|x| x["roman"].as_str().unwrap()).collect();
        assert_eq!(romans.len(), 6);
        assert!(g.iter().any(|x| x["origin"] == "generated"), "not only presets");
        for p in &g {
            assert!(p["roman"].as_str().unwrap().starts_with('i'), "starts on the tonic: {p}");
            for c in p["chords"].as_array().unwrap() {
                assert!(ChordSym::parse(c.as_str().unwrap()).is_ok());
            }
        }
        // deterministic for a seed, different for another
        assert_eq!(generate(am, "melodic_techno", &[], None, 3, 7), generate(am, "melodic_techno", &[], None, 3, 7));
        let deep = generate(parse_key("C major").unwrap(), "deep_house", &[], Some(4), 5, 2);
        let colour = deep.iter().flat_map(|x| x["chords"].as_array().unwrap().clone()).filter(|c| { let s = c.as_str().unwrap(); s.contains('7') || s.contains('9') }).count();
        assert!(colour >= 5, "deep house uses 7th/9th colour: {deep:?}");
        let hyp = generate(am, "techno", &["hypnotic".into()], None, 3, 3);
        assert!(hyp.iter().all(|x| x["chords"].as_array().unwrap().len() <= 2), "hypnotic: one or two chords");
        // every style produces something
        for st in ["techno", "melodic_techno", "progressive_house", "deep_house", "house", "edm", "cinematic", "ambient", "trance", "minimal", "tech_house"] {
            assert!(!generate(am, st, &[], None, 2, 5).is_empty(), "{st}");
            assert!(!generate(parse_key("D major").unwrap(), st, &[], None, 2, 5).is_empty(), "{st} major");
        }
    }
}
