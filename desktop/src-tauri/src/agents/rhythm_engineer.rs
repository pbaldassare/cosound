//! B. Rhythm Engineer: drums, transitions and timing on the M2 engine.
use super::{unknown, Agent, AgentError, Ctx, OpResult, Outcome};
use crate::music::caps;
use serde_json::Value;

pub struct RhythmEngineer;

impl Agent for RhythmEngineer {
    fn id(&self) -> &'static str {
        "rhythm_engineer"
    }
    fn operations(&self) -> &'static [&'static str] {
        &["generate_drums", "generate_transition", "groove_timing", "rhythm_pattern", "humanize"]
    }
    fn run(&self, op: &str, input: &Value, cx: &mut Ctx) -> OpResult {
        match op {
            "rhythm_pattern" => {
                cx.tool("music.rhythm");
                return crate::music::api::rhythm_api(input).map(|v| Outcome::new(v, "rule_based")).map_err(AgentError::input);
            }
            "humanize" => {
                cx.tool("music.midi_editor");
                let notes: Vec<crate::music::generate::MidiNote> = serde_json::from_value(input["notes"].clone()).map_err(|e| AgentError::input(format!("notes: {e}")))?;
                let ops = [crate::music::edit::Op::Humanize { timing: input["timing_ticks"].as_u64().unwrap_or(12).min(60) as u32, velocity: input["velocity"].as_u64().unwrap_or(8).min(30) as u8 }];
                let key = crate::music::theory::Key::parse("C major").unwrap();
                let (out, _) = crate::music::edit::apply(&notes, &ops, key, input["seed"].as_u64().unwrap_or(1)).map_err(AgentError::input)?;
                return Ok(Outcome::new(serde_json::json!({ "notes": out, "diff": crate::music::edit::diff(&notes, &out) }), "rule_based"));
            }
            _ => {}
        }
        cx.tool("music.generator");
        let v = match op {
            "generate_drums" => caps::generate(input, "drums"),
            "generate_transition" => caps::transition(input),
            "groove_timing" => return caps::groove(input).map(|v| Outcome::new(v, "rule_based")).map_err(AgentError::input),
            _ => return Err(unknown(self.id(), op)),
        }
        .map_err(AgentError::input)?;
        cx.tool("music.smf_writer");
        if v["all_checks_passed"] != true {
            return Err(AgentError::new("CHECKS_FAILED", "the generated part did not pass its rhythmic checks"));
        }
        Ok(Outcome::new(v, "rule_based"))
    }
}
