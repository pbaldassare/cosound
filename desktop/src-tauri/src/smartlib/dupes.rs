//! Duplicates, in three clearly separated levels:
//! 1. `identical_files`: same full SHA-256 (byte-identical, any name or folder);
//! 2. `same_audio`: different files whose decoded audio is identical (e.g. the same sound saved as
//!    WAV and AIFF, or with different metadata);
//! 3. `similar_not_duplicates`: sounds whose timbre vectors are very close. Similarity alone is NOT
//!    a duplicate: these are only listed for review. Nothing is ever deleted, renamed or moved.
use super::features::similarity;
use crate::db::Db;
use rusqlite::{params, Connection};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::io::Read;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};

pub fn full_hash(path: &Path) -> Option<String> {
    let mut f = std::fs::File::open(path).ok()?;
    let mut h = Sha256::new();
    let mut buf = vec![0u8; 1 << 16];
    loop {
        let n = f.read(&mut buf).ok()?;
        if n == 0 {
            break;
        }
        h.update(&buf[..n]);
    }
    Some(h.finalize().iter().map(|b| format!("{b:02x}")).collect())
}

pub fn brief(c: &Connection, ids: &[i64]) -> Vec<Value> {
    let mut st = c
        .prepare_cached(
            "select f.id, f.name, f.rel_path, f.size, f.kind, f.status, v.mount_point, v.online, r.rel_path, v.label, f.duration_s, f.category
             from files f join library_roots r on r.id = f.root_id join volumes v on v.id = r.volume_id where f.id = ?1",
        )
        .unwrap();
    ids.iter()
        .filter_map(|id| {
            st.query_row([id], |r| {
                let (mount, online, root_rel, rel): (Option<String>, bool, String, String) = (r.get(6)?, r.get(7)?, r.get(8)?, r.get(2)?);
                Ok(json!({
                    "id": r.get::<_, i64>(0)?, "name": r.get::<_, String>(1)?, "rel_path": rel, "size": r.get::<_, i64>(3)?, "kind": r.get::<_, String>(4)?,
                    "status": r.get::<_, String>(5)?, "device": r.get::<_, String>(9)?, "device_online": online,
                    "path": mount.filter(|_| online).map(|m| Path::new(&m).join(&root_rel).join(&rel).to_string_lossy().to_string()),
                    "duration_s": r.get::<_, Option<f64>>(10)?, "category": r.get::<_, Option<String>>(11)?,
                }))
            })
            .ok()
        })
        .collect()
}

fn groups(c: &Connection, sql: &str) -> Vec<Vec<i64>> {
    let mut st = c.prepare(sql).unwrap();
    st.query_map([], |r| r.get::<_, String>(0))
        .map(|m| m.filter_map(Result::ok).map(|s| s.split(',').filter_map(|x| x.parse().ok()).collect()).collect())
        .unwrap_or_default()
}

pub fn find(db: &Db, cancel: &AtomicBool, include_similar: bool, threshold: f64) -> Value {
    // full hashes only where another file has the same size (cheap, lazy, stored for next time)
    let todo: Vec<(i64, PathBuf)> = {
        let c = db.lock().unwrap();
        let mut st = c
            .prepare(
                "select f.id, v.mount_point, r.rel_path, f.rel_path from files f join library_roots r on r.id = f.root_id join volumes v on v.id = r.volume_id
                 where f.full_hash is null and f.status != 'missing' and v.online = 1 and f.kind != 'project'
                   and f.size in (select size from files where status != 'missing' and kind != 'project' group by size having count(*) > 1)",
            )
            .unwrap();
        st.query_map([], |r| Ok((r.get(0)?, Path::new(&r.get::<_, String>(1)?).join(r.get::<_, String>(2)?).join(r.get::<_, String>(3)?))))
            .map(|m| m.filter_map(Result::ok).collect())
            .unwrap_or_default()
    };
    let mut hashed = 0;
    for (id, p) in todo {
        if cancel.load(Ordering::SeqCst) {
            break;
        }
        if let Some(h) = full_hash(&p) {
            let _ = db.lock().unwrap().execute("update files set full_hash = ?2 where id = ?1", params![id, h]);
            hashed += 1;
        }
    }
    let c = db.lock().unwrap();
    let identical = groups(&c, "select group_concat(id) from files where full_hash is not null and status != 'missing' group by full_hash having count(*) > 1");
    let same_audio: Vec<Vec<i64>> = groups(&c, "select group_concat(id) from files where audio_hash is not null and status != 'missing' group by audio_hash having count(*) > 1")
        .into_iter()
        .filter(|g| !identical.iter().any(|i| g.iter().all(|x| i.contains(x))))
        .collect();
    let mut similar = Vec::new();
    if include_similar {
        let mut st = c.prepare("select id, vec, duration_s, category from files where vec is not null and status != 'missing' and duration_s is not null order by duration_s limit 50000").unwrap();
        let rows: Vec<(i64, Vec<f32>, f64, Option<String>)> = st
            .query_map([], |r| Ok((r.get(0)?, serde_json::from_str(&r.get::<_, String>(1)?).unwrap_or_default(), r.get(2)?, r.get(3)?)))
            .map(|m| m.filter_map(Result::ok).collect())
            .unwrap_or_default();
        let dup = |a: i64, b: i64| identical.iter().chain(&same_audio).any(|g| g.contains(&a) && g.contains(&b));
        for i in 0..rows.len() {
            for j in i + 1..rows.len() {
                if rows[j].2 > rows[i].2 * 1.03 + 0.01 {
                    break; // sorted by duration: later rows are longer still
                }
                if rows[i].3 != rows[j].3 || dup(rows[i].0, rows[j].0) {
                    continue;
                }
                let s = similarity(&rows[i].1, &rows[j].1);
                if s >= threshold {
                    similar.push(json!({ "similarity": s, "files": brief(&c, &[rows[i].0, rows[j].0]) }));
                }
            }
            if similar.len() >= 200 {
                break;
            }
        }
    }
    json!({
        "identical_files": identical.iter().map(|g| json!({ "method": "full SHA-256", "files": brief(&c, g) })).collect::<Vec<_>>(),
        "same_audio": same_audio.iter().map(|g| json!({ "method": "SHA-256 of the decoded audio", "files": brief(&c, g) })).collect::<Vec<_>>(),
        "similar_not_duplicates": similar,
        "hashed_now": hashed,
        "note": "Nothing was deleted, renamed or moved. Similar sounds are not duplicates.",
    })
}
