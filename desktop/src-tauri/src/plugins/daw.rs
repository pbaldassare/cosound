//! DAW Integration Adapter for plug-ins: what each DAW lets COSOUND do with plug-ins, by route, with the
//! source in the knowledge base, and the plan handed to the DAW Controller (which alone talks to DAWs).
//! Statuses: "available" = works today without a connector (files COSOUND writes and the DAW reads);
//! "planned" = needs the DAW connector (milestone M4); "none" = no documented route.
use serde_json::{json, Value};

/// Operations every DAW entry declares (checked by the tests).
#[cfg(test)]
pub const OPERATIONS: [&str; 7] = ["list_project_instruments", "read_parameters", "set_parameters", "load_instrument", "presets", "configure_effects", "verify_result"];

pub fn capabilities(daw: &str) -> Option<Value> {
    let (name, source, rows): (&str, &str, [(&str, &str, &str); 7]) = match daw {
        "logic" => ("Logic Pro", "docs/knowledge/logic-pro/README.md; docs/knowledge/integration-sdks/README.md §3–4", [
            ("list_project_instruments", "none", "no public API to read the project; control surfaces show only the selected bank"),
            ("read_parameters", "planned", "Mackie Control / OSC parameter pages (bank-relative); COSOUND reads parameters by hosting the same Audio Unit itself (available)"),
            ("set_parameters", "planned", "Scripter TargetEvent (same channel strip, after 'Learn plug-in parameter'), MIDI learn, MCU/OSC; needs the connector (M4)"),
            ("load_instrument", "planned", "UI automation (Accessibility) of the plug-in slot menu, to verify"),
            ("presets", "available", "COSOUND writes .aupreset files into ~/Library/Audio/Presets/<Manufacturer>/<Plug-in>/, the Audio Unit user-preset folder; Logic not installed on the test Mac: listing in Logic's preset menu not verified"),
            ("configure_effects", "planned", "same routes as set_parameters"),
            ("verify_result", "available", "COSOUND renders the same Audio Unit with the same state and measures it; inside Logic: planned (bounce via UI)"),
        ]),
        "ableton" => ("Ableton Live", "docs/knowledge/ableton-live/README.md; integration-sdks §3(a)", [
            ("list_project_instruments", "planned", "Live Object Model via Max for Live / Remote Script (connector M4)"),
            ("read_parameters", "planned", "Device → DeviceParameter (name, value, min/max); up to 64 plug-in parameters unless configured"),
            ("set_parameters", "planned", "DeviceParameter.value via the connector"),
            ("load_instrument", "planned", "browser API via the connector, to verify"),
            ("presets", "available", "AU presets: same Audio Unit preset folder (macOS) — reading by Live not verified"),
            ("configure_effects", "planned", "via the connector"),
            ("verify_result", "planned", "via the connector"),
        ]),
        "reaper" => ("REAPER", "docs/knowledge/reaper/README.md", [
            ("list_project_instruments", "planned", "ReaScript TrackFX_GetCount / TrackFX_GetFXName (official API, connector M4)"),
            ("read_parameters", "planned", "TrackFX_GetNumParams / TrackFX_GetParam"),
            ("set_parameters", "planned", "TrackFX_SetParam"),
            ("load_instrument", "planned", "TrackFX_AddByName(\"AU:…\"/\"VST3:…\"/\"CLAP:…\")"),
            ("presets", "planned", "TrackFX_SetPreset; AU user presets folder (macOS)"),
            ("configure_effects", "planned", "TrackFX_* calls"),
            ("verify_result", "planned", "render via ReaScript"),
        ]),
        "fl_studio" => ("FL Studio", "docs/knowledge/fl-studio/README.md", [
            ("list_project_instruments", "planned", "MIDI scripting API, to verify"),
            ("read_parameters", "planned", "plugins module of the MIDI scripting API, to verify"),
            ("set_parameters", "planned", "plugins.setParamValue, to verify"),
            ("load_instrument", "none", "no documented route"),
            ("presets", "planned", "to verify"),
            ("configure_effects", "planned", "to verify"),
            ("verify_result", "none", "no documented route"),
        ]),
        "cubase" => ("Cubase", "docs/knowledge/cubase/README.md", [
            ("list_project_instruments", "none", "MIDI Remote API sees the focused/selected objects only, to verify"),
            ("read_parameters", "planned", "MIDI Remote: focused plug-in parameters (Quick Controls), to verify"),
            ("set_parameters", "planned", "MIDI Remote mapping, to verify"),
            ("load_instrument", "none", "no documented route"),
            ("presets", "available", "VST3 presets in the documented VST3 preset folders (.vstpreset) — writing them needs the plug-in's own state: not done by COSOUND"),
            ("configure_effects", "planned", "to verify"),
            ("verify_result", "none", "no documented route"),
        ]),
        "pro_tools" => ("Pro Tools", "docs/knowledge/pro-tools/README.md", [
            ("list_project_instruments", "none", "AAX only; external control via EUCON/HUI, to verify"),
            ("read_parameters", "none", "COSOUND cannot host AAX (integration-sdks §1.4)"),
            ("set_parameters", "planned", "EUCON/HUI, to verify"),
            ("load_instrument", "none", "no documented route"),
            ("presets", "none", "AAX presets (.tfx) format not documented"),
            ("configure_effects", "none", "no documented route"),
            ("verify_result", "none", "no documented route"),
        ]),
        _ => return None,
    };
    Some(json!({ "daw": daw, "name": name, "source": source, "operations": rows.iter().map(|(op, st, how)| json!({ "operation": op, "status": st, "route": how })).collect::<Vec<_>>() }))
}

/// Steps for the DAW Controller to bring a saved design into a DAW. Each step says how it is done today.
pub fn setup_plan(design: &Value, daw: &str) -> Result<Value, String> {
    let caps = capabilities(daw).ok_or_else(|| format!("unknown DAW {daw}"))?;
    let status = |op: &str| caps["operations"].as_array().unwrap().iter().find(|x| x["operation"] == op).map(|x| x["status"].clone()).unwrap_or(json!("none"));
    let inst = &design["spec"]["instrument"];
    let presets: Vec<Value> = design["files"].as_array().into_iter().flatten().filter(|f| f["kind"] == "preset").cloned().collect();
    let mut steps = vec![];
    if presets.is_empty() {
        steps.push(json!({ "step": "save the design as a preset in COSOUND first", "by": "user", "status": "required" }));
    } else if !presets.iter().any(|f| f["location"] == "plugin_presets") {
        steps.push(json!({ "step": "install the saved preset into the plug-in preset folder (asks for your approval)", "by": "plugin_intelligence.install_preset", "status": status("presets") }));
    }
    steps.push(json!({ "step": format!("insert {} on a software-instrument track", inst["name"].as_str().unwrap_or("the instrument")), "by": "daw_controller", "status": status("load_instrument"), "fallback": "user does it in the DAW" }));
    steps.push(json!({ "step": "choose the COSOUND preset from the plug-in's preset menu", "by": "daw_controller", "status": status("presets"), "fallback": "user picks it in the plug-in header" }));
    for (slot, u) in design["spec"]["chain"].as_array().into_iter().flatten().enumerate() {
        steps.push(json!({ "step": format!("insert {} after the instrument (slot {})", u["name"].as_str().unwrap_or("effect"), slot + 1), "by": "daw_controller", "status": status("configure_effects"), "fallback": "user inserts it and loads the saved preset" }));
    }
    steps.push(json!({ "step": "import the MIDI part generated by the music engine", "by": "daw_controller.import (MIDI file)", "status": "available" }));
    Ok(json!({ "daw": daw, "design": design["id"], "steps": steps, "capabilities": caps,
               "note": "COSOUND does not operate the DAW from here: these steps go to the DAW Controller, which executes only verified routes and asks for approval" }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_daw_declares_every_operation() {
        for d in ["logic", "ableton", "reaper", "fl_studio", "cubase", "pro_tools"] {
            let c = capabilities(d).unwrap();
            let ops: Vec<&str> = c["operations"].as_array().unwrap().iter().map(|o| o["operation"].as_str().unwrap()).collect();
            assert_eq!(ops, OPERATIONS.to_vec(), "{d}");
            assert!(c["operations"].as_array().unwrap().iter().all(|o| ["available", "planned", "none"].contains(&o["status"].as_str().unwrap())));
        }
        assert!(capabilities("garageband").is_none());
        let logic = capabilities("logic").unwrap();
        assert_eq!(logic["operations"][4]["status"], "available", "presets via the Audio Unit preset folder");
        assert_eq!(logic["operations"][2]["status"], "planned", "no parameter control in Logic without the connector");
    }
}
