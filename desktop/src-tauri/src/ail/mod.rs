//! Agent Integration Layer (docs/architecture/AGENT-INTEGRATION-LAYER.md).
//! The ONLY module that knows a central Agent exists. It does not plan or decide anything.
//! Since Prompt 06 a port reaches the specialized agents only through the JSON Agent API
//! (`crate::agents::runtime::Runtime::handle`): the same interface the adapter for the partner's
//! central Agent will use, whatever its transport (decision D-1).
use crate::agents::protocol::TaskResult;
use crate::agents::runtime::Runtime;
use serde::Serialize;
use serde_json::Value;

#[cfg(feature = "temp-connector")]
mod temp_test_connector;

#[derive(Serialize, Clone, Debug)]
pub struct AilError {
    pub code: String, // AGENT_UNAVAILABLE | SCHEMA_INVALID | …, plus every TaskResult error code
    pub message: String,
}

#[derive(Serialize, Clone, Debug)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum AgentEvent {
    #[allow(dead_code)] // emitted by the real Agent adapter and the MOCK connector
    AssistantText { text: String },
    #[allow(dead_code)]
    TaskResult { result: Box<TaskResult> },
    Error { error: AilError },
}

#[derive(Serialize, Clone)]
pub struct PortHealth {
    pub port_id: &'static str,
    pub is_temporary: bool,
    pub connected: bool,
}

/// Agent API: one JSON message in, one out (registry.get, task.submit, task.status, task.list,
/// task.cancel, approval.respond, approval.list, events.poll). See docs/agents/AGENT-PROTOCOL.md.
pub type AgentApi<'a> = &'a dyn Fn(&Value) -> Value;

/// Stable internal interface. The real Agent gets its own implementation (decision D-1: transport).
pub trait AgentPort: Send {
    fn health(&self) -> PortHealth;
    /// Send a user turn. `api` is the Agent API toward the specialized agents.
    fn send(&mut self, text: &str, api: AgentApi) -> Vec<AgentEvent>;
}

/// Default port: the central Agent is not connected. Says so; never pretends to answer.
#[cfg_attr(feature = "temp-connector", allow(dead_code))]
pub struct NoAgent;
impl AgentPort for NoAgent {
    fn health(&self) -> PortHealth {
        PortHealth { port_id: "none", is_temporary: false, connected: false }
    }
    fn send(&mut self, _: &str, _: AgentApi) -> Vec<AgentEvent> {
        vec![AgentEvent::Error { error: AilError { code: "AGENT_UNAVAILABLE".into(), message: "central Agent not connected".into() } }]
    }
}

pub fn default_port() -> Box<dyn AgentPort> {
    #[cfg(feature = "temp-connector")]
    return Box::new(temp_test_connector::MockCentralAgent);
    #[cfg(not(feature = "temp-connector"))]
    Box::new(NoAgent)
}

pub struct Ail {
    pub port: Box<dyn AgentPort>,
}

impl Ail {
    pub fn new() -> Self {
        Ail { port: default_port() }
    }

    pub fn send(&mut self, rt: &Runtime, text: &str) -> Vec<AgentEvent> {
        self.port.send(text, &|m| rt.handle(m))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn no_agent_reports_unavailable() {
        let db = std::sync::Arc::new(std::sync::Mutex::new(crate::db::open_memory()));
        let rt = Runtime::new(db, std::env::temp_dir());
        let mut ail = Ail { port: Box::new(NoAgent) };
        let ev = ail.send(&rt, "make a bassline");
        assert!(matches!(&ev[0], AgentEvent::Error { error } if error.code == "AGENT_UNAVAILABLE"));
        assert!(rt.tasks(10).is_empty(), "nothing is sent anywhere");
    }
}
