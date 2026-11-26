//! ⚠ MOCK CENTRAL AGENT CONNECTOR – SIMULATION, NOT THE CENTRAL AGENT.
//! Compiled only with the `temp-connector` Cargo feature (development builds). Remove when the real
//! Agent adapter exists. No language understanding and no planning: it only executes explicit
//! commands, through the same JSON Agent API the real adapter will use:
//!   /call <agent>.<operation> [json-input]   one task
//!   /scenario [melodic-techno-124]           the fixed collaboration script of Prompt 06 §9
//!   /status <task_id>   /cancel <task_id>   /agents
use super::{AgentApi, AgentEvent, AgentPort, AilError, PortHealth};
use crate::agents::protocol::TaskResult;
use crate::agents::scenario::{self, ApiLink};
use serde_json::{json, Value};

pub struct MockCentralAgent;

fn reply(v: Value) -> AgentEvent {
    if v["ok"] == true {
        match serde_json::from_value::<TaskResult>(v["data"].clone()) {
            Ok(r) => AgentEvent::TaskResult { result: Box::new(r) },
            Err(_) => AgentEvent::AssistantText { text: v["data"].to_string() },
        }
    } else {
        AgentEvent::Error { error: AilError { code: v["error"]["code"].as_str().unwrap_or("ERROR").into(), message: v["error"]["message"].as_str().unwrap_or("").into() } }
    }
}

impl AgentPort for MockCentralAgent {
    fn health(&self) -> PortHealth {
        PortHealth { port_id: "mock-central-agent", is_temporary: true, connected: true }
    }

    fn send(&mut self, text: &str, api: AgentApi) -> Vec<AgentEvent> {
        let t = text.trim();
        let (cmd, rest) = t.split_once(char::is_whitespace).map(|(a, b)| (a, b.trim())).unwrap_or((t, ""));
        match cmd {
            "/call" => {
                let (cap, input) = rest.split_once(char::is_whitespace).unwrap_or((rest, "{}"));
                let Some((agent, operation)) = cap.split_once('.') else {
                    return vec![AgentEvent::Error { error: AilError { code: "SCHEMA_INVALID".into(), message: "use /call <agent>.<operation> {json}".into() } }];
                };
                let input: Value = match serde_json::from_str(input) {
                    Ok(v) => v,
                    Err(e) => return vec![AgentEvent::Error { error: AilError { code: "SCHEMA_INVALID".into(), message: format!("input is not JSON: {e}") } }],
                };
                let req = json!({ "request_id": crate::db::new_id(), "agent": agent, "operation": operation, "input": input, "requested_by": "mock" });
                vec![reply(api(&json!({ "type": "task.submit", "request": req })))]
            }
            "/scenario" => {
                if !rest.is_empty() && rest != scenario::NAME {
                    return vec![AgentEvent::AssistantText { text: format!("MOCK: only the scenario {} exists", scenario::NAME) }];
                }
                let results = scenario::melodic_techno_124(&mut ApiLink(api), "mock");
                let mut ev: Vec<AgentEvent> = results.iter().cloned().map(|r| AgentEvent::TaskResult { result: Box::new(r) }).collect();
                ev.push(AgentEvent::AssistantText { text: scenario::summary(&results) });
                ev
            }
            "/status" => vec![reply(api(&json!({ "type": "task.status", "task_id": rest })))],
            "/cancel" => vec![reply(api(&json!({ "type": "task.cancel", "task_id": rest })))],
            "/agents" => {
                let r = api(&json!({ "type": "registry.get" }));
                let lines: Vec<String> = r["data"]["agents"]
                    .as_array()
                    .into_iter()
                    .flatten()
                    .map(|a| {
                        let ops = a["operations"].as_array().map(|o| o.iter().filter(|x| x["status"] == "available").map(|x| x["name"].as_str().unwrap_or("")).collect::<Vec<_>>().join(", ")).unwrap_or_default();
                        format!("· {} v{} [{}]: {}", a["id"].as_str().unwrap_or(""), a["version"].as_str().unwrap_or(""), a["state"].as_str().unwrap_or(""), if ops.is_empty() { "—".into() } else { ops })
                    })
                    .collect();
                vec![AgentEvent::AssistantText { text: format!("MOCK (simulation) – agents in the registry:\n{}", lines.join("\n")) }]
            }
            _ => vec![AgentEvent::AssistantText { text: "TEMP_CONNECTOR_EXPLICIT_ONLY".into() }],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::super::*;
    use crate::agents::runtime::Runtime;

    #[test]
    fn explicit_commands_only() {
        let db = std::sync::Arc::new(std::sync::Mutex::new(crate::db::open_memory()));
        let dir = tempfile::tempdir().unwrap();
        let rt = Runtime::new(db, dir.path().canonicalize().unwrap());
        let mut ail = Ail { port: Box::new(super::MockCentralAgent) };
        assert!(matches!(&ail.send(&rt, "hello")[0], AgentEvent::AssistantText { text } if text == "TEMP_CONNECTOR_EXPLICIT_ONLY"));
        assert!(matches!(&ail.send(&rt, "/call smart_library.stats")[0], AgentEvent::TaskResult { result } if result.status.as_str() == "COMPLETED"));
        assert!(matches!(&ail.send(&rt, "/call music_theory.generate_bassline {\"genre\":\"techno\"}")[0], AgentEvent::TaskResult { result } if result.status.as_str() == "COMPLETED"));
        assert!(matches!(&ail.send(&rt, "/call voice_to_music.transcribe_melody {\"path\":\"x\"}")[0], AgentEvent::TaskResult { result } if result.errors[0].code == "NOT_IMPLEMENTED"));
        assert!(matches!(&ail.send(&rt, "/call smart_library.search {bad")[0], AgentEvent::Error { .. }));
        assert!(matches!(&ail.send(&rt, "/call nodot")[0], AgentEvent::Error { .. }));
        let sc = ail.send(&rt, "/scenario");
        assert_eq!(sc.len(), 7);
        assert!(matches!(&sc[6], AgentEvent::AssistantText { text } if text.contains("6/6 tasks COMPLETED")));
        let id = rt.tasks(1)[0].task_id.clone();
        assert!(matches!(&ail.send(&rt, &format!("/status {id}"))[0], AgentEvent::TaskResult { .. }));
        assert!(matches!(&ail.send(&rt, &format!("/cancel {id}"))[0], AgentEvent::Error { error } if error.code == "ALREADY_FINISHED"));
        assert!(matches!(&ail.send(&rt, "/agents")[0], AgentEvent::AssistantText { text } if text.contains("adaptive_workflow")));
    }
}
