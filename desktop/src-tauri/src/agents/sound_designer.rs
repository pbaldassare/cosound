//! C. Sound Designer: recipes for a role, the instruments actually installed, matching presets.
use super::{s, unknown, Agent, AgentError, Ctx, OpResult, Outcome};
use crate::sound;
use serde_json::{json, Value};

pub struct SoundDesigner;

impl Agent for SoundDesigner {
    fn id(&self) -> &'static str {
        "sound_designer"
    }
    fn operations(&self) -> &'static [&'static str] {
        &["suggest_sounds", "patch_variants"]
    }
    fn run(&self, op: &str, input: &Value, cx: &mut Ctx) -> OpResult {
        cx.tool("sound.recipes");
        match op {
            "suggest_sounds" => {
                let role = s(input, "role").unwrap_or("bass");
                let genre = s(input, "genre");
                let recipes: Vec<Value> = sound::recipes_for(role, genre)
                    .iter()
                    .map(|r| json!({ "recipe": r, "variants": sound::variants(r).iter().map(|v| json!({ "name": v.0, "changes": v.1 })).collect::<Vec<_>>() }))
                    .collect();
                let mut installed = Vec::new();
                if cx.granted("plugins.scan") {
                    cx.tool("plugins.scanner");
                    cx.tool("plugins.catalog");
                    for p in crate::plugins::scan().into_iter().filter(|p| p.kind != "effect") {
                        let known = sound::known_synth(&p.name);
                        installed.push(json!({ "name": p.name, "vendor": p.vendor, "format": p.format, "kind": p.kind,
                            "known_as": known.map(|k| k.name), "synthesis": known.map(|k| k.synthesis), "detected": true }));
                    }
                    installed.sort_by_key(|v| v["known_as"].is_null());
                    if installed.is_empty() {
                        cx.limit("no software instrument was detected on this computer: the recipes apply to any synth, including the DAW's own instruments");
                    }
                } else {
                    cx.limit("plugin scan is disabled in Settings: installed instruments were not checked");
                }
                cx.tool("library.index");
                let presets = crate::library::search(&cx.db.lock().unwrap(), role, Some("preset"), 20).map_err(|e| AgentError::new("EXECUTION_FAILED", e.to_string()))?;
                Ok(Outcome::new(
                    json!({ "role": role, "genre": genre, "recipes": recipes, "installed_instruments": installed, "presets_in_library": presets, "source": sound::SOURCE_RECIPES }),
                    "rule_based",
                ))
            }
            "patch_variants" => {
                let id = s(input, "recipe_id").unwrap_or("");
                let r = sound::RECIPES.iter().find(|r| r.id == id).ok_or_else(|| AgentError::input(format!("unknown recipe {id}")))?;
                let variants: Vec<Value> = sound::variants(r).iter().map(|v| json!({ "name": v.0, "changes": v.1 })).collect();
                Ok(Outcome::new(json!({ "recipe": r, "variants": variants, "source": sound::SOURCE_RECIPES }), "rule_based"))
            }
            _ => Err(unknown(self.id(), op)),
        }
    }
}
