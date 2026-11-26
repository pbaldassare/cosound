//! Smart Library API: JSON results for the Smart Library Agent, the app UI, the DAW Controller and
//! any other module. Read functions never touch the user's files; `scan` only reads authorized folders.
use super::{dupes, index, query, scan, search};
use crate::db::Db;
use crate::library;
use rusqlite::Connection;
use serde_json::{json, Value};
use std::sync::atomic::AtomicBool;

fn devices(c: &Connection) -> Vec<String> {
    let mut st = c.prepare("select label from volumes").unwrap();
    st.query_map([], |r| r.get(0)).map(|m| m.filter_map(Result::ok).collect()).unwrap_or_default()
}

/// Natural-language (or keyword) search. Returns how the request was read, with the results.
pub fn search_text(c: &Connection, text: &str, limit: usize, kind: Option<&str>) -> Value {
    let mut q = query::parse(text, &devices(c));
    if let Some(k) = kind {
        q.kind = Some(k.into());
    }
    let results = search::search(c, &q, &search::Options { limit, include_offline: true });
    let offline = results.iter().filter(|r| r["path"].is_null()).count();
    json!({ "query": text, "understood": q, "count": results.len(), "offline_results": offline, "results": results })
}

pub fn search_structured(c: &Connection, q: &query::Query, limit: usize) -> Value {
    let results = search::search(c, q, &search::Options { limit, include_offline: true });
    json!({ "understood": q, "count": results.len(), "results": results })
}

/// Original paths for the DAW Controller (null while the drive is not connected).
pub fn resolve(c: &Connection, ids: &[i64]) -> Vec<Value> {
    ids.iter()
        .map(|id| match search::detail(c, *id) {
            Some(d) => json!({ "id": id, "name": d["name"], "kind": d["kind"], "format": d["format"], "path": d["path"], "status": d["status"], "device": d["device"], "device_online": d["device_online"],
                "usable": !d["path"].is_null() && d["status"] != "missing" }),
            None => json!({ "id": id, "error": "file not in the library" }),
        })
        .collect()
}

pub fn detail(c: &Connection, id: i64) -> Result<Value, String> {
    search::detail(c, id).ok_or_else(|| "file not in the library".into())
}

pub fn similar(c: &Connection, id: i64, limit: usize) -> Result<Value, String> {
    search::similar(c, id, limit)
}

pub fn loops(c: &Connection, bpm: f64, key: Option<&str>, category: Option<&str>, limit: usize) -> Value {
    search::loops_for(c, bpm, key, category, limit)
}

pub fn duplicates(db: &Db, cancel: &AtomicBool, include_similar: bool) -> Value {
    dupes::find(db, cancel, include_similar, 0.97)
}

pub fn overview(c: &Connection) -> Value {
    let disks = library::current_disks();
    let _ = library::refresh_volumes(c, &disks);
    let mut st = c.prepare("select r.id, r.scan_state, r.error, r.last_scan_at, (select count(*) from files f where f.root_id = r.id and f.status = 'missing'), (select count(*) from files f where f.root_id = r.id and f.analysis_state = 'pending' and f.status != 'missing') from library_roots r").unwrap();
    let states: Vec<Value> = st
        .query_map([], |r| Ok(json!({ "root_id": r.get::<_, String>(0)?, "scan_state": r.get::<_, String>(1)?, "error": r.get::<_, Option<String>>(2)?, "last_scan_at": r.get::<_, Option<String>>(3)?, "missing_files": r.get::<_, i64>(4)?, "pending_analysis": r.get::<_, i64>(5)? })))
        .map(|m| m.filter_map(Result::ok).collect())
        .unwrap_or_default();
    json!({
        "stats": index::stats(c),
        "volumes": library::volumes(c, &disks).unwrap_or_default(),
        "roots": library::roots(c).unwrap_or_default(),
        "root_states": states,
        "exclusions": index::exclusions(c),
    })
}

/// Incremental scan of one root, or of every root on a connected drive.
pub fn scan(db: &Db, root_id: Option<&str>, cancel: &AtomicBool) -> Vec<scan::ScanSummary> {
    let roots: Vec<String> = match root_id {
        Some(r) => vec![r.to_string()],
        None => library::roots(&db.lock().unwrap()).unwrap_or_default().into_iter().map(|r| r.id).collect(),
    };
    roots.iter().map(|r| scan::scan_root(db, r, cancel, &|_| {})).collect()
}

/// What the library can read, by kind (see docs/smart-library/FORMATS.md).
pub fn formats() -> Value {
    json!({
        "audio_decoded": super::decode::AUDIO_EXTS,
        "midi": ["mid", "midi"],
        "presets_read": super::presets::READABLE,
        "presets_recognized_only": ["nksf", "nmsv", "serumpreset", "h2p", "adg", "adv", "pst", "patch"],
        "instruments_recognized_only": ["nki", "nkm", "nkc", "exs", "sfz", "sf2"],
        "projects_recognized_only": ["logicx", "als", "flp", "ptx", "cpr", "rpp", "dawproject"],
    })
}

pub fn analysis_status(c: &Connection) -> Value {
    let n = |sql: &str| c.query_row(sql, [], |r| r.get::<_, i64>(0)).unwrap_or(0);
    json!({
        "pending": n("select count(*) from files f join library_roots r on r.id = f.root_id join volumes v on v.id = r.volume_id where f.analysis_state = 'pending' and f.status != 'missing' and v.online = 1"),
        "pending_offline": n("select count(*) from files f join library_roots r on r.id = f.root_id join volumes v on v.id = r.volume_id where f.analysis_state = 'pending' and f.status != 'missing' and v.online = 0"),
        "done": n("select count(*) from files where analysis_state = 'done' and status != 'missing'"),
        "errors": n("select count(*) from files where analysis_state = 'error' and status != 'missing'"),
    })
}
