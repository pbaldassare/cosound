//! E. Voice-to-Music. Declared in the registry with all operations "planned": the transcription
//! engine (milestone M5) does not exist yet, so nothing is executed and nothing is simulated.
//! The runtime answers NOT_IMPLEMENTED before reaching this code.
use super::{unknown, Agent, Ctx, OpResult};
use serde_json::Value;

pub struct VoiceToMusic;

impl Agent for VoiceToMusic {
    fn id(&self) -> &'static str {
        "voice_to_music"
    }
    fn operations(&self) -> &'static [&'static str] {
        &[]
    }
    fn run(&self, op: &str, _input: &Value, _cx: &mut Ctx) -> OpResult {
        Err(unknown(self.id(), op))
    }
}
