//! Plugin Registry: the local database of plug-ins actually detected (tables in migrations/004_plugins.sql).
//! One product per instrument/effect, with one install per format; installs that disappear are kept as
//! "missing" with the date, and come back when the plug-in is reinstalled.
use super::categories::{self, Category};
use super::knowledge;
use super::scanner::Install;
use rusqlite::{params, Connection};
use serde_json::{json, Value};

/// Plug-in formats each DAW loads, as documented in COSOUND's knowledge base (not tested here).
pub const DAW_FORMATS: [(&str, &str, &[&str], &[&str], &str); 6] = [
    ("logic", "Logic Pro", &["AU", "AUv3"], &[], "docs/knowledge/logic-pro/README.md (Feature facts)"),
    ("ableton", "Ableton Live", &["VST2", "VST3", "AU", "AUv3"], &["VST2", "VST3"], "docs/knowledge/ableton-live/README.md"),
    ("fl_studio", "FL Studio", &["VST2", "VST3", "AU", "CLAP"], &["VST2", "VST3", "CLAP"], "docs/knowledge/fl-studio/README.md"),
    ("pro_tools", "Pro Tools", &["AAX"], &["AAX"], "docs/knowledge/pro-tools/README.md"),
    ("cubase", "Cubase", &["VST3", "VST2"], &["VST3", "VST2"], "docs/knowledge/cubase/README.md (VST2 disabled by default)"),
    ("reaper", "REAPER", &["VST2", "VST3", "AU", "CLAP"], &["VST2", "VST3", "CLAP"], "docs/knowledge/reaper/README.md"),
];

/// DAWs that can load an install, by format and OS ("documented", never "verified" without a test in that DAW).
pub fn daw_compat(format: &str, os: &str) -> Vec<Value> {
    DAW_FORMATS
        .iter()
        .filter(|(_, _, mac, win, _)| if os == "windows" { win.contains(&format) } else { mac.contains(&format) })
        .map(|(id, name, _, _, src)| json!({ "daw": id, "name": name, "basis": "documented", "source": src }))
        .collect()
}

fn norm(s: &str) -> String {
    let lower = s.to_lowercase();
    let mut n = lower.as_str();
    for suffix in [" (vst3)", " (vst)", " (au)", " (clap)", " (aax)", " vst3", " x64", " (x64)", "_x64", " 64"] {
        n = n.strip_suffix(suffix).unwrap_or(n);
    }
    n.chars().filter(|c| c.is_alphanumeric()).collect()
}

/// Same product in different formats → same id (vendor + name, normalized).
pub fn product_id(i: &Install) -> String {
    let vendor = i.vendor.as_deref().map(norm).filter(|v| !v.is_empty()).unwrap_or_else(|| "unknown".into());
    format!("{vendor}/{}", norm(&i.name))
}

#[derive(serde::Serialize, Default, Debug)]
pub struct SyncSummary {
    pub installs: usize,
    pub products: usize,
    pub new: usize,
    pub missing: usize,
    pub back: usize,
    pub incompatible: usize,
    pub errors: usize,
    pub notes: Vec<String>,
}

/// Store a scan. Installs not found any more become "missing" (kept, never deleted automatically).
pub fn sync(c: &Connection, found: &[Install], notes: Vec<String>) -> rusqlite::Result<SyncSummary> {
    let tx = c.unchecked_transaction()?;
    let mut s = SyncSummary { notes, ..Default::default() };
    let before: std::collections::HashMap<String, String> =
        tx.prepare("select id, status from plugin_installs")?.query_map([], |r| Ok((r.get(0)?, r.get(1)?)))?.filter_map(Result::ok).collect();
    let mut seen = std::collections::HashSet::new();
    let mut products = std::collections::HashMap::<String, (String, Option<String>, String, Vec<String>, &'static str)>::new();
    for i in found {
        let source = if i.format.starts_with("AU") { "au_tags" } else { "vst3_categories" };
        let e = products.entry(product_id(i)).or_insert_with(|| (i.name.clone(), i.vendor.clone(), i.kind.clone(), vec![], source));
        if e.2 == "unknown" && i.kind != "unknown" {
            e.2 = i.kind.clone();
        }
        e.3.extend(i.declared_categories.iter().cloned());
    }
    for (pid, (name, vendor, kind, tags, source)) in &products {
        tx.execute(
            "insert into plugin_products(id, name, vendor, kind, last_verified) values (?1, ?2, ?3, ?4, datetime('now'))
             on conflict(id) do update set name = ?2, vendor = coalesce(?3, vendor), kind = case when ?4 = 'unknown' then kind else ?4 end, last_verified = datetime('now')",
            params![pid, name, vendor, kind],
        )?;
        let cats: Vec<Category> = categories::classify(name, kind, tags, source, &knowledge::known_categories(name));
        tx.execute("delete from plugin_categories where product_id = ?1 and source != 'user'", [pid])?;
        for c in cats {
            tx.execute("insert or ignore into plugin_categories(product_id, category, source, confidence) values (?1, ?2, ?3, ?4)", params![pid, c.category, c.source, c.confidence])?;
        }
    }
    for i in found {
        let pid = product_id(i);
        seen.insert(i.id.clone());
        match before.get(&i.id) {
            None => s.new += 1,
            Some(st) if st == "missing" => s.back += 1,
            _ => {}
        }
        s.incompatible += (i.status == "incompatible") as usize;
        s.errors += (i.status == "error") as usize;
        let mut details = i.details.clone();
        details["declared_categories"] = json!(i.declared_categories);
        details["daw_compat"] = json!(daw_compat(&i.format, &i.os));
        tx.execute(
            "insert into plugin_installs(id, product_id, format, name, vendor, version, path, scope, os, archs, au_code, bundle_id, registered, status, status_reason, details, last_verified, missing_since)
             values (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, datetime('now'), null)
             on conflict(id) do update set product_id = ?2, format = ?3, name = ?4, vendor = ?5, version = ?6, path = ?7, scope = ?8, os = ?9, archs = ?10,
               au_code = ?11, bundle_id = ?12, registered = ?13, status = ?14, status_reason = ?15, details = ?16, last_verified = datetime('now'), missing_since = null",
            params![i.id, pid, i.format, i.name, i.vendor, i.version, i.path, i.scope, i.os, json!(i.archs).to_string(), i.au_code, i.bundle_id, i.registered, i.status, i.status_reason, details.to_string()],
        )?;
    }
    for (id, st) in &before {
        if !seen.contains(id) && st != "missing" {
            tx.execute("update plugin_installs set status = 'missing', status_reason = 'not found at the last scan (uninstalled or moved)', missing_since = datetime('now') where id = ?1", [id])?;
            s.missing += 1;
        }
    }
    tx.execute("update plugin_products set available = exists(select 1 from plugin_installs i where i.product_id = plugin_products.id and i.status = 'available')", [])?;
    tx.commit()?;
    s.installs = found.len();
    s.products = products.len();
    Ok(s)
}

fn install_row(r: &rusqlite::Row) -> rusqlite::Result<Value> {
    let archs: String = r.get::<_, Option<String>>(9)?.unwrap_or_else(|| "[]".into());
    let details: String = r.get::<_, Option<String>>(15)?.unwrap_or_else(|| "{}".into());
    Ok(json!({
        "id": r.get::<_, String>(0)?, "product_id": r.get::<_, String>(1)?, "format": r.get::<_, String>(2)?, "name": r.get::<_, String>(3)?,
        "vendor": r.get::<_, Option<String>>(4)?, "version": r.get::<_, Option<String>>(5)?, "path": r.get::<_, Option<String>>(6)?,
        "scope": r.get::<_, String>(7)?, "os": r.get::<_, String>(8)?, "archs": serde_json::from_str::<Value>(&archs).unwrap_or(json!([])),
        "au_code": r.get::<_, Option<String>>(10)?, "bundle_id": r.get::<_, Option<String>>(11)?, "registered": r.get::<_, Option<bool>>(12)?,
        "status": r.get::<_, String>(13)?, "status_reason": r.get::<_, Option<String>>(14)?, "details": serde_json::from_str::<Value>(&details).unwrap_or(json!({})),
        "last_verified": r.get::<_, String>(16)?, "missing_since": r.get::<_, Option<String>>(17)?,
    }))
}

const INSTALL_COLS: &str = "id, product_id, format, name, vendor, version, path, scope, os, archs, au_code, bundle_id, registered, status, status_reason, details, last_verified, missing_since";

pub fn installs_of(c: &Connection, product_id: &str) -> Vec<Value> {
    let mut st = c.prepare(&format!("select {INSTALL_COLS} from plugin_installs where product_id = ?1 order by format")).unwrap();
    st.query_map([product_id], install_row).map(|m| m.filter_map(Result::ok).collect()).unwrap_or_default()
}

pub fn install(c: &Connection, id: &str) -> Option<Value> {
    c.query_row(&format!("select {INSTALL_COLS} from plugin_installs where id = ?1"), [id], install_row).ok()
}

pub fn categories_of(c: &Connection, product_id: &str) -> Vec<Value> {
    let mut st = c.prepare("select category, source, confidence from plugin_categories where product_id = ?1 order by confidence desc").unwrap();
    st.query_map([product_id], |r| Ok(json!({ "category": r.get::<_, String>(0)?, "source": r.get::<_, String>(1)?, "confidence": r.get::<_, f64>(2)? })))
        .map(|m| m.filter_map(Result::ok).collect())
        .unwrap_or_default()
}

/// Product list with filters. `text` matches name and vendor; category words (IT/EN) in it filter by category.
pub fn list(c: &Connection, text: &str, kind: Option<&str>, category: Option<&str>, include_missing: bool) -> Vec<Value> {
    let words: Vec<&str> = categories::from_words(text);
    let mut st = c
        .prepare(
            "select p.id, p.name, p.vendor, p.kind, p.available, p.last_verified,
                    (select group_concat(category, ',') from plugin_categories pc where pc.product_id = p.id),
                    (select group_concat(format, ',') from plugin_installs i where i.product_id = p.id and i.status != 'missing'),
                    (select count(*) from plugin_params pp join plugin_installs i on i.id = pp.install_id where i.product_id = p.id),
                    (select count(*) from plugin_presets pr where pr.product_id = p.id)
             from plugin_products p order by p.kind, lower(p.name)",
        )
        .unwrap();
    let t = text.to_lowercase();
    let free: Vec<String> = t.split_whitespace().filter(|w| categories::from_words(w).is_empty() && w.len() > 1).map(String::from).collect();
    st.query_map([], |r| {
        Ok(json!({
            "id": r.get::<_, String>(0)?, "name": r.get::<_, String>(1)?, "vendor": r.get::<_, Option<String>>(2)?, "kind": r.get::<_, String>(3)?,
            "available": r.get::<_, bool>(4)?, "last_verified": r.get::<_, Option<String>>(5)?,
            "categories": r.get::<_, Option<String>>(6)?.map(|s| s.split(',').map(String::from).collect::<Vec<_>>()).unwrap_or_default(),
            "formats": r.get::<_, Option<String>>(7)?.map(|s| s.split(',').map(String::from).collect::<Vec<_>>()).unwrap_or_default(),
            "parameters": r.get::<_, i64>(8)?, "presets": r.get::<_, i64>(9)?,
        }))
    })
    .map(|m| m.filter_map(Result::ok).collect::<Vec<Value>>())
    .unwrap_or_default()
    .into_iter()
    .filter(|p| include_missing || p["available"].as_bool() == Some(true) || !p["formats"].as_array().unwrap().is_empty())
    .filter(|p| kind.is_none_or(|k| p["kind"] == k))
    .filter(|p| category.is_none_or(|cat| p["categories"].as_array().unwrap().iter().any(|x| x == cat)))
    .filter(|p| words.is_empty() || words.iter().any(|w| p["categories"].as_array().unwrap().iter().any(|x| x == w)))
    .filter(|p| {
        let hay = format!("{} {}", p["name"].as_str().unwrap_or(""), p["vendor"].as_str().unwrap_or("")).to_lowercase();
        free.iter().all(|w| hay.contains(w.as_str()))
    })
    .collect()
}

pub fn product(c: &Connection, id: &str) -> Option<Value> {
    c.query_row("select id, name, vendor, kind, available, first_seen, last_verified from plugin_products where id = ?1", [id], |r| {
        Ok(json!({ "id": r.get::<_, String>(0)?, "name": r.get::<_, String>(1)?, "vendor": r.get::<_, Option<String>>(2)?, "kind": r.get::<_, String>(3)?,
                   "available": r.get::<_, bool>(4)?, "first_seen": r.get::<_, String>(5)?, "last_verified": r.get::<_, Option<String>>(6)? }))
    })
    .ok()
}

pub fn log_error(c: &Connection, install_id: Option<&str>, op: &str, err: &str) {
    let _ = c.execute("insert into plugin_errors(install_id, operation, error) values (?1, ?2, ?3)", params![install_id, op, err]);
}

pub fn errors(c: &Connection, install_id: Option<&str>, limit: i64) -> Vec<Value> {
    let mut st = c.prepare("select install_id, operation, error, at from plugin_errors where (?1 is null or install_id = ?1) order by id desc limit ?2").unwrap();
    st.query_map(params![install_id, limit], |r| Ok(json!({ "install_id": r.get::<_, Option<String>>(0)?, "operation": r.get::<_, String>(1)?, "error": r.get::<_, String>(2)?, "at": r.get::<_, String>(3)? })))
        .map(|m| m.filter_map(Result::ok).collect())
        .unwrap_or_default()
}
