//! Local SQLite database (single writer: the Rust core).
use rusqlite::{params, Connection, OptionalExtension};
use serde::Serialize;
use serde_json::Value;
use std::collections::HashMap;
use std::path::Path;

pub type Db = std::sync::Mutex<Connection>;

const MIGRATIONS: &[&str] = &[include_str!("../migrations/001_init.sql")];

pub fn open(path: &Path) -> rusqlite::Result<Connection> {
    let c = Connection::open(path)?;
    init(&c)?;
    Ok(c)
}

#[cfg(test)]
pub fn open_memory() -> Connection {
    let c = Connection::open_in_memory().unwrap();
    init(&c).unwrap();
    c
}

fn init(c: &Connection) -> rusqlite::Result<()> {
    c.pragma_update(None, "journal_mode", "WAL")?;
    c.pragma_update(None, "foreign_keys", "ON")?;
    let v: i64 = c.pragma_query_value(None, "user_version", |r| r.get(0))?;
    for (i, sql) in MIGRATIONS.iter().enumerate().skip(v as usize) {
        c.execute_batch(&format!("begin; {sql}; pragma user_version = {}; commit;", i + 1))?;
    }
    Ok(())
}

pub fn new_id() -> String {
    uuid::Uuid::new_v4().to_string()
}

// ---------- settings ----------

pub fn settings_all(c: &Connection) -> rusqlite::Result<HashMap<String, String>> {
    let mut st = c.prepare("select key, value from settings")?;
    let rows = st.query_map([], |r| Ok((r.get(0)?, r.get(1)?)))?;
    rows.collect()
}

pub fn setting(c: &Connection, key: &str) -> rusqlite::Result<Option<String>> {
    c.query_row("select value from settings where key = ?1", [key], |r| r.get(0)).optional()
}

pub fn set_setting(c: &Connection, key: &str, value: &str) -> rusqlite::Result<()> {
    let old = setting(c, key)?;
    c.execute(
        "insert into settings(key, value) values (?1, ?2) on conflict(key) do update set value = excluded.value",
        params![key, value],
    )?;
    // Settings that matter for privacy/permissions or identity go to the operation register.
    if LOGGED_SETTINGS.contains(&key) && old.as_deref() != Some(value) {
        log_op(c, None, "settings.change", "low", &serde_json::json!({"key": key, "from": old, "to": value}), None)?;
    }
    Ok(())
}

const LOGGED_SETTINGS: &[&str] = &[
    "assistant_name",
    "perm_microphone",
    "perm_external_devices",
    "perm_daw_detection",
    "perm_plugin_scan",
    "cloud_sync_assistant_name",
];

// ---------- operation register ----------

#[derive(Serialize)]
pub struct OpEntry {
    pub id: i64,
    pub ts: String,
    pub project_id: Option<String>,
    pub op: String,
    pub risk: String,
    pub params: Value,
    pub undoable: bool,
    pub undone_at: Option<String>,
}

pub fn log_op(c: &Connection, project_id: Option<&str>, op: &str, risk: &str, params: &Value, undo: Option<&Value>) -> rusqlite::Result<i64> {
    c.execute(
        "insert into op_log(project_id, op, risk, params_json, undo_json) values (?1, ?2, ?3, ?4, ?5)",
        params![project_id, op, risk, params.to_string(), undo.map(|u| u.to_string())],
    )?;
    Ok(c.last_insert_rowid())
}

pub fn ops(c: &Connection, project_id: Option<&str>, limit: i64) -> rusqlite::Result<Vec<OpEntry>> {
    let mut st = c.prepare(
        "select id, ts, project_id, op, risk, params_json, undo_json is not null, undone_at from op_log
         where ?1 is null or project_id = ?1 order by id desc limit ?2",
    )?;
    let rows = st.query_map(params![project_id, limit], |r| {
        Ok(OpEntry {
            id: r.get(0)?,
            ts: r.get(1)?,
            project_id: r.get(2)?,
            op: r.get(3)?,
            risk: r.get(4)?,
            params: serde_json::from_str(&r.get::<_, String>(5)?).unwrap_or(Value::Null),
            undoable: r.get(6)?,
            undone_at: r.get(7)?,
        })
    })?;
    rows.collect()
}

/// Undo an operation using the inverse stored at logging time. Only ops that stored `undo_json` can be undone.
pub fn undo_op(c: &Connection, id: i64) -> Result<(), String> {
    let (undo, undone, project_id): (Option<String>, Option<String>, Option<String>) = c
        .query_row("select undo_json, undone_at, project_id from op_log where id = ?1", [id], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)))
        .map_err(|e| e.to_string())?;
    if undone.is_some() {
        return Err("already undone".into());
    }
    let undo: Value = serde_json::from_str(&undo.ok_or("operation cannot be undone")?).map_err(|e| e.to_string())?;
    let pid = project_id.as_deref().ok_or("operation has no project")?;
    match undo["kind"].as_str() {
        Some("project.set_name") => {
            c.execute("update projects set name = ?2, updated_at = datetime('now') where id = ?1", params![pid, undo["name"].as_str()])
        }
        Some("project.set_archived") => c.execute(
            "update projects set archived_at = case when ?2 then datetime('now') else null end where id = ?1",
            params![pid, undo["archived"].as_bool().unwrap_or(false)],
        ),
        other => return Err(format!("unknown undo kind {other:?}")),
    }
    .map_err(|e| e.to_string())?;
    c.execute("update op_log set undone_at = datetime('now') where id = ?1", [id]).map_err(|e| e.to_string())?;
    log_op(c, Some(pid), "op.undo", "low", &serde_json::json!({"undone_op": id}), None).map_err(|e| e.to_string())?;
    Ok(())
}
