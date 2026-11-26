//! Incremental scanner. Walks an authorized root (never anything else), skips excluded folders and
//! hidden entries, and updates the index in place:
//! - unchanged (same size and mtime): only marked as seen, nothing is read;
//! - changed: `modified`, queued for analysis;
//! - new: inserted, queued for analysis; if it matches a missing file (same size and content
//!   fingerprint, or same name and size when no fingerprint exists yet) it is that file `moved`;
//! - gone while the drive IS connected and the walk completed: `missing` (kept, never deleted);
//! - drive not connected: nothing changes (`offline`); folder gone on a connected drive: `unavailable`.
//! Files are never opened for writing. A cancelled walk never marks anything missing.
use crate::db::{new_id, Db};
use crate::library;
use rusqlite::{params, Connection, OptionalExtension};
use serde::Serialize;
use sha2::{Digest, Sha256};
use std::collections::{HashMap, HashSet};
use std::io::{Read, Seek, SeekFrom};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};

#[derive(Serialize, Default, Clone, Debug)]
pub struct ScanSummary {
    pub job_id: String,
    pub root_id: String,
    /// done | cancelled | offline | unavailable | excluded | error
    pub state: String,
    pub seen: usize,
    pub new: usize,
    pub changed: usize,
    pub moved: usize,
    pub missing: usize,
    pub back: usize,
    pub present: usize,
    pub error: Option<String>,
    pub duration_ms: u64,
}

pub enum RootState {
    Online(PathBuf),
    Offline(String),
    Unavailable(PathBuf),
    Excluded(String),
}

pub fn root_state(c: &Connection, root_id: &str) -> Result<RootState, String> {
    let row: Option<(Option<String>, bool, String, String, bool)> = c
        .query_row(
            "select v.mount_point, v.online, r.rel_path, v.label, v.excluded from library_roots r join volumes v on v.id = r.volume_id where r.id = ?1",
            [root_id],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?, r.get(4)?)),
        )
        .optional()
        .map_err(|e| e.to_string())?;
    let (mount, online, rel, label, excluded) = row.ok_or("folder not found")?;
    if excluded {
        return Ok(RootState::Excluded(label));
    }
    let Some(mount) = mount.filter(|_| online) else { return Ok(RootState::Offline(label)) };
    let p = Path::new(&mount).join(rel);
    Ok(if p.is_dir() { RootState::Online(p) } else { RootState::Unavailable(p) })
}

/// SHA-256 over the size and the first and last 64 KiB: a cheap content fingerprint.
pub fn quick_hash(path: &Path) -> Option<String> {
    let mut f = std::fs::File::open(path).ok()?;
    let size = f.metadata().ok()?.len();
    let mut h = Sha256::new();
    h.update(size.to_le_bytes());
    let mut buf = vec![0u8; 65536];
    let n = f.read(&mut buf).ok()?;
    h.update(&buf[..n]);
    if size > 131_072 {
        f.seek(SeekFrom::End(-65536)).ok()?;
        let n = f.read(&mut buf).ok()?;
        h.update(&buf[..n]);
    }
    Some(h.finalize().iter().map(|b| format!("{b:02x}")).collect())
}

struct Existing {
    id: i64,
    size: i64,
    mtime: Option<i64>,
    status: String,
    name: String,
    quick_hash: Option<String>,
}

struct Found {
    rel: String,
    name: String,
    ext: String,
    kind: &'static str,
    size: i64,
    mtime: Option<i64>,
    path: PathBuf,
}

fn finish_job(c: &Connection, s: &ScanSummary) {
    let _ = c.execute(
        "update scan_jobs set state = ?2, files_seen = ?3, files_new = ?4, files_changed = ?5, files_moved = ?6, files_missing = ?7, files_back = ?8, finished_at = datetime('now'), error = ?9 where id = ?1",
        params![s.job_id, s.state, s.seen as i64, s.new as i64, s.changed as i64, s.moved as i64, s.missing as i64, s.back as i64, s.error],
    );
}

pub fn scan_root(db: &Db, root_id: &str, cancel: &AtomicBool, progress: &dyn Fn(usize)) -> ScanSummary {
    let t0 = std::time::Instant::now();
    let mut s = ScanSummary { job_id: new_id(), root_id: root_id.into(), ..Default::default() };
    let (state, exclusions, existing) = {
        let c = db.lock().unwrap();
        let _ = c.execute("insert into scan_jobs(id, root_id, kind, state) values (?1, ?2, 'incremental', 'running')", params![s.job_id, root_id]);
        let state = match root_state(&c, root_id) {
            Ok(st) => st,
            Err(e) => {
                s.state = "error".into();
                s.error = Some(e);
                finish_job(&c, &s);
                return s;
            }
        };
        let mut st = c.prepare("select rel_path from library_exclusions where root_id = ?1").unwrap();
        let excl: Vec<String> = st.query_map([root_id], |r| r.get(0)).map(|m| m.filter_map(Result::ok).collect()).unwrap_or_default();
        let mut st = c.prepare("select id, rel_path, size, mtime, status, name, quick_hash from files where root_id = ?1").unwrap();
        let ex: HashMap<String, Existing> = st
            .query_map([root_id], |r| Ok((r.get::<_, String>(1)?, Existing { id: r.get(0)?, size: r.get(2)?, mtime: r.get(3)?, status: r.get(4)?, name: r.get(5)?, quick_hash: r.get(6)? })))
            .map(|m| m.filter_map(Result::ok).collect())
            .unwrap_or_default();
        (state, excl, ex)
    };
    let base = match state {
        RootState::Online(p) => p,
        other => {
            let (st, msg) = match other {
                RootState::Offline(l) => ("offline", format!("folder offline: drive not connected ({l})")),
                RootState::Unavailable(p) => ("unavailable", format!("folder not available: {} (the drive is connected)", p.display())),
                RootState::Excluded(l) => ("excluded", format!("device excluded by the user ({l})")),
                RootState::Online(_) => unreachable!(),
            };
            let c = db.lock().unwrap();
            let _ = c.execute("update library_roots set scan_state = ?2, error = ?3 where id = ?1", params![root_id, st, msg]);
            s.state = st.into();
            s.error = Some(msg);
            finish_job(&c, &s);
            return s;
        }
    };
    let _ = db.lock().unwrap().execute("update library_roots set scan_state = 'scanning', error = null where id = ?1", [root_id]);

    let mut seen: HashSet<String> = HashSet::new();
    let mut changed: Vec<(i64, i64, Option<i64>)> = Vec::new();
    let mut back: Vec<i64> = Vec::new();
    let mut found_new: Vec<Found> = Vec::new();
    let mut cancelled = false;
    let mut it = walkdir::WalkDir::new(&base).follow_links(false).into_iter();
    while let Some(entry) = it.next() {
        if seen.len() % 256 == 0 && cancel.load(Ordering::SeqCst) {
            cancelled = true;
            break;
        }
        let Ok(e) = entry else { continue }; // unreadable entries are skipped
        if e.depth() == 0 {
            continue;
        }
        let rel = e.path().strip_prefix(&base).map(|r| r.to_string_lossy().to_string()).unwrap_or_default();
        if e.file_name().to_string_lossy().starts_with('.') || exclusions.iter().any(|x| rel == *x || rel.starts_with(&format!("{x}/"))) {
            if e.file_type().is_dir() {
                it.skip_current_dir();
            }
            continue;
        }
        let ext = e.path().extension().map(|x| x.to_string_lossy().to_ascii_lowercase()).unwrap_or_default();
        let package = e.file_type().is_dir() && ext == "logicx";
        if !(e.file_type().is_file() || package) {
            continue;
        }
        if package {
            it.skip_current_dir();
        }
        let Some(kind) = library::kind_of(&ext) else { continue };
        let meta = e.metadata().ok();
        let size = meta.as_ref().map_or(0, |m| m.len() as i64);
        let mtime = meta.and_then(|m| m.modified().ok()).and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok()).map(|d| d.as_secs() as i64);
        seen.insert(rel.clone());
        if seen.len() % 500 == 0 {
            progress(seen.len());
        }
        match existing.get(&rel) {
            Some(x) if x.size == size && x.mtime == mtime => {
                if x.status == "missing" {
                    back.push(x.id);
                }
            }
            Some(x) => changed.push((x.id, size, mtime)),
            None => found_new.push(Found { rel, name: e.file_name().to_string_lossy().to_string(), ext, kind, size, mtime, path: e.path().to_path_buf() }),
        }
    }
    s.seen = seen.len();
    // files that disappeared (only when the walk completed on a connected drive)
    let mut gone: Vec<(&String, &Existing)> = if cancelled { vec![] } else { existing.iter().filter(|(rel, x)| !seen.contains(*rel) && x.status != "missing").collect() };

    let c = db.lock().unwrap();
    let tx = c.unchecked_transaction().unwrap();
    // files missing from other roots (a move across drives), loaded once and grouped by size
    let mut elsewhere: HashMap<i64, Vec<(i64, String, Option<String>, String)>> = HashMap::new();
    if !found_new.is_empty() {
        if let Ok(mut st) = tx.prepare("select id, rel_path, quick_hash, name, size from files where status = 'missing' and root_id != ?1") {
            if let Ok(rows) = st.query_map([root_id], |r| Ok((r.get::<_, i64>(4)?, (r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?)))) {
                for (size, m) in rows.filter_map(Result::ok) {
                    elsewhere.entry(size).or_default().push(m);
                }
            }
        }
    }
    for id in &back {
        let _ = tx.execute("update files set status = 'present', missing_since = null where id = ?1", [id]);
    }
    for (id, size, mtime) in &changed {
        let _ = tx.execute(
            "update files set size = ?2, mtime = ?3, status = 'modified', quick_hash = null, full_hash = null, audio_hash = null, analysis_state = 'pending', indexed_at = datetime('now') where id = ?1",
            params![id, size, mtime],
        );
    }
    for f in &found_new {
        // a missing file with the same size: same content fingerprint, or same name when it has none
        let mut moved_from: Option<(i64, String)> = None;
        let mut qh: Option<String> = None;
        let sized: Vec<(usize, &(&String, &Existing))> = gone.iter().enumerate().filter(|(_, (_, x))| x.size == f.size).collect();
        let db_missing = elsewhere.get(&f.size).map(Vec::as_slice).unwrap_or_default();
        if !sized.is_empty() || !db_missing.is_empty() {
            qh = quick_hash(&f.path);
            let by_hash = sized.iter().find(|(_, (_, x))| x.quick_hash.is_some() && x.quick_hash == qh).or_else(|| sized.iter().find(|(_, (_, x))| x.quick_hash.is_none() && x.name == f.name));
            if let Some((i, (rel, x))) = by_hash {
                moved_from = Some((x.id, (*rel).clone()));
                gone.remove(*i);
            } else if let Some(j) = db_missing.iter().position(|m| (m.2.is_some() && m.2 == qh) || (m.2.is_none() && m.3 == f.name)) {
                moved_from = Some((db_missing[j].0, db_missing[j].1.clone()));
                if let Some(v) = elsewhere.get_mut(&f.size) {
                    v.remove(j);
                }
            }
        }
        if let Some((id, old)) = moved_from {
            let _ = tx.execute(
                "update files set root_id = ?2, rel_path = ?3, name = ?4, ext = ?5, size = ?6, mtime = ?7, status = 'moved', moved_from = ?8, missing_since = null, quick_hash = coalesce(?9, quick_hash), indexed_at = datetime('now') where id = ?1",
                params![id, root_id, f.rel, f.name, f.ext, f.size, f.mtime, old, qh],
            );
            s.moved += 1;
        } else {
            let _ = tx
                .prepare_cached("insert into files(root_id, rel_path, name, ext, kind, size, mtime, status, quick_hash, analysis_state, indexed_at) values (?1, ?2, ?3, ?4, ?5, ?6, ?7, 'present', ?8, 'pending', datetime('now'))")
                .and_then(|mut st| st.execute(params![root_id, f.rel, f.name, f.ext, f.kind, f.size, f.mtime, qh]));
            s.new += 1;
        }
    }
    for (_, x) in &gone {
        let _ = tx.execute("update files set status = 'missing', missing_since = datetime('now') where id = ?1", [x.id]);
    }
    s.missing = gone.len();
    s.changed = changed.len();
    s.back = back.len();
    let _ = tx.commit();
    s.present = c.query_row("select count(*) from files where root_id = ?1 and status != 'missing'", [root_id], |r| r.get::<_, i64>(0)).unwrap_or(0) as usize;
    s.state = if cancelled { "cancelled" } else { "done" }.into();
    let _ = c.execute(
        "update library_roots set scan_state = ?2, file_count = ?3, last_scan_at = datetime('now'), error = ?4 where id = ?1",
        params![root_id, if cancelled { "idle" } else { "done" }, s.present as i64, if cancelled { Some("scan stopped by the user: missing files not checked") } else { None }],
    );
    s.duration_ms = t0.elapsed().as_millis() as u64;
    finish_job(&c, &s);
    s
}

/// Jobs left `running` by a previous session are marked `interrupted` (their roots get rescanned).
pub fn recover(c: &Connection) -> Vec<String> {
    let mut st = c.prepare("select distinct root_id from scan_jobs where state = 'running'").unwrap();
    let roots: Vec<String> = st.query_map([], |r| r.get(0)).map(|m| m.filter_map(Result::ok).collect()).unwrap_or_default();
    let _ = c.execute("update scan_jobs set state = 'interrupted', finished_at = datetime('now') where state = 'running'", []);
    let _ = c.execute("update library_roots set scan_state = 'idle' where scan_state = 'scanning'", []);
    roots
}
