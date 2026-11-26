//! ⚠ MOCK – SIMULATION OF THE CENTRAL AGENT'S SIDE, NOT THE CENTRAL AGENT.
//! Compiled only in tests and development builds (feature `temp-connector`). A fixed script for the
//! collaboration example of Prompt 06 §9 ("Melodic Techno at 124 BPM"). It makes no decisions: the
//! six steps, their order and their inputs are written here; it only pipes one output into the next
//! input. The real central Agent (partner's code) replaces it without any change to the agents.
use super::protocol::{TaskRequest, TaskResult, CONTRACT_VERSION};
use super::runtime::Runtime;
use serde_json::{json, Value};

pub const NAME: &str = "melodic-techno-124";

/// How the script reaches the agents: in-process calls or JSON messages (as an external Agent).
pub trait Link {
    fn submit(&mut self, req: TaskRequest) -> TaskResult;
}

impl Link for Runtime {
    fn submit(&mut self, req: TaskRequest) -> TaskResult {
        Runtime::submit(self, req)
    }
}

/// Talks to the agents only through the JSON Agent API, like an adapter for the real Agent would.
pub struct ApiLink<'a>(pub &'a dyn Fn(&Value) -> Value);

impl Link for ApiLink<'_> {
    fn submit(&mut self, req: TaskRequest) -> TaskResult {
        let reply = (self.0)(&json!({ "contract_version": CONTRACT_VERSION, "type": "task.submit", "request": req }));
        serde_json::from_value(reply["data"].clone()).expect("task.submit answers with a TaskResult")
    }
}

fn request(agent: &str, operation: &str, input: Value, by: &str, trace: &str, n: usize) -> TaskRequest {
    TaskRequest {
        contract_version: CONTRACT_VERSION.into(),
        request_id: format!("{NAME}-{n}"),
        task_id: None,
        trace_id: Some(trace.into()),
        session_id: None,
        project_id: None,
        agent: agent.into(),
        operation: operation.into(),
        input,
        requested_by: Some(by.into()),
        run_async: false,
    }
}

pub fn melodic_techno_124(link: &mut dyn Link, by: &str) -> Vec<TaskResult> {
    let trace = crate::db::new_id();
    let genre = "melodic_techno";
    // 1. Music Producer: structure
    let plan = link.submit(request("music_producer", "production_plan", json!({ "genre": genre, "bpm": 124 }), by, &trace, 1));
    let p = plan.result.clone().unwrap_or(Value::Null);
    let key = p["key"].as_str().unwrap_or("A minor").to_string();
    let sections: Vec<Value> = p["arrangement"]["sections"].as_array().map(|v| v.iter().map(|s| json!({ "name": s["name"], "start_bar": s["start_bar"] })).collect()).unwrap_or_default();
    // 2. Rhythm Engineer: drums
    let drums = link.submit(request("rhythm_engineer", "generate_drums", json!({ "genre": genre, "bpm": 124, "bars": 8, "seed": 1 }), by, &trace, 2));
    // 3. Music Theory: key and progression
    let harmony = link.submit(request("music_theory", "progression", json!({ "genre": genre, "key": key, "bpm": 124 }), by, &trace, 3));
    // 4. Smart Library: samples and loops
    let samples = link.submit(request("smart_library", "search", json!({ "query": "kick", "bpm_min": 120, "bpm_max": 128 }), by, &trace, 4));
    // 5. Sound Designer: sounds and presets
    let sounds = link.submit(request("sound_designer", "suggest_sounds", json!({ "role": "bass", "genre": genre }), by, &trace, 5));
    // 6. DAW Controller: prepare the DAW operations
    let midi = drums.result.as_ref().and_then(|r| r["midi"]["base64"].as_str()).unwrap_or("").to_string();
    let daw = link.submit(request(
        "daw_controller",
        "plan_operations",
        json!({ "daw": "logic", "bpm": 124, "key": key, "project_name": "Melodic Techno 124", "sections": sections, "parts": [{ "name": "Drums", "role": "drums", "midi_base64": midi }] }),
        by,
        &trace,
        6,
    ));
    vec![plan, drums, harmony, samples, sounds, daw]
}

/// One line per step, for the MOCK connector's reply.
pub fn summary(results: &[TaskResult]) -> String {
    let done = results.iter().filter(|r| r.status.as_str() == "COMPLETED").count();
    let mut s = format!("MOCK (simulation) – scenario {NAME}: {done}/{} tasks COMPLETED", results.len());
    for r in results {
        s.push_str(&format!("\n· {}.{} → {}", r.agent, r.operation, r.status.as_str()));
        if let Some(e) = r.errors.first() {
            s.push_str(&format!(" ({}: {})", e.code, e.message));
        }
    }
    s
}
