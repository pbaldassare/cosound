//! G. Reference Track: a profile where every value carries its source (web facts given by the
//! central Agent, or measurements on a file the user provided) and a brief for an ORIGINAL track.
use super::{s, unknown, Agent, AgentError, Ctx, OpResult, Outcome};
use crate::music::arrangement;
use serde_json::{json, Value};

pub struct ReferenceTrack;

const RULES: [&str; 4] = [
    "Do not reuse the melody, hook, lyrics, samples or recordings of the reference",
    "Do not reproduce its arrangement bar by bar: use a genre template and your own sections",
    "Use only general characteristics as inspiration: tempo, energy curve, sound palette, loudness",
    "Keys and tempos are not protected, but the combination with a recognizable motif is: write new themes",
];

impl Agent for ReferenceTrack {
    fn id(&self) -> &'static str {
        "reference_track"
    }
    fn operations(&self) -> &'static [&'static str] {
        &["build_profile", "compose_inspired"]
    }
    fn run(&self, op: &str, input: &Value, cx: &mut Ctx) -> OpResult {
        if op == "compose_inspired" {
            for t in ["music.reference_bridge", "music.composer", "music.harmony", "music.melody", "music.validation"] {
                cx.tool(t);
            }
            let styles = crate::music::styles::Styles::load(Some(cx.data_dir));
            let mut i = input.clone();
            i["compose"] = json!(true);
            let v = crate::music::api::reference_api(&cx.db.lock().unwrap(), &styles, &i).map_err(AgentError::input)?;
            cx.limit("only general traits of the reference are reused; the result is checked for a different progression and a dissimilar melody");
            return Ok(Outcome::new(v, "rule_based"));
        }
        if op != "build_profile" {
            return Err(unknown(self.id(), op));
        }
        let web = &input["web_facts"];
        let audio = s(input, "audio_path");
        if !web.is_object() && audio.is_none() {
            return Err(AgentError::input("give web_facts (found by the central Agent) and/or audio_path (a file the user provided)"));
        }
        let sources = web["sources"].clone();
        let mut profile = serde_json::Map::new();
        let mut put = |k: &str, value: Value, basis: &str, source: Value| {
            if !value.is_null() {
                profile.insert(k.into(), json!({ "value": value, "basis": basis, "source": source }));
            }
        };
        put("bpm", web["bpm"].clone(), "web", sources.clone());
        put("key", web["key"].clone(), "web", sources.clone());
        put("genre", input["genre"].clone().as_str().map_or(web["genre"].clone(), |g| json!(g)), if input["genre"].is_string() { "user" } else { "web" }, sources.clone());
        if web.is_object() {
            cx.limit("web facts were provided by the caller: COSOUND did not verify them");
        }
        let mut measured = Value::Null;
        if let Some(a) = audio {
            let (path, m) = cx.measure_wav(a)?;
            for (k, v) in [("integrated_lufs", json!(m.integrated_lufs)), ("true_peak_dbtp", json!(m.true_peak_dbtp)), ("plr_db", json!(m.plr_db)), ("duration_s", json!(m.duration_s)), ("stereo_correlation", json!(m.stereo_correlation)), ("band_energy", json!(m.band_energy))] {
                put(k, v, "measured", json!(path));
            }
            measured = json!(m);
            cx.limit("BPM, key and structure of the audio are not measured yet (engine M6)");
        }
        let genre = profile.get("genre").and_then(|g| g["value"].as_str()).map(|g| g.to_lowercase().replace([' ', '-'], "_"));
        let template = genre.as_deref().and_then(|g| {
            cx.tool("music.arrangement");
            arrangement::plan(g, profile.get("bpm").and_then(|b| b["value"].as_f64()))
        });
        let bpm = profile.get("bpm").and_then(|b| b["value"].as_f64());
        let brief = json!({
            "tempo_range": bpm.map(|b| [b - 2.0, b + 2.0]),
            "key": profile.get("key").map(|k| json!({ "same_or_relative": k["value"], "note": "write new melodies and chords on your own" })),
            "structure": template.as_ref().map(|t| json!({ "template": t["genre"], "sections": t["sections"], "source": t["source"] })),
            "loudness": profile.get("integrated_lufs").map(|l| json!({ "reference_lufs": l["value"], "note": "compare with the release destination before matching it" })),
            "rules": RULES,
        });
        let basis = match (web.is_object(), audio.is_some()) {
            (true, true) => "mixed",
            (true, false) => "web",
            _ => "measured",
        };
        Ok(Outcome::new(json!({ "title": input["title"], "artist": input["artist"], "profile": profile, "measurements": measured, "brief_for_original_track": brief }), basis))
    }
}
