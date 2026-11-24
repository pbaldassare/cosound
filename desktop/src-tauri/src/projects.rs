//! COSOUND projects and imported material. Projects hold references (paths + hashes), never copies of audio.
use crate::db::{log_op, new_id};
use crate::library::kind_of;
use rusqlite::{params, Connection, OptionalExtension};
use serde::{Deserialize, Serialize};
use serde_json::json;
use sha2::{Digest, Sha256};
use std::io::Read;
use std::path::Path;

type R<T> = Result<T, String>;
fn e(err: impl ToString) -> String {
    err.to_string()
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Project {
    pub id: String,
    pub name: String,
    pub daw: Option<String>,
    pub bpm: Option<f64>,
    pub musical_key: Option<String>,
    pub notes: String,
    pub created_at: String,
    pub updated_at: String,
    pub opened_at: Option<String>,
    pub archived_at: Option<String>,
}

const COLS: &str = "id, name, daw, bpm, musical_key, notes, created_at, updated_at, opened_at, archived_at";

fn row(r: &rusqlite::Row) -> rusqlite::Result<Project> {
    Ok(Project {
        id: r.get(0)?,
        name: r.get(1)?,
        daw: r.get(2)?,
        bpm: r.get(3)?,
        musical_key: r.get(4)?,
        notes: r.get(5)?,
        created_at: r.get(6)?,
        updated_at: r.get(7)?,
        opened_at: r.get(8)?,
        archived_at: r.get(9)?,
    })
}

pub fn get(c: &Connection, id: &str) -> R<Project> {
    c.query_row(&format!("select {COLS} from projects where id = ?1"), [id], row).optional().map_err(e)?.ok_or_else(|| "project not found".into())
}

/// Recent first (last opened, then last modified).
pub fn list(c: &Connection, include_archived: bool) -> R<Vec<Project>> {
    let mut st = c
        .prepare(&format!(
            "select {COLS} from projects where ?1 or archived_at is null order by coalesce(opened_at, updated_at) desc"
        ))
        .map_err(e)?;
    let rows = st.query_map([include_archived], row).map_err(e)?;
    rows.collect::<Result<_, _>>().map_err(e)
}

fn clean_name(name: &str) -> R<String> {
    let n = name.trim();
    if n.is_empty() || n.chars().count() > 120 {
        return Err("name must be 1–120 characters".into());
    }
    Ok(n.to_string())
}

fn check_bpm(bpm: Option<f64>) -> R<()> {
    match bpm {
        Some(b) if !(20.0..=400.0).contains(&b) => Err("BPM must be between 20 and 400".into()),
        _ => Ok(()),
    }
}

pub fn create(c: &Connection, name: &str, daw: Option<&str>, bpm: Option<f64>, key: Option<&str>) -> R<Project> {
    let name = clean_name(name)?;
    check_bpm(bpm)?;
    let id = new_id();
    c.execute(
        "insert into projects(id, name, daw, bpm, musical_key, opened_at) values (?1, ?2, ?3, ?4, ?5, datetime('now'))",
        params![id, name, daw, bpm, key],
    )
    .map_err(e)?;
    log_op(c, Some(&id), "project.create", "low", &json!({"name": name}), None).map_err(e)?;
    get(c, &id)
}

#[derive(Deserialize)]
pub struct ProjectFields {
    pub daw: Option<String>,
    pub bpm: Option<f64>,
    pub musical_key: Option<String>,
    pub notes: String,
}

/// Save the editable fields; old values go to the history.
pub fn save(c: &Connection, id: &str, f: &ProjectFields) -> R<Project> {
    check_bpm(f.bpm)?;
    let old = get(c, id)?;
    c.execute(
        "update projects set daw = ?2, bpm = ?3, musical_key = ?4, notes = ?5, updated_at = datetime('now') where id = ?1",
        params![id, f.daw, f.bpm, f.musical_key, f.notes],
    )
    .map_err(e)?;
    log_op(
        c,
        Some(id),
        "project.save",
        "low",
        &json!({"from": {"daw": old.daw, "bpm": old.bpm, "key": old.musical_key}, "to": {"daw": f.daw, "bpm": f.bpm, "key": f.musical_key}}),
        None,
    )
    .map_err(e)?;
    get(c, id)
}

pub fn open(c: &Connection, id: &str) -> R<Project> {
    c.execute("update projects set opened_at = datetime('now') where id = ?1", [id]).map_err(e)?;
    get(c, id)
}

pub fn rename(c: &Connection, id: &str, name: &str) -> R<Project> {
    let name = clean_name(name)?;
    let old = get(c, id)?;
    c.execute("update projects set name = ?2, updated_at = datetime('now') where id = ?1", params![id, name]).map_err(e)?;
    log_op(c, Some(id), "project.rename", "low", &json!({"from": old.name, "to": name}), Some(&json!({"kind": "project.set_name", "name": old.name})))
        .map_err(e)?;
    get(c, id)
}

pub fn set_archived(c: &Connection, id: &str, archived: bool) -> R<Project> {
    get(c, id)?;
    c.execute("update projects set archived_at = case when ?2 then datetime('now') else null end where id = ?1", params![id, archived])
        .map_err(e)?;
    log_op(
        c,
        Some(id),
        if archived { "project.archive" } else { "project.unarchive" },
        "low",
        &json!({}),
        Some(&json!({"kind": "project.set_archived", "archived": !archived})),
    )
    .map_err(e)?;
    get(c, id)
}

/// Duplicate = new project with the same settings and the same references (no file copies).
pub fn duplicate(c: &Connection, id: &str, copy_suffix: &str) -> R<Project> {
    let src = get(c, id)?;
    let new = create(c, &format!("{} {copy_suffix}", src.name), src.daw.as_deref(), src.bpm, src.musical_key.as_deref())?;
    c.execute("update projects set notes = ?2 where id = ?1", params![new.id, src.notes]).map_err(e)?;
    c.execute("insert into project_roots(project_id, root_id) select ?2, root_id from project_roots where project_id = ?1", params![id, new.id])
        .map_err(e)?;
    c.execute(
        "insert into imports(id, project_id, path, name, kind, role, provenance, size, sha256)
         select lower(hex(randomblob(16))), ?2, path, name, kind, role, provenance, size, sha256 from imports where project_id = ?1",
        params![id, new.id],
    )
    .map_err(e)?;
    log_op(c, Some(&new.id), "project.duplicate", "low", &json!({"from": src.name}), None).map_err(e)?;
    get(c, &new.id)
}

pub fn linked_roots(c: &Connection, id: &str) -> R<Vec<String>> {
    let mut st = c.prepare("select root_id from project_roots where project_id = ?1").map_err(e)?;
    let rows = st.query_map([id], |r| r.get(0)).map_err(e)?;
    rows.collect::<Result<_, _>>().map_err(e)
}

pub fn link_root(c: &Connection, id: &str, root_id: &str, linked: bool) -> R<()> {
    if linked {
        c.execute("insert or ignore into project_roots(project_id, root_id) values (?1, ?2)", params![id, root_id]).map_err(e)?;
    } else {
        c.execute("delete from project_roots where project_id = ?1 and root_id = ?2", params![id, root_id]).map_err(e)?;
    }
    log_op(c, Some(id), if linked { "project.link_library" } else { "project.unlink_library" }, "low", &json!({"root_id": root_id}), None).map_err(e)?;
    Ok(())
}

// ---------- imports ----------

#[derive(Serialize)]
pub struct Import {
    pub id: String,
    pub project_id: Option<String>,
    pub path: String,
    pub name: String,
    pub kind: String,
    pub role: String,
    pub provenance: String,
    pub size: i64,
    pub sha256: String,
    pub imported_at: String,
    pub exists: bool,
}

pub const PROVENANCES: &[&str] = &["own", "suno", "udio", "other_ai", "unknown"];
pub const ROLES: &[&str] = &["track", "stem", "loop", "sample", "voice", "midi", "project", "preset"];

fn sha256_file(p: &Path) -> R<String> {
    let mut f = std::fs::File::open(p).map_err(e)?;
    let mut h = Sha256::new();
    let mut buf = vec![0u8; 1 << 16];
    loop {
        let n = f.read(&mut buf).map_err(e)?;
        if n == 0 {
            break;
        }
        h.update(&buf[..n]);
    }
    Ok(h.finalize().iter().map(|b| format!("{b:02x}")).collect())
}

/// Register files as project material. Reads them once to hash; never modifies, copies or moves them.
pub fn import(c: &Connection, project_id: Option<&str>, paths: &[String], provenance: &str, role: Option<&str>) -> R<Vec<String>> {
    if !PROVENANCES.contains(&provenance) {
        return Err(format!("unknown provenance {provenance}"));
    }
    if role.is_some_and(|r| !ROLES.contains(&r)) {
        return Err("unknown role".into());
    }
    let mut ids = Vec::new();
    for raw in paths {
        let p = Path::new(raw);
        let meta = std::fs::metadata(p).map_err(|err| format!("{raw}: {err}"))?;
        let ext = p.extension().map(|x| x.to_string_lossy().to_ascii_lowercase()).unwrap_or_default();
        let kind = kind_of(&ext).unwrap_or("other");
        let default_role = match kind {
            "midi" => "midi",
            "project" => "project",
            "preset" => "preset",
            _ => "track",
        };
        let sha = if meta.is_dir() { String::new() } else { sha256_file(p)? }; // .logicx packages are folders
        let id = new_id();
        let n = c
            .execute(
                "insert or ignore into imports(id, project_id, path, name, kind, role, provenance, size, sha256)
                 values (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
                params![
                    id,
                    project_id,
                    raw,
                    p.file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_default(),
                    kind,
                    role.unwrap_or(default_role),
                    provenance,
                    meta.len() as i64,
                    sha
                ],
            )
            .map_err(e)?;
        if n == 1 {
            log_op(c, project_id, "import.add", "low", &json!({"path": raw, "provenance": provenance}), None).map_err(e)?;
            ids.push(id);
        }
    }
    Ok(ids)
}

pub fn imports(c: &Connection, project_id: Option<&str>) -> R<Vec<Import>> {
    let mut st = c
        .prepare(
            "select id, project_id, path, name, kind, role, provenance, size, sha256, imported_at from imports
             where ?1 is null or project_id = ?1 order by imported_at desc",
        )
        .map_err(e)?;
    let rows = st
        .query_map([project_id], |r| {
            let path: String = r.get(2)?;
            Ok(Import {
                id: r.get(0)?,
                project_id: r.get(1)?,
                exists: Path::new(&path).exists(),
                path,
                name: r.get(3)?,
                kind: r.get(4)?,
                role: r.get(5)?,
                provenance: r.get(6)?,
                size: r.get(7)?,
                sha256: r.get(8)?,
                imported_at: r.get(9)?,
            })
        })
        .map_err(e)?;
    rows.collect::<Result<_, _>>().map_err(e)
}

/// Remove the reference only; the file stays where it is.
pub fn remove_import(c: &Connection, id: &str) -> R<()> {
    let (pid, path): (Option<String>, String) =
        c.query_row("select project_id, path from imports where id = ?1", [id], |r| Ok((r.get(0)?, r.get(1)?))).map_err(e)?;
    c.execute("delete from imports where id = ?1", [id]).map_err(e)?;
    log_op(c, pid.as_deref(), "import.remove", "low", &json!({"path": path}), None).map_err(e)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn project_lifecycle_and_undo() {
        let c = crate::db::open_memory();
        let p = create(&c, "  Night Drive ", Some("logic"), Some(124.0), Some("A minor")).unwrap();
        assert_eq!(p.name, "Night Drive");
        assert!(create(&c, "x", None, Some(1000.0), None).is_err());
        assert!(create(&c, "   ", None, None, None).is_err());

        rename(&c, &p.id, "Night Drive v2").unwrap();
        set_archived(&c, &p.id, true).unwrap();
        assert!(list(&c, false).unwrap().is_empty());
        assert_eq!(list(&c, true).unwrap().len(), 1);

        // Undo archive, then undo rename, from the history.
        let ops = crate::db::ops(&c, Some(&p.id), 10).unwrap();
        crate::db::undo_op(&c, ops.iter().find(|o| o.op == "project.archive").unwrap().id).unwrap();
        crate::db::undo_op(&c, ops.iter().find(|o| o.op == "project.rename").unwrap().id).unwrap();
        let back = get(&c, &p.id).unwrap();
        assert_eq!(back.name, "Night Drive");
        assert!(back.archived_at.is_none());
        assert!(crate::db::undo_op(&c, ops.iter().find(|o| o.op == "project.create").unwrap().id).is_err());

        let saved = save(&c, &p.id, &ProjectFields { daw: Some("ableton".into()), bpm: Some(128.0), musical_key: None, notes: "n".into() }).unwrap();
        assert_eq!(saved.bpm, Some(128.0));
    }

    #[test]
    fn imports_reference_without_copying_and_duplicate() {
        let tmp = tempfile::tempdir().unwrap();
        let wav = tmp.path().join("suno_stem_vocals.wav");
        std::fs::write(&wav, b"abc").unwrap();
        let before = std::fs::read(&wav).unwrap();

        let c = crate::db::open_memory();
        let p = create(&c, "Song", None, None, None).unwrap();
        let paths = vec![wav.to_string_lossy().to_string()];
        assert_eq!(import(&c, Some(&p.id), &paths, "suno", Some("stem")).unwrap().len(), 1);
        assert_eq!(import(&c, Some(&p.id), &paths, "suno", Some("stem")).unwrap().len(), 0, "same file once per project");
        assert!(import(&c, Some(&p.id), &paths, "stolen", None).is_err());

        let list = imports(&c, Some(&p.id)).unwrap();
        assert_eq!(list[0].sha256, "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad");
        assert_eq!((list[0].kind.as_str(), list[0].role.as_str(), list[0].provenance.as_str()), ("audio", "stem", "suno"));
        assert_eq!(std::fs::read(&wav).unwrap(), before, "original untouched");
        assert_eq!(std::fs::read_dir(tmp.path()).unwrap().count(), 1, "no copies made");

        let d = duplicate(&c, &p.id, "(copy)").unwrap();
        assert_eq!(d.name, "Song (copy)");
        assert_eq!(imports(&c, Some(&d.id)).unwrap().len(), 1);

        remove_import(&c, &list[0].id).unwrap();
        assert!(wav.exists());
    }
}
