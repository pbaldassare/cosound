//! Multi-label classification of library files. Every label says where it comes from (file name,
//! folder, embedded metadata, audio analysis, MIDI content, preset data, user) and how sure it is.
//! A file can have several categories; the user can add or remove labels (see index.rs).
use super::features::Features;
use serde::Serialize;
use serde_json::Value;

#[derive(Serialize, Clone, Debug, PartialEq)]
pub struct Label {
    pub tag: String,
    /// category | type | genre | descriptor | plugin
    pub kind: &'static str,
    /// filename | folder | metadata | analysis | midi | preset | user
    pub source: &'static str,
    pub confidence: f64,
}

pub const CATEGORIES: [(&str, &[&str]); 20] = [
    ("kick", &["kick", "kicks", "bd", "bassdrum", "kik", "cassa", "casse"]),
    ("snare", &["snare", "snares", "sd", "rullante"]),
    ("clap", &["clap", "claps", "handclap", "handclaps"]),
    ("hihat", &["hat", "hats", "hihat", "hihats", "hh", "openhat", "closedhat", "charleston"]),
    ("cymbal", &["crash", "ride", "cymbal", "cymbals", "piatto"]),
    ("percussion", &["perc", "percs", "percussion", "percussions", "percussioni", "conga", "congas", "bongo", "bongos", "shaker", "shakers", "tamb", "tambourine", "rim", "rimshot", "tom", "toms", "cowbell", "clave", "claves", "woodblock", "tribal"]),
    ("drums", &["drum", "drums", "drumloop", "beat", "beats", "break", "breaks", "groove", "grooves", "top", "tops", "batteria"]),
    ("bass", &["bass", "basses", "bassline", "basslines", "sub", "subbass", "808", "reese", "basso"]),
    ("synth", &["synth", "synths", "sintetizzatore", "sintetizzatori"]),
    ("lead", &["lead", "leads"]),
    ("pad", &["pad", "pads"]),
    ("pluck", &["pluck", "plucks"]),
    ("arpeggio", &["arp", "arps", "arpeggio", "arpeggios", "arpeggi", "arpeggiator"]),
    ("chords", &["chord", "chords", "accordi", "accordo", "stab", "stabs", "keys", "piano", "rhodes", "organ"]),
    ("melody", &["melody", "melodies", "melodia", "melodie", "hook", "hooks", "riff", "riffs"]),
    ("vocal", &["vocal", "vocals", "vox", "voice", "voices", "voce", "vocale", "vocali", "acapella", "acappella", "chant", "chants", "adlib", "adlibs", "spoken"]),
    ("fx", &["fx", "sfx", "riser", "risers", "sweep", "sweeps", "impact", "impacts", "whoosh", "uplifter", "downlifter", "transition", "effetto", "effetti"]),
    ("atmosphere", &["atmosphere", "atmospheres", "atmos", "atmo", "ambient", "ambience", "texture", "textures", "drone", "drones", "soundscape", "atmosfera"]),
    ("stem", &["stem", "stems", "multitrack"]),
    ("loop", &["loop", "loops"]),
];
pub const TYPES: [(&str, &[&str]); 3] = [
    ("loop", &["loop", "loops"]),
    ("one_shot", &["oneshot", "oneshots", "shot", "shots", "hit", "hits"]),
    ("full_track", &["song", "songs", "fullmix", "mixdown", "canzone", "canzoni", "brano", "brani"]),
];
pub const GENRES: [(&str, &[&str]); 14] = [
    ("techno", &["techno"]),
    ("melodic_techno", &["melodic techno", "melodictechno"]),
    ("house", &["house"]),
    ("deep_house", &["deep house", "deephouse"]),
    ("tech_house", &["tech house", "techhouse"]),
    ("progressive_house", &["progressive house", "progressivehouse", "progressive", "prog house"]),
    ("afro_house", &["afro house", "afrohouse", "afro"]),
    ("trance", &["trance", "psytrance"]),
    ("minimal", &["minimal"]),
    ("edm", &["edm", "big room", "bigroom", "electro house", "future house"]),
    ("dubstep", &["dubstep"]),
    ("dnb", &["dnb", "drum and bass", "drumandbass", "jungle"]),
    ("hiphop", &["hiphop", "hip hop", "trap", "boombap"]),
    ("lofi", &["lofi", "lo fi"]),
];
pub const TEXT_DESCRIPTORS: [(&str, &[&str]); 12] = [
    ("dark", &["dark", "scuro", "cupo"]),
    ("deep", &["deep", "profondo"]),
    ("warm", &["warm", "caldo"]),
    ("bright", &["bright", "brillante"]),
    ("punchy", &["punchy", "punch", "hard", "potente", "powerful"]),
    ("soft", &["soft", "morbido"]),
    ("distorted", &["distorted", "dist", "distorto", "dirty"]),
    ("acid", &["acid", "303"]),
    ("analog", &["analog", "analogue", "vintage"]),
    ("female", &["female", "fem", "woman", "women", "femminile", "femminili", "donna", "donne"]),
    ("male", &["male", "man", "men", "maschile", "maschili", "uomo", "uomini"]),
    ("tribal", &["tribal", "tribale"]),
];

/// Lower-case words of a name: splits on separators, camelCase and letter/digit boundaries.
pub fn words(s: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut cur = String::new();
    let mut prev: Option<char> = None;
    for c in s.chars() {
        let boundary = match prev {
            Some(p) => !c.is_alphanumeric() || (p.is_lowercase() && c.is_uppercase()) || (p.is_alphabetic() && c.is_ascii_digit()) || (p.is_ascii_digit() && c.is_alphabetic()),
            None => !c.is_alphanumeric(),
        };
        if boundary && !cur.is_empty() {
            out.push(std::mem::take(&mut cur).to_lowercase());
        }
        if c.is_alphanumeric() {
            cur.push(c);
        }
        prev = Some(c);
    }
    if !cur.is_empty() {
        out.push(cur.to_lowercase());
    }
    out
}

fn has_phrase(words: &[String], phrase: &str) -> bool {
    let p: Vec<&str> = phrase.split(' ').collect();
    words.windows(p.len()).any(|w| w.iter().zip(&p).all(|(a, b)| a == b))
}

fn add(out: &mut Vec<Label>, tag: &str, kind: &'static str, source: &'static str, confidence: f64) {
    if let Some(l) = out.iter_mut().find(|l| l.tag == tag && l.kind == kind) {
        // independent evidence reinforces a label; keep the strongest source
        let c = 1.0 - (1.0 - l.confidence) * (1.0 - confidence);
        if confidence > l.confidence {
            l.source = source;
        }
        l.confidence = (c.min(0.99) * 1000.0).round() / 1000.0;
    } else {
        out.push(Label { tag: tag.into(), kind, source, confidence });
    }
}

fn from_words(out: &mut Vec<Label>, w: &[String], source: &'static str, conf: f64) {
    for (tag, keys) in CATEGORIES {
        if tag != "loop" && keys.iter().any(|k| has_phrase(w, k)) {
            add(out, tag, "category", source, conf);
        }
    }
    for (tag, keys) in TYPES {
        if keys.iter().any(|k| has_phrase(w, k)) {
            add(out, tag, "type", source, conf);
        }
    }
    for (tag, keys) in GENRES {
        if keys.iter().any(|k| has_phrase(w, k)) {
            add(out, tag, "genre", source, conf * 0.9);
        }
    }
    for (tag, keys) in TEXT_DESCRIPTORS {
        if keys.iter().any(|k| has_phrase(w, k)) {
            add(out, tag, "descriptor", source, conf * 0.9);
        }
    }
}

pub struct Input<'a> {
    pub name: &'a str,
    /// Folder names between the authorized root and the file.
    pub folders: Vec<String>,
    pub kind: &'a str,
    pub features: Option<&'a Features>,
    pub tags: Option<&'a Value>,
    pub midi: Option<&'a Value>,
    pub preset: Option<&'a Value>,
}

pub fn classify(i: &Input) -> Vec<Label> {
    let mut out = Vec::new();
    let stem = i.name.rsplit_once('.').map_or(i.name, |s| s.0);
    from_words(&mut out, &words(stem), "filename", 0.85);
    for f in &i.folders {
        from_words(&mut out, &words(f), "folder", 0.7);
    }
    if let Some(t) = i.tags {
        let text = ["genre", "title", "comment"].iter().filter_map(|k| t[*k].as_str()).collect::<Vec<_>>().join(" ");
        from_words(&mut out, &words(&text), "metadata", 0.8);
    }
    match i.kind {
        "midi" => {
            add(&mut out, "midi", "type", "midi", 1.0);
            if let Some(m) = i.midi {
                let cat = match m["pattern"]["value"].as_str() {
                    Some("arpeggio") => "arpeggio",
                    Some("chords") => "chords",
                    Some("bass") => "bass",
                    Some("drums") => "drums",
                    Some(_) => "melody",
                    None => "",
                };
                if !cat.is_empty() {
                    add(&mut out, cat, "category", "midi", 0.7);
                }
            }
        }
        "preset" | "instrument" => {
            add(&mut out, i.kind, "type", "preset", 1.0);
            if let Some(p) = i.preset {
                let text = ["category", "instrument", "style", "character", "preset_style", "comments", "name"].iter().filter_map(|k| p[*k].as_str()).collect::<Vec<_>>().join(" ");
                from_words(&mut out, &words(&text), "preset", 0.8);
                if let Some(pl) = p["plugin"]["known"].as_str().map(String::from).or_else(|| p["plugin"]["name"].as_str().map(|n| n.to_lowercase())) {
                    add(&mut out, &pl, "plugin", if p["plugin"]["source"] == "path" { "folder" } else { "preset" }, if p["plugin"]["source"] == "path" { 0.7 } else { 0.95 });
                }
            }
        }
        "audio" => {
            if let Some(f) = i.features {
                let d = &f.descriptors;
                if f.is_loop {
                    add(&mut out, "loop", "type", "analysis", 0.7);
                }
                if f.is_one_shot {
                    add(&mut out, "one_shot", "type", "analysis", 0.7);
                }
                // Analysis stops at 90 s, so a full analysis window means a long file: more likely a whole track than a stem.
                if f.analyzed_s >= 89.0 && !f.is_loop {
                    add(&mut out, "full_track", "type", "analysis", 0.4);
                } else if f.analyzed_s > 30.0 && !f.is_loop {
                    add(&mut out, "stem", "category", "analysis", 0.3);
                }
                if f.is_one_shot && d.deep > 0.8 && f.attack_ms.unwrap_or(99.0) < 25.0 && f.analyzed_s < 1.5 {
                    add(&mut out, "kick", "category", "analysis", 0.55);
                }
                if f.is_one_shot && d.bright > 0.8 && d.noisy > 0.5 && f.analyzed_s < 0.6 && d.deep < 0.2 {
                    add(&mut out, "hihat", "category", "analysis", 0.5);
                }
                if f.is_one_shot && d.noisy > 0.4 && d.deep < 0.4 && d.bright > 0.3 && d.bright <= 0.8 && f.analyzed_s < 1.0 {
                    add(&mut out, "snare", "category", "analysis", 0.3);
                    add(&mut out, "clap", "category", "analysis", 0.3);
                }
                if f.is_loop && d.percussive > 0.6 && d.tonal < 0.6 {
                    add(&mut out, "drums", "category", "analysis", 0.5);
                }
                if !f.is_one_shot && d.tonal > 0.6 && d.deep > 0.6 {
                    add(&mut out, "bass", "category", "analysis", 0.45);
                }
                if d.atmospheric > 0.5 && d.tonal > 0.5 {
                    add(&mut out, "pad", "category", "analysis", 0.4);
                    add(&mut out, "atmosphere", "category", "analysis", 0.35);
                }
                if f.is_loop && d.tonal > 0.6 && d.deep < 0.5 && d.percussive > 0.5 {
                    add(&mut out, "arpeggio", "category", "analysis", 0.3);
                }
                for (name, v) in [("deep", d.deep), ("bright", d.bright), ("dark", d.dark), ("punchy", d.punchy), ("warm", d.warm), ("aggressive", d.aggressive), ("round", d.round), ("wide", d.wide), ("atmospheric", d.atmospheric), ("noisy", d.noisy)] {
                    if v >= 0.65 {
                        add(&mut out, name, "descriptor", "analysis", (v * 0.8 * 1000.0).round() / 1000.0);
                    }
                }
            }
            if out.iter().any(|l| l.tag == "loop" && l.kind == "category") {
                out.retain(|l| !(l.tag == "loop" && l.kind == "category"));
            }
        }
        _ => {}
    }
    // "loop" in the name is a type, not an instrument category
    for (tag, keys) in [("loop", ["loop", "loops"])] {
        if keys.iter().any(|k| has_phrase(&words(stem), k)) {
            add(&mut out, tag, "type", "filename", 0.85);
        }
    }
    out.retain(|l| !(l.kind == "category" && l.tag == "loop"));
    out.sort_by(|a, b| b.confidence.total_cmp(&a.confidence));
    out
}

/// Most likely category, if any.
pub fn primary(labels: &[Label]) -> Option<String> {
    labels.iter().filter(|l| l.kind == "category").max_by(|a, b| a.confidence.total_cmp(&b.confidence)).map(|l| l.tag.clone())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tags(name: &str, folders: &[&str]) -> Vec<(String, &'static str, &'static str)> {
        classify(&Input { name, folders: folders.iter().map(|s| s.to_string()).collect(), kind: "audio", features: None, tags: None, midi: None, preset: None })
            .into_iter()
            .map(|l| (l.tag, l.kind, l.source))
            .collect()
    }

    #[test]
    fn names_and_folders() {
        assert_eq!(words("BD_Hard01-Techno"), ["bd", "hard", "01", "techno"]);
        assert_eq!(words("DeepHouseChords"), ["deep", "house", "chords"]);
        let t = tags("Kick_Techno_Deep_01.wav", &["Drums", "Kicks"]);
        assert!(t.contains(&("kick".into(), "category", "filename")));
        assert!(t.contains(&("techno".into(), "genre", "filename")));
        assert!(t.contains(&("deep".into(), "descriptor", "filename")));
        assert!(t.iter().any(|x| x.0 == "drums" && x.2 == "folder"));
        let t = tags("Melodic Techno Bassline 124.wav", &[]);
        assert!(t.iter().any(|x| x.0 == "melodic_techno") && t.iter().any(|x| x.0 == "bass"));
        let t = tags("Perc Loop Tribal.wav", &[]);
        assert!(t.contains(&("percussion".into(), "category", "filename")) && t.contains(&("loop".into(), "type", "filename")));
        assert!(!t.iter().any(|x| x.0 == "loop" && x.1 == "category"));
        let t = tags("Female Vox Chop.wav", &["Vocals"]);
        assert!(t.iter().any(|x| x.0 == "vocal") && t.iter().any(|x| x.0 == "female"));
        assert!(tags("Untitled 3.wav", &[]).is_empty(), "nothing invented");
    }
}
