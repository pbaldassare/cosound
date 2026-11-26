//! D. Music Theory: keys, progressions, chords/bass/melody generation, key finding and coherence
//! checks, on the M2 theory engine.
use super::{unknown, Agent, AgentError, Ctx, OpResult, Outcome};
use crate::music::{caps, theory};
use serde_json::{json, Value};

pub struct MusicTheory;

impl Agent for MusicTheory {
    fn id(&self) -> &'static str {
        "music_theory"
    }
    fn operations(&self) -> &'static [&'static str] {
        &["key_info", "progression", "generate_chords", "generate_bassline", "generate_melody", "detect_key", "check_coherence"]
    }
    fn run(&self, op: &str, input: &Value, cx: &mut Ctx) -> OpResult {
        cx.tool("music.theory");
        let gen = |part| {
            caps::generate(input, part).and_then(|v| if v["all_checks_passed"] == true { Ok(v) } else { Err("the generated part did not pass the musical checks".into()) })
        };
        let v = match op {
            "key_info" => caps::key_info(input),
            "progression" => caps::progression(input),
            "generate_chords" | "generate_bassline" | "generate_melody" => {
                cx.tool("music.generator");
                cx.tool("music.smf_writer");
                gen(match op {
                    "generate_chords" => "chords",
                    "generate_bassline" => "bass",
                    _ => "melody",
                })
            }
            "check_coherence" => {
                cx.tool("music.generator");
                caps::check_parts(input)
            }
            "detect_key" => {
                cx.tool("music.key_detection");
                let mut w = [0.0f64; 12];
                if let Some(ws) = input["weights"].as_array() {
                    for (i, x) in ws.iter().take(12).enumerate() {
                        w[i] = x.as_f64().unwrap_or(0.0).max(0.0);
                    }
                } else {
                    for n in input["notes"].as_array().into_iter().flatten() {
                        if let Some(p) = n["pitch"].as_u64() {
                            w[(p % 12) as usize] += n["dur"].as_f64().unwrap_or(1.0).max(0.0);
                        }
                    }
                }
                if w.iter().all(|x| *x == 0.0) {
                    return Err(AgentError::input("give notes (pitch, dur) or 12 pitch-class weights"));
                }
                let ranked = theory::detect_key(&w);
                let r2 = |x: f64| (x * 1000.0).round() / 1000.0;
                cx.limit("key finding is an estimate (Krumhansl–Kessler profiles), not a measurement");
                return Ok(Outcome::new(
                    json!({
                        "best": ranked[0].0.name(),
                        "correlation": r2(ranked[0].1),
                        "margin": r2(ranked[0].1 - ranked[1].1),
                        "relative_of_best": ranked[0].0.relative().map(|k| k.name()),
                        "candidates": ranked.iter().take(5).map(|(k, c)| json!({ "key": k.name(), "correlation": r2(*c) })).collect::<Vec<_>>(),
                    }),
                    "estimated",
                ));
            }
            _ => return Err(unknown(self.id(), op)),
        }
        .map_err(AgentError::input)?;
        Ok(Outcome::new(v, "rule_based"))
    }
}
