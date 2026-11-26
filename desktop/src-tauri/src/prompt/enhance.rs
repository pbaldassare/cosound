//! Prompt Enhancement: makes a request more precise without changing what the user asked. Four layers are kept apart:
//! the user's words, the semantic interpretation, the technical aspects COSOUND suggests, and the operations the user
//! actually authorized. Suggestions are never authorizations.
use super::context::Target;
use serde_json::{json, Value};

/// Quality → technical aspects (aspect, direction, why). Sources: docs/knowledge (synthesizers §3, mixing §6/§9,
/// plugins-effects), the same knowledge used by the Sound Designer and Mixing Engineer.
const TECH: [(&str, &[(&str, &str, &str)]); 19] = [
    ("aggressive", &[("drive / saturation", "more", "adds harmonics and edge"), ("filter resonance", "slightly more", "a sharper, more present tone"), ("attack", "faster", "a harder front")]),
    ("warm", &[("soft saturation", "a little", "even harmonics feel warm"), ("high frequencies", "gently rolled off", "less harshness")]),
    ("deep", &[("low end / sub", "more", "weight below 100 Hz"), ("register", "lower", "a deeper sound sits lower")]),
    ("bright", &[("filter cutoff", "more open", "more high frequencies"), ("high shelf", "slight boost", "air and presence")]),
    ("dark", &[("filter cutoff", "more closed", "fewer high frequencies")]),
    ("soft", &[("attack", "slower", "a softer start"), ("drive", "less", "fewer harsh harmonics")]),
    ("punchy", &[("attack / transients", "sharper", "a clearer hit"), ("decay", "shorter", "tighter notes")]),
    ("wide", &[("stereo width (unison, detune, chorus)", "more", "a wider image"), ("low end", "kept mono", "a wide sound must not weaken the bass in mono")]),
    ("atmospheric", &[("reverb / ambience", "more", "a sense of space"), ("release", "longer", "notes fade into each other")]),
    ("movement", &[("modulation (LFO on filter or wavetable)", "more", "the timbre changes over time")]),
    ("clean", &[("distortion", "less", "a clearer tone")]),
    ("long", &[("release / decay", "longer", "")]),
    ("short", &[("release / decay", "shorter", "")]),
    ("dynamic", &[("compression", "less or slower", "more difference between soft and loud"), ("velocity variation", "more", "notes breathe")]),
    ("energetic", &[("rhythmic density", "more", "more events per bar"), ("filter / brightness", "opening", "more energy in the highs"), ("percussion layers", "more", "drive in the section")]),
    ("spacey", &[("stereo width", "more", "a wider image"), ("reverb and delay", "more", "an ambient space"), ("depth (pre-delay, early reflections)", "more", "front-to-back depth")]),
    ("beautiful", &[("timbral movement", "more", "a subjective request: COSOUND proposes, the user judges"), ("richness (layers, detune)", "a little more", "")]),
    ("louder", &[("track gain", "up", "")]),
    ("quieter", &[("track gain", "down", "a slight change is about 1–2 dB")]),
];

pub fn quality_noun(q: &str, it: bool) -> &'static str {
    match (q, it) {
        ("aggressive", true) => "aggressività",
        ("aggressive", false) => "aggressiveness",
        ("warm", true) => "calore",
        ("warm", false) => "warmth",
        ("deep", true) => "profondità",
        ("deep", false) => "depth",
        ("bright", true) => "brillantezza",
        ("bright", false) => "brightness",
        ("dark", true) => "scurezza",
        ("dark", false) => "darkness",
        ("soft", true) => "morbidezza",
        ("soft", false) => "softness",
        ("punchy", true) => "incisività",
        ("punchy", false) => "punch",
        ("wide", true) => "ampiezza stereo",
        ("wide", false) => "stereo width",
        ("atmospheric", true) => "atmosfera",
        ("atmospheric", false) => "atmosphere",
        ("movement", true) => "movimento",
        ("movement", false) => "movement",
        ("clean", true) => "pulizia",
        ("clean", false) => "clarity",
        ("long", true) => "durata",
        ("long", false) => "length",
        ("short", true) => "brevità",
        ("short", false) => "shortness",
        ("dynamic", true) => "dinamica",
        ("dynamic", false) => "dynamics",
        ("energetic", true) => "energia",
        ("energetic", false) => "energy",
        ("spacey", true) => "spazialità",
        ("spacey", false) => "space",
        ("beautiful", true) => "ricchezza timbrica",
        ("beautiful", false) => "richness",
        ("louder", true) | ("quieter", true) => "volume",
        ("louder", false) | ("quieter", false) => "level",
        (_, true) => "carattere",
        (_, false) => "character",
    }
}

pub fn element_name(e: &str, it: bool) -> String {
    let (i, en) = match e {
        "bass" => ("il basso", "the bass"),
        "kick" => ("il kick", "the kick"),
        "snare" => ("il rullante", "the snare"),
        "hihat" => ("gli hi-hat", "the hi-hats"),
        "percussion" => ("le percussioni", "the percussion"),
        "drums" => ("la batteria", "the drums"),
        "lead" => ("il lead", "the lead"),
        "pad" => ("il pad", "the pad"),
        "melody" => ("la melodia", "the melody"),
        "chords" => ("gli accordi", "the chords"),
        "arp" => ("l'arpeggio", "the arpeggio"),
        "synth" => ("il sintetizzatore", "the synth"),
        "piano" => ("il piano", "the piano"),
        "strings" => ("gli archi", "the strings"),
        "vocal" => ("la voce", "the vocal"),
        "fx" => ("gli effetti", "the effects"),
        "master" => ("il master", "the master"),
        "mix" => ("il mix", "the mix"),
        "track" => ("il brano", "the track"),
        "drop" => ("il drop", "the drop"),
        "breakdown" => ("il breakdown", "the breakdown"),
        "buildup" => ("il build-up", "the build-up"),
        "intro" => ("l'intro", "the intro"),
        "outro" => ("l'outro", "the outro"),
        "verse" => ("la strofa", "the verse"),
        "chorus" => ("il ritornello", "the chorus"),
        "bridge" => ("il bridge", "the bridge"),
        other => return other.to_string(),
    };
    (if it { i } else { en }).to_string()
}

/// `qualities`: (quality, signed weight, keep). `authorized`: operations the user's words ask for.
pub fn enhance(it: bool, user_words: Vec<String>, qualities: &[(String, f64, bool)], targets: &[Target], authorized: Vec<String>) -> Value {
    let changing: Vec<&(String, f64, bool)> = qualities.iter().filter(|q| !q.2).collect();
    let mut tech = vec![];
    for (q, w, _) in &changing {
        if let Some((_, list)) = TECH.iter().find(|t| t.0 == q) {
            for (aspect, dir, why) in list.iter() {
                let direction = if *w < 0.0 { format!("opposite of: {dir}") } else { dir.to_string() };
                tech.push(json!({ "aspect": aspect, "direction": direction, "why": why }));
            }
        }
    }
    let kept: Vec<&str> = qualities.iter().filter(|q| q.2).map(|q| quality_noun(&q.0, it)).collect();
    let target = targets.iter().find(|t| t.kind != "section").map(|t| t.track_name.clone().unwrap_or_else(|| element_name(&t.element, it)));
    let section = targets.iter().find(|t| t.kind == "section").map(|t| element_name(&t.element, it));
    let aspects: Vec<String> = changing.iter().map(|q| format!("{}{}", if q.1 < 0.0 { if it { "meno " } else { "less " } } else if it { "più " } else { "more " }, quality_noun(&q.0, it))).collect();
    let semantic = if aspects.is_empty() {
        String::new()
    } else if it {
        format!(
            "Individuare {}{} e proporre una configurazione con {}{}, mantenendo la compatibilità con il contesto musicale.",
            target.clone().unwrap_or_else(|| "l'elemento interessato".into()),
            section.map(|s| format!(" nel {}", s.trim_start_matches("il ").trim_start_matches("l'"))).unwrap_or_default(),
            aspects.join(", "),
            if kept.is_empty() { String::new() } else { format!(", conservando {}", kept.join(" e ")) }
        )
    } else {
        format!(
            "Find {}{} and propose settings with {}{}, staying compatible with the musical context.",
            target.unwrap_or_else(|| "the element concerned".into()),
            section.map(|s| format!(" in {s}")).unwrap_or_default(),
            aspects.join(", "),
            if kept.is_empty() { String::new() } else { format!(", keeping its {}", kept.join(" and ")) }
        )
    };
    json!({ "user_words": user_words, "semantic": semantic, "suggested_technical": tech, "authorized_operations": authorized })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn synth_more_beautiful_and_spacey() {
        let q = vec![("beautiful".to_string(), 1.0, false), ("spacey".to_string(), 1.0, false)];
        let t = vec![Target { element: "synth".into(), kind: "instrument".into(), track_id: None, track_name: None, resolved_by: "explicit".into(), confidence: 0.9 }];
        let e = enhance(true, vec!["synth".into(), "più bello".into(), "spaziale".into()], &q, &t, vec!["modify_sound:synth".into()]);
        let s = e["semantic"].as_str().unwrap();
        assert!(s.starts_with("Individuare il sintetizzatore") && s.contains("spazialità"), "{s}");
        let aspects: Vec<&str> = e["suggested_technical"].as_array().unwrap().iter().map(|x| x["aspect"].as_str().unwrap()).collect();
        assert!(aspects.contains(&"stereo width") && aspects.contains(&"reverb and delay") && aspects.contains(&"timbral movement"), "{aspects:?}");
        assert_eq!(e["authorized_operations"], json!(["modify_sound:synth"]), "suggestions are not authorizations");
        let keep = enhance(false, vec![], &[("dynamic".into(), 1.0, false), ("warm".into(), 1.0, true)], &t, vec![]);
        assert!(keep["semantic"].as_str().unwrap().contains("keeping its warmth"));
    }
}
