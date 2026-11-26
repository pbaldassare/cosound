//! Assistant personalization: name, language, communication style, level of detail, interaction mode and operative
//! preferences, stored in COSOUND's settings. The name is the user's name for their assistant: it never replaces the
//! COSOUND brand. Style and detail change how replies are written, never the technical content of an interpretation.
use super::enhance::{element_name, quality_noun};
use rusqlite::Connection;
use serde::Serialize;
use serde_json::{json, Value};

pub const STYLES: [&str; 4] = ["friendly", "professional", "concise", "technical"];
pub const DETAILS: [&str; 3] = ["brief", "normal", "detailed"];
pub const MODES: [&str; 3] = ["proactive", "confirm_first", "step_by_step"];
pub const LANGUAGES: [&str; 3] = ["auto", "it", "en"];

#[derive(Clone, Debug, Serialize)]
pub struct Profile {
    pub name: String,
    pub brand: &'static str,
    /// auto = reply in the user's language
    pub language: String,
    pub style: String,
    pub detail: String,
    pub mode: String,
    pub preferences: Value,
}

fn pick(v: Option<String>, allowed: &[&str], default: &str) -> String {
    v.filter(|x| allowed.contains(&x.as_str())).unwrap_or_else(|| default.to_string())
}

pub fn load(c: &Connection) -> Profile {
    let s = crate::db::settings_all(c).unwrap_or_default();
    let name = s.get("assistant_name").map(|n| n.trim().to_string()).filter(|n| !n.is_empty()).unwrap_or_else(|| "COSOUND".into());
    Profile {
        name,
        brand: "COSOUND",
        language: pick(s.get("assistant_language").cloned(), &LANGUAGES, "auto"),
        style: pick(s.get("assistant_style").cloned(), &STYLES, "friendly"),
        detail: pick(s.get("assistant_detail").cloned(), &DETAILS, "normal"),
        mode: pick(s.get("assistant_mode").cloned(), &MODES, "proactive"),
        preferences: s.get("assistant_preferences").and_then(|p| serde_json::from_str(p).ok()).unwrap_or_else(|| json!({})),
    }
}

/// Validate and store one profile field. The name: 1–40 visible characters.
pub fn set(c: &Connection, field: &str, value: &str) -> Result<Profile, String> {
    let (key, ok) = match field {
        "name" => {
            let v = value.trim();
            ("assistant_name", !v.is_empty() && v.chars().count() <= 40 && !v.chars().any(|ch| ch.is_control()))
        }
        "language" => ("assistant_language", LANGUAGES.contains(&value)),
        "style" => ("assistant_style", STYLES.contains(&value)),
        "detail" => ("assistant_detail", DETAILS.contains(&value)),
        "mode" => ("assistant_mode", MODES.contains(&value)),
        "preferences" => ("assistant_preferences", serde_json::from_str::<Value>(value).is_ok_and(|v| v.is_object())),
        _ => return Err(format!("unknown field {field}")),
    };
    if !ok {
        return Err(format!("invalid value for {field}"));
    }
    crate::db::set_setting(c, key, value.trim()).map_err(|e| e.to_string())?;
    Ok(load(c))
}

/// The reply the assistant shows: what was understood, defaults declared, the question if one is needed.
/// `it` = Italian. Technical content is the same for every style.
pub fn reply(p: &Profile, it: bool, i: &Value) -> String {
    let mut parts: Vec<String> = vec![];
    let concise = p.style == "concise" || p.detail == "brief";
    let act = i["conversation"]["act"].as_str().unwrap_or("");
    if i["clarification"]["needed"] == true {
        let q = i["clarification"]["question"].as_str().unwrap_or("");
        let lower = q.chars().next().map(|f| f.to_lowercase().collect::<String>() + &q[f.len_utf8()..]).unwrap_or_default();
        return if concise || p.style == "technical" { q.to_string() } else if it { format!("Prima di procedere, {lower}") } else { format!("Before I go ahead, {lower}") };
    }
    let errors: Vec<&str> = i["issues"].as_array().into_iter().flatten().filter(|x| x["severity"] == "error").filter_map(|x| x["message"].as_str()).collect();
    if !errors.is_empty() {
        let sug: Vec<&str> = i["issues"].as_array().into_iter().flatten().filter(|x| x["severity"] == "error").filter_map(|x| x["suggestion"].as_str()).collect();
        return format!("{}{}", errors.join(" "), if sug.is_empty() { String::new() } else { format!(" {}", sug.join(" ")) });
    }
    // no cheerful "Ok!" before something that needs the user's confirmation
    let opener = match (p.style.as_str(), it) {
        _ if i["approval"]["required"] == true => "",
        ("friendly", true) => "Ok! ",
        ("friendly", false) => "Got it! ",
        _ => "",
    };
    match act {
        "undo" => return format!("{opener}{}", if it { "Annullo l'ultima modifica." } else { "Undoing the last change." }),
        "restore" => return format!("{opener}{}", if it { "Ripristino la versione precedente." } else { "Restoring the earlier version." }),
        "confirm" => return format!("{opener}{}", if it { "Tengo questa versione." } else { "Keeping this version." }),
        "cancel" => return if it { "Va bene, non faccio nulla.".into() } else { "Fine, I won't do anything.".into() },
        "smalltalk" => return if it { format!("Ciao! Sono {}. Dimmi cosa vuoi creare o modificare.", p.name) } else { format!("Hi! I'm {}. Tell me what you want to create or change.", p.name) },
        _ => {}
    }
    let goal = i["goal"]["interpretation"].as_str().unwrap_or("");
    if p.style == "technical" {
        parts.push(format!("intent={} goal={} targets={}", i["primary_intent"].as_str().unwrap_or("-"), i["goal"]["id"].as_str().unwrap_or("-"),
            i["targets"].as_array().map(|t| t.iter().filter_map(|x| x["track_name"].as_str().or(x["element"].as_str())).collect::<Vec<_>>().join(",")).unwrap_or_default()));
    } else {
        parts.push(format!("{opener}{goal}"));
    }
    let cons: Vec<String> = i["constraints"].as_array().into_iter().flatten().map(|c| {
        let s = element_name(c["subject"].as_str().unwrap_or(""), it);
        match (c["kind"].as_str().unwrap_or(""), it) {
            ("preserve", true) => format!("non tocco {s}"),
            ("preserve", false) => format!("I won't touch {s}"),
            ("only", true) => format!("cambio solo {s}"),
            ("only", false) => format!("I change only {s}"),
            ("exclude", true) => format!("senza {s}"),
            ("exclude", false) => format!("without {s}"),
            ("resource_restriction", true) => "uso solo le tue risorse".into(),
            ("resource_restriction", false) => "I use only your resources".into(),
            ("no_delete", true) => "non elimino nulla".into(),
            ("no_delete", false) => "I won't delete anything".into(),
            ("no_overwrite", true) => "lavoro su una copia, l'originale resta".into(),
            (_, _) => "I work on a copy, the original stays".into(),
        }
    }).collect();
    if !cons.is_empty() {
        parts.push(format!("{}: {}.", if it { "Vincoli" } else { "Constraints" }, cons.join("; ")));
    }
    if !concise {
        let defaults: Vec<String> = i["defaults_used"].as_array().into_iter().flatten().map(|d| {
            let v = d["value"].as_str().map(String::from).unwrap_or_else(|| d["value"].to_string());
            let v = if it && d["name"] == "key" { crate::music::keys::parse_key(&v).map(crate::music::keys::key_name_it).unwrap_or(v) } else { v };
            let n = match (d["name"].as_str().unwrap_or(""), it) { ("key", true) => "tonalità", ("bpm", _) => "BPM", (x, _) => x };
            format!("{n} {v}")
        }).collect();
        if !defaults.is_empty() {
            parts.push(format!("{} {}.", if it { "Uso come base:" } else { "Starting from:" }, defaults.join(", ")));
        }
        for w in i["issues"].as_array().into_iter().flatten().filter(|x| x["severity"] == "warning") {
            parts.push(w["message"].as_str().unwrap_or("").to_string());
        }
    }
    if p.detail == "detailed" {
        if let Some(s) = i["enhancement"]["semantic"].as_str().filter(|s| !s.is_empty()) {
            parts.push(s.to_string());
        }
    }
    if i["approval"]["required"] == true {
        parts.push(if it { "Ti chiederò conferma prima di applicare le modifiche.".into() } else { "I'll ask you to confirm before applying changes.".into() });
    }
    parts.join(" ")
}

/// Short goal sentence in the user's language.
/// Italian articulated prepositions: "di il basso" → "del basso", "in il drop" → "nel drop", "a la" → "alla".
pub fn contract(s: &str) -> String {
    let mut out = format!(" {s} ");
    for (prep, forms) in [("di", ["del", "dello", "della", "dell'", "dei", "degli", "delle"]), ("in", ["nel", "nello", "nella", "nell'", "nei", "negli", "nelle"]), ("a", ["al", "allo", "alla", "all'", "ai", "agli", "alle"]), ("su", ["sul", "sullo", "sulla", "sull'", "sui", "sugli", "sulle"])] {
        for (art, form) in ["il ", "lo ", "la ", "l'", "i ", "gli ", "le "].iter().zip(forms) {
            let sep = if art.ends_with('\'') { "" } else { " " };
            out = out.replace(&format!(" {prep} {art}"), &format!(" {form}{sep}"));
        }
    }
    out.trim().to_string()
}

pub fn goal_sentence(it: bool, verb: &str, target: Option<String>, qualities: &[(String, f64, bool)], extra: &str) -> String {
    let q: Vec<String> = qualities.iter().filter(|q| !q.2).map(|q| format!("{}{}", if q.1 < 0.0 { if it { "meno " } else { "less " } } else if it { "più " } else { "more " }, quality_noun(&q.0, it))).collect();
    let target = target.unwrap_or_default();
    let mut s = format!("{verb} {target}").trim().to_string();
    if !q.is_empty() {
        s = format!("{s}: {}", q.join(", "));
    }
    if it {
        s = contract(&s);
    }
    if !extra.is_empty() {
        s = format!("{s} {extra}");
    }
    let mut c = s.chars();
    match c.next() {
        Some(f) => format!("{}{}.", f.to_uppercase(), c.as_str()),
        None => String::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn italian_prepositions() {
        assert_eq!(contract("Modifico il suono di il basso in il drop"), "Modifico il suono del basso nel drop");
        assert_eq!(contract("Lavoro sull'arrangiamento di l'intro"), "Lavoro sull'arrangiamento dell'intro");
        assert_eq!(contract("Abbasso il volume di gli hi-hat"), "Abbasso il volume degli hi-hat");
    }

    #[test]
    fn name_style_and_brand() {
        let c = crate::db::open_memory();
        assert_eq!(load(&c).name, "COSOUND");
        let p = set(&c, "name", "Nova").unwrap();
        assert_eq!((p.name.as_str(), p.brand), ("Nova", "COSOUND"), "the name never replaces the brand");
        assert!(set(&c, "name", "   ").is_err() && set(&c, "name", &"x".repeat(41)).is_err() && set(&c, "style", "rude").is_err());
        let p = set(&c, "style", "concise").unwrap();
        assert_eq!(p.style, "concise");
        assert_eq!(set(&c, "mode", "confirm_first").unwrap().mode, "confirm_first");
        assert!(reply(&p, true, &json!({ "conversation": { "act": "smalltalk" } })).contains("Nova"));
    }
}
