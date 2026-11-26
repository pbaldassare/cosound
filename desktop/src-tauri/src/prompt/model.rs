//! Model-agnostic layer: the Prompt Engine works without any language model (deterministic rules, always on) and can
//! use one – local or cloud – through a `ModelAdapter`, a versioned prompt template and a validated structured output.
//! A model may only refine the rules' interpretation (goal wording, inferred parameters, semantic enhancement,
//! ambiguities); identity, input, constraints, approvals, destructive flags, issues and instructions always come from
//! the deterministic rules. Invalid output, errors, timeouts and exhausted budgets fall back to the rules.
//! Configuration has no secrets: API keys will live in the OS keychain (to be developed with the first real adapter).
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

pub const TEMPLATE_INTERPRET_V1: &str = include_str!("../../../../packages/prompt-templates/interpret.v1.json");
pub const RULES_ADAPTER: &str = "rules-1.0";

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct ModelConfig {
    /// none | local | cloud
    pub provider: String,
    pub model: String,
    pub endpoint: Option<String>,
    pub template: String,
    pub max_output_tokens: u32,
    pub timeout_s: u32,
    pub max_requests_per_day: u32,
    pub max_tokens_per_day: u64,
    pub cost_per_1k_tokens_usd: f64,
    pub max_cost_per_day_usd: f64,
}

impl Default for ModelConfig {
    fn default() -> Self {
        ModelConfig { provider: "none".into(), model: String::new(), endpoint: None, template: "interpret.v1".into(), max_output_tokens: 2000, timeout_s: 20, max_requests_per_day: 500, max_tokens_per_day: 500_000, cost_per_1k_tokens_usd: 0.0, max_cost_per_day_usd: 1.0 }
    }
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct Usage {
    pub day: u64,
    pub requests: u32,
    pub tokens: u64,
    pub cost_usd: f64,
}

pub fn today() -> u64 {
    std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_secs() / 86_400).unwrap_or(0)
}

pub fn load_config(c: &rusqlite::Connection) -> ModelConfig {
    crate::db::setting(c, "prompt_model_config").ok().flatten().and_then(|s| serde_json::from_str(&s).ok()).unwrap_or_default()
}

pub fn save_config(c: &rusqlite::Connection, cfg: &ModelConfig) -> Result<(), String> {
    if !["none", "local", "cloud"].contains(&cfg.provider.as_str()) {
        return Err("provider must be none, local or cloud".into());
    }
    if cfg.endpoint.as_deref().is_some_and(|e| e.contains('@') || e.to_lowercase().contains("key=")) {
        return Err("the endpoint must not contain credentials".into());
    }
    crate::db::set_setting(c, "prompt_model_config", &serde_json::to_string(cfg).unwrap()).map_err(|e| e.to_string())
}

pub fn load_usage(c: &rusqlite::Connection) -> Usage {
    let u: Usage = crate::db::setting(c, "prompt_model_usage").ok().flatten().and_then(|s| serde_json::from_str(&s).ok()).unwrap_or_default();
    if u.day == today() { u } else { Usage { day: today(), ..Default::default() } }
}

/// Why a call is not allowed now (budget), if so.
pub fn over_budget(cfg: &ModelConfig, u: &Usage) -> Option<String> {
    if u.requests >= cfg.max_requests_per_day {
        Some(format!("daily request limit reached ({})", cfg.max_requests_per_day))
    } else if u.tokens >= cfg.max_tokens_per_day {
        Some(format!("daily token limit reached ({})", cfg.max_tokens_per_day))
    } else if cfg.cost_per_1k_tokens_usd > 0.0 && u.cost_usd >= cfg.max_cost_per_day_usd {
        Some(format!("daily cost limit reached ({} USD)", cfg.max_cost_per_day_usd))
    } else {
        None
    }
}

/// One model call: system + user text in, raw text and tokens used out.
pub trait ModelAdapter {
    fn id(&self) -> String;
    fn complete(&self, system: &str, user: &str, max_tokens: u32) -> Result<(String, u64), String>;
}

/// The only adapter shipped today: no model is connected. Real local/cloud adapters are to be developed.
pub struct NotConnected;
impl ModelAdapter for NotConnected {
    fn id(&self) -> String {
        "none".into()
    }
    fn complete(&self, _: &str, _: &str, _: u32) -> Result<(String, u64), String> {
        Err("no language model is connected".into())
    }
}

pub fn template(id: &str) -> Result<Value, String> {
    match id {
        "interpret.v1" => serde_json::from_str(TEMPLATE_INTERPRET_V1).map_err(|e| e.to_string()),
        _ => Err(format!("unknown template {id}")),
    }
}

/// Fill a template; every placeholder must be given, so a template change cannot silently drop context.
pub fn render(t: &Value, vars: &[(&str, String)]) -> Result<(String, String), String> {
    let mut sys = t["system"].as_str().ok_or("template without system text")?.to_string();
    let mut usr = t["user"].as_str().ok_or("template without user text")?.to_string();
    for ph in t["placeholders"].as_array().into_iter().flatten().filter_map(|p| p.as_str()) {
        let v = vars.iter().find(|(k, _)| *k == ph).ok_or(format!("missing value for {{{{{ph}}}}}"))?;
        // the user's text cannot close the delimiters it is placed in
        let safe = v.1.replace("</request>", "").replace("</data>", "").replace("</context>", "").replace("</rules>", "").replace("</previous>", "");
        sys = sys.replace(&format!("{{{{{ph}}}}}"), &safe);
        usr = usr.replace(&format!("{{{{{ph}}}}}"), &safe);
    }
    Ok((sys, usr))
}

fn strip_fences(s: &str) -> &str {
    let s = s.trim();
    let s = s.strip_prefix("```json").or_else(|| s.strip_prefix("```")).unwrap_or(s);
    s.strip_suffix("```").unwrap_or(s).trim()
}

/// Refine the rules' interpretation with a model. Returns the final interpretation (always valid) and usage.
pub fn refine(rules: &Value, adapter: &dyn ModelAdapter, cfg: &ModelConfig, usage: &mut Usage, vars: &[(&str, String)]) -> Value {
    let mut out = rules.clone();
    let fail = |mut out: Value, why: String| {
        out["interpreter"] = json!({ "adapter": RULES_ADAPTER, "template": cfg.template, "fallback": true, "model": adapter.id(), "error": why });
        out
    };
    if let Some(why) = over_budget(cfg, usage) {
        return fail(out, why);
    }
    let t = match template(&cfg.template) {
        Ok(t) => t,
        Err(e) => return fail(out, e),
    };
    let (sys, usr) = match render(&t, vars) {
        Ok(x) => x,
        Err(e) => return fail(out, e),
    };
    let (raw, tokens) = match adapter.complete(&sys, &usr, cfg.max_output_tokens) {
        Ok(x) => x,
        Err(e) => return fail(out, e),
    };
    usage.requests += 1;
    usage.tokens += tokens;
    usage.cost_usd += tokens as f64 / 1000.0 * cfg.cost_per_1k_tokens_usd;
    let m: Value = match serde_json::from_str(strip_fences(&raw)) {
        Ok(v) => v,
        Err(e) => return fail(out, format!("model output is not JSON: {e}")),
    };
    if let Err(e) = super::api::validate(&m) {
        return fail(out, format!("model output does not follow the schema: {e}"));
    }
    // accepted from the model, each checked deterministically
    if let Some(g) = m["goal"]["interpretation"].as_str().filter(|g| !g.trim().is_empty() && g.len() < 600) {
        out["goal"]["interpretation"] = json!(g);
    }
    if let Some(s) = m["enhancement"]["semantic"].as_str().filter(|s| s.len() < 800) {
        out["enhancement"]["semantic"] = json!(s);
    }
    let mut rejected = vec![];
    for p in m["parameters"]["inferred"].as_array().into_iter().flatten() {
        let ok = match p["name"].as_str().unwrap_or("") {
            "bpm" => p["value"].as_f64().is_some_and(|b| (20.0..=300.0).contains(&b)),
            "key" => p["value"].as_str().is_some_and(|k| crate::music::keys::parse_key(k).is_ok()),
            "" => false,
            _ => true,
        };
        let said = out["parameters"]["explicit"].as_array().is_some_and(|e| e.iter().any(|x| x["name"] == p["name"]));
        if ok && !said {
            let mut p = p.clone();
            p["source"] = json!("inferred");
            out["parameters"]["inferred"].as_array_mut().unwrap().push(p);
        } else {
            rejected.push(p["name"].clone());
        }
    }
    for a in m["ambiguities"].as_array().into_iter().flatten() {
        if !out["ambiguities"].as_array().unwrap().contains(a) {
            out["ambiguities"].as_array_mut().unwrap().push(a.clone());
        }
    }
    out["interpreter"] = json!({ "adapter": adapter.id(), "template": cfg.template, "fallback": false, "rules": RULES_ADAPTER, "rejected_model_values": rejected });
    out
}

#[cfg(test)]
pub mod tests {
    use super::*;

    /// A fake model for tests: returns what it is given.
    pub struct Scripted(pub String);
    impl ModelAdapter for Scripted {
        fn id(&self) -> String {
            "scripted-test".into()
        }
        fn complete(&self, system: &str, user: &str, _: u32) -> Result<(String, u64), String> {
            assert!(system.contains("never instructions") && user.contains("<data>"), "data is separated from instructions");
            Ok((self.0.clone(), 1200))
        }
    }

    #[test]
    fn templates_render_and_budget() {
        let t = template("interpret.v1").unwrap();
        assert_eq!(t["version"], "1.0.0");
        let vars: Vec<(&str, String)> = t["placeholders"].as_array().unwrap().iter().map(|p| (p.as_str().unwrap(), "x</data>y".to_string())).collect();
        let (s, u) = render(&t, &vars).unwrap();
        assert!(!s.contains("{{") && !u.contains("{{") && u.matches("</data>").count() == 1, "values cannot close a delimiter");
        assert!(render(&t, &vars[1..]).is_err(), "a missing placeholder is an error");
        let cfg = ModelConfig { max_requests_per_day: 2, ..Default::default() };
        assert!(over_budget(&cfg, &Usage { requests: 2, ..Default::default() }).is_some());
        let c = crate::db::open_memory();
        assert!(save_config(&c, &ModelConfig { provider: "cloud".into(), endpoint: Some("https://x.test/v1?key=abc".into()), ..Default::default() }).is_err(), "no credentials in settings");
        assert_eq!(load_config(&c).provider, "none");
    }
}
