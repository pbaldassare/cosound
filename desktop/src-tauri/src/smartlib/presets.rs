//! Preset and instrument-library files. Only formats whose structure is documented are READ:
//! - `.vital`      Vital preset, JSON text (verified in Vital's open source, docs/knowledge/synthesizers §4.6)
//! - `.vstpreset`  VST3 preset: header, class ID, chunk list and the XML `Info` metadata chunk (VST3 SDK)
//! - `.aupreset`   Apple Audio Unit preset, property list (name, manufacturer, type, subtype)
//! - `.fxp`/`.fxb` VST2 program/bank container header (plugin ID, program name, parameter count)
//! Other preset formats are RECOGNIZED by extension only ("contents not read"): their layout is not
//! public or not verified (e.g. NKS `.nksf`, Massive `.nmsv`, Serum 2 `.SerumPreset`).
use serde_json::{json, Map, Value};
use std::path::Path;

pub const READABLE: [&str; 5] = ["vital", "vstpreset", "aupreset", "fxp", "fxb"];

fn fourcc(v: i64) -> String {
    let b = (v as u32).to_be_bytes();
    if b.iter().all(|c| c.is_ascii_graphic() || *c == b' ') {
        String::from_utf8_lossy(&b).to_string()
    } else {
        format!("{:08x}", v as u32)
    }
}

/// Plugin the preset belongs to, from what the file says or from the folder names (lower confidence).
fn plugin_from_path(path: &Path) -> Option<(String, &'static str)> {
    path.ancestors().skip(1).take(6).find_map(|a| a.file_name().and_then(|n| crate::sound::known_synth(&n.to_string_lossy())).map(|s| (s.name.to_string(), "path")))
}

fn attr(xml: &str, id: &str) -> Option<String> {
    let k = format!("id=\"{id}\"");
    let at = xml.find(&k)?;
    let rest = &xml[at..];
    let v = rest.find("value=\"")? + 7;
    let end = rest[v..].find('"')?;
    Some(rest[v..v + end].to_string())
}

pub fn read(path: &Path) -> Value {
    let ext = path.extension().map(|e| e.to_string_lossy().to_lowercase()).unwrap_or_default();
    let stem = path.file_stem().map(|s| s.to_string_lossy().to_string()).unwrap_or_default();
    let mut out = Map::new();
    out.insert("format".into(), json!(ext));
    out.insert("name".into(), json!(stem));
    let mut plugin: Option<(String, &str)> = None;
    let res: Result<(), String> = (|| {
        match ext.as_str() {
            "vital" => {
                let text = std::fs::read_to_string(path).map_err(|e| e.to_string())?;
                let v: Value = serde_json::from_str(&text).map_err(|e| format!("not valid JSON: {e}"))?;
                for k in ["preset_name", "author", "comments", "preset_style", "synth_version"] {
                    if let Some(x) = v[k].as_str().filter(|x| !x.is_empty()) {
                        out.insert(if k == "preset_name" { "name".into() } else { k.into() }, json!(x));
                    }
                }
                out.insert("parameters".into(), json!(v["settings"].as_object().map_or(0, |o| o.len())));
                plugin = Some(("Vital".into(), "file"));
            }
            "vstpreset" => {
                let b = std::fs::read(path).map_err(|e| e.to_string())?;
                if b.len() < 48 || &b[0..4] != b"VST3" {
                    return Err("not a VST3 preset".into());
                }
                out.insert("class_id".into(), json!(String::from_utf8_lossy(&b[8..40])));
                let list = u64::from_le_bytes(b[40..48].try_into().unwrap()) as usize;
                if list + 8 <= b.len() && &b[list..list + 4] == b"List" {
                    let n = u32::from_le_bytes(b[list + 4..list + 8].try_into().unwrap()) as usize;
                    for e in 0..n.min(16) {
                        let at = list + 8 + e * 20;
                        if at + 20 > b.len() {
                            break;
                        }
                        let (id, off, size) = (&b[at..at + 4], u64::from_le_bytes(b[at + 4..at + 12].try_into().unwrap()) as usize, u64::from_le_bytes(b[at + 12..at + 20].try_into().unwrap()) as usize);
                        if id == b"Info" && off + size <= b.len() {
                            let xml = String::from_utf8_lossy(&b[off..off + size]).to_string();
                            for (k, key) in [("PlugInName", "plugin_name"), ("PlugInCategory", "plugin_category"), ("MusicalCategory", "category"), ("MusicalInstrument", "instrument"), ("MusicalStyle", "style"), ("MusicalCharacter", "character"), ("Name", "name")] {
                                if let Some(v) = attr(&xml, k).filter(|v| !v.is_empty()) {
                                    out.insert(key.into(), json!(v));
                                }
                            }
                            if let Some(p) = attr(&xml, "PlugInName") {
                                plugin = Some((p, "file"));
                            }
                        }
                    }
                }
            }
            "aupreset" => {
                let v = plist::Value::from_file(path).map_err(|e| format!("not a property list: {e}"))?;
                let d = v.as_dictionary().ok_or("unexpected property list")?;
                if let Some(n) = d.get("name").and_then(|x| x.as_string()) {
                    out.insert("name".into(), json!(n));
                }
                for (k, key) in [("manufacturer", "manufacturer"), ("type", "au_type"), ("subtype", "au_subtype")] {
                    if let Some(i) = d.get(k).and_then(|x| x.as_signed_integer()) {
                        out.insert(key.into(), json!(fourcc(i)));
                    }
                }
            }
            "fxp" | "fxb" => {
                let b = std::fs::read(path).map_err(|e| e.to_string())?;
                if b.len() < 28 || &b[0..4] != b"CcnK" {
                    return Err("not a VST2 preset container".into());
                }
                let magic = String::from_utf8_lossy(&b[8..12]).to_string();
                out.insert("container".into(), json!(match magic.as_str() {
                    "FxCk" => "program (parameters)",
                    "FPCh" => "program (opaque chunk)",
                    "FxBk" => "bank (parameters)",
                    "FBCh" => "bank (opaque chunk)",
                    _ => "unknown",
                }));
                out.insert("plugin_id".into(), json!(String::from_utf8_lossy(&b[16..20])));
                let count = u32::from_be_bytes(b[24..28].try_into().unwrap());
                if magic == "FxCk" || magic == "FPCh" {
                    if b.len() >= 56 {
                        let name = String::from_utf8_lossy(&b[28..56]).trim_end_matches(char::from(0)).trim().to_string();
                        if !name.is_empty() {
                            out.insert("name".into(), json!(name));
                        }
                    }
                    if magic == "FxCk" {
                        out.insert("parameters".into(), json!(count));
                    }
                } else {
                    out.insert("programs".into(), json!(count));
                }
            }
            _ => return Err("contents not read: format not public or not verified".into()),
        }
        Ok(())
    })();
    match res {
        Ok(()) => {
            out.insert("readable".into(), json!(true));
        }
        Err(e) => {
            out.insert("readable".into(), json!(false));
            out.insert("note".into(), json!(e));
        }
    }
    let plugin = plugin.or_else(|| plugin_from_path(path));
    if let Some((p, src)) = plugin {
        let known = crate::sound::known_synth(&p);
        out.insert("plugin".into(), json!({ "name": known.map_or(p.clone(), |k| k.name.to_string()), "source": src, "known": known.map(|k| k.id) }));
    }
    Value::Object(out)
}

#[cfg(test)]
pub mod fixtures {
    use std::path::Path;

    pub fn vital(p: &Path, name: &str, style: &str) {
        std::fs::write(p, serde_json::json!({ "preset_name": name, "author": "COSOUND test", "preset_style": style, "comments": "warm evolving pad", "settings": { "osc_1_level": 0.7, "filter_1_cutoff": 60.0 } }).to_string()).unwrap();
    }
    pub fn vstpreset(p: &Path, plugin: &str, category: &str) {
        let xml = format!(r#"<?xml version="1.0" encoding="utf-8"?><MetaInfo><Attr id="MediaType" value="VstPreset" type="string" flags="writeProtected"/><Attr id="PlugInName" value="{plugin}" type="string"/><Attr id="PlugInCategory" value="Instrument|Synth" type="string"/><Attr id="MusicalCategory" value="{category}" type="string"/></MetaInfo>"#);
        let comp = vec![0u8; 16];
        let mut b = b"VST3".to_vec();
        b.extend_from_slice(&1u32.to_le_bytes());
        b.extend_from_slice(b"ABCDEF0123456789ABCDEF0123456789");
        let list_at = 48 + comp.len() + xml.len();
        b.extend_from_slice(&(list_at as u64).to_le_bytes());
        b.extend_from_slice(&comp);
        b.extend_from_slice(xml.as_bytes());
        b.extend_from_slice(b"List");
        b.extend_from_slice(&2u32.to_le_bytes());
        for (id, off, size) in [(b"Comp", 48u64, comp.len() as u64), (b"Info", (48 + comp.len()) as u64, xml.len() as u64)] {
            b.extend_from_slice(id);
            b.extend_from_slice(&off.to_le_bytes());
            b.extend_from_slice(&size.to_le_bytes());
        }
        std::fs::write(p, b).unwrap();
    }
    pub fn fxp(p: &Path, id: &[u8; 4], name: &str) {
        let mut b = b"CcnK".to_vec();
        b.extend_from_slice(&0u32.to_be_bytes());
        b.extend_from_slice(b"FPCh");
        b.extend_from_slice(&1u32.to_be_bytes());
        b.extend_from_slice(id);
        b.extend_from_slice(&1u32.to_be_bytes());
        b.extend_from_slice(&1u32.to_be_bytes());
        let mut n = name.as_bytes().to_vec();
        n.resize(28, 0);
        b.extend_from_slice(&n);
        b.extend_from_slice(&[0u8; 8]);
        std::fs::write(p, b).unwrap();
    }
    pub fn aupreset(p: &Path, name: &str) {
        let mut d = plist::Dictionary::new();
        d.insert("name".into(), plist::Value::String(name.into()));
        d.insert("manufacturer".into(), plist::Value::Integer((u32::from_be_bytes(*b"appl") as i64).into()));
        d.insert("type".into(), plist::Value::Integer((u32::from_be_bytes(*b"aumu") as i64).into()));
        d.insert("subtype".into(), plist::Value::Integer((u32::from_be_bytes(*b"test") as i64).into()));
        plist::Value::Dictionary(d).to_file_xml(p).unwrap();
    }
}

#[cfg(test)]
mod tests {
    use super::fixtures::*;
    use super::*;

    #[test]
    fn reads_documented_preset_formats() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path().join("Massive X").join("Pads");
        std::fs::create_dir_all(&root).unwrap();
        let v = dir.path().join("Deep Pad.vital");
        vital(&v, "Deep Pad", "Pad");
        let r = read(&v);
        assert_eq!((r["readable"].as_bool(), r["name"].as_str(), r["plugin"]["name"].as_str(), r["parameters"].as_u64()), (Some(true), Some("Deep Pad"), Some("Vital"), Some(2)));
        let vs = dir.path().join("Lead.vstpreset");
        vstpreset(&vs, "Serum", "Lead");
        let r = read(&vs);
        assert_eq!((r["plugin_name"].as_str(), r["category"].as_str(), r["plugin"]["name"].as_str()), (Some("Serum"), Some("Lead"), Some("Serum / Serum 2")));
        let fx = root.join("Atmos.fxp");
        fxp(&fx, b"MasX", "Atmosphere 1");
        let r = read(&fx);
        assert_eq!((r["name"].as_str(), r["plugin_id"].as_str(), r["plugin"]["source"].as_str()), (Some("Atmosphere 1"), Some("MasX"), Some("path")), "{r}");
        let au = dir.path().join("Keys.aupreset");
        aupreset(&au, "Soft Keys");
        let r = read(&au);
        assert_eq!((r["name"].as_str(), r["manufacturer"].as_str(), r["au_type"].as_str()), (Some("Soft Keys"), Some("appl"), Some("aumu")));
        let nk = dir.path().join("x.nksf");
        std::fs::write(&nk, b"RIFF").unwrap();
        let r = read(&nk);
        assert_eq!(r["readable"], false, "unverified formats are not claimed as readable");
        let bad = dir.path().join("y.vital");
        std::fs::write(&bad, b"{ nope").unwrap();
        assert_eq!(read(&bad)["readable"], false);
    }
}
