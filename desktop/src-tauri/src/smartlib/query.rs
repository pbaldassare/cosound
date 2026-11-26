//! Natural-language search requests (Italian and English) turned into structured criteria.
//! Deterministic: no AI. What is not understood stays as free text, and the parse is returned with
//! the results so the user (and the agents) can see how the request was read.
use super::classify::{words, CATEGORIES, GENRES, TEXT_DESCRIPTORS};
use serde::Serialize;

#[derive(Serialize, Default, Debug, Clone, PartialEq)]
pub struct Query {
    pub categories: Vec<String>,
    pub types: Vec<String>,
    pub genres: Vec<String>,
    pub descriptors: Vec<String>,
    pub key: Option<String>,
    pub mode: Option<String>,
    pub bpm: Option<f64>,
    pub bpm_range: Option<(f64, f64)>,
    pub max_duration_s: Option<f64>,
    pub min_duration_s: Option<f64>,
    pub bars: Option<f64>,
    pub formats: Vec<String>,
    pub kind: Option<String>,
    pub plugin: Option<String>,
    pub device: Option<String>,
    pub text: Vec<String>,
}

const STOP: [&str; 96] = [
    "trova", "trovami", "cerca", "cercami", "mostrami", "mostra", "voglio", "vorrei", "dammi", "un", "una", "uno", "il", "lo", "la", "i", "gli", "le", "di", "del", "della", "dello", "dei", "delle", "degli", "a", "ad", "al", "alla", "allo", "ai", "alle", "in", "con", "per", "su", "sul", "sulla", "e", "ed", "o", "adatto", "adatta", "adatti", "adatte", "tutti", "tutte", "tutto", "find", "search", "show", "me", "all", "an", "the", "for", "with", "of", "some", "suitable", "track", "traccia", "produzione", "production", "sample", "samples", "suono", "suoni", "sound", "sounds", "file", "files", "che", "sia", "mi", "my", "da", "to", "want", "like", "tipo", "molto", "very", "piu", "more", "tonalita", "key", "nel", "nella", "on", "disco", "drive", "qualche", "get", "give", "brano",
];

/// Descriptors the audio analysis can score (besides the name-based ones of classify).
const AUDIO_DESCRIPTORS: [(&str, &[&str]); 13] = [
    ("deep", &["deep", "profondo", "profonda", "profondi", "profonde"]),
    ("punchy", &["punchy", "punch", "potente", "potenti", "powerful", "incisivo", "incisiva", "hard"]),
    ("warm", &["warm", "caldo", "calda", "caldi", "calde"]),
    ("bright", &["bright", "brillante", "brillanti"]),
    ("dark", &["dark", "scuro", "scura", "scuri", "cupo", "cupa"]),
    ("round", &["round", "rotondo", "rotonda", "rotondi"]),
    ("aggressive", &["aggressive", "aggressivo", "aggressiva", "aggressivi"]),
    ("atmospheric", &["atmospheric", "atmosferico", "atmosferica", "atmosferici", "ambient"]),
    ("soft", &["soft", "morbido", "morbida", "delicato", "delicata"]),
    ("wide", &["wide", "largo", "larga", "ampio", "ampia", "stereo"]),
    ("noisy", &["noisy", "rumoroso", "rumorosa", "sporco", "sporca"]),
    ("tonal", &["tonal", "tonale", "melodico", "melodica"]),
    ("percussive", &["percussive", "percussivo", "percussiva"]),
];

fn norm(s: &str) -> String {
    s.to_lowercase()
        .chars()
        .map(|c| match c {
            'à' | 'á' | 'â' => 'a',
            'è' | 'é' | 'ê' => 'e',
            'ì' | 'í' => 'i',
            'ò' | 'ó' => 'o',
            'ù' | 'ú' => 'u',
            '♯' => '#',
            '♭' => 'b',
            c => c,
        })
        .collect()
}

const IT_NOTES: [(&str, &str); 7] = [("do", "C"), ("re", "D"), ("mi", "E"), ("fa", "F"), ("sol", "G"), ("la", "A"), ("si", "B")];

/// Key in the text: "A minor", "F# min", "Am", "C#m", "la minore", "fa diesis minore", "Bb major".
fn find_key(raw: &str) -> (Option<String>, Option<String>, Vec<usize>) {
    let t: Vec<String> = norm(raw).split(|c: char| c.is_whitespace() || ",;.!?()".contains(c)).filter(|x| !x.is_empty()).map(String::from).collect();
    let mode_of = |w: &str| match w {
        "minor" | "min" | "minore" | "minori" => Some("minor"),
        "major" | "maj" | "maggiore" | "maggiori" => Some("major"),
        _ => None,
    };
    for i in 0..t.len() {
        let w = t[i].as_str();
        // English note: a..g with optional #/b, followed by a mode word
        let en = w.len() <= 2 && w.chars().next().is_some_and(|c| ('a'..='g').contains(&c)) && (w.len() == 1 || w.ends_with('#') || w.ends_with('b'));
        let it = IT_NOTES.iter().find(|(n, _)| *n == w).map(|(_, e)| *e);
        if en || it.is_some() {
            let mut j = i + 1;
            let mut acc = if en && w.len() == 2 { w[1..].to_string() } else { String::new() };
            if let Some(n) = t.get(j).map(String::as_str) {
                if n == "diesis" || n == "sharp" {
                    acc = "#".into();
                    j += 1;
                } else if n == "bemolle" || n == "flat" {
                    acc = "b".into();
                    j += 1;
                }
            }
            if let Some(m) = t.get(j).and_then(|n| mode_of(n)) {
                let letter = it.map(String::from).unwrap_or_else(|| w[..1].to_uppercase());
                return (Some(format!("{letter}{acc} {m}")), Some(m.into()), (i..=j).collect());
            }
        }
        // compact form: am, c#m, bbm
        if (2..=3).contains(&w.len()) && w.ends_with('m') && w.chars().next().is_some_and(|c| ('a'..='g').contains(&c)) {
            let mid = &w[1..w.len() - 1];
            if mid.is_empty() || mid == "#" || mid == "b" {
                return (Some(format!("{}{mid} minor", w[..1].to_uppercase())), Some("minor".into()), vec![i]);
            }
        }
    }
    for (i, w) in t.iter().enumerate() {
        if let Some(m) = mode_of(w) {
            return (None, Some(m.into()), vec![i]);
        }
    }
    (None, None, vec![])
}

pub fn parse(raw: &str, devices: &[String]) -> Query {
    let mut q = Query::default();
    let (key, mode, _) = find_key(raw);
    q.key = key;
    q.mode = mode;
    let w = words(&norm(raw).replace("hi-hat", "hihat").replace("hi hat", "hihat").replace("one-shot", "oneshot").replace("one shot", "oneshot"));
    let mut used = vec![false; w.len()];
    let phrase = |w: &[String], at: usize, p: &str| {
        let parts: Vec<&str> = p.split(' ').collect();
        at + parts.len() <= w.len() && parts.iter().enumerate().all(|(k, x)| w[at + k] == *x)
    };
    let mark = |used: &mut Vec<bool>, at: usize, n: usize| (at..at + n).for_each(|k| used[k] = true);
    // numbers: BPM, ranges, bars, durations
    for i in 0..w.len() {
        let num = |s: &str| s.parse::<f64>().ok();
        if let Some(n) = num(&w[i]) {
            let next = w.get(i + 1).map(String::as_str).unwrap_or("");
            let prev = if i > 0 { w[i - 1].as_str() } else { "" };
            let unit = ["bars", "bar", "battute", "battuta", "s", "sec", "secs", "second", "seconds", "secondi", "secondo"].contains(&next);
            if !unit && (next == "bpm" || prev == "bpm") {
                if let (Some(a), true) = (if i >= 2 { num(&w[i - 2]) } else { None }, prev != "bpm" && i >= 2 && (w[i - 1] == "e" || w[i - 1] == "and" || w[i - 1] == "to")) {
                    q.bpm_range = Some((a.min(n), a.max(n)));
                    q.bpm = None;
                    mark(&mut used, i - 2, 2);
                } else if (40.0..=250.0).contains(&n) {
                    q.bpm = Some(n);
                }
                mark(&mut used, i, 1);
                if next == "bpm" {
                    mark(&mut used, i + 1, 1);
                }
            } else if ["bars", "bar", "battute", "battuta"].contains(&next) {
                q.bars = Some(n);
                mark(&mut used, i, 2);
            } else if ["s", "sec", "secs", "second", "seconds", "secondi", "secondo"].contains(&next) {
                let more = i >= 2 && ["over", "piu", "oltre", "longer", "almeno", "least"].contains(&w[i - 1].as_str()) || (i >= 2 && w[i - 2] == "piu");
                if more {
                    q.min_duration_s = Some(n);
                } else {
                    q.max_duration_s = Some(n);
                }
                mark(&mut used, i, 2);
            }
        }
        if w[i] == "bpm" {
            used[i] = true;
        }
    }
    // a plain "120-128" range next to bpm was split into two numbers: handled above via "e/and/to";
    // also accept "120 128 bpm" (hyphen removed by tokenizing)
    for i in 1..w.len() {
        if let (Ok(a), Ok(b)) = (w[i - 1].parse::<f64>(), w[i].parse::<f64>()) {
            if w.get(i + 1).is_some_and(|x| x == "bpm") && (40.0..=250.0).contains(&a) && (40.0..=250.0).contains(&b) && a < b {
                q.bpm_range = Some((a, b));
                q.bpm = None;
                used[i - 1] = true;
            }
        }
    }
    // a lone number in the usual tempo range with no unit ("loop di batteria 124") is read as BPM
    if q.bpm.is_none() && q.bpm_range.is_none() {
        if let Some(i) = (0..w.len()).find(|&i| !used[i] && w[i].parse::<f64>().is_ok_and(|n| (60.0..=200.0).contains(&n))) {
            q.bpm = w[i].parse().ok();
            used[i] = true;
        }
    }
    // dictionaries (longest phrases first, so "melodic techno" wins over "techno")
    let mut dicts: Vec<(&str, &str, &str)> = Vec::new();
    for (tag, keys) in CATEGORIES {
        for k in keys.iter() {
            dicts.push((if tag == "loop" { "type" } else { "category" }, tag, k));
        }
    }
    for (tag, keys) in GENRES {
        for k in keys.iter() {
            dicts.push(("genre", tag, k));
        }
    }
    for (tag, keys) in AUDIO_DESCRIPTORS.iter().chain(TEXT_DESCRIPTORS.iter().filter(|(t, _)| ["distorted", "acid", "analog", "female", "male", "tribal"].contains(t))) {
        for k in keys.iter() {
            dicts.push(("descriptor", tag, k));
        }
    }
    for (tag, keys) in [("one_shot", &["oneshot", "oneshots", "colpo"][..]), ("loop", &["loop", "loops"][..]), ("full_track", &["song", "songs", "canzone", "canzoni", "brano", "brani", "fullmix"][..])] {
        for k in keys {
            dicts.push(("type", tag, k));
        }
    }
    dicts.sort_by_key(|d| std::cmp::Reverse(d.2.split(' ').count()));
    for i in 0..w.len() {
        if used[i] {
            continue;
        }
        if let Some((kind, tag, k)) = dicts.iter().find(|(_, _, k)| phrase(&w, i, k)) {
            let n = k.split(' ').count();
            if used[i..i + n].iter().any(|u| *u) {
                continue;
            }
            let list = match *kind {
                "category" => &mut q.categories,
                "genre" => &mut q.genres,
                "type" => &mut q.types,
                _ => &mut q.descriptors,
            };
            if !list.iter().any(|x| x == tag) {
                list.push(tag.to_string());
            }
            mark(&mut used, i, n);
            // "tribal" also means percussion; "atmosferico" is a descriptor, not the atmosphere category
            if *tag == "tribal" && !q.categories.contains(&"percussion".to_string()) {
                q.categories.push("percussion".into());
            }
        }
    }
    // genres hidden in compounds like "melodic" + "techno" already handled; "progressive" alone is a genre hint
    for i in 0..w.len() {
        if used[i] {
            continue;
        }
        let x = w[i].as_str();
        if ["wav", "wave", "aif", "aiff", "mp3", "flac", "m4a", "aac", "ogg"].contains(&x) {
            q.formats.push(if x == "wave" { "wav".into() } else if x == "aif" { "aiff".into() } else { x.into() });
            used[i] = true;
        } else if x == "midi" || x == "mid" {
            q.kind = Some("midi".into());
            used[i] = true;
        } else if ["preset", "presets", "patch", "patches"].contains(&x) {
            q.kind = Some("preset".into());
            used[i] = true;
        } else if let Some(s) = crate::sound::known_synth(x) {
            // "massive x" names a different synth than "massive"
            let two = w.get(i + 1).map(|n| format!("{x} {n}")).and_then(|t| crate::sound::known_synth(&t)).filter(|t| t.id != s.id);
            q.plugin = Some(two.unwrap_or(s).id.to_string());
            used[i] = true;
            if two.is_some() {
                used[i + 1] = true;
            }
        }
    }
    // a key spelled in words: drop its tokens from the free text
    if q.key.is_some() || q.mode.is_some() {
        for i in 0..w.len() {
            if ["minor", "minore", "major", "maggiore", "min", "maj", "diesis", "bemolle", "sharp", "flat"].contains(&w[i].as_str()) {
                used[i] = true;
                if i > 0 && (w[i - 1].len() <= 2 || IT_NOTES.iter().any(|(n, _)| *n == w[i - 1])) {
                    used[i - 1] = true;
                }
                if i > 1 && ["diesis", "bemolle", "sharp", "flat"].contains(&w[i - 1].as_str()) {
                    used[i - 2] = true;
                }
            } else if w[i].len() <= 3 && w[i].ends_with('m') && q.key.as_deref().is_some_and(|k| k.to_lowercase().starts_with(&w[i][..1])) {
                used[i] = true;
            }
        }
    }
    let joined = norm(raw);
    for d in devices {
        if joined.contains(&norm(d)) && !d.trim().is_empty() {
            q.device = Some(d.clone());
            for (i, x) in w.iter().enumerate() {
                if norm(d).split(|c: char| !c.is_alphanumeric()).any(|p| p == x) {
                    used[i] = true;
                }
            }
        }
    }
    if q.kind.is_none() && q.types.contains(&"loop".to_string()) && q.formats.is_empty() {
        // loops can be audio or MIDI: no kind filter
    }
    for (i, x) in w.iter().enumerate() {
        if !used[i] && !STOP.contains(&x.as_str()) && x.parse::<f64>().is_err() && x.len() > 1 {
            q.text.push(x.clone());
        }
    }
    q
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn prompt_examples() {
        let q = parse("Trova un kick profondo e potente adatto a una traccia Melodic Techno a 124 BPM.", &[]);
        assert_eq!((q.categories.clone(), q.descriptors.clone(), q.genres.clone(), q.bpm), (vec!["kick".to_string()], vec!["deep".to_string(), "punchy".to_string()], vec!["melodic_techno".to_string()], Some(124.0)), "{q:?}");
        assert!(q.text.is_empty(), "{q:?}");
        let q = parse("Trova tutti i kick adatti alla Techno.", &[]);
        assert_eq!((q.categories.as_slice(), q.genres.as_slice()), (&["kick".to_string()][..], &["techno".to_string()][..]));
        let q = parse("Cerca bassline a 124 BPM.", &[]);
        assert_eq!((q.categories.as_slice(), q.bpm), (&["bass".to_string()][..], Some(124.0)));
        let q = parse("Voglio un pad atmosferico e profondo.", &[]);
        assert_eq!((q.categories.as_slice(), q.descriptors.as_slice()), (&["pad".to_string()][..], &["atmospheric".to_string(), "deep".to_string()][..]));
        let q = parse("Trova un loop percussivo tribale.", &[]);
        assert!(q.types.contains(&"loop".into()) && q.categories.contains(&"percussion".into()) && q.descriptors.contains(&"tribal".into()), "{q:?}");
        let q = parse("Mostrami i synth adatti a una produzione Progressive House.", &[]);
        assert_eq!((q.categories.as_slice(), q.genres.as_slice()), (&["synth".to_string()][..], &["progressive_house".to_string()][..]));
        let q = parse("Cerca sample vocali femminili.", &[]);
        assert!(q.categories.contains(&"vocal".into()) && q.descriptors.contains(&"female".into()), "{q:?}");
        let q = parse("Trova un arpeggio in tonalità minore.", &[]);
        assert_eq!((q.categories.as_slice(), q.mode.as_deref(), q.key.as_deref()), (&["arpeggio".to_string()][..], Some("minor"), None));
        let q = parse("Trova un basso caldo, profondo, rotondo e aggressivo.", &[]);
        assert_eq!(q.descriptors, ["warm", "deep", "round", "aggressive"]);
        let q = parse("Trova un MIDI di arpeggio melodico in tonalità minore.", &[]);
        assert_eq!((q.kind.as_deref(), q.mode.as_deref()), (Some("midi"), Some("minor")));
        let q = parse("Mostrami delle bassline MIDI adatte alla Progressive House.", &[]);
        assert_eq!((q.kind.as_deref(), q.categories.as_slice(), q.genres.as_slice()), (Some("midi"), &["bass".to_string()][..], &["progressive_house".to_string()][..]));
        let q = parse("Trova un preset di pad atmosferico per Massive.", &[]);
        assert_eq!((q.kind.as_deref(), q.plugin.as_deref(), q.categories.as_slice()), (Some("preset"), Some("massive"), &["pad".to_string()][..]));
        assert_eq!(parse("preset lead per Massive X", &[]).plugin.as_deref(), Some("massive_xl"));
    }

    #[test]
    fn keys_ranges_devices() {
        assert_eq!(parse("bass in F# minor", &[]).key.as_deref(), Some("F# minor"));
        assert_eq!(parse("pad la minore", &[]).key.as_deref(), Some("A minor"));
        assert_eq!(parse("lead fa diesis minore", &[]).key.as_deref(), Some("F# minor"));
        assert_eq!(parse("chords Bb major", &[]).key.as_deref(), Some("Bb major"));
        assert_eq!(parse("loop Am 128 bpm", &[]).key.as_deref(), Some("A minor"));
        let q = parse("kick tra 120 e 128 bpm wav sul disco SSD-EXT", &["SSD-EXT".into(), "Macintosh HD".into()]);
        assert_eq!((q.bpm_range, q.formats.as_slice(), q.device.as_deref()), (Some((120.0, 128.0)), &["wav".to_string()][..], Some("SSD-EXT")), "{q:?}");
        assert_eq!(parse("drum loop 120-128 bpm 8 bars", &[]).bpm_range, Some((120.0, 128.0)));
        assert_eq!(parse("drum loop 120-128 bpm 8 bars", &[]).bars, Some(8.0));
        let q = parse("loop di batteria 124", &[]);
        assert_eq!((q.bpm, q.categories.as_slice(), q.text.is_empty()), (Some(124.0), &["drums".to_string()][..], true), "{q:?}");
        assert_eq!(parse("808 bass", &[]).bpm, None, "808 is a sound, not a tempo");
        assert_eq!(parse("kick 8 bars", &[]).bpm, None);
        let q = parse("one shot snare under 2 seconds", &[]);
        assert_eq!((q.types.as_slice(), q.max_duration_s), (&["one_shot".to_string()][..], Some(2.0)));
        assert_eq!(parse("zxqv glitch", &[]).text, ["zxqv", "glitch"]);
    }
}
