//! Plugin Scanner: finds plug-ins in the standard plug-in folders of macOS and Windows and, on macOS, in the
//! Audio Unit registry kept by the system (which also lists Apple's built-in units and AUv3 app extensions).
//! Plug-ins are never loaded here: only bundle metadata files and executable headers are read.
//! Formats: AU (AUv2 components), AUv3 (app extensions, from the registry), VST3, VST2, CLAP, AAX.
use super::host;
use plist::Value as Plist;
use serde::Serialize;
use serde_json::{json, Value};
use std::path::{Path, PathBuf};

#[derive(Serialize, Debug, Clone, Default)]
pub struct Install {
    pub id: String,
    pub format: String,
    pub name: String,
    pub vendor: Option<String>,
    pub version: Option<String>,
    pub path: Option<String>,
    pub scope: String,
    pub os: String,
    pub archs: Vec<String>,
    pub au_code: Option<String>,
    pub bundle_id: Option<String>,
    /// Audio Units: listed by macOS's component registry (None for other formats).
    pub registered: Option<bool>,
    /// instrument | effect | midi_effect | generator | other | unknown
    pub kind: String,
    /// AU tags or VST3 sub-categories, as declared by the plug-in.
    pub declared_categories: Vec<String>,
    pub status: String,
    pub status_reason: Option<String>,
    pub details: Value,
}

pub fn os() -> &'static str {
    if cfg!(target_os = "macos") {
        "macos"
    } else if cfg!(windows) {
        "windows"
    } else {
        "linux"
    }
}

/// Architecture names as used in Mach-O / VST3 bundles.
pub fn host_arch() -> &'static str {
    match std::env::consts::ARCH {
        "aarch64" => "arm64",
        "x86_64" => "x86_64",
        a => a,
    }
}

/// Standard plug-in folders (documented by each format). Only these are walked: no other part of the disk.
pub fn folders() -> Vec<(PathBuf, &'static str, &'static str)> {
    let env = |k: &str| std::env::var(k).map(PathBuf::from).ok();
    let mut v = Vec::new();
    if cfg!(target_os = "macos") {
        let home = env("HOME");
        for (sub, format) in [("Components", "AU"), ("VST3", "VST3"), ("CLAP", "CLAP"), ("VST", "VST2")] {
            v.push((PathBuf::from("/Library/Audio/Plug-Ins").join(sub), format, "system"));
            if let Some(h) = &home {
                v.push((h.join("Library/Audio/Plug-Ins").join(sub), format, "user"));
            }
        }
        v.push((PathBuf::from("/Library/Application Support/Avid/Audio/Plug-Ins"), "AAX", "system"));
    } else if cfg!(windows) {
        if let Some(c) = env("CommonProgramFiles") {
            v.push((c.join("VST3"), "VST3", "system"));
            v.push((c.join("CLAP"), "CLAP", "system"));
            v.push((c.join("VST2"), "VST2", "system"));
            v.push((c.join("Avid/Audio/Plug-Ins"), "AAX", "system"));
        }
        if let Some(pf) = env("ProgramFiles") {
            v.push((pf.join("VSTPlugins"), "VST2", "system"));
            v.push((pf.join("Steinberg/VSTPlugins"), "VST2", "system"));
        }
        if let Some(l) = env("LOCALAPPDATA") {
            v.push((l.join("Programs/Common/VST3"), "VST3", "user"));
            v.push((l.join("Programs/Common/CLAP"), "CLAP", "user"));
        }
    } else {
        for (d, f) in [("/usr/lib/vst3", "VST3"), ("/usr/local/lib/vst3", "VST3"), ("/usr/lib/clap", "CLAP"), ("/usr/local/lib/clap", "CLAP")] {
            v.push((PathBuf::from(d), f, "system"));
        }
        if let Some(h) = env("HOME") {
            v.push((h.join(".vst3"), "VST3", "user"));
            v.push((h.join(".clap"), "CLAP", "user"));
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
        "aaxplugin" => "AAX",
        "dll" if cfg!(windows) => "VST2",
        _ => return None,
    })
}

/// CPU architectures in a Mach-O executable (thin or universal). Empty when unreadable.
pub fn macho_archs(bytes: &[u8]) -> Vec<String> {
    let name = |cpu: u32| match cpu {
        0x0100_000C => "arm64".to_string(),
        0x0100_0007 => "x86_64".to_string(),
        7 => "i386".to_string(),
        c => format!("cpu{c:#x}"),
    };
    if bytes.len() < 8 {
        return vec![];
    }
    let be = |o: usize| u32::from_be_bytes([bytes[o], bytes[o + 1], bytes[o + 2], bytes[o + 3]]);
    let le = |o: usize| u32::from_le_bytes([bytes[o], bytes[o + 1], bytes[o + 2], bytes[o + 3]]);
    match be(0) {
        0xCAFE_BABE | 0xCAFE_BABF => {
            let n = be(4) as usize;
            let step = if be(0) == 0xCAFE_BABF { 32 } else { 20 };
            (0..n.min(8)).filter_map(|i| (8 + i * step + 4 <= bytes.len()).then(|| name(be(8 + i * step)))).collect()
        }
        0xCFFA_EDFE | 0xCEFA_EDFE => vec![name(le(4))],
        0xFEED_FACF | 0xFEED_FACE => vec![name(be(4))],
        _ => vec![],
    }
}

/// Machine type of a Windows PE file (.dll / .vst3 / .clap).
pub fn pe_arch(bytes: &[u8]) -> Vec<String> {
    if bytes.len() < 0x40 || &bytes[0..2] != b"MZ" {
        return vec![];
    }
    let off = u32::from_le_bytes([bytes[0x3c], bytes[0x3d], bytes[0x3e], bytes[0x3f]]) as usize;
    if off + 6 > bytes.len() || &bytes[off..off + 4] != b"PE\0\0" {
        return vec![];
    }
    match u16::from_le_bytes([bytes[off + 4], bytes[off + 5]]) {
        0x8664 => vec!["x86_64".into()],
        0xAA64 => vec!["arm64".into()],
        0x014c => vec!["i386".into()],
        m => vec![format!("machine{m:#x}")],
    }
}

fn head(path: &Path, n: usize) -> Vec<u8> {
    use std::io::Read;
    let mut buf = vec![0u8; n];
    match std::fs::File::open(path).and_then(|mut f| f.read(&mut buf)) {
        Ok(k) => {
            buf.truncate(k);
            buf
        }
        Err(_) => vec![],
    }
}

fn plist_dict(path: &Path) -> Option<plist::Dictionary> {
    Plist::from_file(path).ok()?.into_dictionary()
}

fn pstr(d: &plist::Dictionary, k: &str) -> Option<String> {
    d.get(k).and_then(|x| x.as_string()).map(str::to_string)
}

/// Fill an install from a bundle folder: Info.plist (macOS bundles), VST3 moduleinfo.json, executable architectures.
fn read_bundle(i: &mut Install, bundle: &Path) {
    let mut details = serde_json::Map::new();
    if let Some(d) = plist_dict(&bundle.join("Contents/Info.plist")) {
        i.version = pstr(&d, "CFBundleShortVersionString").or_else(|| pstr(&d, "CFBundleVersion"));
        i.bundle_id = pstr(&d, "CFBundleIdentifier");
        if let Some(exe) = pstr(&d, "CFBundleExecutable") {
            i.archs = macho_archs(&head(&bundle.join("Contents/MacOS").join(exe), 4096));
        }
        if let Some(acs) = d.get("AudioComponents").and_then(|x| x.as_array()) {
            let list: Vec<Value> = acs
                .iter()
                .filter_map(|x| x.as_dictionary())
                .map(|ac| json!({ "name": pstr(ac, "name"), "type": pstr(ac, "type"), "subtype": pstr(ac, "subtype"), "manufacturer": pstr(ac, "manufacturer"), "tags": ac.get("tags").and_then(|t| t.as_array()).map(|a| a.iter().filter_map(|x| x.as_string()).collect::<Vec<_>>()) }))
                .collect();
            if let Some(first) = list.first() {
                if let Some((vendor, name)) = first["name"].as_str().and_then(|n| n.split_once(':')) {
                    i.vendor.get_or_insert(vendor.trim().to_string());
                    i.name = name.trim().to_string();
                }
                i.kind = kind_of_au(first["type"].as_str().unwrap_or("")).to_string();
                if let (Some(t), Some(s), Some(m)) = (first["type"].as_str(), first["subtype"].as_str(), first["manufacturer"].as_str()) {
                    i.au_code = Some(format!("{t}:{s}:{m}"));
                }
                if let Some(tags) = first["tags"].as_array() {
                    i.declared_categories = tags.iter().filter_map(|t| t.as_str().map(String::from)).collect();
                }
            }
            details.insert("audio_components".into(), json!(list));
        }
    }
    // VST3 SDK 3.7.5+: Contents/Resources/moduleinfo.json (JSON5 in theory; plain JSON in practice)
    if let Ok(txt) = std::fs::read_to_string(bundle.join("Contents/Resources/moduleinfo.json")) {
        if let Ok(v) = serde_json::from_str::<Value>(&txt) {
            i.vendor = v["Factory Info"]["Vendor"].as_str().map(str::to_string).or(i.vendor.take());
            i.version = v["Version"].as_str().map(str::to_string).or(i.version.take());
            if let Some(n) = v["Name"].as_str().filter(|n| !n.is_empty()) {
                i.name = n.to_string();
            }
            let audio: Vec<&Value> = v["Classes"].as_array().map(|c| c.iter().filter(|c| c["Category"] == "Audio Module Class").collect()).unwrap_or_default();
            let subs: Vec<String> = audio.iter().flat_map(|c| c["Sub Categories"].as_array().cloned().unwrap_or_default()).filter_map(|s| s.as_str().map(String::from)).collect();
            if !audio.is_empty() {
                i.kind = if subs.iter().any(|s| s == "Instrument") { "instrument" } else { "effect" }.into();
            }
            i.declared_categories = subs;
            details.insert("vst3_sdk".into(), v["Classes"][0]["SDKVersion"].clone());
            details.insert("vst3_classes".into(), json!(audio.iter().map(|c| json!({ "name": c["Name"], "cid": c["CID"] })).collect::<Vec<_>>()));
        }
    }
    // VST3 bundles on Windows/Linux: Contents/<arch>-win|linux/<name>.vst3
    if i.archs.is_empty() && i.format == "VST3" {
        for (dir, arch) in [("x86_64-win", "x86_64"), ("arm64-win", "arm64"), ("arm64ec-win", "arm64"), ("x86_64-linux", "x86_64"), ("aarch64-linux", "arm64")] {
            if bundle.join("Contents").join(dir).is_dir() {
                i.archs.push(arch.into());
            }
        }
    }
    i.details = Value::Object(details);
}

pub fn kind_of_au(t: &str) -> &'static str {
    match t {
        "aumu" => "instrument",
        "aufx" | "aumf" => "effect",
        "aumi" => "midi_effect",
        "augn" => "generator",
        _ => "other",
    }
}

/// Walk the given plug-in folders. Bundles are not descended into.
pub fn scan_folders(dirs: &[(PathBuf, &'static str, &'static str)]) -> Vec<Install> {
    let mut out = Vec::new();
    for (dir, _, scope) in dirs {
        let mut it = walkdir::WalkDir::new(dir).max_depth(4).into_iter();
        while let Some(Ok(e)) = it.next() {
            let ext = e.path().extension().map(|x| x.to_string_lossy().to_ascii_lowercase()).unwrap_or_default();
            let Some(format) = format_of(&ext) else { continue };
            if e.file_type().is_dir() {
                it.skip_current_dir();
            }
            let path = e.path().to_string_lossy().to_string();
            let mut i = Install {
                id: format!("{format}:{path}"),
                format: format.into(),
                name: e.path().file_stem().unwrap_or_default().to_string_lossy().to_string(),
                path: Some(path),
                scope: scope.to_string(),
                os: os().into(),
                kind: "unknown".into(),
                status: "available".into(),
                ..Default::default()
            };
            if e.file_type().is_dir() {
                read_bundle(&mut i, e.path());
            } else {
                i.archs = pe_arch(&head(e.path(), 4096));
            }
            check(&mut i);
            out.push(i);
        }
    }
    out
}

/// Usability on this computer: missing executable, wrong CPU architecture.
fn check(i: &mut Install) {
    if !i.archs.is_empty() && !i.archs.iter().any(|a| a == host_arch()) {
        i.status = "incompatible".into();
        i.status_reason = Some(if os() == "macos" && host_arch() == "arm64" && i.archs.iter().any(|a| a == "x86_64") {
            format!("built for {} only: it runs only in a DAW started under Rosetta", i.archs.join(", "))
        } else {
            format!("built for {} only; this computer is {}", i.archs.join(", "), host_arch())
        });
    }
}

/// Audio Units known to macOS (registry), merged with the Components folders: one install per component.
/// Units in a scanned bundle that macOS does not list are reported as not registered (macOS rejected or has not
/// validated them yet).
pub fn merge_au_registry(mut found: Vec<Install>, registry: &[Value]) -> Vec<Install> {
    let mut out: Vec<Install> = found.iter().filter(|i| i.format != "AU").cloned().collect();
    let bundles: Vec<Install> = found.drain(..).filter(|i| i.format == "AU").collect();
    let mut matched = std::collections::HashSet::new();
    for c in registry {
        let t = c["type"].as_str().unwrap_or("");
        if !["aumu", "aufx", "aumf", "aumi", "augn"].contains(&t) {
            continue; // outputs, converters, mixers, panners: not production plug-ins
        }
        let code = format!("{t}:{}:{}", c["subtype"].as_str().unwrap_or(""), c["manufacturer_code"].as_str().unwrap_or(""));
        let path = c["path"].as_str().map(String::from);
        let bundle = path.as_ref().and_then(|p| bundles.iter().find(|b| b.path.as_deref() == Some(p.as_str())));
        if let Some(b) = bundle {
            matched.insert(b.id.clone());
        }
        let v3 = c["v3"].as_bool().unwrap_or(false);
        let scope = match (&bundle, &path) {
            (Some(b), _) => b.scope.clone(),
            (None, Some(p)) if p.starts_with("/System/") => "apple".into(),
            (None, Some(p)) if p.contains(".appex") || v3 => "app_extension".into(),
            _ => if v3 { "app_extension".into() } else { "system".into() },
        };
        let mut i = Install {
            id: format!("AU:{code}"),
            format: if v3 { "AUv3".into() } else { "AU".into() },
            name: c["name"].as_str().unwrap_or("?").to_string(),
            vendor: c["manufacturer"].as_str().map(String::from),
            version: c["version"].as_str().map(String::from),
            path,
            scope,
            os: "macos".into(),
            archs: bundle.map(|b| b.archs.clone()).unwrap_or_default(),
            au_code: Some(code),
            bundle_id: bundle.and_then(|b| b.bundle_id.clone()),
            registered: Some(true),
            kind: kind_of_au(t).into(),
            declared_categories: c["tags"].as_array().map(|a| a.iter().filter_map(|x| x.as_str().map(String::from)).collect()).unwrap_or_default(),
            status: "available".into(),
            status_reason: None,
            details: json!({ "type_name": c["type_name"], "has_midi_input": c["has_midi_input"], "sandbox_safe": c["sandbox_safe"], "has_custom_view": c["has_custom_view"] }),
        };
        check(&mut i);
        out.push(i);
    }
    for mut b in bundles.into_iter().filter(|b| !matched.contains(&b.id)) {
        b.registered = Some(false);
        if b.status == "available" {
            b.status = "error".into();
            b.status_reason = Some("the bundle is in the Components folder but macOS does not list it (failed validation, or not registered yet: log out and in, or open the DAW once)".into());
        }
        out.push(b);
    }
    out
}

/// Full scan: standard folders, plus the AU registry on macOS (through the isolated host, which loads nothing).
pub fn scan() -> (Vec<Install>, Vec<String>) {
    let found = scan_folders(&folders());
    let mut notes = vec![];
    if cfg!(target_os = "macos") {
        match host::run("list", &json!({}), 60) {
            Ok(v) => return (merge_au_registry(found, v["components"].as_array().map(Vec::as_slice).unwrap_or(&[])), notes),
            Err(e) => notes.push(format!("Audio Unit registry not read ({e}): only the Components folders were checked")),
        }
    }
    (found, notes)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn reads_bundles_of_every_format() {
        let tmp = tempfile::tempdir().unwrap();
        let vst = tmp.path().join("Vendor/Massive X.vst3/Contents/Resources");
        fs::create_dir_all(&vst).unwrap();
        fs::write(vst.join("moduleinfo.json"), r#"{"Name":"Massive X","Version":"1.5.0","Factory Info":{"Vendor":"Native Instruments"},
               "Classes":[{"CID":"ABC","Name":"Massive X","Category":"Audio Module Class","Sub Categories":["Instrument","Synth"],"SDKVersion":"VST 3.7.9"}]}"#).unwrap();
        fs::create_dir_all(tmp.path().join("Vendor/Massive X.vst3/Contents/x86_64-win")).unwrap();
        let au = tmp.path().join("Comp.component/Contents");
        fs::create_dir_all(au.join("MacOS")).unwrap();
        fs::write(au.join("Info.plist"), r#"<?xml version="1.0" encoding="UTF-8"?><plist version="1.0"><dict>
              <key>CFBundleShortVersionString</key><string>2.1</string><key>CFBundleExecutable</key><string>Comp</string>
              <key>CFBundleIdentifier</key><string>com.acme.comp</string>
              <key>AudioComponents</key><array><dict><key>name</key><string>Acme: Comp</string><key>type</key><string>aufx</string>
              <key>subtype</key><string>Cmp1</string><key>manufacturer</key><string>Acme</string><key>tags</key><array><string>Dynamics</string></array></dict></array>
              </dict></plist>"#).unwrap();
        // universal binary header: arm64 + x86_64
        let mut fat = vec![0xCA, 0xFE, 0xBA, 0xBE, 0, 0, 0, 2];
        for cpu in [0x0100_000Cu32, 0x0100_0007] {
            fat.extend_from_slice(&cpu.to_be_bytes());
            fat.extend_from_slice(&[0u8; 16]);
        }
        fs::write(au.join("MacOS/Comp"), &fat).unwrap();
        let old = tmp.path().join("Old.vst/Contents");
        fs::create_dir_all(old.join("MacOS")).unwrap();
        fs::write(old.join("Info.plist"), r#"<?xml version="1.0" encoding="UTF-8"?><plist version="1.0"><dict><key>CFBundleExecutable</key><string>Old</string><key>CFBundleVersion</key><string>1.0</string></dict></plist>"#).unwrap();
        fs::write(old.join("MacOS/Old"), [0xCF, 0xFA, 0xED, 0xFE, 0x07, 0x00, 0x00, 0x01]).unwrap(); // thin x86_64
        fs::create_dir_all(tmp.path().join("Bare.clap")).unwrap();
        fs::create_dir_all(tmp.path().join("Pro.aaxplugin")).unwrap();
        fs::write(tmp.path().join("readme.txt"), "").unwrap();

        let got = scan_folders(&[(tmp.path().to_path_buf(), "", "user")]);
        assert_eq!(got.len(), 5);
        let mx = got.iter().find(|p| p.name == "Massive X").unwrap();
        assert_eq!((mx.format.as_str(), mx.kind.as_str(), mx.vendor.as_deref(), mx.version.as_deref()), ("VST3", "instrument", Some("Native Instruments"), Some("1.5.0")));
        assert_eq!((mx.declared_categories.clone(), mx.archs.clone()), (vec!["Instrument".to_string(), "Synth".to_string()], vec!["x86_64".to_string()]));
        let comp = got.iter().find(|p| p.name == "Comp").unwrap();
        assert_eq!((comp.format.as_str(), comp.kind.as_str(), comp.vendor.as_deref(), comp.version.as_deref(), comp.au_code.as_deref()), ("AU", "effect", Some("Acme"), Some("2.1"), Some("aufx:Cmp1:Acme")));
        assert_eq!(comp.archs, vec!["arm64", "x86_64"]);
        let old = got.iter().find(|p| p.name == "Old").unwrap();
        assert_eq!((old.format.as_str(), old.version.as_deref(), old.archs.clone()), ("VST2", Some("1.0"), vec!["x86_64".to_string()]));
        if host_arch() == "arm64" {
            assert_eq!(old.status, "incompatible", "x86_64-only plug-in on Apple silicon");
        }
        assert!(got.iter().any(|p| p.format == "CLAP") && got.iter().any(|p| p.format == "AAX"));
        assert!(scan_folders(&[(tmp.path().join("missing"), "", "system")]).is_empty());

        // AU registry merge: the bundle is registered → one install per component; an unregistered bundle is flagged
        let reg = vec![json!({"type":"aufx","subtype":"Cmp1","manufacturer_code":"Acme","name":"Comp","manufacturer":"Acme","version":"2.1.0","tags":["Dynamics"],"path": comp.path, "v3": false}),
                       json!({"type":"aufx","subtype":"lpas","manufacturer_code":"appl","name":"AULowpass","manufacturer":"Apple","version":"1.6.0","tags":["Effects","Filter"],"path":"/System/Library/Components/CoreAudio.component","v3":false}),
                       json!({"type":"auou","subtype":"def ","manufacturer_code":"appl","name":"DefaultOutputUnit","manufacturer":"Apple"})];
        let merged = merge_au_registry(got.clone(), &reg);
        let c = merged.iter().find(|i| i.id == "AU:aufx:Cmp1:Acme").unwrap();
        assert_eq!((c.registered, c.scope.as_str(), c.archs.len()), (Some(true), "user", 2));
        assert_eq!(merged.iter().find(|i| i.name == "AULowpass").unwrap().scope, "apple");
        assert!(!merged.iter().any(|i| i.name == "DefaultOutputUnit"), "output units are not production plug-ins");
        let unreg = merge_au_registry(got, &reg[1..]);
        let c = unreg.iter().find(|i| i.name == "Comp").unwrap();
        assert_eq!((c.registered, c.status.as_str()), (Some(false), "error"));
    }

    #[test]
    fn pe_machine_types() {
        let mut b = vec![0u8; 0x100];
        b[0] = b'M';
        b[1] = b'Z';
        b[0x3c] = 0x80;
        b[0x80..0x84].copy_from_slice(b"PE\0\0");
        b[0x84..0x86].copy_from_slice(&0x8664u16.to_le_bytes());
        assert_eq!(pe_arch(&b), vec!["x86_64"]);
        b[0x84..0x86].copy_from_slice(&0xAA64u16.to_le_bytes());
        assert_eq!(pe_arch(&b), vec!["arm64"]);
        assert!(pe_arch(b"not a pe").is_empty());
    }
}
