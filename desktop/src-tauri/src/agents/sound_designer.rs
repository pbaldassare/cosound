//! C. Sound Designer: recipes for a role, the instruments actually installed, matching presets, and sound design on
//! the real parameters of loadable plug-ins (Plugin Intelligence System: proposals, audition, new presets).
use super::{s, unknown, Agent, AgentError, Ctx, OpResult, Outcome};
use crate::plugins::{api, design};
use crate::sound;
use serde_json::{json, Value};

pub struct SoundDesigner;

impl Agent for SoundDesigner {
    fn id(&self) -> &'static str {
        "sound_designer"
    }
    fn operations(&self) -> &'static [&'static str] {
        &["suggest_sounds", "patch_variants", "design_sound", "modify_sound", "audition", "save_preset", "install_preset", "remove_installed_presets", "list_designs"]
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
                    cx.tool("plugins.registry");
                    cx.tool("plugins.catalog");
                    installed = design::instrument_candidates(&cx.db.lock().unwrap(), role);
                    if installed.is_empty() {
                        cx.limit("no software instrument was detected on this computer (run a plug-in scan): the recipes apply to any synth, including the DAW's own instruments");
                    }
                } else {
                    cx.limit("plugin scan is disabled in Settings: installed instruments were not checked");
                }
                cx.tool("library.index");
                let presets = {
                    let mut q = crate::smartlib::query::parse(role, &[]);
                    q.kind = Some("preset".into());
                    crate::smartlib::api::search_structured(&cx.db.lock().unwrap(), &q, 20)["results"].clone()
                };
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
            "design_sound" => {
                for t in ["plugins.registry", "plugins.host", "plugins.parameters", "plugins.design", "music.generator"] {
                    cx.tool(t);
                }
                let role = s(input, "role").unwrap_or("bass");
                let d = design::create(&cx.db.lock().unwrap(), s(input, "text").unwrap_or(""), role, s(input, "genre"), s(input, "key"), input["bpm"].as_f64(), s(input, "product_id")).map_err(AgentError::input)?;
                for m in d["not_available"].as_array().into_iter().flatten() {
                    cx.limit(m.as_str().unwrap_or(""));
                }
                Ok(Outcome::new(d, "rule_based"))
            }
            "modify_sound" => {
                cx.tool("plugins.design");
                cx.tool("plugins.parameters");
                let id = s(input, "design_id").ok_or_else(|| AgentError::input("design_id is required"))?;
                let d = design::modify(&cx.db.lock().unwrap(), id, s(input, "text").unwrap_or("")).map_err(AgentError::input)?;
                for m in d["not_available"].as_array().into_iter().flatten() {
                    cx.limit(m.as_str().unwrap_or(""));
                }
                Ok(Outcome::new(d, "rule_based"))
            }
            "audition" => {
                cx.tool("plugins.host");
                cx.tool("plugins.design");
                cx.tool("library.audio_analyzer");
                let id = s(input, "design_id").ok_or_else(|| AgentError::input("design_id is required"))?;
                Ok(Outcome::new(api::audition(&cx.db.lock().unwrap(), cx.data_dir, id).map_err(AgentError::input)?, "measured"))
            }
            "save_preset" => {
                cx.tool("plugins.host");
                cx.tool("plugins.design");
                let id = s(input, "design_id").ok_or_else(|| AgentError::input("design_id is required"))?;
                let name = s(input, "name").unwrap_or("COSOUND sound");
                Ok(Outcome::new(design::save(&cx.db.lock().unwrap(), cx.data_dir, id, name).map_err(AgentError::input)?, "measured"))
            }
            "install_preset" => {
                cx.tool("plugins.design");
                let id = s(input, "design_id").ok_or_else(|| AgentError::input("design_id is required"))?;
                Ok(Outcome::new(design::install_preset(&cx.db.lock().unwrap(), id, None).map_err(AgentError::input)?, "measured"))
            }
            "remove_installed_presets" => {
                cx.tool("plugins.design");
                let id = s(input, "design_id").ok_or_else(|| AgentError::input("design_id is required"))?;
                Ok(Outcome::new(design::remove_installed(&cx.db.lock().unwrap(), id).map_err(AgentError::input)?, "measured"))
            }
            "list_designs" => {
                cx.tool("plugins.design");
                let c = cx.db.lock().unwrap();
                Ok(Outcome::new(json!({ "designs": design::list(&c) }), "measured"))
            }
            _ => Err(unknown(self.id(), op)),
        }
    }
}
