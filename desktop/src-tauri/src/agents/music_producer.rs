//! A. Music Producer: structure, arrangement, variants and the production plan. It writes no
//! notes itself: the plan lists the work (`needs`) the central Agent may assign to other agents.
use super::{s, unknown, Agent, AgentError, Ctx, OpResult, Outcome};
use crate::music::{arrangement, generate, theory};
use serde_json::{json, Value};

pub struct MusicProducer;

/// Closest genre with MIDI presets (M2) for genres that only have an arrangement template.
pub fn m2_genre(g: &str) -> &'static str {
    match g {
        "techno" | "minimal" => "techno",
        "melodic_techno" | "melodic_house" => "melodic_techno",
        "progressive_house" | "progressive_techno" => "progressive_house",
        "house" | "deep_house" | "tech_house" | "afro_house" => "house",
        _ => "edm",
    }
}

const MINOR: [&str; 6] = ["i-VI-III-VII", "i-iv-VI-v", "i-VII-VI-VII", "i-VI-iv-V", "i-iv-VII-III", "i-III-VII-iv"];
const MAJOR: [&str; 6] = ["I-V-vi-IV", "vi-IV-I-V", "I-IV-vi-V", "I-vi-IV-V", "IV-V-iii-vi", "I-iii-IV-V"];

impl Agent for MusicProducer {
    fn id(&self) -> &'static str {
        "music_producer"
    }
    fn operations(&self) -> &'static [&'static str] {
        &["plan_arrangement", "propose_variants", "production_plan"]
    }
    fn run(&self, op: &str, input: &Value, cx: &mut Ctx) -> OpResult {
        let genre = s(input, "genre").unwrap_or("melodic_techno");
        match op {
            "plan_arrangement" => {
                cx.tool("music.arrangement");
                let p = arrangement::plan(genre, input["bpm"].as_f64()).ok_or_else(|| AgentError::input(format!("no arrangement template for {genre}")))?;
                if let Some(a) = p["alias"].as_str() {
                    cx.limit(a.to_string());
                }
                cx.limit("arrangement templates are conventions from the knowledge base, to verify against the user's reference tracks");
                Ok(Outcome::new(p, "convention"))
            }
            "propose_variants" => {
                cx.tool("music.theory");
                let preset = generate::genre(genre).ok_or_else(|| AgentError::input(format!("unknown genre {genre}")))?;
                let key = match (s(input, "key"), s(input, "tonic")) {
                    (Some(k), _) => theory::Key::parse(k),
                    (_, Some(t)) => theory::Key::new(t, s(input, "mode").unwrap_or("major")),
                    _ => theory::Key::parse(preset.key),
                }
                .map_err(AgentError::input)?;
                let pool = if key.mode.minor_third() { MINOR } else { MAJOR };
                let count = input["count"].as_u64().unwrap_or(3).clamp(1, 5) as usize;
                let start = input["seed"].as_u64().unwrap_or(0) as usize;
                let mut variants = Vec::new();
                for k in 0..pool.len() {
                    let prog = pool[(start + k) % pool.len()];
                    if prog == preset.progression || variants.len() == count {
                        continue;
                    }
                    let ch = theory::progression(key, prog, false).map_err(AgentError::input)?;
                    variants.push(json!({
                        "progression": prog,
                        "chords": ch.iter().map(|c| c.symbol()).collect::<Vec<_>>(),
                        "functions": ch.iter().map(|c| c.function()).collect::<Vec<_>>(),
                        "tension_curve": ch.iter().map(|c| c.tension()).collect::<Vec<_>>(),
                        "loop_cadence": theory::cadence(&ch[ch.len() - 1], &ch[0]),
                        "try_with": { "genre": genre, "key": key.name(), "progression": prog, "seed": start + k + 1 },
                    }));
                }
                Ok(Outcome::new(json!({ "genre": genre, "key": key.name(), "baseline": preset.progression, "variants": variants }), "rule_based"))
            }
            "production_plan" => {
                cx.tool("music.arrangement");
                cx.tool("music.theory");
                let p = arrangement::plan(genre, input["bpm"].as_f64()).ok_or_else(|| AgentError::input(format!("no arrangement template for {genre}")))?;
                let bpm = p["bpm"].as_f64().unwrap_or(124.0);
                let m2 = m2_genre(genre);
                let key = match s(input, "key") {
                    Some(k) => theory::Key::parse(k).map_err(AgentError::input)?.name(),
                    None => generate::genre(m2).map_or("A minor".into(), |g| theory::Key::parse(g.key).map(|k| k.name()).unwrap_or_default()),
                };
                let build_bars = p["sections"].as_array().and_then(|v| v.iter().find(|s| s["kind"] == "build")).and_then(|s| s["bars"].as_u64()).unwrap_or(8).min(16);
                let sections: Vec<Value> = p["sections"].as_array().map(|v| v.iter().map(|s| json!({ "name": s["name"], "start_bar": s["start_bar"] })).collect()).unwrap_or_default();
                if genre != m2 {
                    cx.limit(format!("MIDI presets exist for 5 genres: the parts use the closest one ({m2})"));
                }
                cx.limit("arrangement templates are conventions from the knowledge base, to verify against the user's reference tracks");
                let loop_in = json!({ "genre": m2, "key": key, "bpm": bpm, "bars": 8 });
                let mut out = Outcome::new(
                    json!({
                        "genre": genre, "bpm": bpm, "key": key, "arrangement": p,
                        "loop": { "bars": 8, "note": "build the full 8-bar loop first, then arrange by removing elements (subtractive arrangement)" },
                        "parts": ["drums", "bass", "chords", "melody", "transition"],
                    }),
                    "convention",
                );
                out.needs = vec![
                    json!({ "agent": "music_theory", "operation": "progression", "why": "harmonic plan in the chosen key", "input": loop_in }),
                    json!({ "agent": "music_theory", "operation": "generate_chords", "why": "chord part of the main loop", "input": loop_in }),
                    json!({ "agent": "music_theory", "operation": "generate_bassline", "why": "bass part of the main loop", "input": loop_in }),
                    json!({ "agent": "music_theory", "operation": "generate_melody", "why": "theme / lead of the main loop", "input": loop_in }),
                    json!({ "agent": "rhythm_engineer", "operation": "generate_drums", "why": "drum part of the main loop", "input": loop_in }),
                    json!({ "agent": "rhythm_engineer", "operation": "generate_transition", "why": "build into the drop", "input": { "kind": "snare_build", "bars": build_bars, "bpm": bpm } }),
                    json!({ "agent": "sound_designer", "operation": "suggest_sounds", "why": "bass sound", "input": { "role": "bass", "genre": m2 } }),
                    json!({ "agent": "sound_designer", "operation": "suggest_sounds", "why": "lead sound", "input": { "role": "lead", "genre": m2 } }),
                    json!({ "agent": "sound_designer", "operation": "suggest_sounds", "why": "pad / chord sound", "input": { "role": "pad", "genre": m2 } }),
                    json!({ "agent": "smart_library", "operation": "search", "why": "kick samples near the tempo", "input": { "query": "kick", "bpm_min": bpm - 4.0, "bpm_max": bpm + 4.0 } }),
                    json!({ "agent": "daw_controller", "operation": "plan_operations", "why": "set up the DAW project once the parts exist", "input": { "bpm": bpm, "key": key, "sections": sections } }),
                ];
                Ok(out)
            }
            _ => Err(unknown(self.id(), op)),
        }
    }
}
