//! Natural-language music requests (Italian and English) → a structured request. Deterministic rules, no AI: what is
//! not understood is returned as such, never guessed. A language model (central Agent / AI Core) may later produce the
//! same structure; the musical engines below always verify the result.
use super::keys::parse_key;

#[derive(Clone, Debug, Default, serde::Serialize)]
pub struct Request {
    pub parts: Vec<String>,
    pub moods: Vec<String>,
    pub key: Option<String>,
    pub bpm: Option<f64>,
    pub bars: Option<u32>,
    pub styles: Vec<(String, f64)>,
    pub melody_style: Option<String>,
    pub bass_style: Option<String>,
    pub arp_pattern: Option<String>,
    pub unknown: Vec<String>,
}

const PARTS: [(&str, &[&str]); 7] = [
    ("melody", &["melodia", "melodie", "melody", "lead", "topline", "tema", "hook"]),
    ("bass", &["basso", "bassline", "bass", "linea di basso", "sub"]),
    ("chords", &["accordi", "chords", "pad", "armonia"]),
    ("progression", &["progressione", "progression", "giro armonico", "giro di accordi"]),
    ("arp", &["arpeggio", "arpeggi", "arp", "sequenza"]),
    ("drums", &["batteria", "drums", "groove", "beat", "ritmica", "pattern ritmico"]),
    ("all", &["brano", "composizione", "composition", "track", "traccia completa", "pezzo", "loop completo"]),
];

const MOODS: [(&str, &[&str]); 14] = [
    ("emotional", &["emozionale", "emozionante", "emotiva", "emotivo", "emotional", "emotive"]),
    ("hypnotic", &["ipnotica", "ipnotico", "ipnotici", "hypnotic"]),
    ("dark", &["scura", "scuro", "dark", "cupa", "cupo"]),
    ("melancholic", &["malinconica", "malinconico", "melancholic", "triste", "sad"]),
    ("cinematic", &["cinematografica", "cinematografico", "cinematica", "cinematic", "epica", "epico", "epic"]),
    ("minimal", &["minimale", "minimalista", "minimal"]),
    ("uplifting", &["energica", "energico", "uplifting", "euforica", "euforico", "felice", "allegra", "allegro", "happy"]),
    ("tense", &["tesa", "teso", "tense", "tensione"]),
    ("simple", &["semplice", "simple"]),
    ("progressive", &["progressiva", "progressivo", "evolutiva", "evolving"]),
    ("rhythmic", &["ritmica", "ritmico", "rhythmic", "percussiva"]),
    ("dreamy", &["sognante", "dreamy", "eterea", "etereo", "ethereal"]),
    ("syncopated", &["sincopata", "sincopato", "syncopated"]),
    ("aggressive", &["aggressiva", "aggressivo", "aggressive"]),
];

const STYLE_WORDS: [(&str, &[&str]); 11] = [
    ("melodic_techno", &["melodic techno", "techno melodica"]),
    ("progressive_house", &["progressive house", "prog house"]),
    ("deep_house", &["deep house"]),
    ("tech_house", &["tech house"]),
    ("techno", &["techno"]),
    ("house", &["house"]),
    ("minimal", &["minimal techno", "minimal"]),
    ("edm", &["edm", "big room"]),
    ("trance", &["trance"]),
    ("ambient", &["ambient"]),
    ("cinematic", &["cinematic", "colonna sonora", "soundtrack", "orchestrale"]),
];

const MODES: [&str; 22] = [
    "maggiore", "minore", "major", "minor", "dorico", "dorica", "dorian", "frigio", "frigia", "phrygian", "lidio", "lidia", "lydian", "misolidio",
    "misolidia", "mixolydian", "locrio", "locrian", "armonica", "harmonic", "melodica", "melodic",
];

pub fn parse(text: &str) -> Request {
    let raw = text.to_lowercase().replace(['’', '\''], " ");
    let t = format!(" {} ", raw.replace([',', '.', ';', ':', '!', '?', '(', ')'], " "));
    let mut r = Request::default();
    let mut used: Vec<String> = vec![];
    let take = |w: &str, used: &mut Vec<String>| used.extend(w.split_whitespace().map(String::from));
    for (id, words) in STYLE_WORDS {
        for w in words.iter() {
            if t.contains(&format!(" {w} ")) && !r.styles.iter().any(|s| s.0 == id) && !(id == "techno" && (t.contains("melodic techno") || t.contains("minimal techno"))) && !(id == "house" && ["deep house", "tech house", "progressive house"].iter().any(|x| t.contains(x))) {
                // "con elementi di X", "with a touch of X": a secondary style
                let secondary = ["elementi di", "elementi", "touch of", "tocco di", "sfumature", "influenze", "hint of", "with some"].iter().any(|m| t.contains(&format!("{m} {w}")));
                r.styles.push((id.to_string(), if secondary { 0.3 } else { 1.0 }));
                take(w, &mut used);
            }
        }
    }
    if r.styles.len() > 1 && r.styles.iter().all(|s| s.1 == 1.0) {
        let n = r.styles.len() as f64;
        r.styles[0].1 = 0.6;
        for s in r.styles.iter_mut().skip(1) {
            s.1 = 0.4 / (n - 1.0);
        }
    }
    for (id, words) in PARTS {
        if words.iter().any(|w| t.contains(&format!(" {w} "))) {
            r.parts.push(id.to_string());
            for w in words.iter().filter(|w| t.contains(&format!(" {w} "))) {
                take(w, &mut used);
            }
        }
    }
    for (id, words) in MOODS {
        if words.iter().any(|w| t.contains(&format!(" {w} "))) {
            r.moods.push(id.to_string());
            for w in words.iter().filter(|w| t.contains(&format!(" {w} "))) {
                take(w, &mut used);
            }
        }
    }
    // key: a note name followed by a mode word ("la minore", "fa diesis minore", "a minor", "re dorico"), or "Am"
    let words: Vec<&str> = t.split_whitespace().collect();
    for i in 0..words.len() {
        for len in [4usize, 3, 2] {
            if i + len > words.len() {
                continue;
            }
            let cand = words[i..i + len].join(" ");
            if MODES.contains(&words[i + len - 1]) {
                if let Ok(k) = parse_key(&cand) {
                    if r.key.is_none() {
                        r.key = Some(k.name());
                        take(&cand, &mut used);
                    }
                }
            }
        }
        // compact chord-style keys: "Am", "F#m", "Ebm" ("am" alone only after "in": it is also an English word)
        let w = words[i];
        let compact = r.key.is_none()
            && (2..=3).contains(&w.chars().count())
            && w.ends_with('m')
            && w.chars().next().is_some_and(|c| "abcdefg".contains(c))
            && !["bpm", "dim"].contains(&w)
            && (w != "am" || (i > 0 && words[i - 1] == "in"));
        if compact {
            if let Ok(k) = parse_key(w) {
                r.key = Some(k.name());
                take(w, &mut used);
            }
        }
    }
    for i in 0..words.len() {
        let num = words[i].trim_end_matches("bpm").parse::<f64>().ok();
        if let Some(n) = num {
            let next = words.get(i + 1).copied().unwrap_or("");
            if next == "bpm" || words[i].ends_with("bpm") || (60.0..=200.0).contains(&n) && !["battute", "bars", "bar", "battuta"].contains(&next) && r.bpm.is_none() {
                if (40.0..=250.0).contains(&n) {
                    r.bpm = Some(n);
                    used.push(words[i].into());
                    if next == "bpm" {
                        used.push("bpm".into());
                    }
                }
            } else if ["battute", "bars", "bar", "battuta", "misure"].contains(&next) {
                r.bars = Some((n as u32).clamp(1, 64));
                used.push(words[i].into());
                used.push(next.into());
            }
        }
    }
    // explicit part styles
    for (m, s) in [("hypnotic", "hypnotic"), ("emotional", "emotional"), ("cinematic", "cinematic"), ("minimal", "minimal"), ("progressive", "progressive"), ("rhythmic", "rhythmic"), ("simple", "simple")] {
        if r.moods.iter().any(|x| x == m) && r.melody_style.is_none() {
            r.melody_style = Some(s.into());
        }
    }
    if r.moods.iter().any(|m| m == "syncopated") {
        r.bass_style = Some("syncopated".into());
    }
    if r.moods.iter().any(|m| m == "hypnotic") && r.parts.iter().any(|p| p == "bass") {
        r.bass_style = Some("hypnotic".into());
    }
    let filler = ["crea", "creami", "fammi", "genera", "scrivi", "componi", "una", "un", "uno", "il", "la", "lo", "le", "gli", "i", "di", "da", "in", "con", "per", "e", "a", "al", "alla", "adatta", "adatto", "produzione", "tonalità", "tonalita", "make", "create", "write", "a", "an", "the", "for", "with", "and", "in", "of", "to", "suitable", "production", "key", "track", "voglio", "vorrei", "mi", "serve", "che", "sia", "elementi", "stile", "style", "nel", "nella", "su", "un'", "dell", "della", "del", "more", "più", "molto"];
    r.unknown = words.iter().filter(|w| !used.iter().any(|u| u == *w) && !filler.contains(w) && w.parse::<f64>().is_err()).map(|w| w.to_string()).collect();
    if r.parts.is_empty() {
        r.parts.push("all".into());
    }
    r
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_prompt_example_and_others() {
        let r = parse("Crea una melodia emozionale e ipnotica in tonalità di La minore, adatta a una produzione Melodic Techno a 124 BPM.");
        assert_eq!(r.parts, ["melody"]);
        assert_eq!(r.moods, ["emotional", "hypnotic"]);
        assert_eq!(r.key.as_deref(), Some("A minor"));
        assert_eq!(r.bpm, Some(124.0));
        assert_eq!(r.styles, [("melodic_techno".to_string(), 1.0)]);
        assert!(r.unknown.is_empty(), "{:?}", r.unknown);
        let b = parse("bassline sincopata in Fa diesis minore 128 bpm 16 battute, techno con elementi di ambient");
        assert_eq!((b.key.as_deref(), b.bpm, b.bars), (Some("F# minor"), Some(128.0), Some(16)));
        assert_eq!(b.bass_style.as_deref(), Some("syncopated"));
        assert_eq!(b.styles, [("techno".to_string(), 1.0), ("ambient".to_string(), 0.3)]);
        let e = parse("an uplifting progressive house track in G major");
        assert_eq!((e.key.as_deref(), e.parts.clone(), e.styles[0].0.as_str()), (Some("G major"), vec!["all".to_string()], "progressive_house"));
        let d = parse("arpeggio ipnotico in Re dorico");
        assert_eq!((d.parts[0].as_str(), d.key.as_deref()), ("arp", Some("D dorian")));
        let u = parse("melodia con il sapore di un violino zigano");
        assert!(u.unknown.iter().any(|w| w == "violino"), "unknown words are reported: {:?}", u.unknown);
        assert_eq!(parse("deep house chords in Am").key.as_deref(), Some("A minor"));
    }
}
