//! Smart Library v1: volumes, authorized folders, scan, search. Never copies, moves or writes user files.
use crate::db::{log_op, new_id};
#[cfg(test)]
use crate::db::Db;
use rusqlite::{params, Connection, OptionalExtension};
use serde::Serialize;
use std::path::{Path, PathBuf};

#[derive(Clone, Debug)]
pub struct Disk {
    pub label: String,
    pub mount: PathBuf,
    pub fs: String,
    pub total: u64,
    pub available: u64,
    pub removable: bool,
    pub disk_type: String,
}

const SKIP_FS: &[&str] = &["tmpfs", "devtmpfs", "overlay", "squashfs", "proc", "sysfs", "efivarfs", "autofs", "devfs", "nullfs", "fuse.snapfuse"];
const NET_FS: &[&str] = &["smbfs", "nfs", "nfs4", "cifs", "afpfs", "webdav", "smb2", "smb3"];

pub fn current_disks() -> Vec<Disk> {
    let Ok(disks) = sysinfo::Disks::new_with_refreshed_list() else { return Vec::new() };
    disks
        .iter()
        .filter_map(|d| {
            let fs = d.file_system().to_string_lossy().to_lowercase();
            let mount = d.mount_point().to_path_buf();
            let m = mount.to_string_lossy().to_string();
            let hidden = ["/System/Volumes/", "/boot", "/snap", "/private/var/vm", "/dev", "/proc", "/sys", "/run/"]
                .iter()
                .any(|p| m.starts_with(p) && !m.starts_with("/run/media"));
            if SKIP_FS.contains(&fs.as_str()) || hidden {
                return None;
            }
            let name = d.name().to_string_lossy().to_string();
            Some(Disk {
                label: if name.is_empty() { m.clone() } else { name },
                removable: d.is_removable() || m.starts_with("/Volumes/") || m.starts_with("/media/") || m.starts_with("/run/media/"),
                disk_type: match d.kind() {
                    sysinfo::DiskKind::SSD => "SSD",
                    sysinfo::DiskKind::HDD => "HDD",
                    _ => "unknown",
                }
                .into(),
                mount,
                fs,
                total: d.total_space(),
                available: d.available_space(),
            })
        })
        .collect()
}

/// Identity of a volume that survives a change of mount point / drive letter.
// ponytail: fs + size + label; two identical blank drives collide. Upgrade to volume UUID
// (macOS DiskArbitration, Windows GetVolumeInformationW serial) per specs/SMART-LIBRARY.md.
pub fn fingerprint(d: &Disk) -> String {
    format!("{}:{}:{}", d.fs, d.total, d.label)
}

fn kind_of_disk(d: &Disk) -> &'static str {
    if NET_FS.contains(&d.fs.as_str()) {
        "network"
    } else if d.removable {
        "external"
    } else {
        "internal"
    }
}

/// Sync the volumes table with the disks currently mounted: present → online (mount point updated), absent → offline.
pub fn refresh_volumes(c: &Connection, disks: &[Disk]) -> rusqlite::Result<()> {
    c.execute("update volumes set online = 0 where kind != 'folder'", [])?;
    for d in disks {
        c.execute(
            "insert into volumes(id, fingerprint, label, kind, disk_type, fs_type, mount_point, total_bytes, online, last_seen_at)
             values (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, 1, datetime('now'))
             on conflict(fingerprint) do update set label = excluded.label, kind = excluded.kind, mount_point = excluded.mount_point,
               online = 1, last_seen_at = excluded.last_seen_at",
            params![new_id(), fingerprint(d), d.label, kind_of_disk(d), d.disk_type, d.fs, d.mount.to_string_lossy(), d.total as i64],
        )?;
    }
    // Folders not on any detected disk (e.g. some network shares): online when the path exists.
    let mut st = c.prepare("select id, mount_point from volumes where kind = 'folder'")?;
    let folders: Vec<(String, String)> = st.query_map([], |r| Ok((r.get(0)?, r.get(1)?)))?.collect::<Result<_, _>>()?;
    for (id, mount) in folders {
        c.execute("update volumes set online = ?2 where id = ?1", params![id, Path::new(&mount).exists()])?;
    }
    Ok(())
}

#[derive(Serialize)]
pub struct VolumeView {
    pub id: String,
    pub label: String,
    pub kind: String,
    pub disk_type: Option<String>,
    pub fs_type: Option<String>,
    pub mount_point: Option<String>,
    pub online: bool,
    pub total_bytes: Option<i64>,
    pub available_bytes: Option<u64>,
    pub root_count: i64,
    pub file_count: i64,
    pub last_seen_at: Option<String>,
    /// The user excluded this device: never scanned or analyzed.
    pub excluded: bool,
}

/// Online volumes plus offline volumes that still hold authorized folders.
pub fn volumes(c: &Connection, disks: &[Disk]) -> rusqlite::Result<Vec<VolumeView>> {
    let mut st = c.prepare(
        "select v.id, v.label, v.kind, v.disk_type, v.fs_type, v.mount_point, v.online, v.total_bytes, v.last_seen_at, v.fingerprint,
           (select count(*) from library_roots r where r.volume_id = v.id),
           (select coalesce(sum(r.file_count), 0) from library_roots r where r.volume_id = v.id), v.excluded
         from volumes v
         where v.online = 1 or exists (select 1 from library_roots r where r.volume_id = v.id)
         order by v.online desc, v.kind, v.label",
    )?;
    let rows = st.query_map([], |r| {
        let fp: String = r.get(9)?;
        Ok(VolumeView {
            id: r.get(0)?,
            label: r.get(1)?,
            kind: r.get(2)?,
            disk_type: r.get(3)?,
            fs_type: r.get(4)?,
            mount_point: r.get(5)?,
            online: r.get(6)?,
            total_bytes: r.get(7)?,
            last_seen_at: r.get(8)?,
            available_bytes: disks.iter().find(|d| fingerprint(d) == fp).map(|d| d.available),
            root_count: r.get(10)?,
            file_count: r.get(11)?,
            excluded: r.get(12)?,
        })
    })?;
    rows.collect()
}

/// Strip the Windows verbatim prefix that `canonicalize` adds (`\\?\C:\…`).
fn plain(p: PathBuf) -> PathBuf {
    let s = p.to_string_lossy();
    match s.strip_prefix(r"\\?\") {
        Some(rest) if !rest.starts_with("UNC") => PathBuf::from(rest),
        _ => p,
    }
}

/// Authorize a folder. It is referenced (volume + relative path), never copied.
pub fn add_root(c: &Connection, disks: &[Disk], path: &str) -> Result<String, String> {
    let p = plain(std::fs::canonicalize(path).map_err(|e| format!("{path}: {e}"))?);
    if !p.is_dir() {
        return Err(format!("{path}: not a folder"));
    }
    refresh_volumes(c, disks).map_err(|e| e.to_string())?;
    let disk = disks.iter().filter(|d| p.starts_with(&d.mount)).max_by_key(|d| d.mount.as_os_str().len());
    let (volume_id, rel) = match disk {
        Some(d) => {
            let vid: String = c
                .query_row("select id from volumes where fingerprint = ?1", [fingerprint(d)], |r| r.get(0))
                .map_err(|e| e.to_string())?;
            (vid, p.strip_prefix(&d.mount).unwrap().to_string_lossy().to_string())
        }
        None => {
            let fp = format!("path:{}", p.to_string_lossy());
            c.execute(
                "insert into volumes(id, fingerprint, label, kind, mount_point, online, last_seen_at)
                 values (?1, ?2, ?3, 'folder', ?4, 1, datetime('now')) on conflict(fingerprint) do nothing",
                params![new_id(), fp, p.file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_default(), p.to_string_lossy()],
            )
            .map_err(|e| e.to_string())?;
            let vid = c.query_row("select id from volumes where fingerprint = ?1", [fp], |r| r.get(0)).map_err(|e| e.to_string())?;
            (vid, String::new())
        }
    };
    let id = new_id();
    c.execute("insert into library_roots(id, volume_id, rel_path) values (?1, ?2, ?3)", params![id, volume_id, rel])
        .map_err(|e| if e.to_string().contains("UNIQUE") { "folder already authorized".to_string() } else { e.to_string() })?;
    log_op(c, None, "library.root_add", "low", &serde_json::json!({"path": p.to_string_lossy()}), None).map_err(|e| e.to_string())?;
    Ok(id)
}

/// Revoke a folder: forgets its index. The files on disk are not touched.
pub fn remove_root(c: &Connection, id: &str) -> Result<(), String> {
    let path = root_path(c, id).map_err(|e| e.to_string())?.map(|p| p.to_string_lossy().to_string());
    let n = c.execute("delete from library_roots where id = ?1", [id]).map_err(|e| e.to_string())?;
    if n == 0 {
        return Err("folder not found".into());
    }
    log_op(c, None, "library.root_remove", "low", &serde_json::json!({"path": path}), None).map_err(|e| e.to_string())?;
    Ok(())
}

/// Current absolute path of a root, or None when its volume is offline.
pub fn root_path(c: &Connection, id: &str) -> rusqlite::Result<Option<PathBuf>> {
    let row: Option<(Option<String>, bool, String)> = c
        .query_row(
            "select v.mount_point, v.online, r.rel_path from library_roots r join volumes v on v.id = r.volume_id where r.id = ?1",
            [id],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
        )
        .optional()?;
    Ok(row.and_then(|(mount, online, rel)| {
        let p = Path::new(&mount?).join(rel);
        (online && p.exists()).then_some(p)
    }))
}

#[derive(Serialize)]
pub struct RootView {
    pub id: String,
    pub volume_id: String,
    pub volume_label: String,
    pub volume_kind: String,
    pub path: String,
    pub online: bool,
    pub scan_state: String,
    pub file_count: i64,
    pub last_scan_at: Option<String>,
    pub error: Option<String>,
}

pub fn roots(c: &Connection) -> rusqlite::Result<Vec<RootView>> {
    let mut st = c.prepare(
        "select r.id, r.volume_id, v.label, v.kind, v.mount_point, r.rel_path, r.scan_state, r.file_count, r.last_scan_at, r.error
         from library_roots r join volumes v on v.id = r.volume_id order by v.label, r.rel_path",
    )?;
    let rows: Vec<RootView> = st
        .query_map([], |r| {
            let mount: Option<String> = r.get(4)?;
            let rel: String = r.get(5)?;
            Ok(RootView {
                id: r.get(0)?,
                volume_id: r.get(1)?,
                volume_label: r.get(2)?,
                volume_kind: r.get(3)?,
                path: Path::new(&mount.unwrap_or_default()).join(rel).to_string_lossy().to_string(),
                online: false,
                scan_state: r.get(6)?,
                file_count: r.get(7)?,
                last_scan_at: r.get(8)?,
                error: r.get(9)?,
            })
        })?
        .collect::<Result<_, _>>()?;
    rows.into_iter()
        .map(|mut v| {
            v.online = root_path(c, &v.id)?.is_some();
            Ok(v)
        })
        .collect()
}

/// File kind by extension; None = not indexed.
pub fn kind_of(ext: &str) -> Option<&'static str> {
    Some(match ext.to_ascii_lowercase().as_str() {
        "wav" | "wave" | "aif" | "aiff" | "aifc" | "caf" | "mp3" | "m4a" | "aac" | "flac" | "ogg" => "audio",
        "mid" | "midi" => "midi",
        "logicx" | "als" | "flp" | "ptx" | "cpr" | "rpp" | "dawproject" => "project",
        "fxp" | "fxb" | "vstpreset" | "aupreset" | "nmsv" | "nksf" | "h2p" | "serumpreset" | "vital" | "adg" | "adv" | "pst" | "patch" => "preset",
        "nki" | "nkm" | "nkc" | "exs" | "sfz" | "sf2" => "instrument",
        _ => return None,
    })
}


/// Walk a root and replace its index. Hidden entries are skipped; package folders (.logicx) count as one project file.
/// Test shortcut over the Smart Library scanner (the app uses smartlib::service).
#[cfg(test)]
pub fn scan_root(db: &Db, id: &str, progress: &dyn Fn(usize)) -> Result<usize, String> {
    // Incremental since Prompt 07 (smartlib::scan): rows keep their ids, missing files are kept.
    let s = crate::smartlib::scan::scan_root(db, id, &std::sync::atomic::AtomicBool::new(false), progress);
    match s.state.as_str() {
        "done" => Ok(s.present),
        _ => Err(s.error.unwrap_or(s.state)),
    }
}

#[derive(Serialize)]
pub struct FileHit {
    pub id: i64,
    pub root_id: String,
    pub name: String,
    pub rel_path: String,
    pub kind: String,
    pub size: i64,
    pub path: Option<String>, // None when the drive is offline
    pub volume_label: String,
}

pub fn search(c: &Connection, q: &str, kind: Option<&str>, limit: i64) -> rusqlite::Result<Vec<FileHit>> {
    let pattern = format!("%{}%", q.replace('\\', "\\\\").replace('%', "\\%").replace('_', "\\_"));
    let mut st = c.prepare(
        "select f.id, f.root_id, f.name, f.rel_path, f.kind, f.size, v.mount_point, v.online, r.rel_path, v.label
         from files f join library_roots r on r.id = f.root_id join volumes v on v.id = r.volume_id
         where f.status != 'missing' and (f.name like ?1 escape '\\' or f.rel_path like ?1 escape '\\') and (?2 is null or f.kind = ?2)
         order by f.name limit ?3",
    )?;
    let rows = st.query_map(params![pattern, kind, limit], hit_row)?;
    rows.collect()
}

fn hit_row(r: &rusqlite::Row) -> rusqlite::Result<FileHit> {
    let mount: Option<String> = r.get(6)?;
    let online: bool = r.get(7)?;
    let root_rel: String = r.get(8)?;
    let rel: String = r.get(3)?;
    Ok(FileHit {
        id: r.get(0)?,
        root_id: r.get(1)?,
        name: r.get(2)?,
        kind: r.get(4)?,
        size: r.get(5)?,
        path: mount.filter(|_| online).map(|m| Path::new(&m).join(&root_rel).join(&rel).to_string_lossy().to_string()),
        rel_path: rel,
        volume_label: r.get(9)?,
    })
}

/// BPM and key written in a file name ("Bass_124_Am.wav", "pad 128bpm F#min"). Read from the name
/// only: nothing is measured, so callers must label these values as coming from the file name.
pub fn filename_meta(name: &str) -> (Option<u32>, Option<String>) {
    let stem = name.rsplit_once('.').map_or(name, |s| s.0);
    let toks: Vec<&str> = stem.split(|c: char| !(c.is_ascii_alphanumeric() || c == '#')).filter(|t| !t.is_empty()).collect();
    let mut bpm = None;
    for (i, t) in toks.iter().enumerate() {
        let l = t.to_ascii_lowercase();
        let num = |s: &str| s.parse::<u32>().ok().filter(|b| (60..=200).contains(b));
        if let Some(b) = l.strip_suffix("bpm").and_then(num).or_else(|| l.strip_prefix("bpm").and_then(num)) {
            bpm = Some(b);
            break;
        }
        if l == "bpm" {
            if let Some(b) = i.checked_sub(1).and_then(|j| num(toks[j])).or_else(|| toks.get(i + 1).and_then(|t| num(t))) {
                bpm = Some(b);
                break;
            }
        }
        if bpm.is_none() && (2..=3).contains(&t.len()) && t.chars().all(|c| c.is_ascii_digit()) {
            bpm = t.parse::<u32>().ok().filter(|b| (70..=180).contains(b));
        }
    }
    let mut key = None;
    for (i, t) in toks.iter().enumerate() {
        let mut ch = t.chars();
        let Some(letter) = ch.next().filter(|c| ('A'..='G').contains(c)) else { continue };
        let rest: String = ch.collect();
        let (acc, mode) = match rest.strip_prefix('#') {
            Some(m) => ("#", m.to_string()),
            None => match rest.strip_prefix('b') {
                Some(m) if !m.is_empty() || toks.get(i + 1).is_some_and(|n| matches!(n.to_ascii_lowercase().as_str(), "minor" | "major" | "min" | "maj")) => ("b", m.to_string()),
                _ => ("", rest.clone()),
            },
        };
        let next = toks.get(i + 1).map(|n| n.to_ascii_lowercase()).unwrap_or_default();
        let m = match (mode.as_str(), next.as_str()) {
            ("m" | "min" | "minor", _) | ("", "minor" | "min") => "minor",
            ("maj" | "major", _) | ("", "major" | "maj") => "major",
            _ => continue,
        };
        key = Some(format!("{letter}{acc} {m}"));
        break;
    }
    (bpm, key)
}

pub fn stats(c: &Connection) -> rusqlite::Result<serde_json::Value> {
    let mut st = c.prepare("select kind, count(*) from files group by kind")?;
    let mut m = serde_json::Map::new();
    for row in st.query_map([], |r| Ok((r.get::<_, String>(0)?, r.get::<_, i64>(1)?)))? {
        let (k, n) = row?;
        m.insert(k, n.into());
    }
    let roots: i64 = c.query_row("select count(*) from library_roots", [], |r| r.get(0))?;
    Ok(serde_json::json!({"roots": roots, "files_by_kind": m}))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn disk(label: &str, mount: &Path) -> Disk {
        Disk { label: label.into(), mount: mount.into(), fs: "exfat".into(), total: 1000, available: 400, removable: true, disk_type: "SSD".into() }
    }

    #[test]
    fn scan_search_unplug_replug() {
        let tmp = tempfile::tempdir().unwrap();
        // canonicalize: on macOS the temp dir lives under /var, a symlink to /private/var, and add_root canonicalizes paths
        let base = tmp.path().canonicalize().unwrap();
        let mount_a = base.join("mountA");
        let lib = mount_a.join("Samples");
        fs::create_dir_all(lib.join("Drums/.hidden")).unwrap();
        fs::create_dir_all(lib.join("Song.logicx/Alternatives")).unwrap();
        fs::write(lib.join("Drums/Kick 01.wav"), b"RIFF").unwrap();
        fs::write(lib.join("Drums/.hidden/x.wav"), b"").unwrap();
        fs::write(lib.join("bass_line.mid"), b"MThd").unwrap();
        fs::write(lib.join("notes.txt"), b"").unwrap();
        fs::write(lib.join("Song.logicx/Alternatives/inner.wav"), b"").unwrap();

        let db: Db = std::sync::Mutex::new(crate::db::open_memory());
        let disks = vec![disk("SSD-EXT", &mount_a)];
        let root = add_root(&db.lock().unwrap(), &disks, lib.to_str().unwrap()).unwrap();
        assert!(add_root(&db.lock().unwrap(), &disks, lib.to_str().unwrap()).is_err(), "duplicate root rejected");

        assert_eq!(scan_root(&db, &root, &|_| {}).unwrap(), 3); // kick, midi, logicx package
        let hits = search(&db.lock().unwrap(), "kick", None, 50).unwrap();
        assert_eq!(hits.len(), 1);
        assert!(hits[0].path.as_ref().unwrap().ends_with("Kick 01.wav"));
        assert_eq!(search(&db.lock().unwrap(), "", Some("midi"), 50).unwrap().len(), 1);

        // Unplug: volume offline, records kept, scan refused, paths hidden.
        refresh_volumes(&db.lock().unwrap(), &[]).unwrap();
        assert!(scan_root(&db, &root, &|_| {}).is_err());
        let hits = search(&db.lock().unwrap(), "kick", None, 50).unwrap();
        assert_eq!(hits.len(), 1);
        assert!(hits[0].path.is_none());
        assert!(!roots(&db.lock().unwrap()).unwrap()[0].online);

        // Replug at a different mount point: recognized by fingerprint, paths remapped.
        let mount_b = base.join("mountB");
        fs::rename(&mount_a, &mount_b).unwrap();
        let disks_b = vec![disk("SSD-EXT", &mount_b)];
        refresh_volumes(&db.lock().unwrap(), &disks_b).unwrap();
        let hits = search(&db.lock().unwrap(), "kick", None, 50).unwrap();
        assert!(hits[0].path.as_ref().unwrap().starts_with(mount_b.to_str().unwrap()));
        let vols = volumes(&db.lock().unwrap(), &disks_b).unwrap();
        assert_eq!(vols.len(), 1);
        assert_eq!(vols[0].available_bytes, Some(400));

        remove_root(&db.lock().unwrap(), &root).unwrap();
        assert!(mount_b.join("Samples/Drums/Kick 01.wav").exists(), "files untouched");
        assert_eq!(search(&db.lock().unwrap(), "kick", None, 50).unwrap().len(), 0);
    }

    #[test]
    fn filename_metadata() {
        for (n, bpm, key) in [
            ("Bass_124_Am.wav", Some(124), Some("A minor")),
            ("pad 128bpm F#min.wav", Some(128), Some("F# minor")),
            ("Lead Eb major 126 BPM.mid", Some(126), Some("Eb major")),
            ("Kick 01.wav", None, None),
            ("TR-909 Clap.wav", None, None),
            ("Dream am loop.wav", None, None),
            ("Bb minor chords.mid", None, Some("Bb minor")),
        ] {
            let (b, k) = filename_meta(n);
            assert_eq!((b, k.as_deref()), (bpm, key), "{n}");
        }
    }
}
