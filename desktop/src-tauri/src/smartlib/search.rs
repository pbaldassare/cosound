//! Search: structured criteria (from query.rs or the API) → ranked files with a match score and the
//! reasons for it. Also sound similarity and loop suggestions. Read-only on the index.
use super::features::similarity;
use super::query::Query;
use crate::music::theory::Key;
use rusqlite::{params_from_iter, Connection};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::path::Path;

struct Row {
    id: i64,
    name: String,
    rel: String,
    kind: String,
    ext: String,
    size: i64,
    status: String,
    duration: Option<f64>,
    bpm: Option<f64>,
    bpm_src: Option<String>,
    bpm_conf: Option<f64>,
    key: Option<String>,
    key_src: Option<String>,
    key_conf: Option<f64>,
    is_loop: Option<bool>,
    loop_bars: Option<f64>,
    category: Option<String>,
    vec: Option<Vec<f32>>,
    format: Option<String>,
    path: Option<String>,
    device: String,
    online: bool,
    analysis: String,
    lufs: Option<f64>,
    peak: Option<f64>,
    sample_rate: Option<i64>,
    channels: Option<i64>,
    bit_depth: Option<i64>,
    details: Option<String>,
}

const COLS: &str = "f.id, f.name, f.rel_path, f.kind, f.ext, f.size, f.status, f.duration_s, f.bpm, f.bpm_src, f.bpm_conf, f.musical_key, f.key_src, f.key_conf,
  f.is_loop, f.loop_bars, f.category, f.vec, f.format, v.mount_point, v.online, r.rel_path, v.label, f.analysis_state, f.lufs, f.peak_dbfs, f.sample_rate, f.channels, f.bit_depth";
const FROM: &str = "from files f join library_roots r on r.id = f.root_id join volumes v on v.id = r.volume_id where f.status != 'missing' and v.excluded = 0";

fn row(r: &rusqlite::Row) -> rusqlite::Result<Row> {
    let (mount, online, root_rel, rel): (Option<String>, bool, String, String) = (r.get(19)?, r.get(20)?, r.get(21)?, r.get(2)?);
    Ok(Row {
        id: r.get(0)?,
        name: r.get(1)?,
        path: mount.filter(|_| online).map(|m| Path::new(&m).join(&root_rel).join(&rel).to_string_lossy().to_string()),
        rel,
        kind: r.get(3)?,
        ext: r.get(4)?,
        size: r.get(5)?,
        status: r.get(6)?,
        duration: r.get(7)?,
        bpm: r.get(8)?,
        bpm_src: r.get(9)?,
        bpm_conf: r.get(10)?,
        key: r.get(11)?,
        key_src: r.get(12)?,
        key_conf: r.get(13)?,
        is_loop: r.get(14)?,
        loop_bars: r.get(15)?,
        category: r.get(16)?,
        vec: r.get::<_, Option<String>>(17)?.and_then(|s| serde_json::from_str(&s).ok()),
        format: r.get(18)?,
        device: r.get(22)?,
        online,
        analysis: r.get(23)?,
        lufs: r.get(24)?,
        peak: r.get(25)?,
        sample_rate: r.get(26)?,
        channels: r.get(27)?,
        bit_depth: r.get(28)?,
        details: None,
    })
}

type Tags = HashMap<i64, Vec<(String, String, String, f64)>>;

fn tags_for(c: &Connection, ids: &[i64]) -> Tags {
    let mut out: Tags = HashMap::new();
    for chunk in ids.chunks(900) {
        let sql = format!("select file_id, tag, kind, source, confidence from file_tags where file_id in ({})", vec!["?"; chunk.len()].join(","));
        let mut st = c.prepare(&sql).unwrap();
        let rows = st.query_map(params_from_iter(chunk.iter()), |r| Ok((r.get::<_, i64>(0)?, r.get::<_, String>(1)?, r.get::<_, String>(2)?, r.get::<_, String>(3)?, r.get::<_, f64>(4)?)));
        for (id, t, k, s, c) in rows.map(|m| m.filter_map(Result::ok).collect::<Vec<_>>()).unwrap_or_default() {
            out.entry(id).or_default().push((t, k, s, c));
        }
    }
    out
}

/// Continuous 0..1 descriptor scores from the stored timbre vector (see features.rs for the layout).
pub fn descriptor(name: &str, v: &[f32], duration: Option<f64>) -> Option<f64> {
    if v.len() < 14 {
        return None;
    }
    let g = |i: usize| v[i] as f64;
    let crest = g(13) * 20.0 + 3.0;
    let onset_rate = g(11) * 8.0;
    let long = ((duration.unwrap_or(0.0) - 1.5) / 3.0).clamp(0.0, 1.0);
    Some(match name {
        "deep" => g(2),
        "bright" => g(0),
        "dark" => 1.0 - g(0),
        "punchy" => g(8),
        "soft" => 1.0 - g(8),
        "noisy" => g(1),
        "tonal" => g(12),
        "warm" => (g(5) / 0.5).min(1.0) * (1.0 - g(0)) * g(12).max(0.3),
        "aggressive" => 0.5 * (g(6) / 0.3).min(1.0) + 0.5 * (1.0 - ((crest - 6.0) / 12.0).clamp(0.0, 1.0)),
        "round" => g(12) * (1.0 - g(0)) * (1.0 - (g(7) * 10.0).min(1.0)),
        "atmospheric" => long * (1.0 - g(8)) * (1.0 - (onset_rate / 4.0).min(1.0)),
        "percussive" => (onset_rate / 4.0).min(1.0).max(if duration.unwrap_or(9.0) < 1.0 && g(8) > 0.5 { 1.0 } else { 0.0 }),
        _ => return None,
    })
}

const SYNTH_FAMILY: [&str; 6] = ["lead", "pad", "pluck", "arpeggio", "chords", "bass"];

fn key_score(file_key: &str, q: &Query) -> Option<(f64, String)> {
    let fk = Key::parse(file_key).ok()?;
    if let Some(qk) = q.key.as_deref().and_then(|k| Key::parse(k).ok()) {
        let (a, b) = (fk.tonic.pc(), qk.tonic.pc());
        let same_mode = fk.mode.minor_third() == qk.mode.minor_third();
        let rel = qk.relative().map(|r| r.tonic.pc() == a && !same_mode).unwrap_or(false);
        let fifth = same_mode && ((a + 12 - b) % 12 == 7 || (b + 12 - a) % 12 == 7);
        return Some(if a == b && same_mode {
            (1.0, "same key".into())
        } else if rel {
            (0.9, "relative key".into())
        } else if fifth {
            (0.7, "key a fifth away (compatible)".into())
        } else if a == b {
            (0.5, "parallel key".into())
        } else {
            (0.05, format!("different key ({file_key})"))
        });
    }
    let m = q.mode.as_deref()?;
    let minor = fk.mode.minor_third();
    Some(if (m == "minor") == minor { (0.95, format!("{m} key ({file_key})")) } else { (0.1, format!("not {m} ({file_key})")) })
}

fn bpm_score(file: Option<f64>, conf: Option<f64>, q: &Query, one_shot: bool) -> Option<(f64, String)> {
    let target = q.bpm.or(q.bpm_range.map(|(a, b)| (a + b) / 2.0))?;
    let Some(b) = file else {
        return Some(if one_shot { (0.6, "one-shot: tempo not relevant".into()) } else { (0.3, "tempo unknown".into()) });
    };
    if let Some((lo, hi)) = q.bpm_range {
        return Some(if b >= lo && b <= hi { (1.0, format!("{b} BPM in range")) } else if b * 2.0 >= lo && b * 2.0 <= hi || b / 2.0 >= lo && b / 2.0 <= hi { (0.6, format!("{b} BPM (half/double time)")) } else { (0.05, format!("{b} BPM out of range")) });
    }
    let d = |x: f64| (x - target).abs() / target;
    let c = conf.unwrap_or(0.5);
    Some(if d(b) <= 0.01 {
        (1.0, format!("{b} BPM"))
    } else if d(b) <= 0.04 {
        (0.8, format!("{b} BPM (close, stretchable)"))
    } else if d(b * 2.0) <= 0.03 || d(b / 2.0) <= 0.03 {
        (0.6, format!("{b} BPM (half/double time)"))
    } else {
        (0.05 + 0.2 * (1.0 - c), format!("{b} BPM"))
    })
}

pub struct Options {
    pub limit: usize,
    pub include_offline: bool,
}

pub fn search(c: &Connection, q: &Query, o: &Options) -> Vec<Value> {
    let mut sql = format!("select {COLS} {FROM}");
    let mut args: Vec<String> = Vec::new();
    if let Some(k) = &q.kind {
        sql.push_str(" and f.kind = ?");
        args.push(k.clone());
    }
    if !q.formats.is_empty() {
        sql.push_str(&format!(" and f.ext in ({})", vec!["?"; q.formats.len() + 1].join(",")));
        args.extend(q.formats.iter().cloned());
        if q.formats.contains(&"aiff".into()) {
            args.push("aif".into());
        } else {
            args.push(q.formats[0].clone());
        }
    }
    if let Some(d) = &q.device {
        sql.push_str(" and lower(v.label) = lower(?)");
        args.push(d.clone());
    }
    if !o.include_offline {
        // offline files stay findable (path null) unless the caller asks for usable files only
    }
    if let Some(max) = q.max_duration_s {
        sql.push_str(&format!(" and (f.duration_s is null or f.duration_s <= {max})"));
    }
    if let Some(min) = q.min_duration_s {
        sql.push_str(&format!(" and f.duration_s >= {min}"));
    }
    // pre-filter on labels or text when the request names something
    let mut wanted: Vec<String> = q.categories.clone();
    if q.categories.iter().any(|c| c == "synth") {
        wanted.extend(SYNTH_FAMILY.iter().map(|s| s.to_string()));
    }
    if let Some(p) = &q.plugin {
        wanted.push(p.clone());
    }
    let mut ors: Vec<String> = Vec::new();
    if !wanted.is_empty() {
        ors.push(format!("f.id in (select file_id from file_tags where tag in ({}))", vec!["?"; wanted.len()].join(",")));
        args.extend(wanted.iter().cloned());
    }
    for t in q.text.iter().chain(q.descriptors.iter().filter(|d| ["female", "male", "tribal", "acid", "analog", "distorted"].contains(&d.as_str()))) {
        ors.push("(lower(f.name) like ? or lower(f.rel_path) like ?)".into());
        args.push(format!("%{t}%"));
        args.push(format!("%{t}%"));
    }
    if !ors.is_empty() {
        sql.push_str(&format!(" and ({})", ors.join(" or ")));
    }
    sql.push_str(" limit 50000");
    let rows: Vec<Row> = {
        let mut st = c.prepare(&sql).unwrap();
        st.query_map(params_from_iter(args.iter()), row).map(|m| m.filter_map(Result::ok).collect()).unwrap_or_default()
    };
    let tags = tags_for(c, &rows.iter().map(|r| r.id).collect::<Vec<_>>());
    let mut scored: Vec<(f64, Value)> = Vec::new();
    for r in rows {
        let t = tags.get(&r.id).cloned().unwrap_or_default();
        let has = |tag: &str, kind: &str| t.iter().filter(|x| x.0 == tag && x.1 == kind).map(|x| x.3).fold(None, |a: Option<f64>, c| Some(a.map_or(c, |a| a.max(c))));
        let mut total = 0.0;
        let mut weight = 0.0;
        let mut why: Vec<String> = Vec::new();
        let mut add = |w: f64, s: f64, reason: String| {
            total += w * s;
            weight += w;
            if s >= 0.5 {
                why.push(reason);
            }
        };
        let one_shot = has("one_shot", "type").is_some();
        if !q.categories.is_empty() {
            let (mut s, mut label) = (0.0, String::new());
            for cat in &q.categories {
                let exact = has(cat, "category").map(|c| (c, cat.clone()));
                let family = if cat == "synth" { SYNTH_FAMILY.iter().filter_map(|f| has(f, "category").map(|c| (c * 0.7, f.to_string()))).max_by(|a, b| a.0.total_cmp(&b.0)) } else { None };
                if let Some((c, l)) = exact.into_iter().chain(family).max_by(|a, b| a.0.total_cmp(&b.0)) {
                    if c > s {
                        s = c;
                        label = l;
                    }
                }
            }
            if s == 0.0 && q.text.is_empty() {
                continue; // asked for a category this file does not have
            }
            add(3.0, s, format!("category {label} ({:.0}%)", s * 100.0));
        }
        for ty in &q.types {
            let s = has(ty, "type").unwrap_or(if ty == "loop" && r.is_loop == Some(true) { 0.7 } else { 0.0 });
            if s == 0.0 && q.categories.is_empty() && q.text.is_empty() {
                continue;
            }
            add(1.5, s, format!("{ty}"));
        }
        if !q.genres.is_empty() {
            let s = q.genres.iter().filter_map(|g| has(g, "genre")).fold(0.0, f64::max);
            add(1.0, if s > 0.0 { s } else { 0.35 }, format!("genre label ({})", q.genres.join(", ")));
        }
        if let Some((s, why_bpm)) = bpm_score(r.bpm, r.bpm_conf, q, one_shot) {
            add(2.0, s, why_bpm);
        }
        if q.key.is_some() || q.mode.is_some() {
            match r.key.as_deref().and_then(|k| key_score(k, q)) {
                Some((s, w)) => add(1.5, s, w),
                None => add(1.5, 0.3, "key unknown".into()),
            }
        }
        for d in &q.descriptors {
            let from_name = has(d, "descriptor");
            let from_audio = r.vec.as_deref().and_then(|v| descriptor(d, v, r.duration));
            let s = match (from_audio, from_name) {
                (Some(a), Some(n)) => a.max(n),
                (Some(a), None) => a,
                (None, Some(n)) => n,
                (None, None) => 0.35,
            };
            add(2.0, s, format!("{d} ({:.0}%)", s * 100.0));
        }
        if let Some(b) = q.bars {
            add(1.0, if r.loop_bars == Some(b) { 1.0 } else { 0.2 }, format!("{b} bars"));
        }
        if let Some(p) = &q.plugin {
            let s = has(p, "plugin").unwrap_or(0.0);
            if s == 0.0 {
                continue; // never offer a preset for a plugin it was not found for
            }
            add(2.0, s, format!("preset for {p}"));
        }
        if !q.text.is_empty() {
            let hay = format!("{} {} {}", r.name.to_lowercase(), r.rel.to_lowercase(), t.iter().map(|x| x.0.as_str()).collect::<Vec<_>>().join(" "));
            let hits = q.text.iter().filter(|w| hay.contains(w.as_str())).count();
            add(2.0, hits as f64 / q.text.len() as f64, format!("text match {hits}/{}", q.text.len()));
        }
        let score = if weight > 0.0 { total / weight } else { 0.5 };
        let mut d = brief(&r, &t);
        d["score"] = json!((score * 100.0).round());
        d["why"] = json!(why);
        scored.push((score, d));
    }
    scored.sort_by(|a, b| b.0.total_cmp(&a.0).then_with(|| a.1["name"].as_str().cmp(&b.1["name"].as_str())));
    scored.into_iter().take(o.limit).map(|s| s.1).collect()
}

fn brief(r: &Row, t: &[(String, String, String, f64)]) -> Value {
    let labels = |kind: &str| -> Vec<Value> {
        let mut v: Vec<&(String, String, String, f64)> = t.iter().filter(|x| x.1 == kind).collect();
        v.sort_by(|a, b| b.3.total_cmp(&a.3));
        v.iter().map(|x| json!({ "tag": x.0, "source": x.2, "confidence": x.3 })).collect()
    };
    json!({
        "id": r.id, "name": r.name, "kind": r.kind, "ext": r.ext, "format": r.format, "size": r.size, "status": r.status,
        "path": r.path, "rel_path": r.rel, "device": r.device, "device_online": r.online,
        "duration_s": r.duration, "sample_rate": r.sample_rate, "channels": r.channels, "bit_depth": r.bit_depth, "lufs": r.lufs, "peak_dbfs": r.peak,
        "bpm": { "value": r.bpm, "source": r.bpm_src, "confidence": r.bpm_conf },
        "key": { "value": r.key, "source": r.key_src, "confidence": r.key_conf },
        "is_loop": r.is_loop, "loop_bars": r.loop_bars, "category": r.category,
        "categories": labels("category"), "types": labels("type"), "genres": labels("genre"), "descriptors": labels("descriptor"), "plugins": labels("plugin"),
        "analysis": r.analysis, "details": r.details.as_deref().and_then(|d| serde_json::from_str::<Value>(d).ok()),
    })
}

/// One file with everything known about it (for detail views and the DAW Controller).
pub fn detail(c: &Connection, id: i64) -> Option<Value> {
    let mut st = c.prepare(&format!("select {COLS} from files f join library_roots r on r.id = f.root_id join volumes v on v.id = r.volume_id where f.id = ?1")).ok()?;
    let mut r = st.query_row([id], row).ok()?;
    r.details = c.query_row("select details from files where id = ?1", [id], |x| x.get(0)).ok().flatten();
    let t = tags_for(c, &[id]).remove(&id).unwrap_or_default();
    let mut v = brief(&r, &t);
    let mut cs = c.prepare("select field, value, updated_at from file_corrections where file_id = ?1").ok()?;
    v["corrections"] = json!(cs.query_map([id], |x| Ok(json!({ "field": x.get::<_, String>(0)?, "value": x.get::<_, Option<String>>(1)?, "at": x.get::<_, String>(2)? }))).map(|m| m.filter_map(Result::ok).collect::<Vec<_>>()).unwrap_or_default());
    let (ms, mf, err): (Option<String>, Option<String>, Option<String>) = c.query_row("select missing_since, moved_from, analysis_error from files where id = ?1", [id], |x| Ok((x.get(0)?, x.get(1)?, x.get(2)?))).ok()?;
    v["missing_since"] = json!(ms);
    v["moved_from"] = json!(mf);
    v["analysis_error"] = json!(err);
    Some(v)
}

/// Files whose timbre is closest to `id` (estimated similarity, never a guarantee of a match).
pub fn similar(c: &Connection, id: i64, limit: usize) -> Result<Value, String> {
    let target: Vec<f32> = c
        .query_row("select vec from files where id = ?1", [id], |r| r.get::<_, Option<String>>(0))
        .map_err(|_| "file not found".to_string())?
        .and_then(|s| serde_json::from_str(&s).ok())
        .ok_or("this file has no sound analysis yet")?;
    let mut st = c.prepare(&format!("select {COLS} {FROM} and f.vec is not null and f.id != ?1 limit 50000")).unwrap();
    let rows: Vec<Row> = st.query_map([id], row).map(|m| m.filter_map(Result::ok).collect()).unwrap_or_default();
    let mut s: Vec<(f64, Row)> = rows.into_iter().filter_map(|r| r.vec.as_deref().map(|v| similarity(&target, v)).map(|x| (x, r))).filter(|x| x.0 >= 0.1).collect();
    s.sort_by(|a, b| b.0.total_cmp(&a.0));
    let top: Vec<(f64, Row)> = s.into_iter().take(limit).collect();
    let tags = tags_for(c, &top.iter().map(|x| x.1.id).collect::<Vec<_>>());
    Ok(json!({
        "reference": id,
        "note": "estimated timbre similarity (spectrum, envelope, rhythm density): not a guarantee that two sounds match musically",
        "results": top.iter().map(|(sim, r)| { let mut b = brief(r, tags.get(&r.id).map(|v| v.as_slice()).unwrap_or(&[])); b["similarity"] = json!((sim * 100.0).round()); b }).collect::<Vec<_>>(),
    }))
}

/// Loops that fit a project tempo (and key, when given). Files are never modified: the stretch
/// needed is reported for the DAW.
pub fn loops_for(c: &Connection, bpm: f64, key: Option<&str>, category: Option<&str>, limit: usize) -> Value {
    let mut sql = format!("select {COLS} {FROM} and f.is_loop = 1 and f.bpm is not null");
    if let Some(cat) = category {
        sql.push_str(&format!(" and f.id in (select file_id from file_tags where tag = '{}' and kind = 'category')", cat.replace('\'', "")));
    }
    let mut st = c.prepare(&sql).unwrap();
    let rows: Vec<Row> = st.query_map([], row).map(|m| m.filter_map(Result::ok).collect()).unwrap_or_default();
    let q = Query { key: key.map(String::from), ..Default::default() };
    let mut out: Vec<(f64, Value)> = Vec::new();
    for r in rows {
        let b = r.bpm.unwrap_or(0.0);
        let (ratio, how) = [(bpm / b, "same time"), (bpm / (b * 2.0), "double time"), (bpm / (b / 2.0), "half time")].into_iter().min_by(|a, b| (a.0 - 1.0).abs().total_cmp(&(b.0 - 1.0).abs())).unwrap();
        let stretch = (ratio - 1.0) * 100.0;
        if stretch.abs() > 8.0 {
            continue;
        }
        let mut score = 1.0 - stretch.abs() / 16.0;
        let mut key_note = Value::Null;
        if key.is_some() {
            match r.key.as_deref().and_then(|k| key_score(k, &q)) {
                Some((s, w)) => {
                    score *= 0.5 + 0.5 * s;
                    key_note = json!(w);
                }
                None => {
                    score *= 0.6;
                    key_note = json!("key unknown");
                }
            }
        }
        let t = tags_for(c, &[r.id]).remove(&r.id).unwrap_or_default();
        let mut v = brief(&r, &t);
        v["score"] = json!((score * 100.0).round());
        v["fit"] = json!({ "project_bpm": bpm, "loop_bpm": b, "relation": how, "stretch_percent": (stretch * 100.0).round() / 100.0, "key": key_note });
        out.push((score, v));
    }
    out.sort_by(|a, b| b.0.total_cmp(&a.0));
    json!({ "project_bpm": bpm, "key": key, "note": "the original files are not modified; time-stretch, if needed, happens in the DAW", "results": out.into_iter().take(limit).map(|x| x.1).collect::<Vec<_>>() })
}
