//! COSOUND desktop core. Each module is independent and tested on its own; this file only exposes
//! them to the UI as Tauri commands. Engines still to come (MIDI, audio analysis, voice, reference,
//! DAW connectors) plug in as new modules + capabilities in packages/agents/registry.json.
mod ail;
mod daw;
mod db;
mod library;
mod plugins;
mod projects;

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

/// Starts a scan in the background; progress arrives as `library-scan` events.
#[tauri::command]
fn library_scan(handle: tauri::AppHandle, app: State<App>, id: String) {
    let db = app.db.clone();
    std::thread::spawn(move || {
        let emit = |count: usize, done: bool, error: Option<String>| {
            let _ = handle.emit("library-scan", json!({"root_id": id, "count": count, "done": done, "error": error}));
        };
        match library::scan_root(&db, &id, &|n| emit(n, false, None)) {
            Ok(n) => emit(n, true, None),
            Err(err) => {
                if let Ok(c) = db.lock() {
                    let _ = c.execute("update library_roots set scan_state = case when scan_state = 'offline' then 'offline' else 'error' end, error = ?2 where id = ?1", rusqlite::params![id, err]);
                }
                emit(0, true, Some(err))
            }
        }
    });
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

#[tauri::command]
async fn plugins_scan() -> Vec<plugins::Plugin> {
    plugins::scan()
}

// ---------- Agent Integration Layer ----------

#[tauri::command]
fn ail_status(app: State<App>) -> ail::PortHealth {
    app.ail.lock().unwrap().port.health()
}

#[tauri::command]
fn ail_send(app: State<App>, text: String) -> Vec<ail::AgentEvent> {
    app.ail.lock().unwrap().send(&app.db, &text)
}

#[tauri::command]
fn ail_tasks(app: State<App>) -> Vec<ail::Task> {
    app.ail.lock().unwrap().tasks.clone()
}

#[tauri::command]
fn agents_registry() -> Value {
    serde_json::from_str(ail::REGISTRY_JSON).unwrap_or(Value::Null)
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
            app.manage(App { db: Arc::new(Mutex::new(conn)), ail: Mutex::new(ail::Ail::new()), data_dir });
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
            ail_status,
            ail_send,
            ail_tasks,
            agents_registry,
        ])
        .run(tauri::generate_context!())
        .expect("error while running COSOUND");
}
