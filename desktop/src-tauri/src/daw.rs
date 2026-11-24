//! DAW detection: looks for installed DAWs in their standard install locations. Read-only.
//! Detection is NOT a connection: no connector exists yet, so every DAW reports `connected: false`.
use serde::Serialize;
use std::path::{Path, PathBuf};

#[derive(Serialize, Debug)]
pub struct DawFound {
    pub id: &'static str,
    pub supported_on_this_os: bool,
    pub detected: bool,
    pub path: Option<String>,
    pub version: Option<String>,
    pub connected: bool,
}

/// (daw id, folder to look in, entry name prefix). Order = product priority (Prompt 04 §9).
fn locations() -> Vec<(&'static str, PathBuf, &'static str)> {
    let env = |k: &str| std::env::var(k).map(PathBuf::from).unwrap_or_default();
    if cfg!(target_os = "macos") {
        let apps = PathBuf::from("/Applications");
        vec![
            ("logic", apps.clone(), "Logic Pro"),
            ("ableton", apps.clone(), "Ableton Live"),
            ("fl", apps.clone(), "FL Studio"),
            ("protools", apps.clone(), "Pro Tools"),
            ("cubase", apps.clone(), "Cubase"),
            ("reaper", apps, "REAPER"),
        ]
    } else if cfg!(windows) {
        let pf = env("ProgramFiles");
        vec![
            ("ableton", env("ProgramData").join("Ableton"), "Live"),
            ("fl", pf.join("Image-Line"), "FL Studio"),
            ("protools", pf.join("Avid"), "Pro Tools"),
            ("cubase", pf.join("Steinberg"), "Cubase"),
            ("reaper", pf.clone(), "REAPER"),
        ]
    } else {
        vec![("reaper", env("HOME").join("opt"), "REAPER"), ("reaper", PathBuf::from("/opt"), "REAPER")]
    }
}

pub const ALL: &[&str] = &["logic", "ableton", "fl", "protools", "cubase", "reaper"];

fn supported(id: &str) -> bool {
    if cfg!(target_os = "macos") {
        true
    } else if cfg!(windows) {
        id != "logic"
    } else {
        id == "reaper"
    }
}

/// macOS bundle version from Contents/Info.plist. On Windows the version is not read (reported as unavailable).
fn bundle_version(app: &Path) -> Option<String> {
    let v = plist::Value::from_file(app.join("Contents/Info.plist")).ok()?;
    v.as_dictionary()?.get("CFBundleShortVersionString")?.as_string().map(str::to_string)
}

fn find(dir: &Path, prefix: &str) -> Option<PathBuf> {
    let mut hits: Vec<PathBuf> = std::fs::read_dir(dir)
        .ok()?
        .filter_map(|e| e.ok())
        .filter(|e| e.file_name().to_string_lossy().starts_with(prefix))
        .map(|e| e.path())
        .collect();
    hits.sort(); // newest numbered version last (e.g. "Cubase 15" after "Cubase 14")
    hits.pop()
}

pub fn detect_in(locs: &[(&'static str, PathBuf, &'static str)]) -> Vec<DawFound> {
    ALL.iter()
        .map(|&id| {
            let hit = locs.iter().filter(|l| l.0 == id).find_map(|l| find(&l.1, l.2));
            DawFound {
                id,
                supported_on_this_os: supported(id),
                detected: hit.is_some(),
                version: hit.as_deref().and_then(bundle_version),
                path: hit.map(|p| p.to_string_lossy().to_string()),
                connected: false,
            }
        })
        .collect()
}

pub fn detect() -> Vec<DawFound> {
    detect_in(&locations())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_app_bundles_and_reads_version() {
        let tmp = tempfile::tempdir().unwrap();
        let logic = tmp.path().join("Logic Pro.app/Contents");
        std::fs::create_dir_all(&logic).unwrap();
        std::fs::write(
            logic.join("Info.plist"),
            r#"<?xml version="1.0" encoding="UTF-8"?><plist version="1.0"><dict>
               <key>CFBundleShortVersionString</key><string>13.0</string></dict></plist>"#,
        )
        .unwrap();
        std::fs::create_dir_all(tmp.path().join("Cubase 14.app")).unwrap();
        std::fs::create_dir_all(tmp.path().join("Cubase 15.app")).unwrap();
        let d = tmp.path().to_path_buf();
        let found = detect_in(&[("logic", d.clone(), "Logic Pro"), ("cubase", d.clone(), "Cubase"), ("ableton", d, "Ableton Live")]);
        let get = |id| found.iter().find(|f| f.id == id).unwrap();
        assert_eq!(get("logic").version.as_deref(), Some("13.0"));
        assert!(get("cubase").path.as_ref().unwrap().ends_with("Cubase 15.app"));
        assert!(get("cubase").version.is_none());
        assert!(!get("ableton").detected);
        assert!(found.iter().all(|f| !f.connected));
        assert_eq!(found.len(), 6);
    }
}
