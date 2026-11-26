//! Index services: user corrections, exclusions, file details and statistics.
use rusqlite::{params, Connection, OptionalExtension};
use serde_json::{json, Value};

/// Re-apply a file's corrections: user BPM/key win, user labels are added, removed labels stay removed.
pub fn apply_corrections(c: &Connection, id: i64) -> rusqlite::Result<()> {
    let mut st = c.prepare("select field, value from file_corrections where file_id = ?1")?;
    let rows: Vec<(String, Option<String>)> = st.query_map([id], |r| Ok((r.get(0)?, r.get(1)?)))?.collect::<Result<_, _>>()?;
    for (field, value) in rows {
        match field.as_str() {
            "bpm" => {
                c.execute("update files set bpm = ?2, bpm_src = 'user', bpm_conf = 1.0 where id = ?1", params![id, value.and_then(|v| v.parse::<f64>().ok())])?;
            }
            "key" => {
                c.execute("update files set musical_key = ?2, key_src = 'user', key_conf = 1.0 where id = ?1", params![id, value])?;
            }
            f if f.starts_with("add:") => {
                let mut p = f.splitn(3, ':').skip(1);
                let (kind, tag) = (p.next().unwrap_or("category"), p.next().unwrap_or(""));
                c.execute("insert or replace into file_tags(file_id, tag, kind, source, confidence) values (?1, ?2, ?3, 'user', 1.0)", params![id, tag, kind])?;
            }
            f if f.starts_with("remove:") => {
                let mut p = f.splitn(3, ':').skip(1);
                let (kind, tag) = (p.next().unwrap_or("category"), p.next().unwrap_or(""));
                c.execute("delete from file_tags where file_id = ?1 and tag = ?2 and kind = ?3", params![id, tag, kind])?;
            }
            _ => {}
        }
    }
    // primary category: a user category first, else the most confident one
    let cat: Option<String> = c
        .query_row("select tag from file_tags where file_id = ?1 and kind = 'category' order by (source = 'user') desc, confidence desc limit 1", [id], |r| r.get(0))
        .optional()?;
    c.execute("update files set category = ?2 where id = ?1", params![id, cat])?;
    Ok(())
}

/// A user correction. `field`: bpm | key | add | remove. For add/remove, `value` is "<kind>:<tag>"
/// (e.g. "category:kick"). An empty value for bpm/key removes the correction.
pub fn correct(c: &Connection, id: i64, field: &str, value: &str) -> Result<(), String> {
    let exists: bool = c.query_row("select count(*) from files where id = ?1", [id], |r| r.get::<_, i64>(0)).map_err(|e| e.to_string())? == 1;
    if !exists {
        return Err("file not found".into());
    }
    match field {
        "bpm" if value.is_empty() => c.execute("delete from file_corrections where file_id = ?1 and field = 'bpm'", [id]).map(|_| ()),
        "bpm" => {
            let b: f64 = value.parse().map_err(|_| "BPM must be a number")?;
            if !(20.0..=400.0).contains(&b) {
                return Err("BPM out of range".into());
            }
            c.execute("insert or replace into file_corrections(file_id, field, value) values (?1, 'bpm', ?2)", params![id, b.to_string()]).map(|_| ())
        }
        "key" if value.is_empty() => c.execute("delete from file_corrections where file_id = ?1 and field = 'key'", [id]).map(|_| ()),
        "key" => {
            let k = crate::music::theory::Key::parse(value).map_err(|e| format!("key: {e}"))?;
            c.execute("insert or replace into file_corrections(file_id, field, value) values (?1, 'key', ?2)", params![id, k.name()]).map(|_| ())
        }
        "add" | "remove" => {
            let (kind, tag) = value.split_once(':').ok_or("value must be <kind>:<tag>")?;
            if !["category", "type", "genre", "descriptor", "plugin"].contains(&kind) || tag.trim().is_empty() || tag.len() > 40 {
                return Err("invalid label".into());
            }
            let (other, this) = if field == "add" { ("remove", "add") } else { ("add", "remove") };
            c.execute("delete from file_corrections where file_id = ?1 and field = ?2", params![id, format!("{other}:{kind}:{tag}")]).map_err(|e| e.to_string())?;
            if field == "remove" {
                c.execute("delete from file_tags where file_id = ?1 and tag = ?2 and kind = ?3 and source = 'user'", params![id, tag, kind]).map_err(|e| e.to_string())?;
            }
            c.execute("insert or replace into file_corrections(file_id, field, value) values (?1, ?2, null)", params![id, format!("{this}:{kind}:{}", tag.trim().to_lowercase())]).map(|_| ())
        }
        _ => return Err(format!("unknown field {field}")),
    }
    .map_err(|e| e.to_string())?;
    apply_corrections(c, id).map_err(|e| e.to_string())?;
    crate::db::log_op(c, None, "library.correct", "low", &json!({ "file_id": id, "field": field, "value": value }), None).map_err(|e| e.to_string())?;
    Ok(())
}

pub fn set_exclusion(c: &Connection, root_id: &str, rel_path: &str, excluded: bool) -> Result<(), String> {
    let norm = rel_path.replace('\\', "/");
    let rel = norm.trim_matches('/');
    if rel.is_empty() || rel.contains("..") {
        return Err("give a folder inside the authorized folder".into());
    }
    if excluded {
        c.execute("insert or ignore into library_exclusions(root_id, rel_path) values (?1, ?2)", params![root_id, rel]).map_err(|e| e.to_string())?;
    } else {
        c.execute("delete from library_exclusions where root_id = ?1 and rel_path = ?2", params![root_id, rel]).map_err(|e| e.to_string())?;
    }
    crate::db::log_op(c, None, if excluded { "library.exclude" } else { "library.include" }, "low", &json!({ "root_id": root_id, "folder": rel }), None).map_err(|e| e.to_string())?;
    Ok(())
}

pub fn exclusions(c: &Connection) -> Vec<Value> {
    let mut st = c.prepare("select root_id, rel_path from library_exclusions order by root_id, rel_path").unwrap();
    st.query_map([], |r| Ok(json!({ "root_id": r.get::<_, String>(0)?, "folder": r.get::<_, String>(1)? }))).map(|m| m.filter_map(Result::ok).collect()).unwrap_or_default()
}

pub fn set_volume_excluded(c: &Connection, volume_id: &str, excluded: bool) -> Result<(), String> {
    let n = c.execute("update volumes set excluded = ?2 where id = ?1", params![volume_id, excluded]).map_err(|e| e.to_string())?;
    if n == 0 {
        return Err("device not found".into());
    }
    crate::db::log_op(c, None, if excluded { "library.exclude_device" } else { "library.include_device" }, "low", &json!({ "volume_id": volume_id }), None).map_err(|e| e.to_string())?;
    Ok(())
}

/// Counts by status, kind, category and analysis state.
pub fn stats(c: &Connection) -> Value {
    let group = |sql: &str| -> Value {
        let mut st = c.prepare(sql).unwrap();
        let m: serde_json::Map<String, Value> = st.query_map([], |r| Ok((r.get::<_, Option<String>>(0)?.unwrap_or_else(|| "unknown".into()), json!(r.get::<_, i64>(1)?)))).map(|m| m.filter_map(Result::ok).collect()).unwrap_or_default();
        Value::Object(m)
    };
    json!({
        "files": c.query_row("select count(*) from files where status != 'missing'", [], |r| r.get::<_, i64>(0)).unwrap_or(0),
        "by_status": group("select status, count(*) from files group by status"),
        "by_kind": group("select kind, count(*) from files where status != 'missing' group by kind"),
        "by_category": group("select category, count(*) from files where status != 'missing' group by category"),
        "analysis": group("select analysis_state, count(*) from files where status != 'missing' group by analysis_state"),
        "offline_files": c.query_row("select count(*) from files f join library_roots r on r.id = f.root_id join volumes v on v.id = r.volume_id where v.online = 0 and f.status != 'missing'", [], |r| r.get::<_, i64>(0)).unwrap_or(0),
        "roots": c.query_row("select count(*) from library_roots", [], |r| r.get::<_, i64>(0)).unwrap_or(0),
    })
}
