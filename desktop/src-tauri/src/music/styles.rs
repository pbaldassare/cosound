//! Style Intelligence Engine – style profiles (BPM, groove, harmony, melody density, bass type, rhythm, phrase
//! structure, arrangement and energy) loaded from `packages/music-styles/styles.json`, overridable by JSON files in
//! `<app data>/music-styles/`, and blendable ("techno with ambient elements"): a genre is a starting point, not a cage.
use serde_json::{json, Map, Value};
use std::path::Path;

const BUILTIN: &str = include_str!("../../../../packages/music-styles/styles.json");

pub struct Styles {
    list: Vec<Value>,
}

impl Styles {
    pub fn load(data_dir: Option<&Path>) -> Styles {
        let base: Value = serde_json::from_str(BUILTIN).expect("styles.json is valid");
        let mut list: Vec<Value> = base["styles"].as_array().cloned().unwrap_or_default();
        if let Some(dir) = data_dir.map(|d| d.join("music-styles")) {
            if let Ok(rd) = std::fs::read_dir(dir) {
                let mut files: Vec<_> = rd.filter_map(Result::ok).map(|e| e.path()).filter(|p| p.extension().is_some_and(|e| e == "json")).collect();
                files.sort();
                for f in files {
                    let Some(v) = std::fs::read_to_string(&f).ok().and_then(|t| serde_json::from_str::<Value>(&t).ok()) else { continue };
                    for s in v["styles"].as_array().cloned().unwrap_or_else(|| vec![v.clone()]) {
                        let Some(id) = s["id"].as_str() else { continue };
                        match list.iter_mut().find(|x| x["id"] == id) {
                            Some(x) => merge(x, &s), // user values override field by field
                            None => list.push(s),
                        }
                    }
                }
            }
        }
        Styles { list }
    }

    pub fn ids(&self) -> Vec<String> {
        self.list.iter().filter_map(|s| s["id"].as_str().map(String::from)).collect()
    }

    pub fn get(&self, id: &str) -> Option<&Value> {
        self.list.iter().find(|s| s["id"] == id)
    }

    pub fn all(&self) -> &[Value] {
        &self.list
    }

    /// Blend styles with weights: numbers are averaged, the other fields come from the heaviest style.
    pub fn blend(&self, parts: &[(String, f64)]) -> Result<Value, String> {
        let mut found: Vec<(&Value, f64)> = vec![];
        for (id, w) in parts {
            found.push((self.get(id).ok_or_else(|| format!("unknown style {id:?} (use {})", self.ids().join(", ")))?, w.max(0.0)));
        }
        if found.is_empty() {
            return Err("no style".into());
        }
        found.sort_by(|a, b| b.1.total_cmp(&a.1));
        let total: f64 = found.iter().map(|x| x.1).sum::<f64>().max(1e-9);
        let mut out = found[0].0.clone();
        let avg = |path: &[&str]| -> Option<f64> {
            let mut s = 0.0;
            for (v, w) in &found {
                s += path.iter().fold(*v, |x, k| &x[*k]).as_f64()? * w;
            }
            Some(s / total)
        };
        for path in [&["bpm", "min"][..], &["bpm", "max"], &["bpm", "default"], &["swing"], &["melody", "notes_per_bar"], &["harmony", "bars_per_chord"]] {
            if let Some(x) = avg(path) {
                let target = path[..path.len() - 1].iter().fold(&mut out, |o, k| &mut o[*k]);
                target[path[path.len() - 1]] = json!((x * 10.0).round() / 10.0);
            }
        }
        // modes: weighted union
        let mut modes = Map::new();
        for (v, w) in &found {
            for (m, x) in v["modes"].as_object().into_iter().flatten() {
                let cur = modes.get(m).and_then(|y| y.as_f64()).unwrap_or(0.0);
                modes.insert(m.clone(), json!(((cur + x.as_f64().unwrap_or(0.0) * w / total) * 100.0).round() / 100.0));
            }
        }
        out["modes"] = Value::Object(modes);
        if found.len() > 1 {
            out["id"] = json!(found.iter().map(|x| x.0["id"].as_str().unwrap_or("")).collect::<Vec<_>>().join("+"));
            out["label"] = json!(found.iter().map(|x| format!("{} {:.0}%", x.0["label"].as_str().unwrap_or(""), x.1 / total * 100.0)).collect::<Vec<_>>().join(" + "));
            out["blended_from"] = json!(found.iter().map(|x| json!({ "style": x.0["id"], "weight": (x.1 / total * 100.0).round() / 100.0 })).collect::<Vec<_>>());
            // the secondary styles' harmonic colour is kept as a mood hint
            out["harmony"]["secondary_flavors"] = json!(found.iter().skip(1).map(|x| x.0["harmony"]["flavor"].clone()).collect::<Vec<_>>());
        }
        Ok(out)
    }
}

fn merge(base: &mut Value, over: &Value) {
    match (base, over) {
        (Value::Object(b), Value::Object(o)) => {
            for (k, v) in o {
                match b.get_mut(k) {
                    Some(x) if x.is_object() && v.is_object() => merge(x, v),
                    _ => {
                        b.insert(k.clone(), v.clone());
                    }
                }
            }
        }
        (b, o) => *b = o.clone(),
    }
}

/// Most likely mode of a style (for a request without a key).
pub fn main_mode(style: &Value) -> String {
    style["modes"].as_object().and_then(|m| m.iter().max_by(|a, b| a.1.as_f64().unwrap_or(0.0).total_cmp(&b.1.as_f64().unwrap_or(0.0))).map(|x| x.0.clone())).unwrap_or_else(|| "natural_minor".into())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn profiles_load_override_and_blend() {
        let s = Styles::load(None);
        for id in ["techno", "melodic_techno", "progressive_house", "deep_house", "house", "tech_house", "minimal", "edm", "trance", "ambient", "cinematic"] {
            let p = s.get(id).unwrap_or_else(|| panic!("{id}"));
            let (lo, hi, d) = (p["bpm"]["min"].as_f64().unwrap(), p["bpm"]["max"].as_f64().unwrap(), p["bpm"]["default"].as_f64().unwrap());
            assert!(lo <= d && d <= hi, "{id}");
            assert!(p["sources"].as_array().is_some_and(|a| !a.is_empty()), "{id} cites a source");
            assert!(crate::music::harmony::flavor(p["harmony"]["flavor"].as_str().unwrap(), &[]).length > 0);
        }
        assert_eq!(s.get("melodic_techno").unwrap()["bpm"]["default"], 124);
        // user override: changes one field, keeps the others
        let dir = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(dir.path().join("music-styles")).unwrap();
        std::fs::write(dir.path().join("music-styles/mine.json"), r#"{"id":"techno","bpm":{"default":135},"label":"My Techno"}"#).unwrap();
        let o = Styles::load(Some(dir.path()));
        assert_eq!((o.get("techno").unwrap()["bpm"]["default"].as_f64(), o.get("techno").unwrap()["bpm"]["min"].as_f64()), (Some(135.0), Some(120.0)));
        // blend: techno 70 % + ambient 30 %
        let b = s.blend(&[("techno".into(), 0.7), ("ambient".into(), 0.3)]).unwrap();
        assert_eq!(b["id"], "techno+ambient");
        assert!((b["bpm"]["default"].as_f64().unwrap() - 115.0).abs() < 0.01, "{}", b["bpm"]);
        assert_eq!(b["harmony"]["secondary_flavors"], json!(["ambient"]));
        assert!(b["modes"]["lydian"].as_f64().unwrap() > 0.0);
        assert!(s.blend(&[("polka".into(), 1.0)]).is_err());
        assert_eq!(main_mode(s.get("deep_house").unwrap()), "dorian");
    }
}
