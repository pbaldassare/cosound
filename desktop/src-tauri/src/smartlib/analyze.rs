//! Analysis of indexed files (background, resumable): audio measurements and features, embedded and
//! file-name metadata, MIDI analysis, preset reading, classification. Results are written with the
//! source and confidence of every value; user corrections are re-applied after every analysis.
use super::classify::{self, Label};
use super::decode::{self, DecodeError};
use super::features::{Analyzer, Estimate, VERSION};
use super::{midi, presets};
use crate::db::Db;
use crate::library;
use rusqlite::{params, Connection};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};

/// Long files are analyzed on their first part only (the result says so).
pub const MAX_SECONDS: f64 = 90.0;

#[derive(Debug, Default, Clone)]
pub struct Analysis {
    pub format: Option<String>,
    pub duration_s: Option<f64>,
    pub sample_rate: Option<u32>,
    pub channels: Option<u32>,
    pub bit_depth: Option<u32>,
    pub peak_dbfs: Option<f64>,
    pub rms_dbfs: Option<f64>,
    pub lufs: Option<f64>,
    pub bpm: Estimate<f64>,
    pub key: Estimate<String>,
    pub is_loop: Option<bool>,
    pub loop_bars: Option<f64>,
    pub labels: Vec<Label>,
    pub details: Value,
    pub vec: Option<Vec<f32>>,
    pub audio_hash: Option<String>,
    pub quick_hash: Option<String>,
}

fn est<T>(value: Option<T>, confidence: f64, source: &str) -> Estimate<T> {
    Estimate { value, confidence, source: source.into(), note: None }
}

/// Pick the best of several candidates: user > embedded > confident analysis > metadata > file name > weak analysis.
fn best<T: Clone>(cands: Vec<Estimate<T>>) -> Estimate<T> {
    let rank = |e: &Estimate<T>| match e.source.as_str() {
        "embedded" => 5.0,
        s if s.starts_with("analysis") && e.confidence >= 0.5 => 4.0 + e.confidence,
        "metadata" => 3.5,
        "filename" => 3.0,
        _ => e.confidence,
    };
    cands.into_iter().filter(|e| e.value.is_some()).max_by(|a, b| rank(a).total_cmp(&rank(b))).unwrap_or(Estimate { value: None, confidence: 0.0, source: "unknown".into(), note: Some("not determinable from this file".into()) })
}

pub fn analyze_file(path: &Path, kind: &str, name: &str, folders: Vec<String>, cancel: &AtomicBool) -> Result<Analysis, String> {
    let mut a = Analysis { quick_hash: super::scan::quick_hash(path), ..Default::default() };
    let (fname_bpm, fname_key) = library::filename_meta(name);
    let mut bpm_c: Vec<Estimate<f64>> = vec![est(fname_bpm.map(|b| b as f64), 0.6, "filename")];
    let mut key_c: Vec<Estimate<String>> = vec![est(fname_key.clone(), 0.6, "filename")];
    let mut features = None;
    let mut tags = None;
    let mut midi_v = None;
    let mut preset_v = None;
    match kind {
        "audio" => {
            let mut an: Option<Analyzer> = None;
            let mut hasher = Sha256::new();
            let dec = decode::decode(path, Some(MAX_SECONDS), &|| cancel.load(Ordering::SeqCst), &mut |buf, ch, rate| {
                let x = an.get_or_insert_with(|| Analyzer::new(rate, ch));
                if x.channels() != ch {
                    return;
                }
                x.push(buf, ch);
                for v in buf {
                    hasher.update(((v.clamp(-1.0, 1.0) * 32767.0).round() as i16).to_le_bytes());
                }
            });
            let d = match dec {
                Ok(d) => d,
                Err(DecodeError::Cancelled) => return Err("cancelled".into()),
                Err(DecodeError::Unsupported(e)) | Err(DecodeError::Io(e)) => return Err(e),
            };
            let an = an.ok_or("no audio decoded")?;
            let m = an.measurements(if d.codec.starts_with("pcm") { "pcm" } else { "decoded" }, d.bit_depth.unwrap_or(0) as u16);
            let dur = d.duration_s.unwrap_or(m.duration_s);
            let f = an.finish(dur, &m);
            a.format = Some(format!("{} / {}", d.container, d.codec));
            a.duration_s = Some((dur * 1000.0).round() / 1000.0);
            a.sample_rate = Some(d.sample_rate);
            a.channels = Some(d.channels as u32);
            a.bit_depth = d.bit_depth;
            a.peak_dbfs = Some(m.sample_peak_dbfs);
            a.rms_dbfs = Some(m.rms_dbfs);
            a.lufs = m.integrated_lufs;
            a.is_loop = Some(f.is_loop);
            a.loop_bars = f.loop_bars;
            let hex: String = hasher.finalize().iter().map(|b| format!("{b:02x}")).collect();
            a.audio_hash = Some(format!("{}:{}:{hex}", d.sample_rate, d.channels));
            bpm_c.push(f.bpm.clone());
            key_c.push(f.key.clone());
            if let Some(b) = d.tags.get("bpm").and_then(|v| v.as_f64().or_else(|| v.as_str().and_then(|s| s.parse().ok()))).filter(|b| (40.0..=300.0).contains(b)) {
                bpm_c.push(est(Some(b), 0.7, "metadata"));
            }
            if let Some(k) = d.tags.get("key").and_then(|v| v.as_str()).and_then(|k| crate::music::theory::Key::parse(k).ok()) {
                key_c.push(est(Some(k.name()), 0.7, "metadata"));
            }
            let embedded = if path.extension().is_some_and(|e| e.eq_ignore_ascii_case("wav") || e.eq_ignore_ascii_case("wave")) { decode::wav_loop_info(path) } else { None };
            if let Some(t) = embedded.as_ref().and_then(|e| e["acid_tempo"].as_f64()) {
                bpm_c.push(est(Some(t), 0.9, "embedded"));
            }
            a.vec = Some(f.vector.clone());
            a.details = json!({
                "analyzer_version": VERSION, "decoder": "symphonia", "container": d.container, "codec": d.codec,
                "analyzed_s": f.analyzed_s, "truncated": d.truncated, "measurements": m, "features": f, "metadata": d.tags, "embedded": embedded,
            });
            tags = Some(Value::Object(d.tags));
            features = Some(f);
        }
        "midi" => {
            let bytes = std::fs::read(path).map_err(|e| e.to_string())?;
            let s = midi::parse(&bytes)?;
            let m = midi::analyze(&s);
            if let Some(b) = m["bpm"]["value"].as_f64() {
                bpm_c.push(est(Some(b), 1.0, "embedded"));
            }
            if let Some(k) = m["key"]["value"].as_str() {
                key_c.push(est(Some(k.to_string()), m["key"]["confidence"].as_f64().unwrap_or(0.5), "analysis"));
            }
            a.duration_s = m["duration_s"].as_f64();
            a.format = Some(format!("SMF type {}", s.format));
            a.is_loop = Some(true);
            a.loop_bars = m["bars"].as_f64();
            a.details = json!({ "midi": m });
            midi_v = Some(m);
        }
        "preset" | "instrument" => {
            let p = presets::read(path);
            a.format = p["format"].as_str().map(String::from);
            a.details = json!({ "preset": p });
            preset_v = Some(p);
        }
        _ => {}
    }
    a.bpm = best(bpm_c);
    a.key = best(key_c);
    a.labels = classify::classify(&classify::Input { name, folders, kind, features: features.as_ref(), tags: tags.as_ref(), midi: midi_v.as_ref(), preset: preset_v.as_ref() });
    if let Some(d) = a.details.as_object_mut() {
        d.insert("bpm".into(), json!(a.bpm));
        d.insert("key".into(), json!(a.key));
    }
    Ok(a)
}

/// Write an analysis, then re-apply the user's corrections (which always win).
pub fn store(c: &Connection, id: i64, a: &Analysis) -> rusqlite::Result<()> {
    let tx = c.unchecked_transaction()?;
    tx.execute(
        "update files set analysis_state = 'done', analysis_version = ?2, analysis_error = null, analyzed_at = datetime('now'), format = ?3, duration_s = ?4, sample_rate = ?5,
           channels = ?6, bit_depth = ?7, peak_dbfs = ?8, rms_dbfs = ?9, lufs = ?10, bpm = ?11, bpm_src = ?12, bpm_conf = ?13, musical_key = ?14, key_src = ?15, key_conf = ?16,
           is_loop = ?17, loop_bars = ?18, category = ?19, details = ?20, vec = ?21, audio_hash = ?22, quick_hash = coalesce(?23, quick_hash),
           status = case when status in ('modified', 'moved') then 'present' else status end where id = ?1",
        params![
            id,
            VERSION,
            a.format,
            a.duration_s,
            a.sample_rate,
            a.channels,
            a.bit_depth,
            a.peak_dbfs,
            a.rms_dbfs,
            a.lufs,
            a.bpm.value,
            a.bpm.value.map(|_| a.bpm.source.clone()),
            a.bpm.value.map(|_| a.bpm.confidence),
            a.key.value,
            a.key.value.as_ref().map(|_| a.key.source.clone()),
            a.key.value.as_ref().map(|_| a.key.confidence),
            a.is_loop,
            a.loop_bars,
            classify::primary(&a.labels),
            a.details.to_string(),
            a.vec.as_ref().map(|v| serde_json::to_string(v).unwrap_or_default()),
            a.audio_hash,
            a.quick_hash,
        ],
    )?;
    tx.execute("delete from file_tags where file_id = ?1 and source != 'user'", [id])?;
    for l in &a.labels {
        tx.execute("insert or ignore into file_tags(file_id, tag, kind, source, confidence) values (?1, ?2, ?3, ?4, ?5)", params![id, l.tag, l.kind, l.source, l.confidence])?;
    }
    super::index::apply_corrections(&tx, id)?;
    tx.commit()
}

pub struct Pending {
    pub id: i64,
    pub path: PathBuf,
    pub kind: String,
    pub name: String,
    pub folders: Vec<String>,
}

/// Files waiting for analysis on connected drives, oldest first.
pub fn pending(c: &Connection, limit: usize) -> Vec<Pending> {
    let mut st = c
        .prepare(
            "select f.id, v.mount_point, r.rel_path, f.rel_path, f.kind, f.name from files f join library_roots r on r.id = f.root_id join volumes v on v.id = r.volume_id
             where f.analysis_state = 'pending' and f.status != 'missing' and v.online = 1 and v.excluded = 0 and f.kind in ('audio', 'midi', 'preset', 'instrument', 'project')
             order by f.id limit ?1",
        )
        .unwrap();
    st.query_map([limit as i64], |r| {
        let (mount, root_rel, rel): (String, String, String) = (r.get(1)?, r.get(2)?, r.get(3)?);
        let folders = Path::new(&rel).parent().map(|p| p.iter().map(|x| x.to_string_lossy().to_string()).collect()).unwrap_or_default();
        Ok(Pending { id: r.get(0)?, path: Path::new(&mount).join(root_rel).join(&rel), kind: r.get(4)?, name: r.get(5)?, folders })
    })
    .map(|m| m.filter_map(Result::ok).collect())
    .unwrap_or_default()
}

/// Analyze up to `limit` pending files. Decoding happens without holding the database lock.
pub fn run_batch(db: &Db, limit: usize, cancel: &AtomicBool, pause_ms: u64) -> usize {
    let todo = pending(&db.lock().unwrap(), limit);
    let mut done = 0;
    for p in todo {
        if cancel.load(Ordering::SeqCst) {
            break;
        }
        let res = if p.kind == "project" {
            Ok(Analysis { labels: classify::classify(&classify::Input { name: &p.name, folders: p.folders.clone(), kind: "project", features: None, tags: None, midi: None, preset: None }), details: json!({}), ..Default::default() })
        } else {
            analyze_file(&p.path, &p.kind, &p.name, p.folders.clone(), cancel)
        };
        let c = db.lock().unwrap();
        match res {
            Ok(a) => {
                if let Err(e) = store(&c, p.id, &a) {
                    // never leave a file pending forever: record why it could not be stored
                    let _ = c.execute("update files set analysis_state = 'error', analysis_error = ?2 where id = ?1", params![p.id, format!("could not store the analysis: {e}")]);
                }
            }
            Err(e) if e == "cancelled" => break,
            Err(e) => {
                let _ = c.execute("update files set analysis_state = 'error', analysis_error = ?2, analyzed_at = datetime('now') where id = ?1", params![p.id, e]);
                let labels = classify::classify(&classify::Input { name: &p.name, folders: p.folders.clone(), kind: &p.kind, features: None, tags: None, midi: None, preset: None });
                for l in labels {
                    let _ = c.execute("insert or ignore into file_tags(file_id, tag, kind, source, confidence) values (?1, ?2, ?3, ?4, ?5)", params![p.id, l.tag, l.kind, l.source, l.confidence]);
                }
            }
        }
        done += 1;
        if pause_ms > 0 {
            std::thread::sleep(std::time::Duration::from_millis(pause_ms));
        }
    }
    done
}
