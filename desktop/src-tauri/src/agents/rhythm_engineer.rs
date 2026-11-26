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
        &["generate_drums", "generate_transition", "groove_timing"]
    }
    fn run(&self, op: &str, input: &Value, cx: &mut Ctx) -> OpResult {
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
