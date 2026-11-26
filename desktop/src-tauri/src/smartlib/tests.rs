//! End-to-end tests of the Smart Library System on a sample library written to a temporary "drive".
use super::decode::write_test_aiff;
use super::features::fixtures::*;
use super::presets::fixtures as pf;
use super::*;
use crate::audio::wav::write_test_wav;
use crate::db::{self, Db};
use crate::library::{self, Disk};
use crate::music::{generate::*, smf, theory::{progression, Key}};
use serde_json::Value;
use std::path::{Path, PathBuf};
use std::sync::atomic::AtomicBool;
use std::sync::Arc;

pub struct Lib {
    pub _tmp: tempfile::TempDir,
    pub base: PathBuf,
    pub mount: PathBuf,
    pub root: PathBuf,
    pub db: Arc<Db>,
    pub root_id: String,
}

fn disk(mount: &Path) -> Disk {
    Disk { label: "TEST-SSD".into(), mount: mount.into(), fs: "exfat".into(), total: 64_000_000_000, available: 1_000_000, removable: true, disk_type: "SSD".into() }
}

fn wav(p: &Path, s: &[f64], ch: u16) {
    std::fs::create_dir_all(p.parent().unwrap()).unwrap();
    write_test_wav(p, 44100, ch, 16, false, s);
}

fn mid(p: &Path, part: Part, bpm: f64, key: &str) {
    std::fs::create_dir_all(p.parent().unwrap()).unwrap();
    std::fs::write(p, smf::write(&[part], bpm, Key::parse(key).unwrap(), "t")).unwrap();
}

/// The sample library. Returns it scanned and analyzed.
pub fn lib() -> Lib {
    let tmp = tempfile::tempdir().unwrap();
    let base = crate::library::plain(tmp.path().canonicalize().unwrap());
    let mount = base.join("TEST-SSD");
    let root = mount.join("Samples");
    let k = kick(44100);
    wav(&root.join("Drums/Kicks/Kick_Techno_Deep.wav"), &k, 1);
    wav(&root.join("Drums/Hats/Hat 01.wav"), &hat(44100), 1);
    wav(&root.join("Loops/Groove 01.wav"), &drum_loop(44100, 124.0, 2), 1);
    wav(&root.join("Loops/Arp Loop Am.wav"), &arp_loop(44100, 124.0, 2, &[57, 60, 64, 69, 72, 76, 74, 71]), 1);
    wav(&root.join("Bass/Rolling Bass.wav"), &arp_loop(44100, 124.0, 2, &[33, 33, 45, 33, 40, 33, 45, 36]), 1);
    std::fs::create_dir_all(root.join("Pads")).unwrap();
    write_test_aiff(&root.join("Pads/Atmos Pad.aif"), 44100, 2, &pad(44100, 6.0, &[57, 60, 64]));
    wav(&root.join("Copies/BD new name.wav"), &k, 1);
    write_test_aiff(&root.join("Copies/Kick as AIFF.aif"), 44100, 1, &k);
    wav(&root.join("Private/secret.wav"), &k, 1);
    std::fs::write(root.join("notes.txt"), "not a music file").unwrap();
    wav(&base.join("outside.wav"), &k, 1);
    let key = Key::parse("G major").unwrap();
    let ch = progression(key, "vi-IV-I-V", false).unwrap();
    let spec = Spec { key, chords: &ch, bars: 4, seed: 1, swing: 0 };
    mid(&root.join("MIDI/Bassline Prog House.mid"), bass(&spec, "pulse").unwrap(), 126.0, "G major");
    mid(&root.join("MIDI/Chords.mid"), chords(&spec, "sustain").unwrap(), 126.0, "G major");
    let arp: Vec<MidiNote> = (0..64u32).map(|i| MidiNote { pitch: [57, 60, 64, 69, 64, 60][i as usize % 6], start: i * 120, dur: 100, vel: 100 }).collect();
    mid(&root.join("MIDI/Arp minor.mid"), Part { id: "melody", name: "Arp", channel: 0, program: 81, notes: arp }, 124.0, "A minor");
    std::fs::create_dir_all(root.join("Presets/Massive/Pads")).unwrap();
    std::fs::create_dir_all(root.join("Presets/Vital")).unwrap();
    pf::fxp(&root.join("Presets/Massive/Pads/Atmos.fxp"), b"Mssv", "Atmosphere Wide");
    pf::vital(&root.join("Presets/Vital/Deep Pad.vital"), "Deep Pad", "Pad");
    pf::vstpreset(&root.join("Presets/Serum Lead.vstpreset"), "Serum", "Lead");
    let db: Arc<Db> = Arc::new(std::sync::Mutex::new(db::open_memory()));
    let root_id = {
        let c = db.lock().unwrap();
        library::add_root(&c, &[disk(&mount)], root.to_str().unwrap()).unwrap()
    };
    index::set_exclusion(&db.lock().unwrap(), &root_id, "Private", true).unwrap();
    let l = Lib { _tmp: tmp, base, mount, root, db, root_id };
    let s = scan::scan_root(&l.db, &l.root_id, &AtomicBool::new(false), &|_| {});
    assert_eq!(s.state, "done", "{s:?}");
    analyze_all(&l.db);
    l
}

pub fn analyze_all(db: &Db) {
    for _ in 0..100 {
        if analyze::run_batch(db, 50, &AtomicBool::new(false), 0) == 0 {
            return;
        }
    }
    panic!("analysis did not converge");
}

fn id(c: &rusqlite::Connection, name: &str) -> i64 {
    c.query_row("select id from files where name = ?1", [name], |r| r.get(0)).unwrap()
}

fn search(l: &Lib, text: &str) -> Value {
    api::search_text(&l.db.lock().unwrap(), text, 20, None)
}

fn names(v: &Value) -> Vec<String> {
    v["results"].as_array().unwrap().iter().map(|r| r["name"].as_str().unwrap().to_string()).collect()
}

#[test]
fn scans_indexes_and_analyzes_every_format() {
    let l = lib();
    let c = l.db.lock().unwrap();
    let count = |sql: &str| c.query_row(sql, [], |r| r.get::<_, i64>(0)).unwrap();
    assert_eq!(count("select count(*) from files where kind = 'audio'"), 8, "Private/ is excluded, notes.txt ignored");
    assert_eq!(count("select count(*) from files where kind = 'midi'"), 3);
    assert_eq!(count("select count(*) from files where kind = 'preset'"), 3);
    assert_eq!(count("select count(*) from files where name = 'secret.wav' or name = 'outside.wav'"), 0, "excluded folder and files outside the root are never indexed");
    assert_eq!(count("select count(*) from files where analysis_state != 'done'"), 0, "{:?}", c.query_row("select group_concat(name || ': ' || coalesce(analysis_error, '')) from files where analysis_state != 'done'", [], |r| r.get::<_, Option<String>>(0)).unwrap());
    let d = search::detail(&c, id(&c, "Atmos Pad.aif")).unwrap();
    assert_eq!((d["sample_rate"].as_i64(), d["channels"].as_i64(), d["bit_depth"].as_i64()), (Some(44100), Some(2), Some(16)), "AIFF read: {}", d["format"]);
    assert!((d["duration_s"].as_f64().unwrap() - 6.0).abs() < 0.01);
    assert!(d["lufs"].as_f64().is_some() && d["details"]["measurements"]["true_peak_dbtp"].is_number());
    assert_eq!(d["category"], "pad");
}

#[test]
fn estimates_tempo_key_and_categories_with_sources() {
    let l = lib();
    let c = l.db.lock().unwrap();
    let get = |n: &str| search::detail(&c, id(&c, n)).unwrap();
    let g = get("Groove 01.wav");
    assert!((g["bpm"]["value"].as_f64().unwrap() - 124.0).abs() < 0.3, "{}", g["bpm"]);
    assert_eq!((g["bpm"]["source"].as_str(), g["loop_bars"].as_f64(), g["is_loop"].as_bool()), (Some("analysis+loop_length"), Some(2.0), Some(true)));
    // a drum loop has no key to read: the kick's pitch dominates, so the key stays unknown
    assert!(g["key"]["value"].is_null(), "drum loop key: {}", g["key"]);
    assert_eq!(g["category"], "drums");
    let a = get("Arp Loop Am.wav");
    assert_eq!(a["key"]["value"], "A minor", "{}", a["key"]);
    let k = get("Kick_Techno_Deep.wav");
    assert_eq!(k["category"], "kick");
    assert!(k["bpm"]["value"].is_null(), "a one-shot has no tempo, it is not invented");
    assert!(k["types"].as_array().unwrap().iter().any(|t| t["tag"] == "one_shot"));
    assert!(k["genres"].as_array().unwrap().iter().any(|t| t["tag"] == "techno"));
    let h = get("Hat 01.wav");
    assert!(h["categories"].as_array().unwrap().iter().any(|t| t["tag"] == "hihat"));
    assert!(h["key"]["value"].is_null());
    let p = get("Atmos Pad.aif");
    assert!(p["descriptors"].as_array().unwrap().iter().any(|t| t["tag"] == "atmospheric" && t["source"] == "analysis"), "{}", p["descriptors"]);
    let m = get("Bassline Prog House.mid");
    assert_eq!((m["category"].as_str(), m["bpm"]["value"].as_f64(), m["bpm"]["source"].as_str()), (Some("bass"), Some(126.0), Some("embedded")));
    assert!(m["genres"].as_array().unwrap().iter().any(|t| t["tag"] == "progressive_house"));
    assert_eq!(get("Arp minor.mid")["category"], "arpeggio");
    let fx = get("Atmos.fxp");
    assert!(fx["plugins"].as_array().unwrap().iter().any(|t| t["tag"] == "massive" && t["source"] == "folder"), "{}", fx["plugins"]);
}

#[test]
fn natural_language_examples_of_the_prompt() {
    let l = lib();
    let r = search(&l, "Trova un kick profondo e potente adatto a una traccia Melodic Techno a 124 BPM.");
    assert!(r["results"][0]["categories"].as_array().unwrap().iter().any(|t| t["tag"] == "kick"), "{:?}", names(&r));
    assert!(names(&r).iter().take(3).all(|n| n.to_lowercase().contains("kick") || n.contains("BD")), "{:?}", names(&r));
    let r = search(&l, "Cerca loop a 124 BPM");
    let top: Vec<String> = names(&r).into_iter().take(4).collect();
    assert!(top.contains(&"Groove 01.wav".to_string()) && top.contains(&"Arp Loop Am.wav".to_string()), "{top:?}");
    let r = search(&l, "Voglio un pad atmosferico e profondo.");
    assert!(names(&r).iter().take(2).any(|n| n == "Atmos Pad.aif"), "{:?}", names(&r));
    let r = search(&l, "Trova un arpeggio in tonalità minore.");
    assert!(names(&r).iter().take(2).any(|n| n == "Arp Loop Am.wav" || n == "Arp minor.mid"), "{:?}", names(&r));
    let r = search(&l, "Trova un MIDI di arpeggio melodico in tonalità minore.");
    assert_eq!(names(&r)[0], "Arp minor.mid");
    assert!(r["results"].as_array().unwrap().iter().all(|x| x["kind"] == "midi"));
    let r = search(&l, "Mostrami delle bassline MIDI adatte alla Progressive House.");
    assert_eq!(names(&r)[0], "Bassline Prog House.mid");
    let r = search(&l, "Trova un preset di pad atmosferico per Massive.");
    assert_eq!(names(&r), ["Atmos.fxp"], "only presets actually found for that plugin");
    let r = search(&l, "Trova un preset di pad per Massive X");
    assert!(names(&r).is_empty(), "Massive presets are not offered for Massive X: {:?}", names(&r));
    let r = search(&l, "Trova un preset per Sylenth1");
    assert!(names(&r).is_empty(), "no preset is claimed for a plugin that has none");
    let r = search(&l, "Trova un basso caldo, profondo, rotondo e aggressivo.");
    assert_eq!(names(&r)[0], "Rolling Bass.wav", "{:?}", names(&r));
    let r = search(&l, "Mostrami i synth adatti a una produzione Progressive House.");
    assert!(!names(&r).is_empty());
    assert!(r["understood"]["genres"][0] == "progressive_house");
    assert!(r["results"][0]["why"].as_array().is_some());
}

#[test]
fn similarity_loops_and_duplicates() {
    let l = lib();
    let kick = id(&l.db.lock().unwrap(), "Kick_Techno_Deep.wav");
    let s = api::similar(&l.db.lock().unwrap(), kick, 5).unwrap();
    let top: Vec<String> = s["results"].as_array().unwrap().iter().take(2).map(|r| r["name"].as_str().unwrap().to_string()).collect();
    assert!(top.contains(&"BD new name.wav".into()) && top.contains(&"Kick as AIFF.aif".into()), "{top:?}");
    assert!(s["results"][0]["similarity"].as_f64().unwrap() >= 95.0);
    assert!(s["note"].as_str().unwrap().contains("not a guarantee"));
    let lp = api::loops(&l.db.lock().unwrap(), 126.0, Some("A minor"), None, 10);
    let arp = lp["results"].as_array().unwrap().iter().find(|r| r["name"] == "Arp Loop Am.wav").expect("the 124 BPM loop fits 126 BPM");
    assert!((arp["fit"]["stretch_percent"].as_f64().unwrap() - 1.61).abs() < 0.05, "{}", arp["fit"]);
    assert_eq!(arp["fit"]["key"], "same key");
    let d = api::duplicates(&l.db, &AtomicBool::new(false), true);
    let ident = &d["identical_files"][0]["files"];
    let group: Vec<&str> = ident.as_array().unwrap().iter().map(|f| f["name"].as_str().unwrap()).collect();
    assert!(group.contains(&"Kick_Techno_Deep.wav") && group.contains(&"BD new name.wav"), "{d}");
    let same: Vec<&str> = d["same_audio"][0]["files"].as_array().unwrap().iter().map(|f| f["name"].as_str().unwrap()).collect();
    assert!(same.contains(&"Kick as AIFF.aif"), "same audio in another container: {same:?}");
    assert!(std::fs::metadata(l.root.join("Copies/BD new name.wav")).is_ok(), "nothing deleted");
}

#[test]
fn incremental_updates_and_device_states() {
    let l = lib();
    let rescan = || scan::scan_root(&l.db, &l.root_id, &AtomicBool::new(false), &|_| {});
    let s = rescan();
    assert_eq!((s.new, s.changed, s.moved, s.missing), (0, 0, 0, 0), "unchanged library: nothing to do");
    let hat_id = id(&l.db.lock().unwrap(), "Hat 01.wav");
    // user corrections
    {
        let c = l.db.lock().unwrap();
        index::correct(&c, hat_id, "bpm", "128").unwrap();
        index::correct(&c, hat_id, "add", "category:percussion").unwrap();
        index::correct(&c, hat_id, "remove", "category:hihat").unwrap();
        assert!(index::correct(&c, hat_id, "key", "H major").is_err());
    }
    // modified file: re-analyzed, corrections kept
    std::thread::sleep(std::time::Duration::from_millis(1100));
    wav(&l.root.join("Drums/Hats/Hat 01.wav"), &hat(44100).iter().chain(hat(44100).iter()).cloned().collect::<Vec<_>>(), 1);
    let s = rescan();
    assert_eq!(s.changed, 1, "{s:?}");
    assert_eq!(l.db.lock().unwrap().query_row("select status from files where id = ?1", [hat_id], |r| r.get::<_, String>(0)).unwrap(), "modified");
    analyze_all(&l.db);
    let d = search::detail(&l.db.lock().unwrap(), hat_id).unwrap();
    assert_eq!((d["status"].as_str(), d["bpm"]["value"].as_f64(), d["bpm"]["source"].as_str(), d["category"].as_str()), (Some("present"), Some(128.0), Some("user"), Some("percussion")));
    assert!(!d["categories"].as_array().unwrap().iter().any(|t| t["tag"] == "hihat"), "removed label stays removed");
    // moved file keeps its id and analysis
    let groove = id(&l.db.lock().unwrap(), "Groove 01.wav");
    std::fs::create_dir_all(l.root.join("Loops/Moved")).unwrap();
    std::fs::rename(l.root.join("Loops/Groove 01.wav"), l.root.join("Loops/Moved/Groove 01.wav")).unwrap();
    let s = rescan();
    assert_eq!((s.moved, s.new, s.missing), (1, 0, 0), "{s:?}");
    let d = search::detail(&l.db.lock().unwrap(), groove).unwrap();
    assert_eq!((d["status"].as_str(), d["rel_path"].as_str(), d["moved_from"].as_str()), (Some("moved"), Some("Loops/Moved/Groove 01.wav"), Some("Loops/Groove 01.wav")));
    assert!(d["bpm"]["value"].as_f64().is_some(), "analysis kept");
    // deleted file: kept as missing, hidden from search
    std::fs::remove_file(l.root.join("Copies/BD new name.wav")).unwrap();
    let s = rescan();
    assert_eq!(s.missing, 1);
    assert!(!names(&search(&l, "BD new name")).contains(&"BD new name.wav".into()));
    assert_eq!(l.db.lock().unwrap().query_row("select status from files where name = 'BD new name.wav'", [], |r| r.get::<_, String>(0)).unwrap(), "missing");
    // new file
    wav(&l.root.join("Loops/New Perc.wav"), &hat(44100), 1);
    assert_eq!(rescan().new, 1);
    // drive unplugged: nothing becomes missing, results stay findable without a path
    library::refresh_volumes(&l.db.lock().unwrap(), &[]).unwrap();
    let s = rescan();
    assert_eq!((s.state.as_str(), s.missing), ("offline", 0), "{s:?}");
    let r = search(&l, "kick");
    assert!(!r["results"].as_array().unwrap().is_empty() && r["results"].as_array().unwrap().iter().all(|x| x["path"].is_null() && x["device_online"] == false));
    assert_eq!(l.db.lock().unwrap().query_row("select count(*) from files where status = 'missing'", [], |r| r.get::<_, i64>(0)).unwrap(), 1, "still only the deleted file");
    // replugged at another mount point: recognized, paths restored, no duplicate rows
    let before: i64 = l.db.lock().unwrap().query_row("select count(*) from files", [], |r| r.get(0)).unwrap();
    let mount2 = l.base.join("TEST-SSD-2");
    std::fs::rename(&l.mount, &mount2).unwrap();
    library::refresh_volumes(&l.db.lock().unwrap(), &[disk(&mount2)]).unwrap();
    let s = rescan();
    assert_eq!((s.state.as_str(), s.new, s.missing), ("done", 0, 0), "{s:?}");
    assert_eq!(l.db.lock().unwrap().query_row("select count(*) from files", [], |r| r.get::<_, i64>(0)).unwrap(), before);
    let r = search(&l, "kick");
    assert!(r["results"][0]["path"].as_str().unwrap().starts_with(mount2.to_str().unwrap()));
    // folder renamed on a connected drive: "unavailable", files not missing
    std::fs::rename(mount2.join("Samples"), mount2.join("Samples renamed")).unwrap();
    let s = rescan();
    assert_eq!((s.state.as_str(), s.missing), ("unavailable", 0));
    assert!(s.error.unwrap().contains("not available"));
    // a cancelled scan never marks files missing
    std::fs::rename(mount2.join("Samples renamed"), mount2.join("Samples")).unwrap();
    let s = scan::scan_root(&l.db, &l.root_id, &AtomicBool::new(true), &|_| {});
    assert_eq!((s.state.as_str(), s.missing), ("cancelled", 0));
}

#[test]
fn resolves_original_paths_for_the_daw_controller() {
    let l = lib();
    let ids = {
        let c = l.db.lock().unwrap();
        [id(&c, "Arp Loop Am.wav"), id(&c, "Arp minor.mid"), 999_999]
    };
    let r = api::resolve(&l.db.lock().unwrap(), &ids);
    assert_eq!(r[0]["path"].as_str().unwrap(), l.root.join("Loops").join("Arp Loop Am.wav").to_str().unwrap());
    assert_eq!(r[1]["usable"], true);
    assert!(r[2]["error"].is_string());
    library::refresh_volumes(&l.db.lock().unwrap(), &[]).unwrap();
    let r = api::resolve(&l.db.lock().unwrap(), &ids[..1]);
    assert_eq!((r[0]["path"].is_null(), r[0]["usable"].as_bool()), (true, Some(false)), "no path while the drive is unplugged");
}

/// Large-library check (run with `cargo test --release -- --ignored large_library --nocapture`).
#[test]
fn originals_never_change_and_bad_files_do_not_stop_the_rest() {
    use sha2::{Digest, Sha256};
    let l = lib();
    std::fs::write(l.root.join("Drums/Kicks/broken.wav"), b"RIFF\x10\x00\x00\x00WAVEnot really audio").unwrap();
    let digest = |root: &Path| -> Vec<(String, String)> {
        let mut v: Vec<(String, String)> = walkdir::WalkDir::new(root).into_iter().filter_map(Result::ok).filter(|e| e.file_type().is_file())
            .map(|e| (e.path().display().to_string(), Sha256::digest(std::fs::read(e.path()).unwrap()).iter().map(|b| format!("{b:02x}")).collect()))
            .collect();
        v.sort();
        v
    };
    let before = digest(&l.base);
    scan::scan_root(&l.db, &l.root_id, &AtomicBool::new(false), &|_| {});
    analyze_all(&l.db);
    let _ = api::duplicates(&l.db, &AtomicBool::new(false), true);
    {
        let c = l.db.lock().unwrap();
        let kick = id(&c, "Kick_Techno_Deep.wav");
        index::correct(&c, kick, "bpm", "126").unwrap();
        let _ = api::similar(&c, kick, 10);
        let _ = api::loops(&c, 124.0, None, None, 10);
        let _ = api::search_text(&c, "kick", 10, None);
        let (state, err): (String, Option<String>) = c.query_row("select analysis_state, analysis_error from files where name = 'broken.wav'", [], |r| Ok((r.get(0)?, r.get(1)?))).unwrap();
        assert_eq!(state, "error", "an unreadable file is marked, not hidden");
        assert!(err.is_some_and(|e| !e.is_empty()));
        let bad: i64 = c.query_row("select count(*) from files where analysis_state != 'done' and name != 'broken.wav'", [], |r| r.get(0)).unwrap();
        assert_eq!(bad, 0, "the other files are analyzed");
    }
    assert_eq!(before, digest(&l.base), "scan, analysis, duplicates, corrections and searches never modify the user's files");
}

#[test]
fn interrupted_scan_is_resumed() {
    let l = lib();
    // the app stopped in the middle of a scan: the job is still "running" when it starts again
    {
        let c = l.db.lock().unwrap();
        c.execute("insert into scan_jobs(id, root_id, kind, state) values ('j-crash', ?1, 'incremental', 'running')", [&l.root_id]).unwrap();
        c.execute("update library_roots set scan_state = 'scanning' where id = ?1", [&l.root_id]).unwrap();
    }
    let todo = scan::recover(&l.db.lock().unwrap());
    assert_eq!(todo, vec![l.root_id.clone()], "the folder is queued again");
    let (job, root): (String, String) = l.db.lock().unwrap().query_row(
        "select (select state from scan_jobs where id = 'j-crash'), (select scan_state from library_roots where id = ?1)", [&l.root_id], |r| Ok((r.get(0)?, r.get(1)?))).unwrap();
    assert_eq!((job.as_str(), root.as_str()), ("interrupted", "idle"));
    let s = scan::scan_root(&l.db, &l.root_id, &AtomicBool::new(false), &|_| {});
    assert_eq!((s.state.as_str(), s.new, s.missing), ("done", 0, 0), "the resumed scan finds the index intact");
}

#[test]
#[ignore]
fn large_library() {
    let tmp = tempfile::tempdir().unwrap();
    let mount = crate::library::plain(tmp.path().canonicalize().unwrap()).join("BIG");
    let root = mount.join("Lib");
    let n = std::env::var("COSOUND_LIB_FILES").ok().and_then(|v| v.parse().ok()).unwrap_or(20_000usize);
    let cats = ["Kick", "Snare", "Hat", "Bass", "Pad", "Lead", "Perc", "FX", "Vocal", "Arp"];
    let t = std::time::Instant::now();
    for i in 0..n {
        let d = root.join(format!("{}/{:03}", cats[i % 10], i / 200));
        if i % 200 < 10 {
            std::fs::create_dir_all(&d).unwrap();
        }
        std::fs::write(d.join(format!("{} {} {} BPM.wav", cats[i % 10], i, 120 + i % 12)), [0u8; 64]).unwrap();
    }
    eprintln!("created {n} files in {:?}", t.elapsed());
    let db: Arc<Db> = Arc::new(std::sync::Mutex::new(db::open_memory()));
    let rid = library::add_root(&db.lock().unwrap(), &[disk(&mount)], root.to_str().unwrap()).unwrap();
    let t = std::time::Instant::now();
    let s = scan::scan_root(&db, &rid, &AtomicBool::new(false), &|_| {});
    eprintln!("initial scan: {} files in {:?}", s.new, t.elapsed());
    assert_eq!(s.new, n);
    let t = std::time::Instant::now();
    let s = scan::scan_root(&db, &rid, &AtomicBool::new(false), &|_| {});
    eprintln!("incremental rescan (unchanged): {} seen, {} changed in {:?}", s.seen, s.changed, t.elapsed());
    // labels from names (what analysis would add without decoding), then searches
    {
        let c = db.lock().unwrap();
        let mut st = c.prepare("select id, name from files").unwrap();
        let rows: Vec<(i64, String)> = st.query_map([], |r| Ok((r.get(0)?, r.get(1)?))).unwrap().map(Result::unwrap).collect();
        let tx = c.unchecked_transaction().unwrap();
        for (id, name) in rows {
            for l in classify::classify(&classify::Input { name: &name, folders: vec![], kind: "audio", features: None, tags: None, midi: None, preset: None }) {
                tx.execute("insert or ignore into file_tags(file_id, tag, kind, source, confidence) values (?1, ?2, ?3, ?4, ?5)", rusqlite::params![id, l.tag, l.kind, l.source, l.confidence]).unwrap();
            }
            let (b, _) = library::filename_meta(&name);
            tx.execute("update files set bpm = ?2, bpm_src = 'filename', bpm_conf = 0.6, category = (select tag from file_tags where file_id = ?1 and kind = 'category' limit 1) where id = ?1", rusqlite::params![id, b]).unwrap();
        }
        tx.commit().unwrap();
    }
    for q in ["kick 124 bpm", "Trova un pad atmosferico", "bassline a 126 BPM", "vocal"] {
        let t = std::time::Instant::now();
        let r = api::search_text(&db.lock().unwrap(), q, 50, None);
        eprintln!("search {q:?}: {} results in {:?}", r["count"], t.elapsed());
    }
}

/// Real files encoded by other software (e.g. macOS afconvert). Point COSOUND_REAL_FILES at a folder:
/// `COSOUND_REAL_FILES=/path cargo test --features temp-connector -- --ignored real_files --nocapture`
#[test]
#[ignore]
fn real_files() {
    let dir = std::env::var("COSOUND_REAL_FILES").expect("set COSOUND_REAL_FILES");
    let mut paths: Vec<_> = std::fs::read_dir(&dir).unwrap().map(|e| e.unwrap().path()).collect();
    paths.sort();
    for p in paths {
        let name = p.file_name().unwrap().to_string_lossy().to_string();
        let t = std::time::Instant::now();
        match analyze::analyze_file(&p, "audio", &name, vec![], &AtomicBool::new(false)) {
            Ok(a) => eprintln!(
                "{name}: format={:?} dur={:?} sr={:?} ch={:?} bits={:?} lufs={:?} bpm={:?}({}, {:.2}) key={:?}({}, {:.2}) loop={:?} bars={:?} cat={:?} in {:?}",
                a.format, a.duration_s.map(|d| (d * 100.0).round() / 100.0), a.sample_rate, a.channels, a.bit_depth, a.lufs,
                a.bpm.value, a.bpm.source, a.bpm.confidence, a.key.value, a.key.source, a.key.confidence, a.is_loop, a.loop_bars,
                a.labels.iter().filter(|l| l.kind == "category").map(|l| l.tag.clone()).collect::<Vec<_>>(), t.elapsed()
            ),
            Err(e) => eprintln!("{name}: ERROR {e}"),
        }
    }
}


/// Real drives on macOS: the internal disk (a folder of real audio files) and an external volume made from a disk image,
/// which is attached, scanned, analyzed, detached (offline: index kept, no path), attached again (same ids, files back).
/// `COSOUND_REAL_FILES=/folder/with/audio COSOUND_REAL_DMG=/path/image.dmg cargo test --release --features temp-connector -- --ignored real_drives --nocapture`
#[cfg(target_os = "macos")]
#[test]
#[ignore]
fn real_drives() {
    use std::process::Command;
    let files = PathBuf::from(std::env::var("COSOUND_REAL_FILES").expect("set COSOUND_REAL_FILES"));
    let dmg = std::env::var("COSOUND_REAL_DMG").expect("set COSOUND_REAL_DMG");
    let attach = || {
        let out = Command::new("hdiutil").args(["attach", &dmg]).output().unwrap();
        assert!(out.status.success(), "{}", String::from_utf8_lossy(&out.stderr));
        let text = String::from_utf8_lossy(&out.stdout).to_string();
        PathBuf::from(text.lines().filter_map(|l| l.split('\t').last()).find(|m| m.starts_with("/Volumes/")).expect("mount point").trim())
    };
    let db: Arc<Db> = Arc::new(std::sync::Mutex::new(db::open_memory()));
    let state = |id: &str| {
        let c = db.lock().unwrap();
        let disks = library::current_disks();
        library::refresh_volumes(&c, &disks).unwrap();
        library::roots(&c).unwrap().into_iter().find(|r| r.id == id).map(|r| (r.volume_kind.clone(), r.online, r.scan_state.clone(), r.file_count)).unwrap()
    };

    // 1. internal disk
    let internal = library::add_root(&db.lock().unwrap(), &library::current_disks(), files.to_str().unwrap()).unwrap();
    let s = scan::scan_root(&db, &internal, &AtomicBool::new(false), &|_| {});
    analyze_all(&db);
    eprintln!("internal: {:?} · scan new={} · {:?}", state(&internal), s.new, api::analysis_status(&db.lock().unwrap()));
    assert_eq!(state(&internal).0, "internal");

    // 2-3. external volume: copy the same files to it
    let mount = attach();
    let dir = mount.join("Samples");
    std::fs::create_dir_all(&dir).unwrap();
    for e in std::fs::read_dir(&files).unwrap() {
        let p = e.unwrap().path();
        std::fs::copy(&p, dir.join(p.file_name().unwrap())).unwrap();
    }
    let ext = library::add_root(&db.lock().unwrap(), &library::current_disks(), dir.to_str().unwrap()).unwrap();
    let s = scan::scan_root(&db, &ext, &AtomicBool::new(false), &|_| {});
    analyze_all(&db);
    let ids: Vec<i64> = db.lock().unwrap().prepare("select id from files where root_id = ?1 order by id").unwrap().query_map([&ext], |r| r.get(0)).unwrap().map(Result::unwrap).collect();
    eprintln!("external {}: {:?} · scan new={}", mount.display(), state(&ext), s.new);
    assert_eq!(state(&ext).0, "external");
    assert!(s.new > 0 && ids.len() == s.new);
    let dup = api::duplicates(&db, &AtomicBool::new(false), false);
    eprintln!("duplicates across drives: {} identical groups, {} same-audio groups", dup["identical_files"].as_array().unwrap().len(), dup["same_audio"].as_array().unwrap().len());

    // 4. unplug: index kept, state offline, no usable path, scan refuses
    let out = Command::new("hdiutil").args(["detach", mount.to_str().unwrap()]).output().unwrap();
    assert!(out.status.success(), "{}", String::from_utf8_lossy(&out.stderr));
    let st = state(&ext);
    let s = scan::scan_root(&db, &ext, &AtomicBool::new(false), &|_| {});
    let r = api::resolve(&db.lock().unwrap(), &ids);
    eprintln!("after detach: {st:?} · scan state={} · resolve[0]={}", s.state, r[0]);
    assert!(!st.1 && s.state == "offline" && st.3 == ids.len() as i64);
    assert!(r.iter().all(|x| x["path"].is_null() && x["usable"] == false));

    // replug: same volume recognized, same file ids, nothing new
    let mount2 = attach();
    let st = state(&ext);
    let s = scan::scan_root(&db, &ext, &AtomicBool::new(false), &|_| {});
    let r = api::resolve(&db.lock().unwrap(), &ids);
    eprintln!("after re-attach at {}: {st:?} · scan new={} changed={} · resolve[0]={}", mount2.display(), s.new, s.changed, r[0]["path"]);
    assert!(st.1 && s.new == 0 && s.changed == 0);
    assert!(r.iter().all(|x| x["usable"] == true));
    let _ = Command::new("hdiutil").args(["detach", mount2.to_str().unwrap()]).output();
}
