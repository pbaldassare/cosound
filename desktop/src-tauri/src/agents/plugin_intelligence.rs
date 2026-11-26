//! I. Plugin Intelligence: a thin layer over the Plugin Intelligence System (`crate::plugins`). It never claims a
//! plug-in that was not detected; every answer says what was measured, read from documentation, or inferred.
use super::{s, unknown, Agent, AgentError, Ctx, OpResult, Outcome};
use crate::plugins::{api, chains, daw, design, knowledge::Knowledge, params, presets, registry};
use serde_json::{json, Value};

pub struct PluginIntelligence;

fn n(input: &Value, k: &str) -> Option<f64> {
    input[k].as_f64()
}

impl Agent for PluginIntelligence {
    fn id(&self) -> &'static str {
        "plugin_intelligence"
    }
    fn operations(&self) -> &'static [&'static str] {
        &["scan", "list", "find_for_role", "plugin_info", "map_parameters", "find_presets", "analyze_preset", "compare", "propose_chain", "check_chain", "daw_capabilities", "diagnostics"]
    }
    fn run(&self, op: &str, input: &Value, cx: &mut Ctx) -> OpResult {
        let err = |e: String| AgentError::input(e);
        match op {
            "scan" => {
                cx.tool("plugins.scanner");
                cx.tool("plugins.registry");
                cx.tool("plugins.presets");
                let r = api::refresh(&cx.db.lock().unwrap()).map_err(err)?;
                if r["host_available"] == false {
                    cx.limit("the isolated plug-in host is not available on this system: plug-ins are recognized but not loaded");
                }
                let ov = api::overview(&cx.db.lock().unwrap());
                Ok(Outcome::new(json!({ "scan": r, "overview": ov }), "measured"))
            }
            "list" => {
                cx.tool("plugins.registry");
                let c = cx.db.lock().unwrap();
                let list = registry::list(&c, s(input, "text").unwrap_or(""), s(input, "kind"), s(input, "category"), input["include_missing"].as_bool().unwrap_or(false));
                Ok(Outcome::new(json!({ "count": list.len(), "plugins": list }), "measured"))
            }
            "find_for_role" => {
                cx.tool("plugins.registry");
                cx.tool("plugins.catalog");
                cx.tool("sound.recipes");
                cx.tool("plugins.presets");
                let role = s(input, "role").unwrap_or("bass");
                let c = cx.db.lock().unwrap();
                let cands = design::instrument_candidates(&c, role);
                if cands.is_empty() {
                    cx.limit("no software instrument detected: nothing is proposed as available");
                }
                let query = format!("{role} {}", s(input, "text").unwrap_or(""));
                let found = presets::search(&c, &query, None, 10);
                let recipes: Vec<&str> = crate::sound::recipes_for(role, s(input, "genre")).iter().map(|r| r.id).collect();
                Ok(Outcome::new(json!({ "role": role, "candidates": cands, "presets": found["results"], "recipes": recipes }), "rule_based"))
            }
            "plugin_info" => {
                cx.tool("plugins.registry");
                cx.tool("plugins.knowledge");
                cx.tool("plugins.catalog");
                cx.tool("library.index");
                let c = cx.db.lock().unwrap();
                let id = match s(input, "product_id") {
                    Some(i) => i.to_string(),
                    None => {
                        let name = s(input, "name").ok_or_else(|| AgentError::input("give product_id or name"))?;
                        match registry::list(&c, name, None, None, true).first() {
                            Some(p) => p["id"].as_str().unwrap_or("").to_string(),
                            None => {
                                cx.limit(format!("{name} is not installed on this computer (not detected)"));
                                let k = Knowledge::load(Some(cx.data_dir));
                                return Ok(Outcome::new(json!({ "name": name, "installed": false, "knowledge": k.describe(name, None, None, &[]) }), "rule_based"));
                            }
                        }
                    }
                };
                let d = api::detail(&c, &id, &Knowledge::load(Some(cx.data_dir))).map_err(err)?;
                for l in d["knowledge"]["limits"].as_array().into_iter().flatten() {
                    cx.limit(l.as_str().unwrap_or(""));
                }
                Ok(Outcome::new(d, "measured"))
            }
            "map_parameters" => {
                cx.tool("plugins.host");
                cx.tool("plugins.parameters");
                let iid = s(input, "install_id").ok_or_else(|| AgentError::input("install_id is required"))?;
                let r = params::discover(&cx.db.lock().unwrap(), iid, 30).map_err(err)?;
                Ok(Outcome::new(r, "measured"))
            }
            "find_presets" => {
                cx.tool("plugins.presets");
                cx.tool("library.search");
                let r = presets::search(&cx.db.lock().unwrap(), s(input, "text").unwrap_or(""), s(input, "product_id"), input["limit"].as_u64().unwrap_or(20) as usize);
                if r["results"].as_array().is_some_and(|a| a.is_empty()) {
                    cx.limit("no preset found on this computer for this request: none is proposed");
                }
                Ok(Outcome::new(r, "mixed"))
            }
            "analyze_preset" => {
                cx.tool("plugins.host");
                cx.tool("plugins.presets");
                cx.tool("library.audio_analyzer");
                let id = input["preset_id"].as_i64().ok_or_else(|| AgentError::input("preset_id is required"))?;
                let work = cx.data_dir.join("generated").join("preset-analysis");
                Ok(Outcome::new(presets::analyze(&cx.db.lock().unwrap(), id, &work).map_err(err)?, "estimated"))
            }
            "compare" => {
                cx.tool("plugins.registry");
                cx.tool("plugins.parameters");
                let ids: Vec<String> = input["product_ids"].as_array().map(|a| a.iter().filter_map(|x| x.as_str().map(String::from)).collect()).unwrap_or_default();
                Ok(Outcome::new(api::compare(&cx.db.lock().unwrap(), &ids), "measured"))
            }
            "propose_chain" | "check_chain" => {
                cx.tool("plugins.chains");
                cx.tool("plugins.registry");
                let goal = s(input, "goal").unwrap_or("techno_bass");
                let file = match s(input, "file") {
                    Some(f) => Some(cx.authorized_file(f)?),
                    None => None,
                };
                if file.is_some() {
                    cx.tool("library.audio_analyzer");
                }
                let c = cx.db.lock().unwrap();
                let mut p = chains::propose(&c, goal, file.as_deref(), n(input, "bpm")).map_err(err)?;
                if file.is_none() {
                    cx.limit("no audio given: the chain is a generic starting point, not adapted to the material and not to be applied as is");
                }
                if op == "check_chain" {
                    let f = file.ok_or_else(|| AgentError::input("check_chain needs the audio file to process"))?;
                    cx.tool("plugins.host");
                    p["check"] = chains::check(&c, &p, &f, &cx.data_dir.join("generated").join("chain-check")).map_err(err)?;
                }
                Ok(Outcome::new(p, if op == "check_chain" { "measured" } else { "rule_based" }))
            }
            "daw_capabilities" => {
                cx.tool("plugins.daw_adapter");
                let d = s(input, "daw").unwrap_or("logic");
                Ok(Outcome::new(daw::capabilities(d).ok_or_else(|| AgentError::input(format!("unknown DAW {d}")))?, "convention"))
            }
            "diagnostics" => {
                cx.tool("plugins.registry");
                let c = cx.db.lock().unwrap();
                let problems: Vec<Value> = c
                    .prepare("select id, name, format, status, status_reason from plugin_installs where status in ('incompatible', 'error', 'missing')")
                    .unwrap()
                    .query_map([], |r| Ok(json!({ "install_id": r.get::<_, String>(0)?, "name": r.get::<_, String>(1)?, "format": r.get::<_, String>(2)?, "status": r.get::<_, String>(3)?, "reason": r.get::<_, Option<String>>(4)? })))
                    .map(|m| m.filter_map(Result::ok).collect())
                    .unwrap_or_default();
                Ok(Outcome::new(json!({ "problems": problems, "errors": registry::errors(&c, None, 50), "host_available": crate::plugins::host::available() }), "measured"))
            }
            _ => Err(unknown(self.id(), op)),
        }
    }
}
