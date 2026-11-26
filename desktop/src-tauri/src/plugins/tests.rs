//! Plugin Intelligence System tests on the REAL plug-ins of the machine running them.
//! On macOS they use Apple's built-in Audio Units (always installed): AULowpass, AUDistortion, AUReverb2, AUDelay,
//! DLSMusicDevice, AUSampler … On other systems (CI on Linux/Windows) the host is unavailable and the tests that
//! need it say so and stop early; the format/scan/catalog tests with synthetic bundles run everywhere.
use super::*;
use crate::db::{self, Db};
use serde_json::{json, Value};
use std::path::Path;
use std::sync::{Arc, Mutex};

fn real() -> Option<(Arc<Db>, tempfile::TempDir)> {
    if !host::available() {
        eprintln!("plug-in host not available on this system: real-plug-in test skipped");
        return None;
    }
    let db: Arc<Db> = Arc::new(Mutex::new(db::open_memory()));
    api::refresh(&db.lock().unwrap()).unwrap();
    Some((db, tempfile::tempdir().unwrap()))
}

fn install_id(c: &rusqlite::Connection, name: &str) -> String {
    c.query_row("select id from plugin_installs where name = ?1", [name], |r| r.get(0)).unwrap_or_else(|_| panic!("{name} not detected"))
}

#[test]
fn recognizes_real_plugins_with_versions_and_categories() {
    let Some((db, _t)) = real() else { return };
    let c = db.lock().unwrap();
    let ov = api::overview(&c);
    assert!(ov["products"].as_i64().unwrap() >= 20, "{ov}");
    let lp = registry::list(&c, "AULowpass", None, None, false);
    assert_eq!(lp.len(), 1, "one product");
    assert_eq!(lp[0]["kind"], "effect");
    let cats: Vec<&str> = lp[0]["categories"].as_array().unwrap().iter().map(|x| x.as_str().unwrap()).collect();
    assert!(cats.contains(&"filter") && cats.contains(&"eq"), "{cats:?}");
    let i = registry::install(&c, &install_id(&c, "AULowpass")).unwrap();
    assert_eq!((i["format"].as_str(), i["scope"].as_str(), i["status"].as_str(), i["registered"].as_bool()), (Some("AU"), Some("apple"), Some("available"), Some(true)));
    assert!(i["version"].as_str().is_some_and(|v| v.split('.').count() == 3), "version read from the registry: {}", i["version"]);
    assert!(i["details"]["daw_compat"].as_array().unwrap().iter().any(|d| d["daw"] == "logic" && d["basis"] == "documented"));
    // instruments, Italian category search
    assert!(registry::list(&c, "", Some("instrument"), None, false).iter().any(|p| p["name"] == "DLSMusicDevice"));
    assert!(registry::list(&c, "riverbero", None, None, false).iter().any(|p| p["name"] == "AUReverb2"));
    assert!(registry::list(&c, "compressore", None, None, false).iter().any(|p| p["name"] == "AUMultibandCompressor"));
    // rescan: same products, nothing new; an install that disappears is "missing", and comes back
    let before = ov["products"].as_i64().unwrap();
    let again = api::refresh(&c).unwrap();
    assert_eq!(again["plugins"]["new"], 0);
    assert_eq!(api::overview(&c)["products"].as_i64().unwrap(), before);
    let (found, _) = scanner::scan();
    let fewer: Vec<_> = found.iter().filter(|i| i.name != "AUDelay").cloned().collect();
    let s = registry::sync(&c, &fewer, vec![]).unwrap();
    assert_eq!(s.missing, 1);
    assert!(!registry::list(&c, "AUDelay", None, None, false).first().map(|p| p["available"] == true).unwrap_or(false));
    let s = registry::sync(&c, &found, vec![]).unwrap();
    assert_eq!(s.back, 1);
}

#[test]
fn discovers_real_parameters_and_factory_presets() {
    let Some((db, _t)) = real() else { return };
    let c = db.lock().unwrap();
    let lp = params::discover(&c, &install_id(&c, "AULowpass"), 30).unwrap();
    let ps = lp["parameters"].as_array().unwrap();
    let cutoff = ps.iter().find(|p| p["name"] == "Cutoff Frequency").unwrap();
    assert_eq!((cutoff["role"].as_str(), cutoff["unit"].as_str(), cutoff["min"].as_f64()), (Some("cutoff"), Some("hertz"), Some(10.0)));
    // the documentation (Apple header) agrees with what the plug-in exposes
    let k = knowledge::Knowledge::load(None).about(Some("aufx:lpas:appl"), "AULowpass").unwrap();
    assert_eq!(k["parameters"][0]["id"].as_i64(), cutoff["address"].as_i64());
    let dist = params::discover(&c, &install_id(&c, "AUDistortion"), 30).unwrap();
    assert!(dist["factory_presets"].as_array().unwrap().len() >= 10);
    let pid = registry::install(&c, &install_id(&c, "AUDistortion")).unwrap()["product_id"].as_str().unwrap().to_string();
    assert!(presets::of_product(&c, &pid).len() >= 10, "factory presets stored");
    let d = api::detail(&c, &pid, &knowledge::Knowledge::load(None)).unwrap();
    let lv = &d["installs"][0]["levels"];
    assert_eq!((lv["recognized"].as_bool(), lv["analyzed"].as_bool()), (Some(true), Some(true)));
    assert!(lv["parameters_controllable"].as_u64().unwrap() > 5 && lv["presets_found"].as_i64().unwrap() >= 10);
}

#[test]
fn preset_search_and_measured_timbre() {
    let Some((db, t)) = real() else { return };
    let c = db.lock().unwrap();
    params::discover(&c, &install_id(&c, "AUReverb2"), 30).unwrap();
    let r = presets::search(&c, "large hall", None, 10);
    assert_eq!(r["results"][0]["name"], "Large Hall", "{}", r["results"][0]);
    assert!(presets::search(&c, "preset di lead potente per progressive house", None, 10)["results"].as_array().unwrap().iter().all(|p| p["origin"] == "library" || p["score"].as_f64().unwrap() > 0.0), "only matching, existing presets");
    params::discover(&c, &install_id(&c, "AUDistortion"), 30).unwrap();
    let id: i64 = c.query_row("select id from plugin_presets where source = 'factory' and name like 'Multi- Decimated 1%'", [], |r| r.get(0)).unwrap();
    let a = presets::analyze(&c, id, t.path()).unwrap();
    assert!(a["timbre"]["descriptors"]["aggressive"].is_number(), "{a}");
    assert!(presets::get(&c, id).unwrap()["details"]["timbre"].is_object(), "stored");
}

#[test]
fn designs_controls_auditions_and_saves_a_sound_without_touching_originals() {
    let Some((db, t)) = real() else { return };
    let c = db.lock().unwrap();
    // the prompt's example: deep, warm bass for melodic techno at 124 BPM with the installed synths
    let d = design::create(&c, "basso profondo e caldo", "bass", Some("melodic_techno"), Some("A minor"), Some(124.0), None).unwrap();
    assert!(d["instrument_choice"]["controllable"] == true, "{}", d["instrument_choice"]);
    assert_eq!(d["music"]["key"], "A minor");
    assert_eq!(d["music"]["bpm"], 124.0);
    assert!(d["spec"]["notes"].as_array().unwrap().len() > 4, "notes from the music engine");
    let chain: Vec<&str> = d["spec"]["chain"].as_array().unwrap().iter().filter_map(|u| u["name"].as_str()).collect();
    assert!(chain.contains(&"AULowpass") && chain.contains(&"AUDistortion"), "the GM synth has no filter/drive: effects added {chain:?}");
    // "più aggressivo, ma mantienilo caldo e profondo"
    let m = design::modify(&c, d["id"].as_str().unwrap(), "Rendi questo basso più aggressivo, ma mantienilo caldo e profondo").unwrap();
    let changes = m["changes"].as_array().unwrap();
    let roles: Vec<&str> = changes.iter().filter_map(|x| x["role"].as_str()).collect();
    assert!(roles.contains(&"drive") && roles.contains(&"resonance"), "{roles:?}");
    let cut = changes.iter().find(|x| x["role"] == "cutoff").unwrap();
    let (from, to) = (cut["from"].as_f64().unwrap(), cut["to"].as_f64().unwrap());
    assert!(to > from && to / from < 1.5, "aggressive opens the filter, warm/deep hold it back: {from} → {to}");
    // audition: rendered, measured, checked
    let dir = t.path();
    let a = api::audition(&c, dir, d["id"].as_str().unwrap()).unwrap();
    assert!(Path::new(a["before"]["file"].as_str().unwrap()).exists() && Path::new(a["after"]["file"].as_str().unwrap()).exists());
    for chk in a["checks"].as_array().unwrap() {
        assert_eq!(chk["as_requested"], true, "measured: {chk}");
    }
    assert_eq!(a["checks"].as_array().unwrap().len(), 3, "aggressive (more), warm and deep (kept) were all measured");
    let lp = registry::install(&c, &install_id(&c, "AULowpass")).unwrap();
    assert!(lp["details"]["control_verified_at"].is_string(), "control verified in the isolated host");
    // save twice: two different files, never overwritten
    let s1 = design::save(&c, dir, d["id"].as_str().unwrap(), "Deep Warm Bass").unwrap();
    let s2 = design::save(&c, dir, d["id"].as_str().unwrap(), "Deep Warm Bass").unwrap();
    let p1: Vec<String> = s1["files"].as_array().unwrap().iter().filter(|f| f["kind"] == "preset").map(|f| f["path"].as_str().unwrap().to_string()).collect();
    let p2: Vec<String> = s2["files"].as_array().unwrap().iter().filter(|f| f["kind"] == "preset").map(|f| f["path"].as_str().unwrap().to_string()).collect();
    assert!(p1.iter().all(|p| Path::new(p).exists()) && p2.iter().any(|p| p.ends_with("Deep Warm Bass 2.aupreset")), "{p2:?}");
    // the saved filter state reloads in the plug-in with the designed cutoff
    let lp_preset = p1.iter().find(|p| p.contains("AULowpass")).unwrap();
    let r = host::run("describe", &json!({ "type": "aufx", "subtype": "lpas", "manufacturer": "appl", "state_file": lp_preset }), 30).unwrap();
    let reloaded = r["parameters_after_state"].as_array().unwrap().iter().find(|p| p["address"] == 0).unwrap()["value"].as_f64().unwrap();
    assert!((reloaded - to).abs() / to < 0.01, "preset holds the cutoff {to}, reloaded {reloaded}");
    // install into a (test) plug-in preset folder, then undo
    let target = dir.join("Presets");
    let inst = design::install_preset(&c, d["id"].as_str().unwrap(), Some(&target)).unwrap();
    let installed: Vec<String> = inst["installed"].as_array().unwrap().iter().map(|f| f["path"].as_str().unwrap().to_string()).collect();
    assert!(installed.iter().all(|p| Path::new(p).exists() && p.starts_with(target.to_str().unwrap())));
    // the preset folder scan finds the new preset and attributes it to the right plug-in (AU codes inside the file)
    let found = presets::scan(&c, &[target.clone()]);
    assert!(found["stored"].as_u64().unwrap() >= 2, "{found}");
    let lp_pid = registry::install(&c, &install_id(&c, "AULowpass")).unwrap()["product_id"].as_str().unwrap().to_string();
    assert!(presets::of_product(&c, &lp_pid).iter().any(|p| p["source"] == "preset_folder" && p["name"] == "Deep Warm Bass"));
    let undo = design::remove_installed(&c, d["id"].as_str().unwrap()).unwrap();
    assert_eq!(undo["removed"].as_array().unwrap().len(), installed.len());
    assert!(installed.iter().all(|p| !Path::new(p).exists()));
    assert!(p1.iter().all(|p| Path::new(p).exists()), "COSOUND's own saved presets are kept");
    // plan for Logic Pro: presets available, parameter control planned
    let plan = daw::setup_plan(&design::get(&c, d["id"].as_str().unwrap()).unwrap(), "logic").unwrap();
    assert!(plan["steps"].as_array().unwrap().iter().any(|s| s["status"] == "available") && plan["steps"].as_array().unwrap().iter().any(|s| s["status"] == "planned"));
}

#[test]
fn pad_with_progressive_filter_opening_and_unknown_requests() {
    let Some((db, t)) = real() else { return };
    let c = db.lock().unwrap();
    let d = design::create(&c, "pad atmosferico con movimento lento, apertura progressiva del filtro e texture cinematografica", "pad", None, Some("A minor"), Some(122.0), None).unwrap();
    assert!(d["automation"].as_str().is_some_and(|a| a.contains("cutoff")), "{}", d["automation"]);
    assert!(d["spec"]["chain"].as_array().unwrap().iter().any(|u| u["name"] == "AUReverb2" || u["name"] == "AUMatrixReverb"), "reverb added for the pad");
    // "movimento lento": moved on a real modulation parameter if one exists, otherwise declared missing (never invented)
    let moved = d["changes"].as_array().unwrap().iter().any(|x| x["role"] == "lfo_depth" || x["role"] == "lfo_rate");
    let declared = d["not_available"].as_array().unwrap().iter().any(|x| x.as_str().unwrap().contains("lfo"));
    assert!(moved || declared, "{}", d["changes"]);
    for ch in d["changes"].as_array().unwrap() {
        let p = params::stored(&c, d["spec"]["chain"][ch["slot"].as_u64().unwrap() as usize - 1]["install_id"].as_str().unwrap_or(""));
        assert!(p.iter().any(|x| x["address"] == ch["address"] && x["name"] == ch["parameter"]), "every change targets a parameter the plug-in exposes");
    }
    api::audition(&c, t.path(), d["id"].as_str().unwrap()).unwrap();
    let none = design::modify(&c, d["id"].as_str().unwrap(), "fallo suonare come un violino").unwrap();
    assert!(none["changes"].as_array().unwrap().is_empty() && !none["not_available"].as_array().unwrap().is_empty());
}

#[test]
fn effect_chains_adapt_to_material_and_check_overload() {
    let Some((db, t)) = real() else { return };
    let c = db.lock().unwrap();
    let generic = chains::propose(&c, "techno_bass", None, Some(130.0)).unwrap();
    assert_eq!(generic["adapted_to_material"], false);
    assert!(generic["stages"].as_array().unwrap().iter().all(|s| s["plugin"].is_object()), "every stage has an installed Apple unit");
    // material: a loud bass rendered by the GM synth
    let bass = t.path().join("bass.wav");
    host::run("render", &json!({ "seconds": 4, "out": bass, "program": 38, "instrument": { "type": "aumu", "subtype": "dls ", "manufacturer": "appl" },
                                 "notes": [{ "pitch": 33, "start_s": 0, "dur_s": 1.8, "vel": 127 }, { "pitch": 36, "start_s": 2, "dur_s": 1.8, "vel": 127 }] }), 30).unwrap();
    let adapted = chains::propose(&c, "melodic_lead", Some(&bass), Some(124.0)).unwrap();
    assert_eq!(adapted["adapted_to_material"], true);
    assert!(adapted["stages"].as_array().unwrap().iter().any(|s| s["settings"].as_array().unwrap().iter().any(|x| x["role"] == "delay_time")), "tempo-synced delay");
    let check = chains::check(&c, &adapted, &bass, t.path()).unwrap();
    assert!(check["rendered_stages"].as_u64().unwrap() >= 3, "{check}");
    assert!(check["output"]["peak_dbfs"].is_number() && check["risks"].is_array());
    // the bass chain cleans the rumble with a high-pass, not a band-pass: the bass keeps its level
    let bass_chain = chains::propose(&c, "techno_bass", Some(&bass), Some(124.0)).unwrap();
    assert_eq!(bass_chain["stages"][0]["plugin"]["name"], "AUHipass");
    let chk = chains::check(&c, &bass_chain, &bass, t.path()).unwrap();
    assert!(chk["rms_change_db"].as_f64().unwrap() > -6.0, "{}", chk["rms_change_db"]);
}

#[test]
fn faulty_plugins_are_isolated_and_logged() {
    let Some((db, _t)) = real() else { return };
    let c = db.lock().unwrap();
    let e = host::run("describe", &json!({ "type": "aufx", "subtype": "none", "manufacturer": "nope" }), 30).unwrap_err();
    assert!(e.contains("could not load"), "{e}");
    let hang = host::run("render", &json!({ "seconds": 120, "out": "/dev/null", "instrument": { "type": "aumu", "subtype": "dls ", "manufacturer": "appl" } }), 0).unwrap_err();
    assert!(hang.contains("did not answer"), "a plug-in that does not answer in time is stopped: {hang}");
    // an install that is not usable is refused before loading, and the error is logged
    c.execute("update plugin_installs set status = 'incompatible', status_reason = 'test' where name = 'AUDelay'", []).unwrap();
    assert!(params::discover(&c, &install_id(&c, "AUDelay"), 30).is_err());
    registry::log_error(&c, Some("x"), "test", "boom");
    assert!(!registry::errors(&c, None, 5).is_empty());
}

#[test]
fn knowledge_and_daw_compatibility_are_sourced() {
    let k = knowledge::Knowledge::load(None);
    let d = k.describe("AUReverb2", Some("Apple"), Some("aufx:rvb2:appl"), &["reverb".into()]);
    assert!(d["documentation"]["source"]["title"].as_str().unwrap().contains("AudioUnitParameters.h"));
    assert!(d["effect_facts"][0]["source"].as_str().unwrap().starts_with("docs/knowledge/"));
    let v: Vec<Value> = registry::daw_compat("AAX", "macos");
    assert_eq!(v.len(), 1);
    assert_eq!(v[0]["daw"], "pro_tools");
    assert!(registry::daw_compat("AU", "macos").iter().any(|d| d["daw"] == "logic"));
    assert!(!registry::daw_compat("VST3", "macos").iter().any(|d| d["daw"] == "logic"), "Logic Pro loads Audio Units only");
}

#[test]
fn smart_library_presets_come_through_its_api() {
    // the Smart Library's own test library (presets for Massive, Vital, Serum in authorized folders)
    let l = crate::smartlib::tests::lib();
    let c = l.db.lock().unwrap();
    let r = presets::search(&c, "pad preset per Massive", None, 20);
    let lib: Vec<&Value> = r["results"].as_array().unwrap().iter().filter(|p| p["origin"] == "library").collect();
    assert!(lib.iter().any(|p| p["name"] == "Atmos.fxp"), "{lib:?}");
    assert!(lib.iter().all(|p| p["plugin_detected"] == false), "Massive is not installed here: the preset is found, the plug-in is not claimed");
    // nothing is copied into the plug-in tables
    assert_eq!(c.query_row("select count(*) from plugin_presets", [], |r| r.get::<_, i64>(0)).unwrap(), 0);
}
