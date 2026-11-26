//! Preset Analyzer: presets that really exist for detected plug-ins.
//! - factory presets reported by the plug-in itself (parameter discovery);
//! - preset files in the standard preset folders (VST3 SDK and Apple conventions);
//! - presets in the user's authorized library folders, read through the Smart Library API (never copied here).
//! Readable formats are parsed by `smartlib::presets`; other formats are "found, contents not read".
//! Timbre: an Audio Unit preset can be rendered in the isolated host and measured with the Smart Library analyzer.
use super::{host, registry};
use crate::smartlib;
use rusqlite::{params, Connection};
use serde_json::{json, Value};
use std::path::{Path, PathBuf};
use std::sync::atomic::AtomicBool;

pub const PRESET_CATEGORIES: [(&str, &[&str]); 12] = [
    ("sub_bass", &["sub", "subbass", "sub bass", "808"]),
    ("bass", &["bass", "basso", "bs", "reese"]),
    ("lead", &["lead", "ld", "solo"]),
    ("pad", &["pad", "pads", "pd"]),
    ("pluck", &["pluck", "plk"]),
    ("arpeggio", &["arp", "arpeggio", "seq", "sequence"]),
    ("keys", &["keys", "key", "piano", "ep", "organ", "rhodes", "bell", "bells"]),
    ("strings", &["strings", "string", "str", "archi"]),
    ("atmosphere", &["atmo", "atmos", "atmosphere", "ambient", "texture", "drone", "soundscape", "atmosfera"]),
    ("fx", &["fx", "sfx", "riser", "impact", "sweep", "noise"]),
    ("percussion", &["perc", "drum", "drums", "kick", "snare", "hat", "percussion"]),
    ("chords", &["chord", "chords", "stab", "stabs"]),
];

const FORMATS: [&str; 10] = ["aupreset", "vstpreset", "fxp", "fxb", "vital", "nksf", "nmsv", "serumpreset", "h2p", "adv"];

/// Preset category from its name and folders (first match in the category list).
pub fn category_of(name: &str, folders: &[String]) -> Option<&'static str> {
    let mut words = smartlib::classify::words(name);
    for f in folders {
        words.extend(smartlib::classify::words(f));
    }
    let text = words.join(" ");
    PRESET_CATEGORIES.iter().find(|(_, ks)| ks.iter().any(|k| if k.contains(' ') { text.contains(k) } else { words.iter().any(|w| w == k) })).map(|(c, _)| *c)
}

/// Standard preset folders (documented: VST3 SDK preset locations, Apple's Audio/Presets).
pub fn folders() -> Vec<PathBuf> {
    let env = |k: &str| std::env::var(k).map(PathBuf::from).ok();
    let mut v = vec![];
    if cfg!(target_os = "macos") {
        if let Some(h) = env("HOME") {
            v.push(h.join("Library/Audio/Presets"));
        }
        v.push(PathBuf::from("/Library/Audio/Presets"));
    } else if cfg!(windows) {
        if let Some(h) = env("USERPROFILE") {
            v.push(h.join("Documents/VST3 Presets"));
        }
        if let Some(a) = env("APPDATA") {
            v.push(a.join("VST3 Presets"));
        }
        if let Some(p) = env("PROGRAMDATA") {
            v.push(p.join("VST3 Presets"));
        }
    } else if let Some(h) = env("HOME") {
        v.push(h.join(".vst3/presets"));
    }
    v
}

fn fourcc(n: i64) -> String {
    (n as u32).to_be_bytes().iter().map(|b| *b as char).collect()
}

/// Product of a preset file: AU codes inside an .aupreset, else the <Vendor>/<Plugin>/ folder names.
fn product_for(c: &Connection, path: &Path, root: &Path) -> Option<String> {
    if path.extension().is_some_and(|e| e.eq_ignore_ascii_case("aupreset")) {
        if let Some(d) = plist::Value::from_file(path).ok().and_then(|v| v.into_dictionary()) {
            let code = |k: &str| d.get(k).and_then(|x| x.as_signed_integer()).map(fourcc);
            if let (Some(t), Some(s), Some(m)) = (code("type"), code("subtype"), code("manufacturer")) {
                if let Ok(p) = c.query_row("select product_id from plugin_installs where au_code = ?1", [format!("{t}:{s}:{m}")], |r| r.get::<_, String>(0)) {
                    return Some(p);
                }
            }
        }
    }
    let rel = path.strip_prefix(root).ok()?;
    let parts: Vec<String> = rel.components().map(|c| c.as_os_str().to_string_lossy().to_lowercase().chars().filter(|c| c.is_alphanumeric()).collect()).collect();
    if parts.len() < 3 {
        return None;
    }
    let (vendor, plugin) = (&parts[0], &parts[1]);
    c.query_row(
        "select id from plugin_products where id = ?1 or (id like ?2 and replace(lower(name), ' ', '') like ?3) limit 1",
        params![format!("{vendor}/{plugin}"), format!("%/{plugin}"), format!("%{plugin}%")],
        |r| r.get(0),
    )
    .ok()
}

/// Scan the standard preset folders. Presets of plug-ins that are not detected are counted, not stored.
pub fn scan(c: &Connection, dirs: &[PathBuf]) -> Value {
    let (mut stored, mut orphan, mut unreadable) = (0, 0, 0);
    let tx = c.unchecked_transaction().unwrap();
    tx.execute("delete from plugin_presets where source = 'preset_folder'", []).unwrap();
    for root in dirs {
        for e in walkdir::WalkDir::new(root).max_depth(6).into_iter().filter_map(Result::ok).filter(|e| e.file_type().is_file()) {
            let ext = e.path().extension().map(|x| x.to_string_lossy().to_lowercase()).unwrap_or_default();
            if !FORMATS.contains(&ext.as_str()) {
                continue;
            }
            let Some(pid) = product_for(&tx, e.path(), root) else {
                orphan += 1;
                continue;
            };
            let info = smartlib::presets::read(e.path());
            let readable = info["contents_read"].as_bool().unwrap_or(info["error"].is_null() && smartlib::presets::READABLE.contains(&ext.as_str()) || ext == "aupreset");
            unreadable += (!readable) as usize;
            let folders: Vec<String> = e.path().strip_prefix(root).ok().map(|r| r.parent().into_iter().flat_map(|p| p.components()).map(|c| c.as_os_str().to_string_lossy().to_string()).collect()).unwrap_or_default();
            let name = info["name"].as_str().map(String::from).unwrap_or_else(|| e.path().file_stem().unwrap_or_default().to_string_lossy().to_string());
            let cat = info["category"].as_str().and_then(|x| category_of(x, &[])).or_else(|| category_of(&name, &folders));
            tx.execute(
                "insert or ignore into plugin_presets(product_id, source, name, path, format, category, readable, details) values (?1, 'preset_folder', ?2, ?3, ?4, ?5, ?6, ?7)",
                params![pid, name, e.path().to_string_lossy(), ext, cat, readable, info.to_string()],
            )
            .unwrap();
            stored += 1;
        }
    }
    tx.commit().unwrap();
    json!({ "stored": stored, "for_plugins_not_detected": orphan, "contents_not_read": unreadable, "folders": dirs })
}

fn row(r: &rusqlite::Row) -> rusqlite::Result<Value> {
    let details: Option<String> = r.get(9)?;
    Ok(json!({
        "id": r.get::<_, i64>(0)?, "product_id": r.get::<_, String>(1)?, "install_id": r.get::<_, Option<String>>(2)?, "source": r.get::<_, String>(3)?,
        "name": r.get::<_, String>(4)?, "number": r.get::<_, Option<i64>>(5)?, "path": r.get::<_, Option<String>>(6)?, "format": r.get::<_, Option<String>>(7)?,
        "category": r.get::<_, Option<String>>(8)?, "details": details.and_then(|d| serde_json::from_str::<Value>(&d).ok()).unwrap_or(json!({})),
        "readable": r.get::<_, bool>(10)?, "plugin": r.get::<_, String>(11)?,
    }))
}

const COLS: &str = "pr.id, pr.product_id, pr.install_id, pr.source, pr.name, pr.number, pr.path, pr.format, pr.category, pr.details, pr.readable, p.name";

pub fn of_product(c: &Connection, product_id: &str) -> Vec<Value> {
    let mut st = c.prepare(&format!("select {COLS} from plugin_presets pr join plugin_products p on p.id = pr.product_id where pr.product_id = ?1 order by pr.source, pr.name")).unwrap();
    st.query_map([product_id], row).map(|m| m.filter_map(Result::ok).collect()).unwrap_or_default()
}

pub fn get(c: &Connection, id: i64) -> Option<Value> {
    c.query_row(&format!("select {COLS} from plugin_presets pr join plugin_products p on p.id = pr.product_id where pr.id = ?1"), [id], row).ok()
}

/// Semantic preset search over presets that really exist: category, sound character (from the name, or from the
/// measured timbre when the preset was analyzed), genre words, plug-in. Library presets come from the Smart Library.
pub fn search(c: &Connection, text: &str, product: Option<&str>, limit: usize) -> Value {
    let q = smartlib::query::parse(text, &[]);
    let lower = text.to_lowercase();
    let wanted_cat: Vec<&str> = PRESET_CATEGORIES.iter().filter(|(cat, ks)| ks.iter().any(|k| smartlib::classify::words(&lower).iter().any(|w| w == k)) || q.categories.iter().any(|c| c == cat)).map(|(c, _)| *c).collect();
    let mut st = c.prepare(&format!("select {COLS} from plugin_presets pr join plugin_products p on p.id = pr.product_id where p.available = 1 and (?1 is null or pr.product_id = ?1)")).unwrap();
    let rows: Vec<Value> = st.query_map([product], row).map(|m| m.filter_map(Result::ok).collect()).unwrap_or_default();
    let mut scored: Vec<(f64, Value)> = vec![];
    for mut p in rows {
        let (mut score, mut why) = (0.0, vec![]);
        let name_words = smartlib::classify::words(p["name"].as_str().unwrap_or(""));
        if let Some(cat) = p["category"].as_str() {
            if wanted_cat.contains(&cat) {
                score += 3.0;
                why.push(format!("category {cat}"));
            }
        }
        for d in &q.descriptors {
            let measured = p["details"]["timbre"]["descriptors"][d.as_str()].as_f64();
            if let Some(m) = measured.filter(|m| *m >= 0.5) {
                score += 2.0 * m;
                why.push(format!("{d} (measured {:.0}%)", m * 100.0));
            } else if name_words.iter().any(|w| w == d) {
                score += 1.5;
                why.push(format!("{d} (in the name)"));
            }
        }
        for g in &q.genres {
            let gw = g.replace('_', " ");
            if p["name"].as_str().unwrap_or("").to_lowercase().contains(&gw) || p["path"].as_str().unwrap_or("").to_lowercase().contains(&gw) {
                score += 1.0;
                why.push(format!("genre {gw} in name/folder"));
            }
        }
        for t in &q.text {
            if p["name"].as_str().unwrap_or("").to_lowercase().contains(t.as_str()) || p["plugin"].as_str().unwrap_or("").to_lowercase().contains(t.as_str()) {
                score += 1.0;
                why.push(format!("“{t}”"));
            }
        }
        if score > 0.0 || (wanted_cat.is_empty() && q.descriptors.is_empty() && q.text.is_empty()) {
            p["why"] = json!(why);
            p["score"] = json!((score * 10.0).round() / 10.0);
            p["origin"] = json!("plugin");
            scored.push((score, p));
        }
    }
    scored.sort_by(|a, b| b.0.total_cmp(&a.0));
    let mut results: Vec<Value> = scored.into_iter().take(limit).map(|x| x.1).collect();
    // presets in the authorized library folders (Smart Library), marked when their plug-in is not detected
    let mut lq = q.clone();
    lq.kind = Some("preset".into());
    let lib = smartlib::api::search_structured(c, &lq, limit);
    for mut r in lib["results"].as_array().cloned().unwrap_or_default() {
        let plugin = r["plugins"].as_array().and_then(|a| a.first()).and_then(|x| x["tag"].as_str()).map(String::from);
        let detected = plugin.as_ref().is_some_and(|p| {
            c.query_row("select count(*) from plugin_products where available = 1 and (id like ?1 or lower(name) like ?1)", [format!("%{}%", p.replace('_', ""))], |r| r.get::<_, i64>(0)).unwrap_or(0) > 0
        });
        r["origin"] = json!("library");
        r["plugin_detected"] = json!(detected);
        results.push(r);
    }
    json!({ "understood": { "categories": wanted_cat, "descriptors": q.descriptors, "genres": q.genres, "text": q.text }, "results": results,
            "note": "only presets found on this computer; timbre is measured only for presets analyzed by rendering, otherwise inferred from the name" })
}

/// Render an Audio Unit preset with a test phrase in the isolated host and measure its timbre.
pub fn analyze(c: &Connection, id: i64, work_dir: &Path) -> Result<Value, String> {
    let p = get(c, id).ok_or("unknown preset")?;
    let install_id = match p["install_id"].as_str() {
        Some(i) => i.to_string(),
        None => c
            .query_row("select id from plugin_installs where product_id = ?1 and au_code is not null and status = 'available' limit 1", [p["product_id"].as_str()], |r| r.get(0))
            .map_err(|_| "this preset belongs to a plug-in COSOUND cannot host (Audio Units on macOS only)".to_string())?,
    };
    let inst = registry::install(c, &install_id).ok_or("install not found")?;
    let code: Vec<String> = inst["au_code"].as_str().unwrap_or("").split(':').map(String::from).collect();
    let mut unit = json!({ "type": code[0], "subtype": code.get(1), "manufacturer": code.get(2) });
    match p["source"].as_str() {
        Some("factory") => unit["factory_preset"] = p["number"].clone(),
        _ if p["format"] == "aupreset" => unit["state_file"] = p["path"].clone(),
        _ => return Err("only Audio Unit presets (factory or .aupreset) can be rendered".into()),
    }
    std::fs::create_dir_all(work_dir).map_err(|e| e.to_string())?;
    let out = work_dir.join(format!("preset-{id}.wav"));
    let _ = std::fs::remove_file(&out);
    let instrument = inst["details"]["has_midi_input"].as_bool().unwrap_or(false) && code[0] == "aumu";
    let cat = p["category"].as_str().unwrap_or("");
    let pitch = if cat.contains("bass") { 33 } else { 57 };
    let notes = json!([{ "pitch": pitch, "start_s": 0.0, "dur_s": 1.5, "vel": 100 }, { "pitch": pitch + 7, "start_s": 0.0, "dur_s": 1.5, "vel": 90 }]);
    let req = if instrument {
        json!({ "seconds": 2.5, "out": out, "instrument": unit, "notes": notes })
    } else {
        // an effect preset is measured on COSOUND's General MIDI test tone (Apple DLS synth) through the effect
        json!({ "seconds": 2.5, "out": out, "instrument": { "type": "aumu", "subtype": "dls ", "manufacturer": "appl" }, "program": 81, "chain": [unit], "notes": notes })
    };
    host::run("render", &req, 60).map_err(|e| {
        registry::log_error(c, Some(&install_id), "analyze_preset", &e);
        e
    })?;
    let a = smartlib::analyze::analyze_file(&out, "audio", p["name"].as_str().unwrap_or(""), vec![], &AtomicBool::new(false))?;
    let timbre = json!({ "descriptors": a.details["features"]["descriptors"], "centroid_hz": a.details["features"]["centroid_hz"], "lufs": a.lufs, "peak_dbfs": a.peak_dbfs, "test": if instrument { "two notes rendered with the preset" } else { "test tone through the effect" } });
    let mut details = p["details"].clone();
    details["timbre"] = timbre.clone();
    c.execute("update plugin_presets set details = ?2 where id = ?1", params![id, details.to_string()]).map_err(|e| e.to_string())?;
    let _ = std::fs::remove_file(&out);
    Ok(json!({ "preset": p["name"], "timbre": timbre, "basis": "measured on a rendering (estimate of the sound character)" }))
}
