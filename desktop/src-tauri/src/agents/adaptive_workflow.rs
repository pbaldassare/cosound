//! M. Adaptive Workflow: remembers shortcuts, procedures and preferences the user (or the central
//! Agent on the user's behalf) declares explicitly. No keystroke logging, no passwords or codes.
//! Off until the user enables it; everything is viewable, editable and deletable. Storage is the
//! Music Memory (`memory_facts`, category "workflow"): not a second memory.
use super::{s, unknown, Agent, AgentError, Ctx, OpResult, Outcome};
use crate::memory;
use serde_json::{json, Value};

pub struct AdaptiveWorkflow;
const CAT: &str = "workflow";

fn mem_err(e: String) -> AgentError {
    if e.starts_with("not stored") {
        AgentError::new("PRIVATE_DATA_REFUSED", e)
    } else if e.contains("not found") {
        AgentError::new("NOT_FOUND", e)
    } else {
        AgentError::new("EXECUTION_FAILED", e)
    }
}

fn value_of(input: &Value, base: &Value) -> Value {
    let mut v = if base.is_object() { base.clone() } else { json!({}) };
    for k in ["kind", "daw", "keys", "steps", "value", "source"] {
        if !input[k].is_null() {
            v[k] = input[k].clone();
        }
    }
    v
}

impl Agent for AdaptiveWorkflow {
    fn id(&self) -> &'static str {
        "adaptive_workflow"
    }
    fn operations(&self) -> &'static [&'static str] {
        &["remember", "recall", "list", "update", "forget", "clear", "status", "remember_plugin_preference", "plugin_preferences"]
    }
    fn run(&self, op: &str, input: &Value, cx: &mut Ctx) -> OpResult {
        cx.tool("memory.facts");
        let list = |cx: &Ctx| memory::list(&cx.db.lock().unwrap(), Some(CAT)).map_err(mem_err);
        match op {
            "remember" => {
                let kind = s(input, "kind").unwrap_or("");
                let need = match kind {
                    "shortcut" => "keys",
                    "procedure" => "steps",
                    _ => "value",
                };
                if input[need].is_null() {
                    return Err(AgentError::input(format!("a {kind} needs {need}")));
                }
                let name = s(input, "name").unwrap_or("");
                let scope = s(input, "scope").unwrap_or("custom");
                if scope == "standard" && s(input, "source").is_none() {
                    cx.limit("marked as a standard command without a source: the user can correct it");
                }
                let value = value_of(input, &Value::Null);
                let existing = list(cx)?.into_iter().find(|f| f.key.eq_ignore_ascii_case(name) && f.value["daw"] == value["daw"]);
                let c = cx.db.lock().unwrap();
                let (fact, updated) = match existing {
                    Some(f) => (memory::update(&c, &f.id, Some(name), Some(&value), Some(scope)).map_err(mem_err)?, true),
                    None => (memory::add(&c, CAT, name, &value, "explicit", scope, 1.0, s(input, "source")).map_err(mem_err)?, false),
                };
                Ok(Outcome::new(json!({ "fact": fact, "updated_existing": updated }), "none"))
            }
            "recall" => {
                let q: Vec<String> = s(input, "query").unwrap_or("").to_lowercase().split_whitespace().filter(|w| w.len() > 1).map(String::from).collect();
                let mut scored: Vec<(usize, memory::Fact)> = list(cx)?
                    .into_iter()
                    .filter(|f| s(input, "daw").is_none_or(|d| f.value["daw"] == d) && s(input, "kind").is_none_or(|k| f.value["kind"] == k))
                    .map(|f| {
                        let text = format!("{} {}", f.key, f.value).to_lowercase();
                        (q.iter().filter(|w| text.contains(w.as_str())).count(), f)
                    })
                    .filter(|(sc, _)| q.is_empty() || *sc > 0)
                    .collect();
                scored.sort_by(|a, b| b.0.cmp(&a.0).then(b.1.updated_at.cmp(&a.1.updated_at)));
                let facts: Vec<Value> = scored.into_iter().take(20).map(|(sc, f)| json!({ "score": sc, "fact": f })).collect();
                Ok(Outcome::new(json!({ "count": facts.len(), "facts": facts }), "none"))
            }
            "list" => Ok(Outcome::new(json!({ "facts": list(cx)? }), "none")),
            "update" => {
                let id = s(input, "id").unwrap_or("");
                let c = cx.db.lock().unwrap();
                let old = memory::get(&c, id).map_err(mem_err)?.filter(|f| f.category == CAT).ok_or_else(|| AgentError::new("NOT_FOUND", "memory item not found"))?;
                let f = memory::update(&c, id, s(input, "name"), Some(&value_of(input, &old.value)), s(input, "scope")).map_err(mem_err)?;
                Ok(Outcome::new(json!({ "fact": f }), "none"))
            }
            "forget" => {
                let id = s(input, "id").unwrap_or("");
                let c = cx.db.lock().unwrap();
                memory::get(&c, id).map_err(mem_err)?.filter(|f| f.category == CAT).ok_or_else(|| AgentError::new("NOT_FOUND", "memory item not found"))?;
                memory::delete(&c, id).map_err(mem_err)?;
                Ok(Outcome::new(json!({ "deleted": id }), "none"))
            }
            "clear" => {
                let n = memory::clear(&cx.db.lock().unwrap(), CAT).map_err(mem_err)?;
                Ok(Outcome::new(json!({ "deleted": n }), "none"))
            }
            // Plug-in habits (usual instrument, preset or effect chain for a role) go to the same memory, category
            // "plugin": the Plugin Intelligence System reads them; it never keeps a memory of its own.
            "remember_plugin_preference" => {
                let kind = s(input, "kind").unwrap_or("instrument");
                if !["instrument", "preset", "chain"].contains(&kind) {
                    return Err(AgentError::input("kind must be instrument, preset or chain"));
                }
                let scope = s(input, "role").or(s(input, "goal")).ok_or_else(|| AgentError::input("role (or goal for a chain) is required"))?;
                let c = cx.db.lock().unwrap();
                let key = if kind == "instrument" { format!("preferred_instrument:{scope}") } else { format!("preferred_{kind}:{scope}") };
                let value = json!({ "product_id": input["product_id"], "preset_id": input["preset_id"], "stages": input["stages"] });
                if let Some(old) = memory::list(&c, Some("plugin")).map_err(mem_err)?.into_iter().find(|f| f.key == key) {
                    memory::delete(&c, &old.id).map_err(mem_err)?;
                }
                let f = memory::add(&c, "plugin", &key, &value, "explicit", "custom", 1.0, Some("adaptive_workflow")).map_err(mem_err)?;
                Ok(Outcome::new(json!({ "fact": f }), "none"))
            }
            "plugin_preferences" => Ok(Outcome::new(json!({ "facts": memory::list(&cx.db.lock().unwrap(), Some("plugin")).map_err(mem_err)? }), "none")),
            "status" => {
                let n = list(cx)?.len();
                Ok(Outcome::new(json!({ "enabled": cx.granted("memory.workflow"), "items": n, "keystroke_logging": false }), "none"))
            }
            _ => Err(unknown(self.id(), op)),
        }
    }
}
