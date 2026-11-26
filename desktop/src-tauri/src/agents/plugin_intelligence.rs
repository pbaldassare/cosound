//! I. Plugin Intelligence: what is installed (never claims a plugin that was not detected) and what
//! COSOUND's knowledge base says about well-known synths.
use super::{s, unknown, Agent, Ctx, OpResult, Outcome};
use crate::sound;
use serde_json::{json, Value};

pub struct PluginIntelligence;

impl Agent for PluginIntelligence {
    fn id(&self) -> &'static str {
        "plugin_intelligence"
    }
    fn operations(&self) -> &'static [&'static str] {
        &["scan", "find_for_role", "plugin_info"]
    }
    fn run(&self, op: &str, input: &Value, cx: &mut Ctx) -> OpResult {
        let annotate = |p: &crate::plugins::Plugin| {
            let k = sound::known_synth(&p.name);
            json!({ "plugin": p, "known_as": k.map(|k| k.name), "synthesis": k.map(|k| k.synthesis) })
        };
        match op {
            "scan" => {
                cx.tool("plugins.scanner");
                cx.tool("plugins.catalog");
                let list = crate::plugins::scan();
                let count = |f: &dyn Fn(&crate::plugins::Plugin) -> bool| list.iter().filter(|p| f(p)).count();
                Ok(Outcome::new(
                    json!({
                        "total": list.len(),
                        "instruments": count(&|p| p.kind == "instrument"), "effects": count(&|p| p.kind == "effect"),
                        "by_format": { "VST3": count(&|p| p.format == "VST3"), "AU": count(&|p| p.format == "AU"), "CLAP": count(&|p| p.format == "CLAP"), "VST2": count(&|p| p.format == "VST2") },
                        "plugins": list.iter().map(annotate).collect::<Vec<_>>(),
                    }),
                    "measured",
                ))
            }
            "find_for_role" => {
                cx.tool("plugins.scanner");
                cx.tool("plugins.catalog");
                cx.tool("sound.recipes");
                let role = s(input, "role").unwrap_or("bass");
                let mut cands: Vec<Value> = crate::plugins::scan().iter().filter(|p| p.kind != "effect").map(annotate).collect();
                cands.sort_by_key(|c| c["known_as"].is_null());
                if cands.is_empty() {
                    cx.limit("no software instrument detected: nothing is proposed as available");
                }
                let recipes: Vec<&str> = sound::recipes_for(role, None).iter().map(|r| r.id).collect();
                Ok(Outcome::new(json!({ "role": role, "candidates": cands, "recipes": recipes }), "rule_based"))
            }
            "plugin_info" => {
                cx.tool("plugins.catalog");
                let name = s(input, "name").unwrap_or("");
                let known = sound::known_synth(name);
                let installed = if cx.granted("plugins.scan") {
                    cx.tool("plugins.scanner");
                    let n = name.to_lowercase();
                    Some(crate::plugins::scan().into_iter().filter(|p| match known {
                        Some(k) => sound::known_synth(&p.name).is_some_and(|x| x.id == k.id),
                        None => p.name.to_lowercase().contains(&n),
                    }).collect::<Vec<_>>())
                } else {
                    cx.limit("plugin scan is disabled in Settings: installation not checked");
                    None
                };
                if known.is_none() {
                    cx.limit(format!("{name} is not in COSOUND's synth catalog"));
                }
                Ok(Outcome::new(
                    json!({ "name": name, "catalog": known, "source": known.map(|_| sound::SOURCE_SYNTHS), "installed": installed.as_ref().map(|v| !v.is_empty()), "installed_versions": installed }),
                    "rule_based",
                ))
            }
            _ => Err(unknown(self.id(), op)),
        }
    }
}
