//! Plugin inventory: lists plugin bundles in the standard folders and reads their metadata files.
//! Plugins are never loaded or executed. Control level (host params / hand) is not measured yet.
use serde::Serialize;
use std::path::{Path, PathBuf};

#[derive(Serialize, Debug, Clone)]
pub struct Plugin {
    pub name: String,
    pub format: &'static str, // VST3 | AU | CLAP | VST2
    pub kind: &'static str,   // instrument | effect | unknown
    pub vendor: Option<String>,
    pub version: Option<String>,
    pub scope: &'static str, // system | user
    pub path: String,
}

fn folders() -> Vec<(PathBuf, &'static str)> {
    let env = |k: &str| std::env::var(k).map(PathBuf::from).ok();
    let mut v = Vec::new();
    if cfg!(target_os = "macos") {
        let sys = PathBuf::from("/Library/Audio/Plug-Ins");
        for sub in ["Components", "VST3", "CLAP", "VST"] {
            v.push((sys.join(sub), "system"));
            if let Some(h) = env("HOME") {
                v.push((h.join("Library/Audio/Plug-Ins").join(sub), "user"));
            }
        }
    } else if cfg!(windows) {
        if let Some(c) = env("CommonProgramFiles") {
            v.push((c.join("VST3"), "system"));
            v.push((c.join("CLAP"), "system"));
        }
        if let Some(pf) = env("ProgramFiles") {
            v.push((pf.join("VSTPlugins"), "system"));
            v.push((pf.join("Steinberg").join("VSTPlugins"), "system"));
        }
        if let Some(l) = env("LOCALAPPDATA") {
            v.push((l.join("Programs/Common/VST3"), "user"));
            v.push((l.join("Programs/Common/CLAP"), "user"));
        }
    } else {
        for d in ["/usr/lib/vst3", "/usr/local/lib/vst3", "/usr/lib/clap", "/usr/local/lib/clap"] {
            v.push((PathBuf::from(d), "system"));
        }
        if let Some(h) = env("HOME") {
            v.push((h.join(".vst3"), "user"));
            v.push((h.join(".clap"), "user"));
        }
    }
    v
}

fn format_of(ext: &str) -> Option<&'static str> {
    Some(match ext {
        "vst3" => "VST3",
        "component" => "AU",
        "clap" => "CLAP",
        "vst" => "VST2",
        "dll" if cfg!(windows) => "VST2",
        _ => return None,
    })
}

/// VST3 moduleinfo.json (SDK 3.7.5+): vendor, version, instrument vs effect.
fn read_moduleinfo(p: &mut Plugin, bundle: &Path) {
    let Ok(txt) = std::fs::read_to_string(bundle.join("Contents/Resources/moduleinfo.json")) else { return };
    let Ok(v) = serde_json::from_str::<serde_json::Value>(&txt) else { return }; // ponytail: JSON5 variants are skipped
    p.vendor = v["Factory Info"]["Vendor"].as_str().map(str::to_string);
    p.version = v["Version"].as_str().map(str::to_string);
    if let Some(classes) = v["Classes"].as_array() {
        let audio: Vec<_> = classes.iter().filter(|c| c["Category"] == "Audio Module Class").collect();
        if !audio.is_empty() {
            let instrument = audio.iter().any(|c| c["Sub Categories"].as_array().is_some_and(|s| s.iter().any(|x| x == "Instrument")));
            p.kind = if instrument { "instrument" } else { "effect" };
        }
    }
}

/// macOS bundle Info.plist: version, and for AU the component type and "Vendor: Name".
fn read_plist(p: &mut Plugin, bundle: &Path) {
    let Ok(v) = plist::Value::from_file(bundle.join("Contents/Info.plist")) else { return };
    let Some(d) = v.as_dictionary() else { return };
    if p.version.is_none() {
        p.version = d.get("CFBundleShortVersionString").and_then(|x| x.as_string()).map(str::to_string);
    }
    if let Some(ac) = d.get("AudioComponents").and_then(|x| x.as_array()).and_then(|a| a.first()).and_then(|x| x.as_dictionary()) {
        if let Some((vendor, _)) = ac.get("name").and_then(|x| x.as_string()).and_then(|n| n.split_once(':')) {
            p.vendor.get_or_insert(vendor.trim().to_string());
        }
        p.kind = match ac.get("type").and_then(|x| x.as_string()) {
            Some("aumu") => "instrument",
            Some("aufx" | "aumf") => "effect",
            _ => p.kind,
        };
    }
}

pub fn scan_in(dirs: &[(PathBuf, &'static str)]) -> Vec<Plugin> {
    let mut out = Vec::new();
    for (dir, scope) in dirs {
        let mut it = walkdir::WalkDir::new(dir).max_depth(4).into_iter();
        while let Some(Ok(e)) = it.next() {
            let ext = e.path().extension().map(|x| x.to_string_lossy().to_ascii_lowercase()).unwrap_or_default();
            let Some(format) = format_of(&ext) else { continue };
            if e.file_type().is_dir() {
                it.skip_current_dir(); // bundle: don't descend
            }
            let mut p = Plugin {
                name: e.path().file_stem().unwrap_or_default().to_string_lossy().to_string(),
                format,
                kind: "unknown",
                vendor: None,
                version: None,
                scope,
                path: e.path().to_string_lossy().to_string(),
            };
            if e.file_type().is_dir() {
                read_moduleinfo(&mut p, e.path());
                read_plist(&mut p, e.path());
            }
            out.push(p);
        }
    }
    out.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()).then(a.format.cmp(b.format)));
    out
}

pub fn scan() -> Vec<Plugin> {
    scan_in(&folders())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn reads_vst3_moduleinfo_and_au_plist() {
        let tmp = tempfile::tempdir().unwrap();
        let vst = tmp.path().join("Vendor/Massive X.vst3/Contents/Resources");
        fs::create_dir_all(&vst).unwrap();
        fs::write(
            vst.join("moduleinfo.json"),
            r#"{"Name":"Massive X","Version":"1.5.0","Factory Info":{"Vendor":"Native Instruments"},
               "Classes":[{"Category":"Audio Module Class","Sub Categories":["Instrument","Synth"]}]}"#,
        )
        .unwrap();
        let au = tmp.path().join("Comp.component/Contents");
        fs::create_dir_all(&au).unwrap();
        fs::write(
            au.join("Info.plist"),
            r#"<?xml version="1.0" encoding="UTF-8"?><plist version="1.0"><dict>
              <key>CFBundleShortVersionString</key><string>2.1</string>
              <key>AudioComponents</key><array><dict><key>name</key><string>Acme: Comp</string><key>type</key><string>aufx</string></dict></array>
              </dict></plist>"#,
        )
        .unwrap();
        fs::create_dir_all(tmp.path().join("Bare.clap")).unwrap();
        fs::write(tmp.path().join("readme.txt"), "").unwrap();

        let got = scan_in(&[(tmp.path().to_path_buf(), "user")]);
        assert_eq!(got.len(), 3);
        let mx = got.iter().find(|p| p.name == "Massive X").unwrap();
        assert_eq!((mx.format, mx.kind, mx.vendor.as_deref(), mx.version.as_deref()), ("VST3", "instrument", Some("Native Instruments"), Some("1.5.0")));
        let comp = got.iter().find(|p| p.name == "Comp").unwrap();
        assert_eq!((comp.format, comp.kind, comp.vendor.as_deref(), comp.version.as_deref()), ("AU", "effect", Some("Acme"), Some("2.1")));
        assert_eq!(got.iter().find(|p| p.name == "Bare").unwrap().kind, "unknown");
        assert!(scan_in(&[(tmp.path().join("missing"), "system")]).is_empty());
    }
}
