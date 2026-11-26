//! Tests of the Multi-Agent System: registry consistency, contracts, every available operation of
//! every agent, permissions, approvals, cancellation, errors, MOCK collaboration and replaceability.
use super::protocol::*;
use super::registry::{self, Registry, AGENT_IDS};
use super::runtime::{Actor, Runtime};
use super::scenario::{self, ApiLink};
use super::schema::validate;
use crate::audio::wav::{sine, write_test_wav};
use crate::db::{self, Db};
use serde_json::{json, Value};
use std::path::PathBuf;
use std::sync::Arc;

struct Env {
    rt: Runtime,
    db: Arc<Db>,
    dir: PathBuf,
    _tmp: tempfile::TempDir,
}

fn env() -> Env {
    let tmp = tempfile::tempdir().unwrap();
    let dir = tmp.path().canonicalize().unwrap();
    let db: Arc<Db> = Arc::new(std::sync::Mutex::new(db::open_memory()));
    Env { rt: Runtime::new(db.clone(), dir.clone()), db, dir, _tmp: tmp }
}

fn req(agent: &str, op: &str, input: Value) -> TaskRequest {
    TaskRequest { contract_version: "1".into(), request_id: db::new_id(), task_id: None, trace_id: None, session_id: None, project_id: None, agent: agent.into(), operation: op.into(), input, requested_by: Some("test".into()), run_async: false }
}

impl Env {
    fn run(&self, agent: &str, op: &str, input: Value) -> TaskResult {
        self.rt.submit(req(agent, op, input))
    }
    fn wav(&self, name: &str, secs: f64, amp: f64, stereo: bool) -> String {
        let p = self.dir.join(name);
        let s = sine(48000, secs, 110.0, amp, 0.0);
        let data: Vec<f64> = if stereo { s.iter().flat_map(|&v| [v, v]).collect() } else { s };
        write_test_wav(&p, 48000, if stereo { 2 } else { 1 }, 24, false, &data);
        p.to_string_lossy().to_string()
    }
    /// A small authorized library (kick + 124 BPM drum loop) on a test drive; returns the kick's id.
    fn library(&self) -> i64 {
        use crate::smartlib::features::fixtures::{drum_loop, kick};
        let root = self.dir.join("Drive").join("Lib");
        std::fs::create_dir_all(&root).unwrap();
        write_test_wav(&root.join("Kick Deep.wav"), 44100, 1, 16, false, &kick(44100));
        write_test_wav(&root.join("Drum Loop.wav"), 44100, 1, 16, false, &drum_loop(44100, 124.0, 2));
        let disk = crate::library::Disk { label: "TESTDRIVE".into(), mount: self.dir.join("Drive"), fs: "apfs".into(), total: 1 << 30, available: 1 << 20, removable: true, disk_type: "SSD".into() };
        let rid = crate::library::add_root(&self.db.lock().unwrap(), &[disk], root.to_str().unwrap()).unwrap();
        crate::smartlib::scan::scan_root(&self.db, &rid, &std::sync::atomic::AtomicBool::new(false), &|_| {});
        while crate::smartlib::analyze::run_batch(&self.db, 20, &std::sync::atomic::AtomicBool::new(false), 0) > 0 {}
        self.db.lock().unwrap().query_row("select id from files where name = 'Kick Deep.wav'", [], |r| r.get(0)).unwrap()
    }
    fn set(&self, k: &str, v: &str) {
        db::set_setting(&self.db.lock().unwrap(), k, v).unwrap();
    }
}

#[test]
fn registry_is_consistent() {
    let reg = Registry::load().unwrap();
    assert_eq!(reg.manifests.len(), 13);
    let ids: Vec<&str> = reg.manifests.iter().map(|m| m.id.as_str()).collect();
    assert_eq!(ids, AGENT_IDS);
    let schema: Value = serde_json::from_str(registry::MANIFEST_SCHEMA).unwrap();
    let agents = super::all_agents();
    for m in &reg.manifests {
        validate(&schema, &serde_json::to_value(m).unwrap(), &m.id).unwrap();
        let imp = agents.iter().find(|a| a.id() == m.id).unwrap_or_else(|| panic!("{} has no implementation", m.id));
        let mut avail: Vec<&str> = m.operations.iter().filter(|o| o.status == "available").map(|o| o.name.as_str()).collect();
        let mut done: Vec<&str> = imp.operations().to_vec();
        avail.sort();
        done.sort();
        assert_eq!(avail, done, "{}: manifest 'available' ops must equal implemented ops", m.id);
        for o in &m.operations {
            assert_eq!(o.input["type"], "object", "{}.{} input schema", m.id, o.name);
            assert!(o.status == "available" || o.requires.is_some(), "{}.{} planned without 'requires'", m.id, o.name);
            for t in &o.tools {
                assert!(reg.tools.iter().any(|x| &x.id == t), "{}.{} uses unknown tool {t}", m.id, o.name);
                assert!(m.tools.contains(t));
            }
            for p in &o.permissions {
                assert!(reg.permissions.iter().any(|x| &x.id == p), "{}.{} uses unknown permission {p}", m.id, o.name);
            }
            if o.status == "available" {
                assert!(o.tools.iter().all(|t| reg.tools.iter().find(|x| &x.id == t).unwrap().status == "available"), "{}.{} is available but uses a planned tool", m.id, o.name);
            }
        }
        for c in &m.collaborates_with {
            assert!(AGENT_IDS.contains(&c.as_str()) && c != &m.id, "{}: collaborates_with {c}", m.id);
        }
        assert_eq!(m.competencies.it.len(), m.competencies.en.len(), "{} competencies it/en", m.id);
        assert_eq!(m.limits.it.len(), m.limits.en.len(), "{} limits it/en", m.id);
    }
    assert!(reg.permissions.iter().filter(|p| !p.grantable).all(|p| !p.default));
}

#[test]
fn every_available_operation_runs() {
    let e = env();
    e.set("perm_workflow_memory", "1");
    let kick = e.wav("kick.wav", 1.0, 0.9, false);
    let bass = e.wav("bass.wav", 1.0, 0.5, true);
    let mixdown = e.wav("mix.wav", 2.0, 0.7, true);
    let midi = e.run("music_theory", "generate_bassline", json!({ "genre": "techno", "bars": 2 })).result.unwrap()["midi"]["base64"].as_str().unwrap().to_string();
    let lib_kick = e.library();
    let a_minor = json!([{ "pitch": 57, "dur": 4 }, { "pitch": 60, "dur": 2 }, { "pitch": 64, "dur": 3 }, { "pitch": 62, "dur": 1 }, { "pitch": 65, "dur": 1 }, { "pitch": 67, "dur": 1 }, { "pitch": 59, "dur": 1 }]);
    let cases: Vec<(&str, &str, Value)> = vec![
        ("music_producer", "plan_arrangement", json!({ "genre": "techno" })),
        ("music_producer", "propose_variants", json!({ "genre": "house", "count": 3 })),
        ("music_producer", "production_plan", json!({ "genre": "trance" })),
        ("rhythm_engineer", "generate_drums", json!({ "genre": "techno", "bars": 4 })),
        ("rhythm_engineer", "generate_transition", json!({ "kind": "snare_build", "bars": 4 })),
        ("rhythm_engineer", "groove_timing", json!({ "bpm": 124, "swing": 50 })),
        ("sound_designer", "suggest_sounds", json!({ "role": "bass", "genre": "techno" })),
        ("sound_designer", "patch_variants", json!({ "recipe_id": "pad" })),
        ("music_theory", "key_info", json!({ "key": "F# minor" })),
        ("music_theory", "progression", json!({ "key": "C major", "progression": "ii7-V7-Imaj7" })),
        ("music_theory", "generate_chords", json!({ "genre": "edm" })),
        ("music_theory", "generate_bassline", json!({ "genre": "melodic_techno" })),
        ("music_theory", "generate_melody", json!({ "genre": "house" })),
        ("music_theory", "detect_key", json!({ "notes": a_minor })),
        ("music_theory", "check_coherence", json!({ "key": "A minor", "progression": "i", "bars": 1, "parts": [{ "id": "bass", "notes": [{ "pitch": 33, "start": 0, "dur": 240 }] }] })),
        ("audio_analysis", "analyze_file", json!({ "path": kick })),
        ("audio_analysis", "analyze_stems", json!({ "paths": [kick, bass] })),
        ("reference_track", "build_profile", json!({ "title": "Ref", "genre": "melodic_techno", "web_facts": { "bpm": 124, "key": "A minor", "sources": ["https://example.org/ref"] }, "audio_path": mixdown })),
        ("smart_library", "search", json!({ "text": "kick 124 bpm" })),
        ("smart_library", "describe_sound", json!({ "text": "deep punchy kick" })),
        ("smart_library", "find_similar", json!({ "file_id": lib_kick })),
        ("smart_library", "suggest_loops", json!({ "bpm": 126 })),
        ("smart_library", "find_presets", json!({ "text": "pad", "plugin": "Massive" })),
        ("smart_library", "find_midi", json!({ "text": "bassline" })),
        ("smart_library", "file_info", json!({ "file_ids": [lib_kick], "details": true })),
        ("smart_library", "find_duplicates", json!({ "include_similar": true })),
        ("smart_library", "stats", json!({})),
        ("smart_library", "volumes", json!({})),
        ("smart_library", "scan", json!({})),
        ("smart_library", "analyze_now", json!({ "limit": 5 })),
        ("smart_library", "analysis_status", json!({})),
        ("plugin_intelligence", "scan", json!({})),
        ("plugin_intelligence", "find_for_role", json!({ "role": "lead" })),
        ("plugin_intelligence", "plugin_info", json!({ "name": "Massive X" })),
        ("mixing_engineer", "analyze_mix", json!({ "tracks": [{ "name": "Kick", "path": kick, "role": "kick" }, { "name": "Bass", "path": bass, "role": "bass" }] })),
        ("mastering_engineer", "analyze_master", json!({ "path": mixdown, "destinations": ["spotify", "club"] })),
        ("daw_controller", "detect", json!({})),
        ("daw_controller", "plan_operations", json!({ "daw": "logic", "bpm": 124, "parts": [{ "name": "Bass", "midi_base64": midi }] })),
        ("adaptive_workflow", "remember", json!({ "kind": "shortcut", "name": "Bounce in place", "daw": "logic", "keys": "Ctrl+B" })),
        ("adaptive_workflow", "recall", json!({ "query": "bounce" })),
        ("adaptive_workflow", "list", json!({})),
        ("adaptive_workflow", "status", json!({})),
    ];
    let reg = Registry::load().unwrap();
    for (agent, op, input) in &cases {
        let r = e.run(agent, op, input.clone());
        assert_eq!(r.status, Status::Completed, "{agent}.{op}: {:?}", r.errors);
        assert!(r.result.is_some() && r.basis.is_some() && !r.tools_used.is_empty(), "{agent}.{op} result/basis/tools");
        let m = reg.manifest(agent).unwrap();
        assert!(r.tools_used.iter().all(|t| m.tools.contains(t)), "{agent}.{op} used undeclared tools {:?}", r.tools_used);
        assert_eq!(r.agent_version.as_deref(), Some(m.version.as_str()));
    }
    // update / forget / clear need an id or an approval: covered below; together 36 operations
    let id = e.run("adaptive_workflow", "list", json!({})).result.unwrap()["facts"][0]["id"].as_str().unwrap().to_string();
    assert_eq!(e.run("adaptive_workflow", "update", json!({ "id": id, "keys": "Ctrl+Alt+B" })).status, Status::Completed);
    assert_eq!(e.run("adaptive_workflow", "forget", json!({ "id": id })).status, Status::Completed);
    assert_eq!(e.run("adaptive_workflow", "clear", json!({})).status, Status::WaitingApproval);
    // Prompt 08 operations. Those that load plug-ins need the isolated host (macOS); elsewhere they are skipped.
    let check = |agent: &str, op: &str, input: Value| -> Value {
        let r = e.run(agent, op, input);
        assert_eq!(r.status, Status::Completed, "{agent}.{op}: {:?}", r.errors);
        let m = reg.manifest(agent).unwrap();
        assert!(r.tools_used.iter().all(|t| m.tools.contains(t)), "{agent}.{op} used undeclared tools {:?}", r.tools_used);
        r.result.unwrap()
    };
    let mut plugin_ops = 0;
    for (agent, op, input) in [
        ("plugin_intelligence", "list", json!({ "text": "riverbero" })),
        ("plugin_intelligence", "find_presets", json!({ "text": "hall" })),
        ("plugin_intelligence", "compare", json!({ "product_ids": ["apple/aureverb2", "apple/aumatrixreverb"] })),
        ("plugin_intelligence", "propose_chain", json!({ "goal": "melodic_lead", "bpm": 124 })),
        ("plugin_intelligence", "daw_capabilities", json!({ "daw": "logic" })),
        ("plugin_intelligence", "diagnostics", json!({})),
        ("sound_designer", "list_designs", json!({})),
        ("adaptive_workflow", "remember_plugin_preference", json!({ "kind": "instrument", "role": "bass", "product_id": "apple/dlsmusicdevice" })),
        ("adaptive_workflow", "plugin_preferences", json!({})),
    ] {
        check(agent, op, input);
        plugin_ops += 1;
    }
    let host_ops = 10;
    if crate::plugins::host::available() {
        e.set("perm_preset_write", "1");
        check("plugin_intelligence", "map_parameters", json!({ "install_id": "AU:aufx:dist:appl" }));
        let preset: i64 = e.db.lock().unwrap().query_row("select id from plugin_presets where source = 'factory' limit 1", [], |r| r.get(0)).unwrap();
        check("plugin_intelligence", "analyze_preset", json!({ "preset_id": preset }));
        check("plugin_intelligence", "check_chain", json!({ "goal": "techno_bass", "file": bass }));
        let d = check("sound_designer", "design_sound", json!({ "text": "basso profondo e caldo", "role": "bass", "genre": "melodic_techno", "key": "A minor", "bpm": 124 }));
        let id = d["id"].as_str().unwrap().to_string();
        check("sound_designer", "modify_sound", json!({ "design_id": id, "text": "più aggressivo ma mantienilo caldo" }));
        check("sound_designer", "audition", json!({ "design_id": id }));
        check("sound_designer", "save_preset", json!({ "design_id": id, "name": "Agent Bass" }));
        check("daw_controller", "plugin_setup_plan", json!({ "design_id": id, "daw": "logic" }));
        // writing into the plug-in preset folders (outside COSOUND) waits for the user's approval
        assert_eq!(e.run("sound_designer", "install_preset", json!({ "design_id": id })).status, Status::WaitingApproval);
        assert_eq!(e.run("sound_designer", "remove_installed_presets", json!({ "design_id": id })).status, Status::WaitingApproval);
        plugin_ops += host_ops;
    } else {
        eprintln!("plug-in host not available: {host_ops} plug-in operations not exercised on this system");
        plugin_ops += host_ops;
    }
    let available = reg.manifests.iter().flat_map(|m| &m.operations).filter(|o| o.status == "available").count();
    assert_eq!(cases.len() + 3 + plugin_ops, available, "every available operation is exercised");
}

#[test]
fn outputs_are_honest_and_specific() {
    let e = env();
    // measured loudness of a known tone, and what is NOT measured is said
    let p = e.wav("tone.wav", 3.0, 1.0, false);
    let r = e.run("audio_analysis", "analyze_file", json!({ "path": p }));
    assert_eq!(r.basis.as_deref(), Some("measured"));
    assert!(r.result.as_ref().unwrap()["measurements"]["integrated_lufs"].as_f64().is_some());
    assert!(r.limitations.iter().any(|l| l.contains("not measured yet")));
    // mastering never claims a master was produced
    let m = e.run("mastering_engineer", "analyze_master", json!({ "path": p }));
    assert_eq!(m.result.as_ref().unwrap()["mastered"], false);
    assert!(m.files.is_empty());
    assert!(m.limitations.iter().any(|l| l.contains("no mastering was performed")));
    let spotify = m.result.as_ref().unwrap()["destinations"].as_array().unwrap().iter().find(|d| d["id"] == "spotify").unwrap().clone();
    assert_eq!(spotify["reliability"], "official");
    // mixing proposals are never executed
    let mix = e.run("mixing_engineer", "analyze_mix", json!({ "tracks": [{ "path": p, "role": "kick" }] }));
    assert!(mix.result.unwrap()["suggestions"].as_array().unwrap().iter().all(|s| s["executed"] == false));
    // the DAW plan is prepared, not executed; the MIDI file really exists with its hash
    let midi = e.run("rhythm_engineer", "generate_drums", json!({ "genre": "house", "bars": 2 })).result.unwrap()["midi"]["base64"].as_str().unwrap().to_string();
    let d = e.run("daw_controller", "plan_operations", json!({ "parts": [{ "name": "Drums", "midi_base64": midi }] }));
    assert_eq!(d.status, Status::Completed, "{:?}", d.errors);
    assert_eq!(d.result.as_ref().unwrap()["executed"], false);
    assert!(d.proposed_daw_ops.iter().all(|o| o["executable_now"] == false));
    assert_eq!(d.files.len(), 1);
    let bytes = std::fs::read(&d.files[0].path).unwrap();
    assert!(bytes.starts_with(b"MThd") && d.files[0].sha256.len() == 64 && d.files[0].path.starts_with(e.dir.to_str().unwrap()));
    // reference profile labels the source of every value
    let rf = e.run("reference_track", "build_profile", json!({ "web_facts": { "bpm": 126, "sources": ["https://example.org"] } }));
    let prof = &rf.result.as_ref().unwrap()["profile"];
    assert_eq!((prof["bpm"]["basis"].as_str(), rf.basis.as_deref()), (Some("web"), Some("web")));
    assert!(rf.result.unwrap()["brief_for_original_track"]["rules"].as_array().unwrap().len() >= 3);
    // the producer suggests work for others instead of calling them
    let plan = e.run("music_producer", "production_plan", json!({ "genre": "melodic_techno", "bpm": 124 }));
    assert!(plan.needs.iter().any(|n| n["agent"] == "music_theory") && plan.needs.iter().all(|n| n["agent"] != "music_producer"));
    assert_eq!(plan.tools_used, ["music.arrangement", "music.theory"]);
    // key finding is labelled as an estimate
    let k = e.run("music_theory", "detect_key", json!({ "weights": [6.0, 0, 3, 0, 4, 2, 0, 5, 0, 2, 0, 1] }));
    assert_eq!((k.basis.as_deref(), k.result.unwrap()["best"].as_str()), (Some("estimated"), Some("C major")));
}

#[test]
fn errors_are_typed() {
    let e = env();
    let code = |r: TaskResult| (r.status, r.errors[0].code.clone());
    assert_eq!(code(e.run("nobody", "x", json!({}))), (Status::Failed, "AGENT_UNKNOWN".into()));
    assert_eq!(code(e.run("music_theory", "sing", json!({}))), (Status::Failed, "OPERATION_UNKNOWN".into()));
    let planned = e.run("voice_to_music", "transcribe_melody", json!({ "path": "x" }));
    assert_eq!(planned.errors[0].code, "NOT_IMPLEMENTED");
    assert!(planned.limitations[0].contains("M5"), "the missing component is named");
    assert_eq!(code(e.run("sound_designer", "suggest_sounds", json!({}))), (Status::Failed, "SCHEMA_INVALID".into()));
    assert_eq!(code(e.run("music_theory", "key_info", json!({ "key": "H major" }))), (Status::Failed, "SCHEMA_INVALID".into()));
    let elsewhere = tempfile::tempdir().unwrap();
    let outside = elsewhere.path().join("outside.wav");
    std::fs::write(&outside, "x").unwrap();
    assert_eq!(code(e.run("audio_analysis", "analyze_file", json!({ "path": outside }))).1, "PERMISSION_DENIED");
    let txt = e.dir.join("notes.txt");
    std::fs::write(&txt, "x").unwrap();
    assert_eq!(code(e.run("audio_analysis", "analyze_file", json!({ "path": txt }))).1, "UNSUPPORTED_FORMAT");
    let mut bad = req("music_theory", "key_info", json!({}));
    bad.contract_version = "2".into();
    assert_eq!(code(e.rt.submit(bad)).1, "PROTOCOL_VERSION");
    let mut dup = req("music_theory", "key_info", json!({}));
    dup.task_id = Some("same".into());
    assert_eq!(e.rt.submit(dup.clone()).status, Status::Completed);
    assert_eq!(code(e.rt.submit(dup)).1, "DUPLICATE_TASK");
    // failed requests are logged too
    let n: i64 = e.db.lock().unwrap().query_row("select count(*) from ail_tasks where status = 'FAILED'", [], |r| r.get(0)).unwrap();
    assert!(n >= 7);
}

#[test]
fn permissions_and_enable_disable() {
    let e = env();
    let r = e.run("adaptive_workflow", "remember", json!({ "kind": "shortcut", "name": "x", "keys": "Cmd+K" }));
    assert_eq!(r.errors[0].code, "PERMISSION_DENIED", "workflow memory is off by default");
    e.set("perm_workflow_memory", "1");
    assert_eq!(e.run("adaptive_workflow", "remember", json!({ "kind": "shortcut", "name": "x", "keys": "Cmd+K" })).status, Status::Completed);
    let secret = e.run("adaptive_workflow", "remember", json!({ "kind": "preference", "name": "login", "value": "my password is hunter2" }));
    assert_eq!(secret.errors[0].code, "PRIVATE_DATA_REFUSED");
    e.set("perm_plugin_scan", "0");
    assert_eq!(e.run("plugin_intelligence", "scan", json!({})).errors[0].code, "PERMISSION_DENIED");
    let sd = e.run("sound_designer", "suggest_sounds", json!({ "role": "pad" }));
    assert!(sd.status == Status::Completed && sd.limitations.iter().any(|l| l.contains("plugin scan is disabled")));
    e.rt.set_enabled("music_theory", false).unwrap();
    assert_eq!(e.run("music_theory", "key_info", json!({})).errors[0].code, "AGENT_DISABLED");
    e.rt.set_enabled("music_theory", true).unwrap();
    assert_eq!(e.run("music_theory", "key_info", json!({})).status, Status::Completed);
    assert_eq!(e.rt.registry_view()["permissions"].as_array().unwrap().iter().find(|p| p["id"] == "daw.control").unwrap()["granted"], false);
}

#[test]
fn approvals_only_from_the_user() {
    let e = env();
    e.set("perm_workflow_memory", "1");
    e.run("adaptive_workflow", "remember", json!({ "kind": "procedure", "name": "Vocal chain", "steps": ["EQ", "Compressor"] }));
    let w = e.run("adaptive_workflow", "clear", json!({}));
    assert_eq!(w.status, Status::WaitingApproval);
    let a = w.approval.clone().unwrap();
    assert_eq!(a.approvable_by, "user");
    // the central Agent cannot approve, only reject
    assert_eq!(e.rt.respond(&a.approval_id, true, Actor::CentralAgent).unwrap_err().code, "PERMISSION_DENIED");
    let api = e.rt.handle(&json!({ "type": "approval.respond", "approval_id": a.approval_id, "decision": "approve" }));
    assert_eq!((api["ok"].as_bool(), api["error"]["code"].as_str()), (Some(false), Some("PERMISSION_DENIED")));
    assert_eq!(e.rt.status(&w.task_id).unwrap().status, Status::WaitingApproval);
    // the user approves: it runs
    let done = e.rt.respond(&a.approval_id, true, Actor::User).unwrap();
    assert_eq!(done.status, Status::Completed);
    assert_eq!(done.result.unwrap()["deleted"], 1);
    // a rejection cancels
    let w2 = e.run("adaptive_workflow", "clear", json!({}));
    let r = e.rt.handle(&json!({ "type": "approval.respond", "approval_id": w2.approval.unwrap().approval_id, "decision": "reject" }));
    assert_eq!(r["data"]["status"], "CANCELLED");
    assert_eq!(r["data"]["errors"][0]["code"], "APPROVAL_REJECTED");
    let kinds: Vec<String> = e.rt.events_since(0).into_iter().map(|e| e.kind).collect();
    assert!(kinds.contains(&"approval.requested".into()) && kinds.contains(&"approval.resolved".into()));
}

#[test]
fn background_tasks_can_be_cancelled() {
    let e = env();
    let p = e.wav("long.wav", 40.0, 0.5, true);
    let mut r = req("audio_analysis", "analyze_file", json!({ "path": p }));
    r.run_async = true;
    let started = e.rt.submit(r);
    assert_eq!(started.status, Status::Running);
    let c = e.rt.cancel(&started.task_id, Actor::CentralAgent).unwrap();
    assert!(!c.status.is_final() || c.status == Status::Cancelled);
    let end = e.rt.wait(&started.task_id, 60_000).unwrap();
    assert_eq!(end.status, Status::Cancelled, "{:?}", end.errors);
    assert_eq!(e.rt.cancel(&started.task_id, Actor::User).unwrap_err().code, "ALREADY_FINISHED");
    // background task that completes
    let mut r = req("music_theory", "generate_melody", json!({}));
    r.run_async = true;
    let t = e.rt.submit(r);
    assert_eq!(e.rt.wait(&t.task_id, 10_000).unwrap().status, Status::Completed);
}

#[test]
fn activity_log_survives_restart() {
    let e = env();
    e.set("perm_workflow_memory", "1");
    let w = e.run("adaptive_workflow", "clear", json!({}));
    let ok = e.run("music_theory", "key_info", json!({}));
    let rt2 = Runtime::new(e.db.clone(), e.dir.clone());
    assert_eq!(rt2.status(&ok.task_id).unwrap().status, Status::Completed);
    let lost = rt2.status(&w.task_id).unwrap();
    assert_eq!((lost.status, lost.errors.last().unwrap().code.as_str()), (Status::Cancelled, "INTERRUPTED"));
    assert!(rt2.pending_approvals().is_empty());
}

#[test]
fn contracts_match_the_json_schemas() {
    let e = env();
    let r = e.run("music_theory", "key_info", json!({ "key": "D dorian" }));
    let v = serde_json::to_value(&r).unwrap();
    let schema: Value = serde_json::from_str(registry::RESULT_SCHEMA).unwrap();
    for k in schema["required"].as_array().unwrap() {
        assert!(v.get(k.as_str().unwrap()).is_some(), "TaskResult misses {k}");
    }
    validate(&json!({ "type": "object", "properties": { "status": schema["properties"]["status"] } }), &v, "").unwrap();
    let rq: Value = serde_json::from_str(registry::REQUEST_SCHEMA).unwrap();
    for k in rq["required"].as_array().unwrap() {
        assert!(serde_json::to_value(req("a", "b", json!({}))).unwrap().get(k.as_str().unwrap()).is_some());
    }
    // the documented example request parses
    let ex: TaskRequest = serde_json::from_value(json!({ "request_id": "r1", "agent": "music_theory", "operation": "generate_bassline", "input": { "key": "A minor", "bpm": 124 }, "async": true })).unwrap();
    assert!(ex.run_async && ex.contract_version == "1");
}

#[test]
fn collaboration_melodic_techno_124_via_mock() {
    let e = env();
    let mut link = e.rt.clone();
    let res = scenario::melodic_techno_124(&mut link, "mock");
    assert_eq!(res.len(), 6);
    for r in &res {
        assert_eq!(r.status, Status::Completed, "{}.{}: {:?}", r.agent, r.operation, r.errors);
        assert_eq!(r.requested_by.as_deref(), Some("mock"));
    }
    let agents: Vec<&str> = res.iter().map(|r| r.agent.as_str()).collect();
    assert_eq!(agents, ["music_producer", "rhythm_engineer", "music_theory", "smart_library", "sound_designer", "daw_controller"]);
    let ids: std::collections::HashSet<&str> = res.iter().map(|r| r.task_id.as_str()).collect();
    assert_eq!(ids.len(), 6, "unique task ids");
    assert!(res.iter().all(|r| r.trace_id == res[0].trace_id), "one trace across the scenario");
    // coherent results: same tempo and key everywhere
    let out = |i: usize| res[i].result.clone().unwrap();
    assert_eq!((out(0)["bpm"].as_f64(), out(1)["bpm"].as_f64()), (Some(124.0), Some(124.0)));
    let key = out(0)["key"].as_str().unwrap().to_string();
    assert_eq!(out(2)["key"]["name"], key.as_str());
    assert_eq!(out(5)["operations"][1]["args"]["key"], key.as_str());
    // the DAW step got the drums produced by the Rhythm Engineer, as a real file
    assert_eq!(res[5].files.len(), 1);
    assert!(std::fs::read(&res[5].files[0].path).unwrap().starts_with(b"MThd"));
    assert!(res[5].proposed_daw_ops.iter().any(|o| o["op"] == "import_midi"));
    assert!(res[5].proposed_daw_ops.iter().filter(|o| o["op"] == "create_marker").count() >= 8, "markers from the producer's sections");
    // no agent called another agent: every tool used is a Level-3 tool
    let reg = Registry::load().unwrap();
    for r in &res {
        assert!(r.tools_used.iter().all(|t| reg.tools.iter().any(|x| &x.id == t)));
        assert!(r.tools_used.iter().all(|t| !AGENT_IDS.contains(&t.as_str())));
    }
    assert!(scenario::summary(&res).starts_with("MOCK (simulation)"));
}

#[test]
fn mock_is_replaceable_by_a_json_connector() {
    // The same script through the JSON Agent API only (what an adapter for the real Agent does):
    // identical statuses and outputs, with no change to any agent.
    let a = env();
    let b = env();
    let mut direct = a.rt.clone();
    let via_mock = scenario::melodic_techno_124(&mut direct, "mock");
    let api = |m: &Value| b.rt.handle(m);
    let via_json = scenario::melodic_techno_124(&mut ApiLink(&api), "central_agent");
    for (x, y) in via_mock.iter().zip(&via_json) {
        assert_eq!((x.agent.as_str(), x.operation.as_str(), x.status), (y.agent.as_str(), y.operation.as_str(), y.status));
        assert_eq!(y.requested_by.as_deref(), Some("central_agent"));
        if x.agent != "daw_controller" {
            assert_eq!(x.result, y.result, "{}.{} differs", x.agent, x.operation);
        }
    }
    // the rest of the JSON API
    let reg = b.rt.handle(&json!({ "type": "registry.get" }));
    assert_eq!(reg["data"]["agents"].as_array().unwrap().len(), 13);
    let list = b.rt.handle(&json!({ "type": "task.list", "limit": 3 }));
    assert_eq!(list["data"].as_array().unwrap().len(), 3);
    let st = b.rt.handle(&json!({ "type": "task.status", "task_id": via_json[0].task_id }));
    assert_eq!(st["data"]["status"], "COMPLETED");
    assert!(b.rt.handle(&json!({ "type": "events.poll", "since": 0 }))["data"].as_array().unwrap().len() >= 12);
    assert_eq!(b.rt.handle(&json!({ "type": "nope" }))["error"]["code"], "MESSAGE_UNKNOWN");
    assert_eq!(b.rt.handle(&json!({ "type": "registry.get", "contract_version": "9" }))["error"]["code"], "PROTOCOL_VERSION");
}

/// Prompt 08 example through the JSON Agent API only (what the adapter for the partner's Central Agent will do):
/// "Voglio creare un basso profondo e caldo per una traccia Melodic Techno a 124 BPM. Utilizza i sintetizzatori che
/// ho già installato." Only the user can approve writing into the plug-in preset folder.
#[test]
fn central_agent_mock_designs_a_bass_with_installed_synths() {
    if !crate::plugins::host::available() {
        eprintln!("plug-in host not available: scenario skipped");
        return;
    }
    let e = env();
    e.set("perm_preset_write", "1");
    let presets_root = e.dir.join("AU Presets");
    *crate::plugins::design::TEST_PRESETS_ROOT.lock().unwrap() = Some(presets_root.clone());
    let api = |m: &Value| e.rt.handle(m);
    let mut n = 0;
    let mut call = |agent: &str, op: &str, input: Value| -> TaskResult {
        n += 1;
        let r = api(&json!({ "contract_version": "1", "type": "task.submit", "request": { "contract_version": "1", "request_id": format!("p8-{n}"), "agent": agent, "operation": op, "input": input, "run_async": false } }));
        serde_json::from_value(r["data"].clone()).unwrap()
    };
    let scan = call("plugin_intelligence", "scan", json!({}));
    assert_eq!(scan.status, Status::Completed);
    let synths = call("plugin_intelligence", "find_for_role", json!({ "role": "bass", "genre": "melodic_techno", "text": "profondo caldo" }));
    let cands = synths.result.clone().unwrap()["candidates"].as_array().unwrap().clone();
    assert!(!cands.is_empty() && cands.iter().all(|c| c["product_id"].is_string()), "only detected instruments");
    let d = call("sound_designer", "design_sound", json!({ "text": "basso profondo e caldo", "role": "bass", "genre": "melodic_techno", "key": "A minor", "bpm": 124 }));
    assert_eq!(d.status, Status::Completed, "{:?}", d.errors);
    assert_eq!(d.requested_by.as_deref(), Some("central_agent"));
    let id = d.result.unwrap()["id"].as_str().unwrap().to_string();
    let m = call("sound_designer", "modify_sound", json!({ "design_id": id, "text": "più aggressivo, ma mantienilo caldo e profondo" }));
    assert!(!m.result.unwrap()["changes"].as_array().unwrap().is_empty());
    let a = call("sound_designer", "audition", json!({ "design_id": id }));
    assert_eq!(a.basis.as_deref(), Some("measured"));
    assert!(a.result.unwrap()["checks"].as_array().unwrap().iter().all(|c| c["as_requested"] == true));
    let s = call("sound_designer", "save_preset", json!({ "design_id": id, "name": "Melodic Bass 124" }));
    assert_eq!(s.status, Status::Completed);
    // writing outside COSOUND's folder: waits; the Central Agent cannot approve, the user can
    let inst = call("sound_designer", "install_preset", json!({ "design_id": id }));
    assert_eq!(inst.status, Status::WaitingApproval);
    let ap = inst.approval.as_ref().unwrap().approval_id.clone();
    assert_eq!(api(&json!({ "type": "approval.respond", "approval_id": ap, "decision": "approve" }))["error"]["code"], "PERMISSION_DENIED");
    let done = e.rt.respond(&ap, true, Actor::User).unwrap();
    assert_eq!(done.status, Status::Completed, "{:?}", done.errors);
    let files = done.result.unwrap()["installed"].as_array().unwrap().clone();
    assert!(files.iter().all(|f| std::path::Path::new(f["path"].as_str().unwrap()).starts_with(&presets_root) && std::path::Path::new(f["path"].as_str().unwrap()).exists()));
    let plan = call("daw_controller", "plugin_setup_plan", json!({ "design_id": id, "daw": "logic" }));
    assert_eq!(plan.status, Status::Completed);
    assert!(plan.limitations.iter().any(|l| l.contains("plan only")));
    let st = api(&json!({ "type": "task.status", "task_id": inst.task_id }));
    assert_eq!(st["data"]["status"], "COMPLETED");
    *crate::plugins::design::TEST_PRESETS_ROOT.lock().unwrap() = None;
}

#[test]
fn daw_controller_uses_library_paths() {
    let e = env();
    let kick = e.library();
    let loop_id: i64 = e.db.lock().unwrap().query_row("select id from files where name = 'Drum Loop.wav'", [], |r| r.get(0)).unwrap();
    // the Smart Library Agent finds the loop from a sentence, the DAW Controller gets its original path
    let found = e.run("smart_library", "search", json!({ "text": "loop a 124 BPM" }));
    let res = found.result.clone().unwrap();
    assert_eq!(res["results"][0]["id"], loop_id, "{res}");
    let plan = e.run("daw_controller", "plan_operations", json!({ "daw": "logic", "bpm": 124, "parts": [{ "name": "Drums", "library_file_id": loop_id }, { "name": "Kick", "library_file_id": kick }] }));
    assert_eq!(plan.status, Status::Completed, "{:?}", plan.errors);
    let imports: Vec<&Value> = plan.proposed_daw_ops.iter().filter(|o| o["op"] == "import_audio").collect();
    assert_eq!(imports.len(), 2);
    assert!(std::path::Path::new(imports[0]["args"]["file"].as_str().unwrap()).ends_with(std::path::Path::new("Drive").join("Lib").join("Drum Loop.wav")));
    assert!(plan.files.is_empty(), "library files are referenced in place, never copied");
    // drive unplugged: the plan refuses instead of inventing a path
    crate::library::refresh_volumes(&e.db.lock().unwrap(), &[]).unwrap();
    let r = e.run("daw_controller", "plan_operations", json!({ "parts": [{ "name": "Kick", "library_file_id": kick }] }));
    assert_eq!(r.errors[0].code, "FILE_NOT_AVAILABLE", "{:?}", r.errors);
    let info = e.run("smart_library", "file_info", json!({ "file_ids": [kick] }));
    assert_eq!(info.result.unwrap()["files"][0]["usable"], false);
}
