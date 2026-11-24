//! Agent Integration Layer (docs/architecture/AGENT-INTEGRATION-LAYER.md).
//! The ONLY module that knows a central Agent exists. It does not plan or decide anything.
//! The central Agent is being built by the project partner; until its adapter is written the
//! active port is `NoAgent` (release) or the TEMPORARY test connector (dev builds only).
use crate::db::Db;
use serde::Serialize;
use serde_json::{json, Value};

#[cfg(feature = "temp-connector")]
mod temp_test_connector;

pub const REGISTRY_JSON: &str = include_str!("../../../../packages/agents/registry.json");

#[derive(Serialize, Clone, Debug)]
pub struct AilError {
    pub code: &'static str, // AGENT_UNAVAILABLE | CAPABILITY_UNKNOWN | NOT_IMPLEMENTED | SCHEMA_INVALID | SUBAGENT_FAILED
    pub message: String,
}

#[derive(Serialize, Clone, Debug)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum AgentEvent {
    #[allow(dead_code)] // emitted by the real Agent adapter and the dev test connector
    AssistantText { text: String },
    CapabilityResult { task_id: String, capability: String, ok: bool, output: Value },
    Error { error: AilError },
}

#[derive(Serialize, Clone)]
pub struct PortHealth {
    pub port_id: &'static str,
    pub is_temporary: bool,
    pub connected: bool,
}

/// Stable internal interface. The real Agent gets its own implementation (decision D-1: transport).
pub trait AgentPort: Send {
    fn health(&self) -> PortHealth;
    /// Send a user turn. `call` executes a capability on the Capability Bus.
    fn send(&mut self, text: &str, call: &mut dyn FnMut(&str, Value) -> AgentEvent) -> Vec<AgentEvent>;
}

/// Default port: the central Agent is not connected. Says so; never pretends to answer.
#[cfg_attr(feature = "temp-connector", allow(dead_code))]
pub struct NoAgent;
impl AgentPort for NoAgent {
    fn health(&self) -> PortHealth {
        PortHealth { port_id: "none", is_temporary: false, connected: false }
    }
    fn send(&mut self, _: &str, _: &mut dyn FnMut(&str, Value) -> AgentEvent) -> Vec<AgentEvent> {
        vec![AgentEvent::Error { error: AilError { code: "AGENT_UNAVAILABLE", message: "central Agent not connected".into() } }]
    }
}

pub fn default_port() -> Box<dyn AgentPort> {
    #[cfg(feature = "temp-connector")]
    return Box::new(temp_test_connector::TempTestConnector);
    #[cfg(not(feature = "temp-connector"))]
    Box::new(NoAgent)
}

#[derive(Serialize, Clone)]
pub struct Task {
    pub task_id: String,
    pub capability: String,
    pub input: Value,
    pub status: &'static str, // done | error
    pub output: Value,
    pub started_at_ms: u128,
    pub duration_ms: u128,
}

pub struct Ail {
    pub port: Box<dyn AgentPort>,
    pub tasks: Vec<Task>,
}

fn capability_status(name: &str) -> Option<String> {
    let reg: Value = serde_json::from_str(REGISTRY_JSON).ok()?;
    reg["agents"].as_array()?.iter().flat_map(|a| a["capabilities"].as_array().cloned().unwrap_or_default()).find(|c| c["name"] == name).map(|c| c["status"].as_str().unwrap_or("").to_string())
}

/// Capability Bus: executes an available capability. Only deterministic, read-only tools exist today.
pub fn dispatch(db: &Db, cap: &str, input: &Value) -> Result<Value, AilError> {
    let err = |code, m: String| AilError { code, message: m };
    match capability_status(cap).as_deref() {
        None => return Err(err("CAPABILITY_UNKNOWN", format!("unknown capability {cap}"))),
        Some("planned") => return Err(err("NOT_IMPLEMENTED", format!("{cap} is designed but not implemented yet"))),
        _ => {}
    }
    let fail = |e: String| err("SUBAGENT_FAILED", e);
    match cap {
        "smart_library.search" => {
            let q = input["query"].as_str().ok_or_else(|| err("SCHEMA_INVALID", "input.query (string) required".into()))?;
            let hits = crate::library::search(&db.lock().unwrap(), q, input["kind"].as_str(), input["limit"].as_i64().unwrap_or(50)).map_err(|e| fail(e.to_string()))?;
            Ok(json!(hits))
        }
        "smart_library.stats" => crate::library::stats(&db.lock().unwrap()).map_err(|e| fail(e.to_string())),
        "plugin_intelligence.scan" => Ok(json!(crate::plugins::scan())),
        "daw_controller.detect" => Ok(json!(crate::daw::detect())),
        _ => Err(err("CAPABILITY_UNKNOWN", format!("{cap} marked available but not wired on the bus"))),
    }
}

impl Ail {
    pub fn new() -> Self {
        Ail { port: default_port(), tasks: Vec::new() }
    }

    pub fn send(&mut self, db: &Db, text: &str) -> Vec<AgentEvent> {
        let tasks = &mut self.tasks;
        let mut call = |cap: &str, input: Value| {
            let start = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default();
            let t0 = std::time::Instant::now();
            let res = dispatch(db, cap, &input);
            let task_id = crate::db::new_id();
            let (ok, output) = match res {
                Ok(v) => (true, v),
                Err(e) => (false, json!(e)),
            };
            tasks.insert(
                0,
                Task {
                    task_id: task_id.clone(),
                    capability: cap.into(),
                    input,
                    status: if ok { "done" } else { "error" },
                    output: output.clone(),
                    started_at_ms: start.as_millis(),
                    duration_ms: t0.elapsed().as_millis(),
                },
            );
            tasks.truncate(100);
            AgentEvent::CapabilityResult { task_id, capability: cap.into(), ok, output }
        };
        self.port.send(text, &mut call)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn registry_and_bus_agree() {
        let db: Db = std::sync::Mutex::new(crate::db::open_memory());
        let reg: Value = serde_json::from_str(REGISTRY_JSON).unwrap();
        let agents = reg["agents"].as_array().unwrap();
        assert_eq!(agents.len(), 12);
        for cap in agents.iter().flat_map(|a| a["capabilities"].as_array().unwrap()) {
            let name = cap["name"].as_str().unwrap();
            let r = dispatch(&db, name, &json!({"query": "x"}));
            match cap["status"].as_str().unwrap() {
                "available" => assert!(r.is_ok(), "{name}: {:?}", r.err()),
                _ => assert_eq!(r.unwrap_err().code, "NOT_IMPLEMENTED"),
            }
        }
        assert_eq!(dispatch(&db, "nope.x", &json!({})).unwrap_err().code, "CAPABILITY_UNKNOWN");
        assert_eq!(dispatch(&db, "smart_library.search", &json!({})).unwrap_err().code, "SCHEMA_INVALID");
    }

    #[test]
    fn no_agent_reports_unavailable() {
        let db: Db = std::sync::Mutex::new(crate::db::open_memory());
        let mut ail = Ail { port: Box::new(NoAgent), tasks: vec![] };
        let ev = ail.send(&db, "make a bassline");
        assert!(matches!(&ev[0], AgentEvent::Error { error } if error.code == "AGENT_UNAVAILABLE"));
        assert!(ail.tasks.is_empty());
    }
}
