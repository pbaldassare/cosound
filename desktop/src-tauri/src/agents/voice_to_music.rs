//! E. Voice-to-Music. Transcription from audio (pitch tracking, onsets, speech) is the engine of milestone M5 and
//! is still "planned": nothing is simulated. What works today: turning notes ALREADY estimated (by that engine later,
//! or given by the central Agent) into music — rhythm, key, MIDI, chords — through the Music Intelligence Engine.
use super::{unknown, Agent, AgentError, Ctx, OpResult, Outcome};
use serde_json::Value;

pub struct VoiceToMusic;

impl Agent for VoiceToMusic {
    fn id(&self) -> &'static str {
        "voice_to_music"
    }
    fn operations(&self) -> &'static [&'static str] {
        &["notes_to_music"]
    }
    fn run(&self, op: &str, input: &Value, cx: &mut Ctx) -> OpResult {
        if op != "notes_to_music" {
            return Err(unknown(self.id(), op));
        }
        cx.tool("music.voice_bridge");
        cx.tool("music.smf_writer");
        let v = crate::music::api::voice_api(input).map_err(AgentError::input)?;
        cx.limit("the notes were estimated upstream: tempo and key are estimates; pitches are only rounded unless correction was requested");
        Ok(Outcome::new(v, "estimated"))
    }
}
