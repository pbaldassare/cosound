//! ⚠ TEMPORARY TEST CONNECTOR – NOT THE CENTRAL AGENT.
//! Compiled only with the `temp-connector` Cargo feature (dev builds). Remove when the real Agent adapter exists.
//! No language understanding, no planning: it only executes explicit commands
//!   /call <capability> [json-input]
//! so the Capability Bus, the UI and the error paths can be tested end to end.
use super::{AgentEvent, AgentPort, AilError, PortHealth};
use serde_json::Value;

pub struct TempTestConnector;

impl AgentPort for TempTestConnector {
    fn health(&self) -> PortHealth {
        PortHealth { port_id: "temp-test-connector", is_temporary: true, connected: true }
    }

    fn send(&mut self, text: &str, call: &mut dyn FnMut(&str, Value) -> AgentEvent) -> Vec<AgentEvent> {
        let Some(rest) = text.trim().strip_prefix("/call ") else {
            return vec![AgentEvent::AssistantText { text: "TEMP_CONNECTOR_EXPLICIT_ONLY".into() }];
        };
        let (cap, input) = rest.trim().split_once(char::is_whitespace).unwrap_or((rest.trim(), "{}"));
        match serde_json::from_str::<Value>(input) {
            Ok(v) => vec![call(cap, v)],
            Err(e) => vec![AgentEvent::Error { error: AilError { code: "SCHEMA_INVALID", message: format!("input is not JSON: {e}") } }],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::super::*;

    #[test]
    fn explicit_commands_only() {
        let db: Db = std::sync::Mutex::new(crate::db::open_memory());
        let mut ail = Ail { port: Box::new(super::TempTestConnector), tasks: vec![] };
        assert!(matches!(&ail.send(&db, "hello")[0], AgentEvent::AssistantText { text } if text == "TEMP_CONNECTOR_EXPLICIT_ONLY"));
        assert!(matches!(&ail.send(&db, "/call smart_library.stats")[0], AgentEvent::CapabilityResult { ok: true, .. }));
        assert!(matches!(&ail.send(&db, "/call voice_to_music.transcribe {}")[0], AgentEvent::CapabilityResult { ok: false, .. }));
        assert!(matches!(&ail.send(&db, "/call music_theory.generate_bassline {\"genre\":\"techno\"}")[0], AgentEvent::CapabilityResult { ok: true, .. }));
        assert!(matches!(&ail.send(&db, "/call smart_library.search {bad")[0], AgentEvent::Error { .. }));
        assert_eq!(ail.tasks.len(), 3);
    }
}
