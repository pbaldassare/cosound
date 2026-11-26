//! Musical Parameter Extraction: general, rhythm, harmony, sound, structure and technical parameters, each with the
//! words it came from and whether it was said (explicit) or deduced (inferred). Missing values are never invented:
//! "abbassa leggermente il volume" gives a direction and a qualitative amount, not a number of dB.
//! It reuses the vocabularies of the music engine (genres, keys, moods – `music::request`, `music::keys`,
//! `music::scales`, `music::chords`) and of the plug-in engine (sound qualities – `plugins::design`).
use super::lexicon::SECTIONS;
use super::text::{number, Text};
use serde::Serialize;
use serde_json::{json, Value};

#[derive(Clone, Debug, Serialize, PartialEq)]
pub struct Param {
    pub group: &'static str,
    pub name: String,
    pub value: Value,
    pub unit: Option<String>,
    pub evidence: Option<String>,
    pub source: &'static str,
    pub confidence: f64,
}

pub fn p(group: &'static str, name: &str, value: Value, evidence: Option<&str>, source: &'static str, confidence: f64) -> Param {
    Param { group, name: name.into(), value, unit: None, evidence: evidence.map(String::from), source, confidence }
}

#[derive(Default, Debug)]
pub struct Extracted {
    pub params: Vec<Param>,
    /// (code, message) problems found while extracting: contradictions, impossible values …
    pub problems: Vec<(&'static str, String)>,
    /// Sound qualities as (quality, signed weight, keep) – shared with enhancement
    pub qualities: Vec<(String, f64, bool)>,
    pub sections: Vec<String>,
}

/// Extra sound/energy qualities the plug-in vocabulary does not cover.
const EXTRA_QUALITIES: [(&str, &[&str]); 6] = [
    ("dynamic", &["dinamico", "dinamica", "dinamici", "dynamic", "more dynamic"]),
    ("energetic", &["energico", "energica", "energici", "energetic", "energia", "energy", "carico", "carica"]),
    ("spacey", &["spaziale", "spaziali", "spacey", "spacious", "arioso"]),
    ("beautiful", &["bello", "bella", "belli", "beautiful", "nice", "migliore", "better"]),
    ("louder", &["più forte", "piu forte", "louder", "più alto di volume"]),
    ("quieter", &["più piano", "piu piano", "quieter", "più basso di volume", "più leggero"]),
];

const SOUND_TYPES: [(&str, &[&str]); 12] = [
    ("acid", &["acid", "303", "acido"]),
    ("reese", &["reese"]),
    ("supersaw", &["supersaw", "super saw"]),
    ("pluck", &["pluck", "plucked", "pizzicato"]),
    ("sub", &["sub", "sub bass", "808"]),
    ("fm", &["fm", "fm synth"]),
    ("analog", &["analogico", "analogica", "analog", "analogue", "vintage"]),
    ("saw", &["dente di sega", "saw", "sawtooth"]),
    ("organic", &["organico", "acustico", "organic", "acoustic"]),
    ("distorted", &["distorto", "distorta", "distorted", "saturo"]),
    ("glassy", &["cristallino", "glassy", "vetroso"]),
    ("rolling", &["rolling", "rotolante"]),
];

const RHYTHM: [(&str, &[&str]); 8] = [
    ("four_on_the_floor", &["four on the floor", "cassa dritta", "4 4 dritto", "cassa in quattro", "dritto"]),
    ("breakbeat", &["breakbeat", "break beat", "spezzato"]),
    ("half_time", &["half time", "half-time", "metà tempo"]),
    ("shuffle", &["shuffle", "shuffled", "terzinato"]),
    ("syncopation", &["sincopato", "sincopata", "sincopati", "sincope", "syncopated", "syncopation", "off beat", "offbeat", "in levare"]),
    ("groove", &["groove", "groovy", "più groove", "groovoso"]),
    ("polyrhythm", &["poliritmo", "poliritmico", "polyrhythm", "polyrhythmic"]),
    ("euclidean", &["euclideo", "euclidean"]),
];

const DENSITY: [(&str, &[&str]); 2] = [
    ("increase", &["più denso", "più densa", "più fitto", "più fitta", "più note", "più pieno", "denser", "busier", "more notes", "fuller"]),
    ("decrease", &["meno denso", "meno densa", "meno note", "più vuoto", "più semplice", "più essenziale", "sparser", "fewer notes", "simpler", "minimale"]),
];

fn direction(t: &Text) -> Option<&'static str> {
    let down = t.any(&["abbassa", "riduci", "diminuisci", "togli", "meno", "lower", "reduce", "decrease", "less", "turn down", "cut"]);
    let up = t.any(&["alza", "aumenta", "aggiungi", "più", "piu", "raise", "increase", "boost", "more", "turn up", "add"]);
    match (up, down) {
        (true, false) => Some("increase"),
        (false, true) => Some("decrease"),
        (true, true) => {
            // "abbassa il volume … più …": the verb wins
            if t.any(&["abbassa", "lower", "riduci", "reduce", "turn down"]) {
                Some("decrease")
            } else {
                Some("increase")
            }
        }
        _ => None,
    }
}

fn amount_word(t: &Text) -> Option<&'static str> {
    if t.any(&["leggermente", "un po", "un pochino", "appena", "poco", "slightly", "a bit", "a little", "a touch"]) {
        Some("slight")
    } else if t.any(&["molto", "tanto", "decisamente", "parecchio", "very", "a lot", "much", "way more"]) {
        Some("large")
    } else {
        None
    }
}

/// Numbers followed by a unit word, with the words they came from. "un paio di decibel" → (2, approximate).
fn quantities(t: &Text, units: &[&str]) -> Vec<(f64, bool, String)> {
    let mut out = vec![];
    for i in 0..t.words.len() {
        let w = t.words[i].as_str();
        let (n_idx, unit_idx) = if units.contains(&w) {
            // "2 db", "un paio di db", "-3 db"
            let k = if i >= 2 && t.words[i - 1] == "di" { i - 2 } else if i >= 1 { i - 1 } else { continue };
            (k, i)
        } else {
            continue;
        };
        let nw = t.words[n_idx].as_str();
        if let Some(n) = number(nw) {
            let approx = ["paio", "couple", "qualche", "few"].contains(&nw);
            out.push((n, approx, format!("{} {}", t.words[n_idx..=unit_idx].join(" "), "").trim().to_string()));
        } else if ["qualche", "alcuni", "few", "some"].contains(&nw) {
            out.push((2.0, true, format!("{nw} {}", t.words[unit_idx])));
        }
    }
    out
}

pub fn extract(t: &Text) -> Extracted {
    let mut x = Extracted::default();
    let req = crate::music::request::parse(&t.raw);
    // ---- general
    for (id, w) in &req.styles {
        x.params.push(p("general", "genre", json!({ "id": id, "weight": w }), Some(id.replace('_', " ").as_str()), "explicit", 0.95));
    }
    // BPM only when the word bpm (or "tempo") is next to the number: "100 Hz" or "8 battute" are not tempos
    let mut bpms = vec![];
    for i in 0..t.words.len() {
        let w = t.words[i].as_str();
        let n = number(w).filter(|_| w.chars().next().is_some_and(|c| c.is_ascii_digit()));
        let Some(n) = n else { continue };
        let next = t.words.get(i + 1).map(String::as_str).unwrap_or("");
        let prev = if i > 0 { t.words[i - 1].as_str() } else { "" };
        if next == "bpm" || prev == "bpm" || prev == "tempo" {
            bpms.push((n, format!("{w} bpm")));
        }
    }
    bpms.dedup_by(|a, b| a.0 == b.0);
    if bpms.len() > 1 {
        x.problems.push(("contradictory", format!("two tempos in the same request: {}", bpms.iter().map(|b| b.0.to_string()).collect::<Vec<_>>().join(" and "))));
    }
    if let Some((n, ev)) = bpms.first() {
        if !(20.0..=300.0).contains(n) {
            x.problems.push(("impossible", format!("{n} BPM is outside what DAWs handle (20–300)")));
        } else {
            let mut prm = p("general", "bpm", json!(n), Some(ev), "explicit", 1.0);
            prm.unit = Some("BPM".into());
            x.params.push(prm);
        }
    }
    if t.any(&["più veloce", "accelera", "faster", "speed up"]) && t.any(&["più lento", "rallenta", "slower", "slow down"]) {
        x.problems.push(("contradictory", "faster and slower at the same time".into()));
    } else if t.any(&["più veloce", "accelera", "faster", "speed up"]) {
        x.params.push(p("general", "tempo_change", json!("increase"), Some("faster"), "explicit", 0.9));
    } else if t.any(&["più lento", "rallenta", "slower", "slow down"]) {
        x.params.push(p("general", "tempo_change", json!("decrease"), Some("slower"), "explicit", 0.9));
    }
    if let Some(k) = &req.key {
        x.params.push(p("general", "key", json!(k), Some(k.as_str()), "explicit", 0.95));
    }
    // two different keys named
    let key_words = ["maggiore", "minore", "major", "minor"];
    if key_words.iter().map(|k| t.norm.matches(&format!(" {k} ")).count()).sum::<usize>() >= 2 {
        let firsts: Vec<String> = t.raw.split([',', ';']).filter_map(|part| crate::music::request::parse(part).key).collect();
        let mut uniq = firsts.clone();
        uniq.dedup();
        if uniq.len() > 1 {
            x.problems.push(("contradictory", format!("two keys in the same request: {}", uniq.join(" and "))));
        }
    }
    if let Some(b) = req.bars.filter(|_| t.any(&["battute", "battuta", "bars", "bar", "misure"])) {
        x.params.push(p("general", "length_bars", json!(b), Some(&format!("{b} bars")), "explicit", 1.0));
    }
    for (n, _, ev) in quantities(t, &["minuti", "minuto", "minutes", "minute", "min"]) {
        let mut prm = p("general", "duration", json!(n * 60.0), Some(&ev), "explicit", 0.9);
        prm.unit = Some("s".into());
        x.params.push(prm);
    }
    for (n, _, ev) in quantities(t, &["secondi", "secondo", "seconds", "sec"]) {
        let mut prm = p("general", "duration", json!(n), Some(&ev), "explicit", 0.9);
        prm.unit = Some("s".into());
        x.params.push(prm);
    }
    for m in &req.moods {
        if !["syncopated", "aggressive"].contains(&m.as_str()) {
            x.params.push(p("general", "mood", json!(m), Some(m.as_str()), "explicit", 0.9));
        }
    }
    // references: the words after "come"/"nello stile di"/"like", kept as said (never looked up or completed)
    for cue in ["nello stile di", "nello stile dei", "in the style of", "simile a", "similar to", "ispirato a", "inspired by", "alla maniera di"] {
        let after = t.after(cue, 4);
        if !after.is_empty() {
            let name: Vec<String> = after.into_iter().take_while(|w| !["a", "con", "e", "with", "and", "at", "in", "ma", "but"].contains(&w.as_str())).collect();
            if !name.is_empty() {
                x.params.push(p("general", "reference", json!(name.join(" ")), Some(cue), "explicit", 0.7));
            }
        }
    }
    // ---- rhythm
    for (id, words) in RHYTHM {
        if let Some(w) = t.first(words) {
            x.params.push(p("rhythm", if id == "groove" || id == "syncopation" { id } else { "pattern" }, json!(if id == "groove" || id == "syncopation" { json!(direction(t).unwrap_or("increase")) } else { json!(id) }), Some(w), "explicit", 0.85));
        }
    }
    for (n, _, ev) in quantities(t, &["%", "percento", "percent"]).into_iter().filter(|_| t.any(&["swing"])) {
        let mut prm = p("rhythm", "swing", json!(n), Some(&format!("swing {ev}")), "explicit", 1.0);
        prm.unit = Some("%".into());
        x.params.push(prm);
    }
    if t.has("swing") && !x.params.iter().any(|q| q.name == "swing") {
        x.params.push(p("rhythm", "swing", json!(direction(t).unwrap_or("increase")), Some("swing"), "explicit", 0.8));
    }
    for (dir, words) in DENSITY {
        if let Some(w) = t.first(words) {
            x.params.push(p("rhythm", "density", json!(dir), Some(w), "explicit", 0.8));
        }
    }
    if let Some(w) = t.first(&["più intenso", "più intensa", "intensità", "more intense", "intensity", "più incalzante"]) {
        x.params.push(p("rhythm", "intensity", json!("increase"), Some(w), "explicit", 0.8));
    }
    // ---- harmony
    for (cue, len) in [("scala", 3usize), ("scale", 3), ("modo", 2), ("mode", 2)] {
        let after = t.after(cue, len);
        for n in (1..=after.len()).rev() {
            let cand = after[..n].join(" ");
            if let Some(s) = crate::music::scales::find(&cand) {
                x.params.push(p("harmony", "scale", json!(s.id), Some(&format!("{cue} {cand}")), "explicit", 0.9));
                break;
            }
        }
    }
    // chord symbols and roman numerals only as runs (≥2 chords, ≥3 numerals): "A track" or "i miei" are words, not harmony
    let tokens: Vec<String> = t.raw.split(|c: char| c.is_whitespace() || [',', ';', '–', '-', '|'].contains(&c))
        .map(|w| w.trim_matches(|c: char| !c.is_alphanumeric() && c != '#' && c != '/' && c != '♭' && c != '°').to_string())
        .filter(|w| !w.is_empty())
        .collect();
    let is_chord = |w: &str| w.chars().next().is_some_and(|c| ('A'..='G').contains(&c)) && w.len() <= 8 && crate::music::chords::ChordSym::parse(w).is_ok();
    let is_roman = |w: &str| w.len() <= 5 && w.chars().all(|c| "iIvV♭b#°".contains(c)) && w.chars().any(|c| "iIvV".contains(c));
    let run = |f: &dyn Fn(&str) -> bool, min: usize| -> Vec<String> {
        let (mut best, mut cur): (Vec<String>, Vec<String>) = (vec![], vec![]);
        for w in tokens.iter().chain(std::iter::once(&String::new())) {
            if !w.is_empty() && f(w) {
                cur.push(w.clone());
            } else {
                if cur.len() >= min && cur.len() > best.len() {
                    best = cur.clone();
                }
                cur.clear();
            }
        }
        best
    };
    let chords = run(&is_chord, 2);
    if !chords.is_empty() {
        x.params.push(p("harmony", "chords", json!(chords), Some(&chords.join(" ")), "explicit", 0.9));
    }
    let romans = run(&is_roman, 3);
    if !romans.is_empty() {
        x.params.push(p("harmony", "progression", json!(romans.join("-")), Some(&romans.join(" ")), "explicit", 0.85));
    }
    if let Some(w) = t.first(&["più tensione", "tensione", "più teso", "più tesa", "tension", "tense", "more tension"]) {
        x.params.push(p("harmony", "tension", json!(if t.any(&["meno tensione", "less tension"]) { "decrease" } else { "increase" }), Some(w), "explicit", 0.8));
    }
    if let Some(w) = t.first(&["risolvi", "risoluzione", "risolva", "che risolva", "resolve", "resolution", "resolves"]) {
        x.params.push(p("harmony", "resolution", json!("resolve"), Some(w), "explicit", 0.8));
    }
    // ---- sound
    let mut quals = crate::plugins::design::parse_request(&t.raw);
    for (id, words) in EXTRA_QUALITIES {
        if let Some(w) = t.first(words) {
            if !quals.iter().any(|q| q.0 == id) {
                let neg = t.has(&format!("meno {w}")) || t.has(&format!("less {w}"));
                let keep = t.any(&[&format!("mantieni {w}"), &format!("keep it {w}")]);
                quals.push((id.to_string(), if neg { -1.0 } else { 1.0 }, keep));
            }
        }
    }
    // the plug-in vocabulary reads "potente" as aggressive; "spaziale" is ours (space), not "atmospheric" twice
    for (q, w, keep) in &quals {
        let ev = q.as_str();
        x.params.push(p("sound", q, json!({ "direction": if *keep { "keep" } else if *w < 0.0 { "decrease" } else { "increase" }, "amount": if w.abs() > 1.2 { "large" } else if w.abs() < 0.8 { "slight" } else { "normal" } }), Some(ev), "explicit", 0.85));
    }
    let opposite = [("bright", "dark"), ("aggressive", "soft"), ("wide", "mono"), ("long", "short"), ("louder", "quieter")];
    for (a, b) in opposite {
        let has = |q: &str| quals.iter().any(|x| x.0 == q && x.1 > 0.0 && !x.2);
        if has(a) && has(b) {
            x.problems.push(("contradictory", format!("{a} and {b} at the same time")));
        }
    }
    x.qualities = quals;
    for (id, words) in SOUND_TYPES {
        if let Some(w) = t.first(words) {
            if id == "sub" && t.any(&["sub bass", "sub"]) && !t.any(&["808"]) {
                continue; // "sub" names the element (bass), not a sound type
            }
            x.params.push(p("sound", "sound_type", json!(id), Some(w), "explicit", 0.8));
        }
    }
    if t.any(&["stereo", "stereofonico", "image", "immagine stereo", "più largo", "più larga", "wider", "mono"]) {
        let narrow = t.any(&["mono", "più stretto", "narrower", "meno largo"]);
        x.params.push(p("sound", "stereo_image", json!(if narrow { "narrower" } else { "wider" }), Some(if narrow { "mono" } else { "stereo" }), "explicit", 0.85));
    }
    // ---- structure
    for (id, words) in SECTIONS {
        if let Some(w) = t.first(words) {
            x.sections.push(id.to_string());
            x.params.push(p("structure", "section", json!(id), Some(w), "explicit", 0.9));
        }
    }
    // ---- technical: dB, Hz, LUFS, octaves/semitones are left to the MIDI editor
    for (n, approx, ev) in quantities(t, &["db", "decibel", "decibels", "dB"]) {
        let dir = direction(t);
        let signed = if n < 0.0 { n } else if dir == Some("decrease") { -n } else { n };
        let mut prm = p("technical", "gain_change", json!({ "db": signed, "approximate": approx }), Some(&ev), "explicit", if approx { 0.8 } else { 1.0 });
        prm.unit = Some("dB".into());
        x.params.push(prm);
    }
    for (n, _, ev) in quantities(t, &["hz", "khz", "hertz"]) {
        let hz = if ev.contains("khz") { n * 1000.0 } else { n };
        if !(10.0..=24000.0).contains(&hz) {
            x.problems.push(("impossible", format!("{hz} Hz is outside the audible range")));
            continue;
        }
        let mut prm = p("technical", "frequency", json!(hz), Some(&ev), "explicit", 1.0);
        prm.unit = Some("Hz".into());
        x.params.push(prm);
    }
    for (n, _, ev) in quantities(t, &["lufs"]) {
        let v = -n.abs();
        if !(-40.0..=-3.0).contains(&v) {
            x.problems.push(("impossible", format!("{v} LUFS is not a usable loudness target")));
            continue;
        }
        let mut prm = p("technical", "loudness_target", json!(v), Some(&ev), "explicit", 1.0);
        prm.unit = Some("LUFS".into());
        x.params.push(prm);
    }
    if t.any(&["volume", "livello", "level", "gain"]) && !x.params.iter().any(|q| q.name == "gain_change") {
        if let Some(d) = direction(t) {
            x.params.push(p("technical", "gain_change", json!({ "direction": d, "amount": amount_word(t).unwrap_or("unspecified") }), Some("volume"), "explicit", 0.9));
        }
    }
    x
}

#[cfg(test)]
mod tests {
    use super::super::text::normalize;
    use super::*;

    fn get<'a>(x: &'a Extracted, name: &str) -> Option<&'a Param> {
        x.params.iter().find(|q| q.name == name)
    }

    #[test]
    fn the_prompt_example() {
        let x = extract(&normalize("Voglio un brano Melodic Techno a 124 BPM, con un basso profondo, un'atmosfera cinematografica e una melodia emozionale."));
        assert_eq!(get(&x, "bpm").unwrap().value, json!(124.0));
        assert_eq!(get(&x, "genre").unwrap().value["id"], "melodic_techno");
        let moods: Vec<&Value> = x.params.iter().filter(|q| q.name == "mood").map(|q| &q.value).collect();
        assert!(moods.contains(&&json!("cinematic")) && moods.contains(&&json!("emotional")), "{moods:?}");
        assert!(x.qualities.iter().any(|q| q.0 == "deep"), "{:?}", x.qualities);
        assert!(get(&x, "key").is_none(), "no key was said: none is invented");
        assert!(x.problems.is_empty());
    }

    #[test]
    fn numbers_units_and_no_invented_values() {
        let x = extract(&normalize("Abbassa il kick di un paio di decibel"));
        let g = get(&x, "gain_change").unwrap();
        assert_eq!((g.value["db"].as_f64(), g.value["approximate"].as_bool()), (Some(-2.0), Some(true)));
        let x = extract(&normalize("Adesso abbassa leggermente il volume"));
        assert_eq!(get(&x, "gain_change").unwrap().value, json!({ "direction": "decrease", "amount": "slight" }), "no dB value invented");
        let x = extract(&normalize("taglia sotto i 100 Hz e porta il master a -14 LUFS"));
        assert!(get(&x, "bpm").is_none(), "100 Hz is not a tempo");
        assert_eq!(get(&x, "frequency").unwrap().value, json!(100.0));
        assert_eq!(get(&x, "loudness_target").unwrap().value, json!(-14.0));
        let x = extract(&normalize("loop di 8 battute a 128bpm in fa diesis minore"));
        assert_eq!((get(&x, "length_bars").unwrap().value.clone(), get(&x, "bpm").unwrap().value.clone(), get(&x, "key").unwrap().value.clone()), (json!(8), json!(128.0), json!("F# minor")));
        let x = extract(&normalize("A track at 500 bpm"));
        assert!(x.problems.iter().any(|p| p.0 == "impossible"));
        let x = extract(&normalize("techno a 128 bpm, anzi a 132 bpm"));
        assert!(x.problems.iter().any(|p| p.0 == "contradictory"));
    }

    #[test]
    fn harmony_rhythm_structure_sound() {
        let x = extract(&normalize("Usa la progressione i VI III VII con accordi Am F C G, groove sincopato e swing al 60%, poi un breakdown e il drop"));
        assert_eq!(get(&x, "progression").unwrap().value, json!("i-VI-III-VII"));
        assert_eq!(get(&x, "chords").unwrap().value, json!(["Am", "F", "C", "G"]));
        assert_eq!(get(&x, "swing").unwrap().value, json!(60.0));
        assert!(get(&x, "syncopation").is_some());
        assert_eq!(x.sections, ["breakdown", "drop"]);
        let x = extract(&normalize("Vorrei un pad più largo e atmosferico"));
        assert!(x.qualities.iter().any(|q| q.0 == "wide") && x.qualities.iter().any(|q| q.0 == "atmospheric"));
        assert_eq!(get(&x, "stereo_image").unwrap().value, json!("wider"));
        let x = extract(&normalize("rendilo più brillante e più scuro"));
        assert!(x.problems.iter().any(|p| p.0 == "contradictory"));
        let x = extract(&normalize("scala dorica, più tensione"));
        assert_eq!(get(&x, "scale").unwrap().value, json!("dorian"));
        assert_eq!(get(&x, "tension").unwrap().value, json!("increase"));
    }
}
