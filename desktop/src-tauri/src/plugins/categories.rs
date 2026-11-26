//! Plugin catalog: multi-label categories with source and confidence.
//! Sources, strongest first: what the plug-in declares (AU tags, VST3 sub-categories, AU type), COSOUND's
//! knowledge base, then words in the name. A plug-in can be in several categories (e.g. EQ + filter).
use serde::Serialize;

pub const INSTRUMENT_CATEGORIES: [&str; 9] = ["synth", "sampler", "drum_machine", "rompler", "piano", "bass", "orchestral", "acoustic", "generator"];
pub const EFFECT_CATEGORIES: [&str; 18] = [
    "eq", "compressor", "limiter", "multiband", "saturation", "distortion", "reverb", "delay", "chorus", "flanger", "phaser", "filter",
    "stereo", "modulation", "creative", "analyzer", "pitch", "utility",
];

#[derive(Serialize, Debug, Clone, PartialEq)]
pub struct Category {
    pub category: String,
    pub source: String,
    pub confidence: f64,
}

/// Declared tags/sub-categories (Apple's standard AU tags, Steinberg's VST3 sub-categories) → COSOUND categories.
fn declared(tag: &str) -> &'static [&'static str] {
    match tag.to_lowercase().as_str() {
        "synthesizer" | "synth" => &["synth"],
        "sampler" => &["sampler"],
        "drums" | "drum" => &["drum_machine"],
        "piano" => &["piano"],
        "bass" => &["bass"],
        "orchestral" | "strings" | "brass" => &["orchestral"],
        "guitar" | "acoustic" => &["acoustic"],
        "generator" => &["generator"],
        "equalizer" | "eq" => &["eq"],
        "filter" => &["filter"],
        "dynamics" | "dynamics processor" => &["compressor"],
        "distortion" => &["distortion", "saturation"],
        "reverb" => &["reverb"],
        "delay" => &["delay"],
        "modulation" => &["modulation"],
        "pitch" | "pitch shift" => &["pitch"],
        "spatial" | "surround" | "imaging" => &["stereo"],
        "analyzer" => &["analyzer"],
        "mastering" => &["limiter"],
        "tools" | "utility" => &["utility"],
        _ => &[],
    }
}

/// Words in plug-in names (English and the common abbreviations used by vendors).
const NAME_WORDS: [(&str, &[&str]); 25] = [
    ("synth", &["synth", "synthesizer", "massive", "serum", "vital", "sylenth", "diva", "pigments", "phase plant", "predator", "albino", "spire", "zebra", "hive", "repro", "minimoog", "jup", "prophet", "analog lab"]),
    ("sampler", &["sampler", "kontakt", "sample"]),
    ("drum_machine", &["drum", "808", "909", "battery", "beat"]),
    ("rompler", &["rompler", "nexus", "omnisphere"]),
    ("piano", &["piano", "keys", "rhodes", "wurli", "keyscape"]),
    ("bass", &["bass"]),
    ("orchestral", &["orchestra", "strings", "symphon", "brass", "cinematic"]),
    ("acoustic", &["guitar", "acoustic", "violin", "cello"]),
    ("eq", &["eq", "equalizer", "equaliser", "pro-q"]),
    ("compressor", &["comp", "compressor", "dynamics", "glue", "pro-c", "1176", "la-2a"]),
    ("limiter", &["limiter", "lmtr", "maximizer", "pro-l", "l2"]),
    ("multiband", &["multiband", "multi-band", "mbc"]),
    ("saturation", &["satur", "tape", "tube", "warm", "decapitator", "saturn"]),
    ("distortion", &["distort", "dist", "drive", "fuzz", "overdrive", "bitcrush", "crusher", "trash"]),
    ("reverb", &["reverb", "verb", "hall", "plate", "room", "valhalla"]),
    ("delay", &["delay", "echo", "dly"]),
    ("chorus", &["chorus"]),
    ("flanger", &["flanger", "flange"]),
    ("phaser", &["phaser"]),
    ("filter", &["filter", "lowpass", "highpass", "hipass", "bandpass", "volcano"]),
    ("stereo", &["stereo", "imager", "width", "wider", "spread", "panner"]),
    ("modulation", &["tremolo", "vibrato", "ring mod", "ensemble", "rotary", "lfo"]),
    ("creative", &["glitch", "stutter", "granular", "shaper", "transient", "pitch", "vocoder"]),
    ("analyzer", &["analyzer", "analyser", "meter", "spectrum", "scope", "tuner"]),
    ("pitch", &["pitch", "tune", "harmon"]),
];

fn add(out: &mut Vec<Category>, c: &str, source: &str, conf: f64) {
    match out.iter_mut().find(|x| x.category == c) {
        Some(x) if x.confidence >= conf => {}
        Some(x) => {
            x.source = source.into();
            x.confidence = conf;
        }
        None => out.push(Category { category: c.into(), source: source.into(), confidence: conf }),
    }
}

/// Categories for one product. `kind` is instrument/effect/…; `known` are categories from the knowledge base.
pub fn classify(name: &str, kind: &str, declared_tags: &[String], declared_source: &str, known: &[&str]) -> Vec<Category> {
    let mut out = Vec::new();
    for t in declared_tags {
        for c in declared(t) {
            // an instrument tagged "Bass" is a bass instrument; an effect tagged "Filter" is a filter
            let ok = if INSTRUMENT_CATEGORIES.contains(c) { kind != "effect" } else { kind != "instrument" };
            if ok {
                add(&mut out, c, declared_source, 0.95);
            }
        }
    }
    for c in known {
        add(&mut out, c, "knowledge", 0.9);
    }
    let n = name.to_lowercase();
    for (c, words) in NAME_WORDS {
        let inst = INSTRUMENT_CATEGORIES.contains(&c);
        if (inst && kind == "effect") || (!inst && kind == "instrument") {
            continue;
        }
        if words.iter().any(|w| n.split(|ch: char| !ch.is_alphanumeric() && ch != '-').any(|t| t == *w) || (w.len() > 4 && n.contains(w))) {
            add(&mut out, c, "name", 0.6);
        }
    }
    if out.is_empty() {
        match kind {
            "instrument" => add(&mut out, "synth", "kind", 0.3),
            "generator" => add(&mut out, "generator", "kind", 0.5),
            _ => {}
        }
    }
    // a multiband compressor is also a compressor
    if out.iter().any(|c| c.category == "multiband") {
        add(&mut out, "compressor", "rule", 0.7);
    }
    out.sort_by(|a, b| b.confidence.total_cmp(&a.confidence));
    out
}

/// Search words (Italian and English) → categories. Used by the plug-in search.
pub fn from_words(text: &str) -> Vec<&'static str> {
    let t = text.to_lowercase();
    let map: [(&str, &[&str]); 26] = [
        ("synth", &["synth", "sintetizzatore", "sintetizzatori", "synthesizer"]),
        ("sampler", &["sampler", "campionatore"]),
        ("drum_machine", &["drum machine", "batteria", "drum"]),
        ("rompler", &["rompler"]),
        ("piano", &["piano", "pianoforte", "keys"]),
        ("bass", &["basso", "bassi", "bass"]),
        ("orchestral", &["orchestra", "orchestrale", "orchestrali", "archi", "strings"]),
        ("acoustic", &["acustico", "acustici", "acoustic", "chitarra", "guitar"]),
        ("generator", &["generatore", "generator"]),
        ("eq", &["eq", "equalizzatore", "equalizer", "equalizzazione"]),
        ("compressor", &["compressore", "compressor", "compressione", "compression"]),
        ("limiter", &["limiter", "limitatore"]),
        ("multiband", &["multibanda", "multiband"]),
        ("saturation", &["saturatore", "saturazione", "saturation", "saturator"]),
        ("distortion", &["distorsore", "distorsione", "distortion", "overdrive"]),
        ("reverb", &["riverbero", "reverb", "riverberi"]),
        ("delay", &["delay", "eco", "echo"]),
        ("chorus", &["chorus"]),
        ("flanger", &["flanger"]),
        ("phaser", &["phaser"]),
        ("filter", &["filtro", "filtri", "filter"]),
        ("stereo", &["stereo", "imaging", "ampiezza", "width"]),
        ("modulation", &["modulazione", "modulazioni", "modulation"]),
        ("creative", &["creativo", "creativi", "creative"]),
        ("analyzer", &["analizzatore", "analizzatori", "analyzer", "analyser"]),
        ("pitch", &["pitch", "intonazione"]),
    ];
    map.iter().filter(|(_, ws)| ws.iter().any(|w| t.split(|c: char| !c.is_alphanumeric()).any(|x| x == *w) || (w.contains(' ') && t.contains(w)))).map(|(c, _)| *c).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn categories_from_declarations_names_and_knowledge() {
        let cats = |n: &str, k: &str, tags: &[&str], known: &[&str]| -> Vec<String> {
            classify(n, k, &tags.iter().map(|s| s.to_string()).collect::<Vec<_>>(), "au_tags", known).into_iter().map(|c| c.category).collect()
        };
        let mut lp = cats("AULowpass", "effect", &["Effects", "Filter", "Equalizer"], &[]);
        lp.sort();
        assert_eq!(lp, vec!["eq", "filter"]);
        assert!(cats("AUMultibandCompressor", "effect", &["Effects", "Dynamics Processor"], &[]).contains(&"multiband".to_string()));
        assert_eq!(cats("AUSampler", "instrument", &["Sampler", "Synthesizer"], &[])[..2], ["sampler".to_string(), "synth".to_string()]);
        assert_eq!(cats("Serum", "instrument", &[], &["synth"]), vec!["synth"]);
        assert!(cats("ValhallaVintageVerb", "effect", &[], &[]).contains(&"reverb".to_string()));
        assert!(cats("Unknown Thing", "effect", &[], &[]).is_empty(), "no guess when nothing supports a category");
        let c = classify("Pro-Q 3", "effect", &["Fx".into(), "EQ".into()], "vst3_categories", &[]);
        assert_eq!((c[0].category.as_str(), c[0].source.as_str()), ("eq", "vst3_categories"));
        assert_eq!(from_words("un riverbero e un compressore"), vec!["compressor", "reverb"]);
        assert_eq!(from_words("sintetizzatori per il basso"), vec!["synth", "bass"]);
    }
}
