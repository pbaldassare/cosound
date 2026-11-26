//! COSOUND desktop core. Each module is independent and tested on its own; this file only exposes
//! them to the UI as Tauri commands. Engines still to come (audio analysis, voice, reference,
//! DAW connectors) plug in as new modules + capabilities in packages/agents/registry.json.
mod agents;
mod ail;
mod audio;
mod daw;
mod db;
mod library;
mod memory;
mod music;
mod plugins;
mod projects;
mod smartlib;
mod sound;

use db::Db;
use serde_json::{json, Value};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use tauri::{Emitter, Manager, State};

type R<T> = Result<T, String>;
fn e(err: impl ToString) -> String {
    err.to_string()
}

struct App {
    db: Arc<Db>,
    library: Arc<smartlib::service::Service>,
    agents: agents::runtime::Runtime,
    ail: Mutex<ail::Ail>,
    data_dir: PathBuf,
}

// ---------- app & settings ----------

#[tauri::command]
fn app_info(app: State<App>) -> Value {
    json!({
        "version": env!("CARGO_PKG_VERSION"),
        "os": std::env::consts::OS,
        "arch": std::env::consts::ARCH,
        "data_dir": app.data_dir,
        "temp_connector": cfg!(feature = "temp-connector"),
    })
}

#[tauri::command]
fn settings_get(app: State<App>) -> R<HashMap<String, String>> {
    db::settings_all(&app.db.lock().unwrap()).map_err(e)
}

#[tauri::command]
fn settings_set(app: State<App>, key: String, value: String) -> R<()> {
    if key.len() > 64 || value.len() > 4000 {
        return Err("setting too long".into());
    }
    db::set_setting(&app.db.lock().unwrap(), &key, &value).map_err(e)
}

#[tauri::command]
fn oplog_list(app: State<App>, project_id: Option<String>, limit: Option<i64>) -> R<Vec<db::OpEntry>> {
    db::ops(&app.db.lock().unwrap(), project_id.as_deref(), limit.unwrap_or(200)).map_err(e)
}

#[tauri::command]
fn oplog_undo(app: State<App>, id: i64) -> R<()> {
    db::undo_op(&app.db.lock().unwrap(), id)
}

// ---------- smart library ----------

#[tauri::command]
fn library_volumes(app: State<App>) -> R<Vec<library::VolumeView>> {
    let disks = library::current_disks();
    let c = app.db.lock().unwrap();
    library::refresh_volumes(&c, &disks).map_err(e)?;
    library::volumes(&c, &disks).map_err(e)
}

#[tauri::command]
fn library_roots(app: State<App>) -> R<Vec<library::RootView>> {
    library::roots(&app.db.lock().unwrap()).map_err(e)
}

#[tauri::command]
fn library_add_root(app: State<App>, path: String) -> R<String> {
    library::add_root(&app.db.lock().unwrap(), &library::current_disks(), &path)
}

#[tauri::command]
fn library_remove_root(app: State<App>, id: String) -> R<()> {
    library::remove_root(&app.db.lock().unwrap(), &id)
}

/// Queues an incremental scan (one folder, or all when `id` is empty); progress arrives as
/// `library-scan` events from the Smart Library service.
#[tauri::command]
fn library_scan(app: State<App>, id: String) {
    app.library.request_scan(if id.is_empty() { None } else { Some(id) });
}

#[tauri::command]
fn library_cancel(app: State<App>) {
    app.library.cancel();
}

// ---------- Smart Library System (Prompt 07) ----------

#[tauri::command]
async fn library_query(app: State<'_, App>, text: String, kind: Option<String>, limit: Option<usize>) -> R<Value> {
    Ok(smartlib::api::search_text(&app.db.lock().unwrap(), &text, limit.unwrap_or(100).min(500), kind.as_deref()))
}

#[tauri::command]
fn library_detail(app: State<App>, id: i64) -> R<Value> {
    smartlib::api::detail(&app.db.lock().unwrap(), id)
}

#[tauri::command]
async fn library_similar(app: State<'_, App>, id: i64) -> R<Value> {
    smartlib::api::similar(&app.db.lock().unwrap(), id, 30)
}

#[tauri::command]
async fn library_loops(app: State<'_, App>, bpm: f64, key: Option<String>, category: Option<String>) -> R<Value> {
    if !(40.0..=250.0).contains(&bpm) {
        return Err("BPM must be between 40 and 250".into());
    }
    let key = key.filter(|k| !k.is_empty()).map(|k| music::theory::Key::parse(&k).map(|k| k.name())).transpose()?;
    Ok(smartlib::api::loops(&app.db.lock().unwrap(), bpm, key.as_deref(), category.as_deref().filter(|c| !c.is_empty()), 100))
}

#[tauri::command]
async fn library_duplicates(app: State<'_, App>, similar: bool) -> R<Value> {
    Ok(smartlib::api::duplicates(&app.db, &std::sync::atomic::AtomicBool::new(false), similar))
}

#[tauri::command]
fn library_overview(app: State<App>) -> Value {
    let c = app.db.lock().unwrap();
    let mut v = smartlib::api::overview(&c);
    v["service"] = app.library.status();
    v["analysis"] = smartlib::api::analysis_status(&c);
    v["formats"] = smartlib::api::formats();
    v
}

#[tauri::command]
fn library_correct(app: State<App>, id: i64, field: String, value: String) -> R<Value> {
    let c = app.db.lock().unwrap();
    smartlib::index::correct(&c, id, &field, &value)?;
    smartlib::api::detail(&c, id)
}

#[tauri::command]
fn library_exclude(app: State<App>, root_id: String, folder: String, excluded: bool) -> R<()> {
    smartlib::index::set_exclusion(&app.db.lock().unwrap(), &root_id, &folder, excluded)
}

#[tauri::command]
fn library_exclude_device(app: State<App>, volume_id: String, excluded: bool) -> R<()> {
    smartlib::index::set_volume_excluded(&app.db.lock().unwrap(), &volume_id, excluded)
}

#[tauri::command]
fn library_search(app: State<App>, query: String, kind: Option<String>) -> R<Vec<library::FileHit>> {
    library::search(&app.db.lock().unwrap(), &query, kind.as_deref(), 300).map_err(e)
}

#[tauri::command]
fn library_stats(app: State<App>) -> R<Value> {
    library::stats(&app.db.lock().unwrap()).map_err(e)
}

// ---------- projects & imports ----------

#[tauri::command]
fn projects_list(app: State<App>, include_archived: bool) -> R<Vec<projects::Project>> {
    projects::list(&app.db.lock().unwrap(), include_archived)
}

#[tauri::command]
fn project_create(app: State<App>, name: String, daw: Option<String>, bpm: Option<f64>, musical_key: Option<String>) -> R<projects::Project> {
    projects::create(&app.db.lock().unwrap(), &name, daw.as_deref(), bpm, musical_key.as_deref())
}

#[tauri::command]
fn project_open(app: State<App>, id: String) -> R<projects::Project> {
    let c = app.db.lock().unwrap();
    db::set_setting(&c, "current_project", &id).map_err(e)?;
    projects::open(&c, &id)
}

#[tauri::command]
fn project_save(app: State<App>, id: String, fields: projects::ProjectFields) -> R<projects::Project> {
    projects::save(&app.db.lock().unwrap(), &id, &fields)
}

#[tauri::command]
fn project_rename(app: State<App>, id: String, name: String) -> R<projects::Project> {
    projects::rename(&app.db.lock().unwrap(), &id, &name)
}

#[tauri::command]
fn project_duplicate(app: State<App>, id: String, suffix: String) -> R<projects::Project> {
    projects::duplicate(&app.db.lock().unwrap(), &id, &suffix)
}

#[tauri::command]
fn project_archive(app: State<App>, id: String, archived: bool) -> R<projects::Project> {
    projects::set_archived(&app.db.lock().unwrap(), &id, archived)
}

#[tauri::command]
fn project_roots(app: State<App>, id: String) -> R<Vec<String>> {
    projects::linked_roots(&app.db.lock().unwrap(), &id)
}

#[tauri::command]
fn project_link_root(app: State<App>, id: String, root_id: String, linked: bool) -> R<()> {
    projects::link_root(&app.db.lock().unwrap(), &id, &root_id, linked)
}

/// Hashing large files can take a moment: async so the UI stays responsive.
#[tauri::command]
async fn imports_add(app: State<'_, App>, project_id: Option<String>, paths: Vec<String>, provenance: String, role: Option<String>) -> R<Vec<String>> {
    projects::import(&app.db.lock().unwrap(), project_id.as_deref(), &paths, &provenance, role.as_deref())
}

#[tauri::command]
fn imports_list(app: State<App>, project_id: Option<String>) -> R<Vec<projects::Import>> {
    projects::imports(&app.db.lock().unwrap(), project_id.as_deref())
}

#[tauri::command]
fn imports_remove(app: State<App>, id: String) -> R<()> {
    projects::remove_import(&app.db.lock().unwrap(), &id)
}

/// Saves a microphone take recorded by the UI into the app data folder and registers it as voice material.
/// No transcription or audio→MIDI happens here: the Voice-to-Music engine is not implemented yet.
#[tauri::command]
fn voice_save_take(app: State<App>, project_id: Option<String>, bytes: Vec<u8>, ext: String, mode: String) -> R<String> {
    if !["webm", "ogg", "mp4", "m4a", "wav"].contains(&ext.as_str()) || !mode.chars().all(|c| c.is_ascii_alphanumeric() || c == '_') {
        return Err("invalid take".into());
    }
    let dir = app.data_dir.join("takes");
    std::fs::create_dir_all(&dir).map_err(e)?;
    let path = dir.join(format!("take-{mode}-{}.{ext}", db::new_id()));
    std::fs::write(&path, bytes).map_err(e)?;
    let ids = projects::import(&app.db.lock().unwrap(), project_id.as_deref(), &[path.to_string_lossy().to_string()], "own", Some("voice"))?;
    ids.into_iter().next().ok_or_else(|| "take not registered".into())
}

// ---------- DAW, plugins ----------

#[tauri::command]
async fn daw_detect() -> Vec<daw::DawFound> {
    daw::detect()
}

// ---------- Plugin Intelligence System (Prompt 08) ----------

/// Permission toggle from Settings ("1" on, "0" off, absent = default), same rule as the agent runtime.
fn perm(app: &App, setting: &str, default: bool) -> bool {
    db::setting(&app.db.lock().unwrap(), setting).ok().flatten().map_or(default, |v| v == "1")
}

fn need(app: &App, setting: &str, default: bool, what: &str) -> R<()> {
    if perm(app, setting, default) {
        Ok(())
    } else {
        Err(format!("{what} is turned off in Settings → Permissions"))
    }
}

#[tauri::command]
async fn plugins_scan(app: State<'_, App>) -> R<Value> {
    need(&app, "perm_plugin_scan", true, "Plug-in scanning")?;
    plugins::api::refresh(&app.db.lock().unwrap())
}

#[tauri::command]
fn plugins_overview(app: State<App>) -> Value {
    plugins::api::overview(&app.db.lock().unwrap())
}

#[tauri::command]
fn plugins_list(app: State<App>, text: String, kind: Option<String>, category: Option<String>, missing: Option<bool>) -> Value {
    json!(plugins::registry::list(&app.db.lock().unwrap(), &text, kind.as_deref().filter(|k| !k.is_empty()), category.as_deref().filter(|c| !c.is_empty()), missing.unwrap_or(false)))
}

#[tauri::command]
fn plugins_detail(app: State<App>, id: String) -> R<Value> {
    plugins::api::detail(&app.db.lock().unwrap(), &id, &plugins::knowledge::Knowledge::load(Some(&app.data_dir)))
}

#[tauri::command]
async fn plugins_discover(app: State<'_, App>, install_id: String) -> R<Value> {
    need(&app, "perm_plugin_host", true, "Loading plug-ins in COSOUND's isolated host")?;
    plugins::params::discover(&app.db.lock().unwrap(), &install_id, 30)
}

#[tauri::command]
fn plugins_presets_search(app: State<App>, text: String, product_id: Option<String>) -> Value {
    plugins::presets::search(&app.db.lock().unwrap(), &text, product_id.as_deref(), 50)
}

#[tauri::command]
async fn plugins_preset_analyze(app: State<'_, App>, id: i64) -> R<Value> {
    need(&app, "perm_plugin_host", true, "Loading plug-ins in COSOUND's isolated host")?;
    plugins::presets::analyze(&app.db.lock().unwrap(), id, &app.data_dir.join("generated").join("preset-analysis"))
}

#[tauri::command]
fn plugins_compare(app: State<App>, ids: Vec<String>) -> Value {
    plugins::api::compare(&app.db.lock().unwrap(), &ids)
}

#[tauri::command]
async fn plugins_chain(app: State<'_, App>, goal: String, file: Option<String>, bpm: Option<f64>, check: bool) -> R<Value> {
    let file = file.filter(|f| !f.is_empty()).map(PathBuf::from);
    let c = app.db.lock().unwrap();
    let mut p = plugins::chains::propose(&c, &goal, file.as_deref(), bpm)?;
    if check {
        let f = file.ok_or("choose the audio file the chain will process")?;
        need(&app, "perm_plugin_host", true, "Loading plug-ins in COSOUND's isolated host")?;
        p["check"] = plugins::chains::check(&c, &p, &f, &app.data_dir.join("generated").join("chain-check"))?;
    }
    Ok(p)
}

#[tauri::command]
fn plugins_daw(daw: String) -> R<Value> {
    plugins::daw::capabilities(&daw).ok_or_else(|| format!("unknown DAW {daw}"))
}

#[tauri::command]
async fn plugins_design_create(app: State<'_, App>, text: String, role: String, genre: Option<String>, key: Option<String>, bpm: Option<f64>, product_id: Option<String>) -> R<Value> {
    need(&app, "perm_plugin_host", true, "Loading plug-ins in COSOUND's isolated host")?;
    let f = |o: &Option<String>| o.clone().filter(|x| !x.is_empty());
    plugins::design::create(&app.db.lock().unwrap(), &text, &role, f(&genre).as_deref(), f(&key).as_deref(), bpm, f(&product_id).as_deref())
}

#[tauri::command]
async fn plugins_design_modify(app: State<'_, App>, id: String, text: String) -> R<Value> {
    plugins::design::modify(&app.db.lock().unwrap(), &id, &text)
}

#[tauri::command]
async fn plugins_design_audition(app: State<'_, App>, id: String) -> R<Value> {
    need(&app, "perm_plugin_host", true, "Loading plug-ins in COSOUND's isolated host")?;
    plugins::api::audition(&app.db.lock().unwrap(), &app.data_dir, &id)
}

#[tauri::command]
async fn plugins_design_save(app: State<'_, App>, id: String, name: String) -> R<Value> {
    plugins::design::save(&app.db.lock().unwrap(), &app.data_dir, &id, &name)
}

/// Writes outside COSOUND's folder (the plug-in's user preset folder): needs the permission, and the UI asks first.
#[tauri::command]
async fn plugins_design_install(app: State<'_, App>, id: String) -> R<Value> {
    need(&app, "perm_preset_write", false, "Saving presets into plug-in preset folders")?;
    plugins::design::install_preset(&app.db.lock().unwrap(), &id, None)
}

#[tauri::command]
fn plugins_design_uninstall(app: State<App>, id: String) -> R<Value> {
    plugins::design::remove_installed(&app.db.lock().unwrap(), &id)
}

#[tauri::command]
fn plugins_designs(app: State<App>) -> Value {
    json!(plugins::design::list(&app.db.lock().unwrap()))
}

#[tauri::command]
fn plugins_design_get(app: State<App>, id: String) -> R<Value> {
    plugins::design::get(&app.db.lock().unwrap(), &id).ok_or_else(|| "unknown design".into())
}

#[tauri::command]
fn plugins_design_plan(app: State<App>, id: String, daw: String) -> R<Value> {
    let d = plugins::design::get(&app.db.lock().unwrap(), &id).ok_or("unknown design")?;
    plugins::daw::setup_plan(&d, &daw)
}

/// The user's usual instrument for a role, kept in the shared memory (Adaptive Workflow permission).
#[tauri::command]
fn plugins_remember(app: State<App>, role: String, product_id: String) -> R<Value> {
    need(&app, "perm_workflow_memory", false, "Workflow memory")?;
    plugins::design::remember_instrument(&app.db.lock().unwrap(), &role, &product_id)
}

// ---------- music engine (M2) ----------

/// Chords, bass, melody and drums for the preview page. Deterministic: same input, same notes.
#[tauri::command]
async fn music_sketch(input: Value) -> R<Value> {
    music::caps::sketch(&input)
}

#[tauri::command]
fn music_options() -> Value {
    music::caps::options()
}

/// Writes the sketch as a Standard MIDI File into the app data folder and registers it in
/// "Imported material" (current project, provenance "own"). Nothing outside COSOUND's folder is touched.
#[tauri::command]
fn music_export_midi(app: State<App>, input: Value) -> R<Value> {
    let (bytes, title) = music::caps::export(&input)?;
    let dir = app.data_dir.join("generated");
    std::fs::create_dir_all(&dir).map_err(e)?;
    let slug: String = title.chars().map(|c| if c.is_ascii_alphanumeric() { c.to_ascii_lowercase() } else { '-' }).collect();
    let slug = slug.split('-').filter(|s| !s.is_empty()).collect::<Vec<_>>().join("-");
    let path = dir.join(format!("{slug}-{}.mid", &db::new_id()[..8]));
    std::fs::write(&path, &bytes).map_err(e)?;
    let c = app.db.lock().unwrap();
    let project = db::setting(&c, "current_project").map_err(e)?.filter(|p| !p.is_empty());
    let ids = projects::import(&c, project.as_deref(), &[path.to_string_lossy().to_string()], "own", Some("midi"))?;
    Ok(json!({ "path": path, "bytes": bytes.len(), "import_id": ids.first() }))
}

// ---------- Agent Integration Layer ----------

#[tauri::command]
fn ail_status(app: State<App>) -> ail::PortHealth {
    app.ail.lock().unwrap().port.health()
}

#[tauri::command]
async fn ail_send(app: State<'_, App>, text: String) -> R<Vec<ail::AgentEvent>> {
    Ok(app.ail.lock().unwrap().send(&app.agents, &text))
}

// ---------- Multi-Agent System (Prompt 06) ----------

#[tauri::command]
fn agents_registry(app: State<App>) -> Value {
    app.agents.registry_view()
}

#[tauri::command]
fn agents_tasks(app: State<App>, limit: Option<usize>) -> Vec<agents::protocol::TaskResult> {
    app.agents.tasks(limit.unwrap_or(100))
}

#[tauri::command]
fn agents_events(app: State<App>, since: u64) -> Vec<agents::protocol::Event> {
    app.agents.events_since(since)
}

/// The user's answer to an approval request (Safety Gate): only the user can approve.
#[tauri::command]
fn agents_approval_respond(app: State<App>, approval_id: String, approve: bool) -> R<agents::protocol::TaskResult> {
    app.agents.respond(&approval_id, approve, agents::runtime::Actor::User).map_err(|e| format!("{}: {}", e.code, e.message))
}

#[tauri::command]
fn agents_cancel(app: State<App>, task_id: String) -> R<agents::protocol::TaskResult> {
    app.agents.cancel(&task_id, agents::runtime::Actor::User).map_err(|e| format!("{}: {}", e.code, e.message))
}

#[tauri::command]
fn agents_set_enabled(app: State<App>, agent: String, enabled: bool) -> R<()> {
    app.agents.set_enabled(&agent, enabled)
}

// ---------- Music Memory (workflow memory, user side) ----------

#[tauri::command]
fn memory_list(app: State<App>, category: Option<String>) -> R<Vec<memory::Fact>> {
    memory::list(&app.db.lock().unwrap(), category.as_deref())
}

#[tauri::command]
fn memory_update(app: State<App>, id: String, key: Option<String>, value: Option<Value>, scope: Option<String>) -> R<memory::Fact> {
    if scope.as_deref().is_some_and(|s| s != "standard" && s != "custom") {
        return Err("scope must be standard or custom".into());
    }
    memory::update(&app.db.lock().unwrap(), &id, key.as_deref(), value.as_ref(), scope.as_deref())
}

#[tauri::command]
fn memory_delete(app: State<App>, id: String) -> R<()> {
    memory::delete(&app.db.lock().unwrap(), &id)
}

/// The UI asks the user to confirm before calling this (that confirmation is the approval).
#[tauri::command]
fn memory_clear(app: State<App>, category: String) -> R<usize> {
    memory::clear(&app.db.lock().unwrap(), &category)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            let data_dir = app.path().app_data_dir()?;
            std::fs::create_dir_all(&data_dir)?;
            let conn = db::open(&data_dir.join("cosound.db"))?;
            let db = Arc::new(Mutex::new(conn));
            let agents = agents::runtime::Runtime::new(db.clone(), data_dir.clone());
            let handle = app.handle().clone();
            let library = smartlib::service::Service::start(db.clone(), Box::new(move |ev, payload| {
                let _ = handle.emit(ev, payload);
            }));
            app.manage(App { db, library, agents, ail: Mutex::new(ail::Ail::new()), data_dir });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            app_info,
            settings_get,
            settings_set,
            oplog_list,
            oplog_undo,
            library_volumes,
            library_roots,
            library_add_root,
            library_remove_root,
            library_scan,
            library_cancel,
            library_query,
            library_detail,
            library_similar,
            library_loops,
            library_duplicates,
            library_overview,
            library_correct,
            library_exclude,
            library_exclude_device,
            library_search,
            library_stats,
            projects_list,
            project_create,
            project_open,
            project_save,
            project_rename,
            project_duplicate,
            project_archive,
            project_roots,
            project_link_root,
            imports_add,
            imports_list,
            imports_remove,
            voice_save_take,
            daw_detect,
            plugins_scan,
            plugins_overview,
            plugins_list,
            plugins_detail,
            plugins_discover,
            plugins_presets_search,
            plugins_preset_analyze,
            plugins_compare,
            plugins_chain,
            plugins_daw,
            plugins_design_create,
            plugins_design_modify,
            plugins_design_audition,
            plugins_design_save,
            plugins_design_install,
            plugins_design_uninstall,
            plugins_designs,
            plugins_design_get,
            plugins_design_plan,
            plugins_remember,
            music_options,
            music_sketch,
            music_export_midi,
            ail_status,
            ail_send,
            agents_registry,
            agents_tasks,
            agents_events,
            agents_approval_respond,
            agents_cancel,
            agents_set_enabled,
            memory_list,
            memory_update,
            memory_delete,
            memory_clear,
        ])
        .run(tauri::generate_context!())
        .expect("error while running COSOUND");
}
