//! Structured Instruction Generator: for each purpose of the request, the competent COSOUND component, the agent that
//! owns it (Prompt 06 registry), what it should do, with which inputs and under which constraints. It is not a plan:
//! no order, no execution, no choice between alternatives – the Central Agent decides all of that. An operation is
//! suggested only if it exists in the registry; `operation_available` says whether it works today.
use super::constraints::Constraint;
use super::context::{Ctx, Target};
use super::extract::Param;
use super::intent::Intent;
use super::safety::Destructive;
use super::text::Text;
use serde_json::{json, Value};
use std::sync::OnceLock;

fn registry() -> &'static Vec<(String, String, bool)> {
    static R: OnceLock<Vec<(String, String, bool)>> = OnceLock::new();
    R.get_or_init(|| {
        crate::agents::registry::Registry::load()
            .map(|r| r.manifests.iter().flat_map(|m| m.operations.iter().map(|o| (m.id.clone(), o.name.clone(), o.status == "available")).collect::<Vec<_>>()).collect())
            .unwrap_or_default()
    })
}

/// (exists in the registry, available today)
pub fn operation(agent: &str, op: &str) -> (bool, bool) {
    registry().iter().find(|(a, o, _)| a == agent && o == op).map_or((false, false), |x| (true, x.2))
}

pub struct Input<'a> {
    pub text: &'a Text,
    pub intents: &'a [Intent],
    pub targets: &'a [Target],
    pub params: &'a [Param],
    pub constraints: &'a [Constraint],
    pub destructive: &'a [Destructive],
    pub ctx: &'a Ctx,
    pub creating: bool,
    pub confirm_first: bool,
    pub act: &'a str,
    pub previous_agent: Option<String>,
}

fn param(ps: &[Param], name: &str) -> Option<Value> {
    ps.iter().find(|p| p.name == name).map(|p| p.value.clone())
}

pub fn generate(x: &Input) -> Vec<Value> {
    let mut out: Vec<Value> = vec![];
    let cons: Vec<String> = x.constraints.iter().map(|c| format!("{}:{}", c.kind, c.subject)).collect();
    let elements: Vec<&Target> = x.targets.iter().filter(|t| t.kind != "section").collect();
    let sections: Vec<String> = x.targets.iter().filter(|t| t.kind == "section").map(|t| t.element.clone()).collect();
    let track = elements.first().map(|t| json!({ "element": t.element, "track_id": t.track_id, "track_name": t.track_name })).unwrap_or(Value::Null);
    let forbidden = |el: &str| super::constraints::forbids(x.constraints, el);
    let push = |out: &mut Vec<Value>, component: &str, agent: &str, op: Option<&str>, purpose: String, inputs: Value, approval: bool, destructive: bool| {
        if out.iter().any(|o| o["component"] == component && o["suggested_operation"] == json!(op)) {
            return;
        }
        let (exists, available) = op.map_or((false, false), |o| operation(agent, o));
        out.push(json!({
            "component": component, "agent": agent, "purpose": purpose, "inputs": inputs, "constraints": cons,
            "suggested_operation": if exists { json!(op) } else { Value::Null }, "operation_available": available,
            "requires_approval": approval || x.confirm_first || destructive, "destructive": destructive,
        }));
    };
    match x.act {
        "undo" | "restore" => {
            let agent = x.previous_agent.clone().unwrap_or_else(|| "none".into());
            let op = if agent == "music_theory" { Some(if x.act == "undo" { "undo_edit" } else { "compare_versions" }) } else { None };
            push(&mut out, "versioning", &agent, op, format!("{} the result of the referenced request", if x.act == "undo" { "undo the last change to" } else { "restore an earlier version of" }), json!({ "text": x.text.raw }), false, false);
            return out;
        }
        "confirm" | "cancel" | "smalltalk" => return out,
        _ => {}
    }
    let has = |id: &str| x.intents.iter().any(|i| i.id == id);
    let parts: Vec<String> = elements.iter().map(|t| t.element.clone()).filter(|e| ["bass", "melody", "chords", "arp", "drums", "lead", "pad", "kick", "percussion", "hihat"].contains(&e.as_str()) && !forbidden(e)).collect();
    let general = json!({
        "genre": param(x.params, "genre"), "bpm": param(x.params, "bpm"), "key": param(x.params, "key"), "moods": x.params.iter().filter(|p| p.name == "mood").map(|p| p.value.clone()).collect::<Vec<_>>(),
        "length_bars": param(x.params, "length_bars"), "sections": sections,
    });
    let any_destructive = !x.destructive.is_empty();
    if has("composition") || has("music_creation") {
        let excluded: Vec<&str> = x.constraints.iter().filter(|c| c.kind == "exclude").map(|c| c.subject.as_str()).collect();
        push(&mut out, "music_intelligence", "music_theory", Some("compose"), "define key, harmony and the musical parts; write the MIDI".into(),
            json!({ "text": x.text.raw, "parts": if parts.is_empty() { json!(["all"]) } else { json!(parts) }, "exclude_parts": excluded, "general": general, "harmony": { "scale": param(x.params, "scale"), "chords": param(x.params, "chords"), "progression": param(x.params, "progression") } }), false, false);
        if parts.iter().any(|p| ["drums", "kick", "percussion", "hihat"].contains(&p.as_str())) || x.params.iter().any(|p| p.group == "rhythm") || parts.is_empty() && !forbidden("drums") {
            push(&mut out, "rhythm", "rhythm_engineer", Some("rhythm_pattern"), "define the rhythmic pattern and groove".into(), json!({ "general": general, "rhythm": x.params.iter().filter(|p| p.group == "rhythm").collect::<Vec<_>>() }), false, false);
        }
        if x.params.iter().any(|p| p.group == "sound") && x.creating {
            push(&mut out, "plugin_intelligence", "plugin_intelligence", Some("find_for_role"), "find an installed instrument that fits the sound described, if needed".into(), json!({ "role": parts.first(), "qualities": x.params.iter().filter(|p| p.group == "sound").collect::<Vec<_>>() }), false, false);
        }
        let daw = x.ctx.daw.clone().or_else(|| x.ctx.project.as_ref().and_then(|p| p.daw.clone()));
        if daw.is_some() || has("daw_management") {
            push(&mut out, "daw", "daw_controller", Some("plan_operations"), "prepare the import of the MIDI into the DAW, when the user authorizes it".into(), json!({ "daw": daw }), true, false);
        }
    }
    if has("midi_editing") {
        push(&mut out, "music_intelligence", "music_theory", Some("edit_midi"), "apply the requested MIDI changes to the named part only".into(), json!({ "text": x.text.raw, "track": track }), false, false);
    }
    if has("arrangement") {
        push(&mut out, "arrangement", "music_producer", Some("plan_arrangement"), "adjust the arrangement of the sections named".into(), json!({ "sections": sections, "text": x.text.raw, "general": general }), any_destructive, any_destructive);
    }
    if has("sound_design") {
        let op = if x.creating { "design_sound" } else { "modify_sound" };
        push(&mut out, "sound_design", "sound_designer", Some(op), "change the sound as described, keeping what the user asked to keep".into(),
            json!({ "request": x.text.raw, "target": track, "qualities": x.params.iter().filter(|p| p.group == "sound").collect::<Vec<_>>() }), x.constraints.iter().any(|c| c.subject == "preset"), false);
    }
    if has("plugin_configuration") {
        let chain = x.text.any(&["catena", "chain", "effetti", "effects"]);
        push(&mut out, "plugin_intelligence", "plugin_intelligence", Some(if chain { "propose_chain" } else { "find_for_role" }), "choose or configure the plug-ins concerned".into(), json!({ "text": x.text.raw, "target": track }), false, false);
    }
    if has("sample_selection") || has("library_search") {
        let loops = x.text.any(&["loop", "loops"]);
        let own_only = x.constraints.iter().any(|c| c.subject == "own_samples");
        push(&mut out, "smart_library", "smart_library", Some(if loops && param(x.params, "bpm").is_some() { "suggest_loops" } else { "search" }), "search the user's indexed libraries".into(), json!({ "text": x.text.raw, "bpm": param(x.params, "bpm"), "key": param(x.params, "key"), "only_user_libraries": own_only }), false, false);
    }
    if has("mixing") {
        push(&mut out, "mixing", "mixing_engineer", Some("apply_moves"), "change level, balance or processing of the target only".into(), json!({ "target": track, "technical": x.params.iter().filter(|p| p.group == "technical").collect::<Vec<_>>() }), false, false);
    }
    if has("mastering") {
        push(&mut out, "mastering", "mastering_engineer", Some("master_track"), "master to the requested target".into(), json!({ "loudness_target": param(x.params, "loudness_target") }), false, false);
    }
    if has("audio_analysis") {
        push(&mut out, "audio_analysis", "audio_analysis", Some("analyze_file"), "measure the audio requested".into(), json!({ "files": x.ctx.files }), false, false);
    }
    if has("reference_analysis") {
        push(&mut out, "reference", "reference_track", Some("build_profile"), "read the reference's general traits (never copy it)".into(), json!({ "reference": param(x.params, "reference"), "files": x.ctx.files }), false, false);
    }
    if has("track_editing") || any_destructive {
        push(&mut out, "daw", "daw_controller", Some("plan_operations"), "prepare the track operations requested".into(), json!({ "text": x.text.raw, "target": track, "destructive": x.destructive }), true, any_destructive);
    } else if has("daw_management") && !out.iter().any(|o| o["agent"] == "daw_controller") {
        push(&mut out, "daw", "daw_controller", Some("plan_operations"), "prepare the DAW operations requested".into(), json!({ "text": x.text.raw }), true, false);
    }
    if has("file_import") {
        push(&mut out, "projects", "none", None, "import the file into the project (COSOUND import: reference + SHA-256, no copy)".into(), json!({ "files": x.ctx.files }), false, false);
    }
    if has("project_management") {
        push(&mut out, "projects", "none", None, "project operation in COSOUND".into(), json!({ "text": x.text.raw }), false, false);
    }
    if has("workflow_learning") || x.constraints.iter().any(|c| c.scope == "permanent") {
        push(&mut out, "adaptive_workflow", "adaptive_workflow", Some("remember"), "remember this habit or permanent instruction (shared memory, with the user's permission)".into(), json!({ "text": x.text.raw, "constraints": x.constraints.iter().filter(|c| c.scope == "permanent").collect::<Vec<_>>() }), true, false);
    }
    if has("preferences") && !has("workflow_learning") {
        push(&mut out, "assistant_settings", "none", None, "change the assistant's settings (Settings page)".into(), json!({ "text": x.text.raw }), false, false);
    }
    if has("information") && out.is_empty() {
        let theory = x.text.any(&["scala", "accordo", "accordi", "intervallo", "tonalità", "modo", "scale", "chord", "interval", "key", "mode", "cadenza", "cadence"]);
        if theory {
            push(&mut out, "music_intelligence", "music_theory", Some("theory"), "answer the music theory question with the theory engine".into(), json!({ "text": x.text.raw }), false, false);
        } else {
            push(&mut out, "assistant", "none", None, "answer the question".into(), json!({ "text": x.text.raw }), false, false);
        }
    }
    out
}

#[cfg(test)]
mod tests {
    #[test]
    fn registry_operations_are_known() {
        assert_eq!(super::operation("music_theory", "compose"), (true, true));
        assert_eq!(super::operation("mixing_engineer", "apply_moves"), (true, false), "planned, not available");
        assert_eq!(super::operation("music_theory", "fly"), (false, false));
    }
}
