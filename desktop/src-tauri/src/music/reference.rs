//! Reference Track bridge – musical reading of a reference and creation of ORIGINAL material inspired by its general
//! traits. Every fact keeps its source: "audio_analysis" (measured by COSOUND on a file the user gave), "web"
//! (external sources, not measured), "user". Composition reuses only general traits (key, tempo, style, harmonic
//! function family) and is checked against the reference: a different progression and a melody whose similarity
//! to the reference melody stays low. Melodies and sequences of the reference are never reproduced.
use super::compose::{compose, Composition};
use super::generate::MidiNote;
use super::harmony;
use super::melody::similarity;
use super::request::Request;
use super::styles::Styles;
use serde_json::{json, Value};

fn fact<'a>(input: &'a Value, k: &str) -> (&'a Value, &'a str) {
    let v = &input[k];
    if v.is_object() && v.get("value").is_some() {
        (&v["value"], v["source"].as_str().unwrap_or("unknown"))
    } else {
        (v, "unknown")
    }
}

/// Read a reference: facts by source, harmony, relations between bass/chords/melody, closest style.
pub fn read(input: &Value, styles: &Styles) -> Result<Value, String> {
    let mut measured = serde_json::Map::new();
    let mut external = serde_json::Map::new();
    for k in ["key", "bpm", "chords", "sections", "melody"] {
        let (v, src) = fact(input, k);
        if v.is_null() {
            continue;
        }
        let target = if src == "audio_analysis" { &mut measured } else { &mut external };
        target.insert(k.into(), json!({ "value": v, "source": src }));
    }
    let key = fact(input, "key").0.as_str().and_then(|k| super::keys::parse_key(k).ok());
    let chords: Vec<String> = fact(input, "chords").0.as_array().map(|a| a.iter().filter_map(|x| x.as_str().map(String::from)).collect()).unwrap_or_default();
    let harmony_read = if chords.len() >= 2 { harmony::analyze(key, &chords).ok() } else { None };
    let bpm = fact(input, "bpm").0.as_f64();
    let minor = key.map(|k| k.mode.minor_third());
    // closest style by tempo range and mode
    let mut best: Option<(f64, &Value)> = None;
    for s in styles.all() {
        let (lo, hi) = (s["bpm"]["min"].as_f64().unwrap_or(0.0), s["bpm"]["max"].as_f64().unwrap_or(300.0));
        let mut sc = match bpm {
            Some(b) if b >= lo && b <= hi => 1.0,
            Some(b) => 1.0 - ((b - lo).abs().min((b - hi).abs()) / 20.0).min(1.0),
            None => 0.5,
        };
        // closer to the style's typical tempo is a better fit than merely inside a wide range
        if let (Some(b), Some(d)) = (bpm, s["bpm"]["default"].as_f64()) {
            sc += 0.5 * (1.0 - ((b - d).abs() / 10.0).min(1.0));
        }
        let minor_w: f64 = s["modes"].as_object().map(|m| m.iter().filter(|(k, _)| !["major", "lydian", "mixolydian"].contains(&k.as_str())).map(|(_, v)| v.as_f64().unwrap_or(0.0)).sum()).unwrap_or(0.5);
        sc += match minor {
            Some(true) => minor_w * 0.5,
            Some(false) => (1.0 - minor_w) * 0.5,
            None => 0.25,
        };
        if best.as_ref().is_none_or(|b| sc > b.0) {
            best = Some((sc, s));
        }
    }
    Ok(json!({
        "title": input["title"], "measured_by_cosound": measured, "from_external_sources": external,
        "harmony": harmony_read, "closest_style": best.map(|b| json!({ "id": b.1["id"], "label": b.1["label"], "fit": (b.0 * 100.0).round() / 100.0 })),
        "note": "external facts (web) are not measurements; only audio_analysis values were measured by COSOUND",
    }))
}

/// Compose original material inspired by the reference's general traits, with originality checks.
pub fn compose_inspired(input: &Value, styles: &Styles, parts: &[String], seed: u64) -> Result<(Composition, Value), String> {
    let r = read(input, styles)?;
    let key = fact(input, "key").0.as_str().map(String::from);
    let bpm = fact(input, "bpm").0.as_f64();
    let style = r["closest_style"]["id"].as_str().unwrap_or("melodic_techno").to_string();
    let ref_romans: Option<String> = r["harmony"]["chords"].as_array().map(|c| c.iter().filter_map(|x| x["roman"].as_str()).collect::<Vec<_>>().join("-"));
    let ref_melody: Vec<MidiNote> = fact(input, "melody").0.as_array().map(|a| a.iter().filter_map(|n| Some(MidiNote { pitch: n["pitch"].as_u64()? as u8, start: n["start"].as_u64()? as u32, dur: n["dur"].as_u64().unwrap_or(240) as u32, vel: 100 })).collect()).unwrap_or_default();
    let req = Request { parts: if parts.is_empty() { vec!["all".into()] } else { parts.to_vec() }, key, bpm, styles: vec![(style, 1.0)], bars: Some(8), ..Default::default() };
    let mut tries = vec![];
    for k in 0..20u64 {
        let c = compose(&req, styles, None, seed + k * 101)?;
        let romans = c.report["progression"]["chords"].as_array().map(|a| a.iter().filter_map(|x| x["roman"].as_str()).collect::<Vec<_>>().join("-")).unwrap_or_default();
        let same_prog = ref_romans.as_deref() == Some(romans.as_str());
        let mel: Vec<MidiNote> = c.tracks.iter().find(|t| t.name == "Melody").map(|t| t.notes.clone()).unwrap_or_default();
        let sim = if ref_melody.len() >= 2 && mel.len() >= 2 { similarity(&ref_melody, &mel) } else { 0.0 };
        tries.push(json!({ "seed": seed + k * 101, "same_progression": same_prog, "melody_similarity": sim }));
        if !same_prog && sim < 0.5 {
            let checks = json!({ "different_progression_than_reference": true, "melody_similarity_to_reference": sim, "threshold": 0.5, "attempts": tries.len(),
                                  "reference_reading": r, "rule": "only general traits reused (key, tempo, style); melodies and progressions of the reference are not reproduced" });
            return Ok((c, checks));
        }
    }
    Err(format!("no original result in 20 attempts: {}", json!(tries)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sources_are_separated_and_the_result_is_original() {
        let styles = Styles::load(None);
        let input = json!({
            "title": "Reference",
            "key": { "value": "A minor", "source": "audio_analysis" },
            "bpm": { "value": 123, "source": "audio_analysis" },
            "chords": { "value": ["Am", "F", "C", "G"], "source": "web" },
            "melody": { "value": [ {"pitch": 69, "start": 0}, {"pitch": 72, "start": 240}, {"pitch": 76, "start": 480}, {"pitch": 74, "start": 720}, {"pitch": 72, "start": 960}, {"pitch": 71, "start": 1200} ], "source": "user" },
        });
        let r = read(&input, &styles).unwrap();
        assert!(r["measured_by_cosound"]["key"].is_object() && r["from_external_sources"]["chords"]["source"] == "web");
        assert_eq!(r["harmony"]["chords"][0]["roman"], "i");
        assert!(["melodic_techno", "deep_house", "tech_house", "house", "progressive_house"].contains(&r["closest_style"]["id"].as_str().unwrap()), "{}", r["closest_style"]);
        let (c, checks) = compose_inspired(&input, &styles, &["chords".into(), "melody".into()], 1).unwrap();
        assert_eq!(c.key.name(), "A minor");
        assert_eq!(checks["different_progression_than_reference"], true);
        assert!(checks["melody_similarity_to_reference"].as_f64().unwrap() < 0.5);
        let romans: Vec<&str> = c.report["progression"]["chords"].as_array().unwrap().iter().map(|x| x["roman"].as_str().unwrap()).collect();
        assert_ne!(romans.join("-"), "i-VI-III-VII", "the reference's own progression is not reused");
    }
}
