//! Plugin Knowledge Engine: what COSOUND knows about a plug-in, always with its source and version.
//! Sources, in order:
//! 1. Official documentation shipped as JSON in `packages/plugin-knowledge/` (today: Apple's AudioUnitParameters.h,
//!    extracted by `extract_apple_au.py`), plus any updated file the user or a partner puts in
//!    `<app data>/plugin-knowledge/*.json` (same schema; newer files override older entries).
//! 2. COSOUND's knowledge base (`docs/knowledge/synthesizers`, `plugins-effects`, `audio-engineering`) for synth
//!    profiles and effect categories.
//! When nothing is documented the answer says so: no function or parameter is ever invented.
use crate::sound;
use serde_json::{json, Value};
use std::path::Path;

const APPLE_AU: &str = include_str!("../../../../packages/plugin-knowledge/apple-au.json");

/// What each effect category does and where it usually sits (knowledge base, not vendor documentation).
pub const EFFECT_FACTS: [(&str, &str, &str); 14] = [
    ("eq", "Boosts or cuts frequency bands (bell, shelf, high/low-pass). Prefer cuts; boosts of a few dB", "docs/knowledge/audio-engineering/mixing.md §2"),
    ("filter", "Removes content above (low-pass) or below (high-pass) a cutoff; resonance emphasizes the cutoff", "docs/knowledge/synthesizers/README.md §3"),
    ("compressor", "Reduces level above a threshold by a ratio; attack/release shape transients; make-up gain restores level", "docs/knowledge/audio-engineering/mixing.md §3"),
    ("limiter", "A compressor at ∞:1 that keeps peaks under a ceiling; last in a chain", "docs/knowledge/audio-engineering/mixing.md §3"),
    ("multiband", "Compression in separate frequency bands", "docs/knowledge/audio-engineering/mastering.md §6"),
    ("saturation", "Adds harmonics; helps bass translate on small speakers; can tame harsh peaks", "docs/knowledge/audio-engineering/mixing.md §9"),
    ("distortion", "Clips or waveshapes the signal: more harmonics, more aggression", "docs/knowledge/plugins-effects/README.md §1"),
    ("reverb", "Simulated space; high-pass the return to keep the sub clean", "docs/knowledge/plugins-effects/README.md §2"),
    ("delay", "Repeats the signal; tempo-synced 1/8, dotted 1/8, 1/4 are standard in the genres", "docs/knowledge/plugins-effects/README.md §3"),
    ("chorus", "Detuned, modulated copies: width and movement", "docs/knowledge/plugins-effects/README.md §4"),
    ("flanger", "Short modulated delay with feedback: jet sweep", "docs/knowledge/plugins-effects/README.md §4"),
    ("phaser", "Moving notches from all-pass stages", "docs/knowledge/plugins-effects/README.md §4"),
    ("stereo", "Changes stereo width; keep content below ≈100 Hz mono", "docs/knowledge/audio-engineering/mixing.md §6"),
    ("modulation", "Periodic change of a parameter (tremolo, vibrato, ring modulation)", "docs/knowledge/plugins-effects/README.md §4"),
];

pub struct Knowledge {
    docs: Vec<Value>,
}

impl Knowledge {
    /// Built-in documentation plus updates found in `<data_dir>/plugin-knowledge/`.
    pub fn load(data_dir: Option<&Path>) -> Knowledge {
        let mut docs: Vec<Value> = vec![serde_json::from_str(APPLE_AU).expect("apple-au.json is valid")];
        if let Some(dir) = data_dir.map(|d| d.join("plugin-knowledge")) {
            if let Ok(rd) = std::fs::read_dir(dir) {
                let mut files: Vec<_> = rd.filter_map(Result::ok).map(|e| e.path()).filter(|p| p.extension().is_some_and(|e| e == "json")).collect();
                files.sort();
                for f in files {
                    if let Some(v) = std::fs::read_to_string(&f).ok().and_then(|t| serde_json::from_str::<Value>(&t).ok()) {
                        docs.insert(0, v); // updates win over built-in documents
                    }
                }
            }
        }
        Knowledge { docs }
    }

    /// Documentation for an install: by AU code, else by name. `None` when nothing is documented.
    pub fn about(&self, au_code: Option<&str>, name: &str) -> Option<Value> {
        let n = name.to_lowercase();
        for d in &self.docs {
            let entries = d["units"].as_array().or_else(|| d["plugins"].as_array());
            for e in entries.into_iter().flatten() {
                let by_code = au_code.is_some() && (e["au"].as_str() == au_code || e["match"]["au"].as_str() == au_code);
                let by_name = e["match"]["name_contains"].as_str().is_some_and(|m| n.contains(&m.to_lowercase())) || e["unit"].as_str().is_some_and(|u| u.eq_ignore_ascii_case(name));
                if by_code || by_name {
                    let mut out = e.clone();
                    out["source"] = d["source"].clone();
                    return Some(out);
                }
            }
        }
        None
    }

    /// Everything known for a product, with explicit limits.
    pub fn describe(&self, name: &str, vendor: Option<&str>, au_code: Option<&str>, categories: &[String]) -> Value {
        let doc = self.about(au_code, name);
        let synth = sound::known_synth(name);
        let effects: Vec<Value> = categories
            .iter()
            .filter_map(|c| EFFECT_FACTS.iter().find(|f| f.0 == c))
            .map(|(c, what, src)| json!({ "category": c, "what_it_does": what, "source": src }))
            .collect();
        let mut limits = vec![];
        if doc.is_none() {
            limits.push(format!("no official parameter documentation for {name} is in COSOUND's knowledge base: only what the plug-in itself exposes (parameter discovery) is used"));
        }
        if synth.is_none() && categories.iter().any(|c| c == "synth") {
            limits.push("synth architecture not in the knowledge base".to_string());
        }
        json!({
            "name": name, "vendor": vendor,
            "documentation": doc,
            "synth_profile": synth.map(|s| json!({ "name": s.name, "vendor": s.vendor, "synthesis": s.synthesis, "source": sound::SOURCE_SYNTHS })),
            "effect_facts": effects,
            "limits": limits,
        })
    }
}

/// Categories the knowledge base asserts for a product (synth catalog).
pub fn known_categories(name: &str) -> Vec<&'static str> {
    if sound::known_synth(name).is_some() {
        vec!["synth"]
    } else {
        vec![]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn documented_units_have_sources_and_updates_override() {
        let k = Knowledge::load(None);
        let lp = k.about(Some("aufx:lpas:appl"), "AULowpass").unwrap();
        assert_eq!(lp["parameters"][0]["constant"], "kLowPassParam_CutoffFrequency");
        assert_eq!(lp["parameters"][0]["range"], "10->(SampleRate/2)");
        assert_eq!(lp["source"]["title"], "AudioToolbox/AudioUnitParameters.h");
        assert!(lp["source"]["sdk_version"].is_string());
        let none = k.describe("Mystery Synth", None, None, &["synth".into()]);
        assert!(none["documentation"].is_null() && none["limits"].as_array().unwrap().len() == 2, "{none}");

        let dir = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(dir.path().join("plugin-knowledge")).unwrap();
        std::fs::write(dir.path().join("plugin-knowledge/acme.json"), r#"{"source":{"title":"Acme Comp manual","version":"2.1"},
            "plugins":[{"match":{"name_contains":"acme comp"},"categories":["compressor"],"parameters":[{"name":"Threshold","doc":"dB"}]}]}"#).unwrap();
        let k = Knowledge::load(Some(dir.path()));
        assert_eq!(k.about(None, "Acme Comp 2").unwrap()["source"]["version"].as_str(), Some("2.1"));
        assert!(k.about(Some("aufx:dely:appl"), "AUDelay").is_some(), "built-in documents are still there");
    }
}
