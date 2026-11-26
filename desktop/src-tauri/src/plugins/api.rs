//! Entry points of the Plugin Intelligence System, used by the app commands and by the agents.
use super::{design, host, knowledge::Knowledge, params, presets, registry, scanner};
use crate::smartlib;
use rusqlite::Connection;
use serde_json::{json, Value};
use std::path::Path;

/// Scan plug-ins and the standard preset folders and update the registry.
pub fn refresh(c: &Connection) -> Result<Value, String> {
    let (found, notes) = scanner::scan();
    let s = registry::sync(c, &found, notes).map_err(|e| e.to_string())?;
    let p = presets::scan(c, &presets::folders());
    Ok(json!({ "plugins": s, "preset_folders": p, "host_available": host::available() }))
}

pub fn overview(c: &Connection) -> Value {
    let q = |sql: &str| -> Vec<(String, i64)> { c.prepare(sql).unwrap().query_map([], |r| Ok((r.get(0)?, r.get(1)?))).map(|m| m.filter_map(Result::ok).collect()).unwrap_or_default() };
    let to = |v: Vec<(String, i64)>| Value::Object(v.into_iter().map(|(k, n)| (k, json!(n))).collect());
    json!({
        "products": c.query_row("select count(*) from plugin_products", [], |r| r.get::<_, i64>(0)).unwrap_or(0),
        "by_kind": to(q("select kind, count(*) from plugin_products where available = 1 group by kind")),
        "by_format": to(q("select format, count(*) from plugin_installs where status != 'missing' group by format")),
        "by_status": to(q("select status, count(*) from plugin_installs group by status")),
        "by_category": to(q("select category, count(*) from plugin_categories pc join plugin_products p on p.id = pc.product_id where p.available = 1 group by category")),
        "presets": to(q("select source, count(*) from plugin_presets group by source")),
        "last_scan": c.query_row("select max(last_verified) from plugin_installs", [], |r| r.get::<_, Option<String>>(0)).unwrap_or(None),
        "host_available": host::available(),
        "host_formats": if host::available() { json!(["AU", "AUv3"]) } else { json!([]) },
        "errors": registry::errors(c, None, 20),
        "os": scanner::os(), "arch": scanner::host_arch(),
        "taxonomy": { "instrument": super::categories::INSTRUMENT_CATEGORIES, "effect": super::categories::EFFECT_CATEGORIES },
    })
}

/// Interaction levels of one install (Prompt 08 §8), from what was actually done on this computer.
pub fn levels(c: &Connection, install: &Value) -> Value {
    let id = install["id"].as_str().unwrap_or("");
    let ps = params::stored(c, id);
    let hostable = install["au_code"].is_string() && install["status"] == "available" && host::available();
    let pid = install["product_id"].as_str().unwrap_or("");
    let n_presets: i64 = c.query_row("select count(*) from plugin_presets where product_id = ?1", [pid], |r| r.get(0)).unwrap_or(0);
    let readable: i64 = c.query_row("select count(*) from plugin_presets where product_id = ?1 and readable = 1", [pid], |r| r.get(0)).unwrap_or(0);
    json!({
        "recognized": true,
        "analyzed": !ps.is_empty(),
        "parameters_available": ps.len(),
        "parameters_controllable": if hostable { ps.iter().filter(|p| p["writable"] == true).count() } else { 0 },
        "parameters_with_sound_role": ps.iter().filter(|p| !p["role"].is_null()).count(),
        "presets_found": n_presets,
        "presets_readable": readable,
        // codes, translated by the UI: new_only = new presets from the plug-in's own state, originals never overwritten
        "presets_modifiable": if hostable { "new_only" } else { "no" },
        "control_verified": install["details"]["control_verified_at"].as_str().map(|d| json!({ "where": "cosound_host", "at": d })).unwrap_or(json!(null)),
        "control_in_daw": "needs_connector",
        "hosting": if hostable { "hostable" } else if install["au_code"].is_string() { "au_not_loadable" } else { "not_hosted" },
    })
}

/// Everything about one product.
pub fn detail(c: &Connection, product_id: &str, k: &Knowledge) -> Result<Value, String> {
    let p = registry::product(c, product_id).ok_or("unknown plug-in")?;
    let cats: Vec<String> = registry::categories_of(c, product_id).iter().filter_map(|x| x["category"].as_str().map(String::from)).collect();
    let installs: Vec<Value> = registry::installs_of(c, product_id)
        .into_iter()
        .map(|mut i| {
            i["levels"] = levels(c, &i);
            i["parameters"] = json!(params::stored(c, i["id"].as_str().unwrap_or("")));
            i
        })
        .collect();
    let au = installs.iter().find_map(|i| i["au_code"].as_str().map(String::from));
    let name = p["name"].as_str().unwrap_or("");
    let mut lib_q = smartlib::query::Query { kind: Some("preset".into()), ..Default::default() };
    lib_q.plugin = crate::sound::known_synth(name).map(|s| s.id.to_string());
    let library_presets = if lib_q.plugin.is_some() { smartlib::api::search_structured(c, &lib_q, 50)["results"].clone() } else { json!([]) };
    Ok(json!({
        "product": p, "categories": registry::categories_of(c, product_id), "installs": installs,
        "knowledge": k.describe(name, p["vendor"].as_str(), au.as_deref(), &cats),
        "presets": presets::of_product(c, product_id),
        "library_presets": library_presets,
        "errors": installs.iter().flat_map(|i| registry::errors(c, i["id"].as_str(), 10)).collect::<Vec<_>>(),
    }))
}

/// Side-by-side comparison of similar plug-ins.
pub fn compare(c: &Connection, ids: &[String]) -> Value {
    let rows: Vec<Value> = ids
        .iter()
        .filter_map(|id| {
            let p = registry::product(c, id)?;
            let installs = registry::installs_of(c, id);
            let roles: std::collections::BTreeSet<String> = installs.iter().flat_map(|i| params::stored(c, i["id"].as_str().unwrap_or(""))).filter_map(|p| p["role"].as_str().map(String::from)).collect();
            Some(json!({ "product": p, "categories": registry::categories_of(c, id), "formats": installs.iter().map(|i| i["format"].clone()).collect::<Vec<_>>(),
                         "versions": installs.iter().map(|i| i["version"].clone()).collect::<Vec<_>>(), "sound_roles": roles,
                         "presets": presets::of_product(c, id).len(), "levels": installs.first().map(|i| levels(c, i)) }))
        })
        .collect();
    json!({ "compared": rows })
}

/// Mark that a parameter change was applied and rendered for these installs (control verified in COSOUND's host).
pub fn mark_control_verified(c: &Connection, spec: &Value) {
    let mut ids: Vec<&str> = spec["chain"].as_array().into_iter().flatten().filter_map(|u| u["install_id"].as_str()).collect();
    if let Some(i) = spec["instrument"]["install_id"].as_str() {
        ids.push(i);
    }
    for id in ids {
        let _ = c.execute("update plugin_installs set details = json_set(coalesce(details, '{}'), '$.control_verified_at', datetime('now')) where id = ?1", [id]);
    }
}

/// Audition a design and record the verified control.
pub fn audition(c: &Connection, data_dir: &Path, id: &str) -> Result<Value, String> {
    let r = design::audition(c, data_dir, id)?;
    if let Some(d) = design::get(c, id) {
        if d["changes"].as_array().is_some_and(|a| !a.is_empty()) || d["spec"]["chain"].as_array().is_some_and(|a| !a.is_empty()) {
            mark_control_verified(c, &d["spec"]);
        }
    }
    Ok(r)
}
