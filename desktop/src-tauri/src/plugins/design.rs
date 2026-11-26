//! Plugin Control Interface + Preset Management for sound design.
//! A *design* is a render spec (instrument, effect chain, parameter values, automation, notes) stored in
//! `plugin_designs`. Requests in words ("più aggressivo ma caldo e profondo") become parameter changes on the
//! parameters the plug-ins really expose (roles from parameter discovery). Every change is a proposal with
//! before/after values and a reason; it is applied only in COSOUND's isolated host for audition, measured, and
//! saved as a NEW preset file. Original presets and plug-ins are never modified.
use super::{host, params as pd, registry};
use crate::{memory, music, sound};
use rusqlite::{params, Connection};
use serde_json::{json, Value};
use std::path::{Path, PathBuf};
use std::sync::atomic::AtomicBool;

/// Descriptor → parameter moves (role, change in normalized range). Sources: docs/knowledge/synthesizers §3
/// (variants: brighter/darker/…), plugins-effects §1–4, audio-engineering/mixing §6, §9.
const MOVES: [(&str, &[(&str, f64)]); 13] = [
    ("aggressive", &[("drive", 0.25), ("drive_mix", 0.15), ("resonance", 0.10), ("cutoff", 0.10), ("attack", -0.10)]),
    ("warm", &[("cutoff", -0.04), ("drive", 0.05), ("resonance", -0.05)]),
    ("deep", &[("cutoff", -0.05), ("octave", -1.0), ("sub_level", 0.2)]),
    ("bright", &[("cutoff", 0.15), ("resonance", 0.05)]),
    ("dark", &[("cutoff", -0.15), ("resonance", -0.05)]),
    ("soft", &[("attack", 0.15), ("drive", -0.20), ("drive_mix", -0.15), ("cutoff", -0.08)]),
    ("punchy", &[("attack", -0.20), ("decay", -0.10), ("drive", 0.05)]),
    ("wide", &[("detune", 0.15), ("unison", 0.20), ("width", 0.20), ("space_mix", 0.05)]),
    ("atmospheric", &[("space_mix", 0.20), ("reverb_decay", 0.20), ("release", 0.20), ("attack", 0.10)]),
    ("movement", &[("lfo_depth", 0.20), ("lfo_rate", -0.10)]),
    ("clean", &[("drive", -0.30), ("drive_mix", -0.25), ("resonance", -0.10)]),
    ("long", &[("release", 0.20), ("decay", 0.10), ("reverb_decay", 0.15)]),
    ("short", &[("release", -0.20), ("decay", -0.10), ("reverb_decay", -0.15)]),
];

const WORDS: [(&str, &[&str]); 13] = [
    ("aggressive", &["aggressivo", "aggressiva", "aggressivi", "aggressive", "cattivo", "incisivo", "duro", "grintoso", "potente", "powerful", "hard"]),
    ("warm", &["caldo", "calda", "caldi", "warm", "warmer"]),
    ("deep", &["profondo", "profonda", "profondi", "deep", "deeper", "grave"]),
    ("bright", &["brillante", "brillanti", "bright", "brighter", "luminoso", "aperto"]),
    ("dark", &["scuro", "scura", "dark", "darker", "cupo"]),
    ("soft", &["morbido", "morbida", "soft", "softer", "dolce", "delicato"]),
    ("punchy", &["punchy", "deciso", "secco", "tight"]),
    ("wide", &["largo", "larga", "ampio", "ampia", "wide", "wider", "stereo"]),
    ("atmospheric", &["atmosferico", "atmosferica", "atmospheric", "spazioso", "cinematografico", "cinematografica", "cinematic", "ambient", "riverberato"]),
    ("movement", &["movimento", "movement", "evolving", "evolve", "evolutivo", "lfo"]),
    ("clean", &["pulito", "pulita", "clean", "cleaner"]),
    ("long", &["lungo", "lunga", "long", "sostenuto"]),
    ("short", &["corto", "corta", "short", "breve"]),
];

/// Words of a request → (descriptor, weight). "più/more" +1, "meno/less" −1, "molto/very" ×1.5,
/// "leggermente/slightly/un po'" ×0.5; "mantieni/keep" marks the next descriptors as qualities to preserve.
pub fn parse_request(text: &str) -> Vec<(String, f64, bool)> {
    let t = text.to_lowercase().replace('\'', " ");
    let words: Vec<&str> = t.split(|c: char| !c.is_alphanumeric()).filter(|w| !w.is_empty()).collect();
    let (mut sign, mut scale, mut keep) = (1.0, 1.0, false);
    let mut out: Vec<(String, f64, bool)> = vec![];
    for (i, w) in words.iter().enumerate() {
        match *w {
            "più" | "piu" | "more" | "add" | "aggiungi" => sign = 1.0,
            "meno" | "less" | "togli" | "reduce" | "riduci" => sign = -1.0,
            "molto" | "very" | "much" | "decisamente" => scale = 1.5,
            "leggermente" | "slightly" | "poco" | "bit" | "little" => scale = 0.5,
            "mantieni" | "mantienilo" | "mantienila" | "mantenendo" | "keep" | "keeping" | "conserva" | "resta" | "rimanga" => keep = true,
            "ma" | "but" if words.get(i + 1).is_some_and(|n| !n.starts_with("mant") && *n != "keep") => {
                sign = 1.0;
                scale = 1.0;
            }
            _ => {
                if let Some((d, _)) = WORDS.iter().find(|(_, ws)| ws.contains(w)) {
                    if !out.iter().any(|x| x.0 == *d) {
                        out.push((d.to_string(), sign * scale, keep));
                    }
                    scale = 1.0;
                }
            }
        }
    }
    out
}

fn log_scale(unit: &str, min: f64, max: f64) -> bool {
    unit == "hertz" && min > 0.0 && max / min > 20.0
}

/// Musical range of a parameter: for decibels, the top 48 dB (a −80…+20 dB gain moved "a quarter" must not land
/// at −55 dB, where it does nothing).
fn musical(min: f64, max: f64, unit: &str) -> (f64, f64) {
    if unit == "decibels" && max - min > 48.0 {
        (max - 48.0, max)
    } else {
        (min, max)
    }
}

fn to_norm(v: f64, min: f64, max: f64, unit: &str) -> f64 {
    let (min, max) = musical(min, max, unit);
    if max <= min {
        return 0.0;
    }
    if log_scale(unit, min, max) {
        ((v.max(min) / min).ln() / (max / min).ln()).clamp(0.0, 1.0)
    } else {
        ((v - min) / (max - min)).clamp(0.0, 1.0)
    }
}

fn from_norm(n: f64, min: f64, max: f64, unit: &str) -> f64 {
    let (min, max) = musical(min, max, unit);
    let n = n.clamp(0.0, 1.0);
    if log_scale(unit, min, max) {
        min * (max / min).powf(n)
    } else {
        min + n * (max - min)
    }
}

/// Role used for moves: a plug-in's dry/wet means "how much drive" in a distortion and "how much space" in a
/// reverb/delay/chorus.
fn effective_role(role: Option<&str>, unit_categories: &[String]) -> Option<String> {
    let r = role?;
    if r == "dry_wet" {
        if unit_categories.iter().any(|c| c == "distortion" || c == "saturation") {
            return Some("drive_mix".into());
        }
        if unit_categories.iter().any(|c| ["reverb", "delay", "chorus", "flanger", "phaser"].contains(&c.as_str())) {
            return Some("space_mix".into());
        }
        return None;
    }
    Some(r.to_string())
}

/// The units of a spec with their stored parameters and categories (instrument first, then the chain).
fn units(c: &Connection, spec: &Value) -> Vec<(usize, Value, Vec<Value>, Vec<String>)> {
    let mut list: Vec<Value> = spec["instrument"].is_object().then(|| spec["instrument"].clone()).into_iter().collect();
    list.extend(spec["chain"].as_array().cloned().unwrap_or_default());
    list.into_iter()
        .enumerate()
        .map(|(i, u)| {
            let iid = u["install_id"].as_str().unwrap_or("").to_string();
            let iid = iid.as_str();
            let pid = registry::install(c, iid).and_then(|x| x["product_id"].as_str().map(String::from)).unwrap_or_default();
            let cats = registry::categories_of(c, &pid).iter().filter_map(|x| x["category"].as_str().map(String::from)).collect();
            (i, u, pd::stored(c, iid), cats)
        })
        .collect()
}

fn unit_mut<'a>(spec: &'a mut Value, slot: usize) -> &'a mut Value {
    if spec["instrument"].is_object() {
        if slot == 0 {
            return &mut spec["instrument"];
        }
        &mut spec["chain"][slot - 1]
    } else {
        &mut spec["chain"][slot]
    }
}

/// Current value of a parameter in a spec: explicit value, else the value read from the plug-in (default or preset).
fn current(u: &Value, p: &Value) -> f64 {
    let addr = p["address"].as_i64().unwrap_or(-1);
    u["params"].as_array().and_then(|a| a.iter().find(|x| x["address"].as_i64() == Some(addr))).and_then(|x| x["value"].as_f64()).or_else(|| u["state_values"][addr.to_string()].as_f64()).or(p["default"].as_f64()).unwrap_or(0.0)
}

/// Proposed parameter changes for a request, on the parameters that really exist. Nothing is applied here.
pub fn propose(c: &Connection, spec: &Value, request: &str) -> (Value, Vec<Value>, Vec<String>) {
    let wanted = parse_request(request);
    let mut new_spec = spec.clone();
    let mut changes = vec![];
    let mut missing = vec![];
    let us = units(c, spec);
    // qualities to keep ("mantienilo caldo") do not move anything: they limit the moves that go against them
    let kept: Vec<(&str, f64, &str)> = wanted
        .iter()
        .filter(|w| w.2)
        .filter_map(|w| MOVES.iter().find(|(d, _)| *d == w.0).map(|(d, m)| (*d, m)))
        .flat_map(|(d, m)| m.iter().map(move |(r, v)| (*r, *v, d)))
        .collect();
    for (desc, weight, keep) in &wanted {
        if *keep {
            continue;
        }
        let Some((_, moves)) = MOVES.iter().find(|(d, _)| d == desc) else { continue };
        let mut any = false;
        for (role, delta) in moves.iter() {
            let mut hit = false;
            for (slot, u, ps, cats) in &us {
                for p in ps.iter().filter(|p| p["writable"].as_bool() == Some(true)) {
                    if effective_role(p["role"].as_str(), cats).as_deref() != Some(*role) {
                        continue;
                    }
                    let (min, max, unit) = (p["min"].as_f64().unwrap_or(0.0), p["max"].as_f64().unwrap_or(1.0), p["unit"].as_str().unwrap_or(""));
                    let addr = p["address"].as_i64().unwrap_or(0);
                    let target = unit_mut(&mut new_spec, *slot);
                    let before = current(target, p);
                    let opposed: Vec<&str> = kept.iter().filter(|(r, v, _)| r == role && v.signum() != (delta * weight).signum()).map(|k| k.2).collect();
                    let limit = if opposed.is_empty() { 1.0 } else { 0.3 };
                    let after = if *role == "octave" && unit == "semitones" {
                        if opposed.is_empty() { (before + 12.0 * delta * weight).clamp(min, max) } else { before }
                    } else {
                        from_norm(to_norm(before, min, max, unit) + delta * weight * limit, min, max, unit)
                    };
                    if (after - before).abs() < 1e-6 {
                        continue;
                    }
                    let list = target["params"].as_array_mut();
                    let mut params_list = list.map(|l| l.clone()).unwrap_or_default();
                    match params_list.iter_mut().find(|x| x["address"].as_i64() == Some(addr)) {
                        Some(x) => x["value"] = json!(after),
                        None => params_list.push(json!({ "address": addr, "value": after })),
                    }
                    target["params"] = json!(params_list);
                    let reason = format!("{}{desc}{}", if *weight < 0.0 { "less " } else { "" }, if opposed.is_empty() { String::new() } else { format!(" (limited to keep it {})", opposed.join(", ")) });
                    match changes.iter_mut().find(|x: &&mut Value| x["slot"] == *slot && x["address"] == addr) {
                        Some(x) => {
                            x["to"] = json!(after);
                            x["reasons"].as_array_mut().unwrap().push(json!(reason));
                        }
                        None => changes.push(json!({ "slot": slot, "unit": u["name"], "address": addr, "parameter": p["name"], "role": role, "unit_of_measure": unit, "from": before, "to": after, "reasons": [reason] })),
                    }
                    hit = true;
                }
            }
            any |= hit;
            if !hit && !matches!(*role, "drive_mix" | "space_mix" | "sub_level" | "width" | "unison" | "detune") {
                missing.push(format!("{desc}: no '{role}' parameter is exposed by these plug-ins"));
            }
        }
        if !any {
            missing.push(format!("{desc}: nothing to change on the parameters available"));
        }
    }
    if wanted.iter().all(|w| w.2) {
        missing.push("no sound quality recognized in the request (e.g. aggressive, warm, deep, bright, dark, soft, wide, atmospheric, movement)".into());
    }
    changes.retain(|x| (x["to"].as_f64().unwrap_or(0.0) - x["from"].as_f64().unwrap_or(0.0)).abs() > 1e-6);
    (new_spec, changes, missing)
}

/// GM programs for Apple's DLS synth when it is the only instrument available (General MIDI numbering, 0-based).
fn gm_program(role: &str) -> u8 {
    match role {
        "bass" | "sub" | "acid" => 38,
        "lead" => 81,
        "pad" | "atmosphere" | "texture" => 89,
        "pluck" | "arp" => 84,
        "keys" | "chords" => 4,
        _ => 81,
    }
}

/// Notes for the audition, generated by the Music Intelligence engine (key, genre, tempo); sound and notes stay
/// separate responsibilities: this module only plays them.
pub fn notes_for(role: &str, genre: Option<&str>, key: Option<&str>, bpm: Option<f64>, bars: u32) -> (Vec<Value>, f64, Value) {
    let mut input = json!({ "bars": bars });
    if let Some(g) = genre {
        input["genre"] = json!(g);
    }
    if let Some(k) = key {
        input["key"] = json!(k);
    }
    if let Some(b) = bpm {
        input["bpm"] = json!(b);
    }
    let parsed = music::caps::parse(&input).or_else(|_| music::caps::parse(&json!({ "bars": bars })));
    let Ok(i) = parsed else { return (vec![json!({ "pitch": 45, "start_s": 0.0, "dur_s": 1.0 })], 2.0, json!(null)) };
    let spec = i.spec();
    let part = match role {
        "bass" | "sub" | "acid" => music::generate::bass(&spec, &i.bass_style),
        "pad" | "chords" | "keys" | "atmosphere" | "texture" => music::generate::chords(&spec, "sustain"),
        _ => Ok(music::generate::melody(&spec)),
    };
    let Ok(part) = part else { return (vec![], 2.0, json!(null)) };
    let sec = |ticks: u32| ticks as f64 / music::generate::PPQ as f64 * 60.0 / i.bpm;
    let notes: Vec<Value> = part.notes.iter().map(|n| json!({ "pitch": n.pitch, "start_s": sec(n.start), "dur_s": sec(n.dur), "vel": n.vel })).collect();
    let len = sec(bars * music::generate::BAR) + 1.5;
    (notes, len, json!({ "key": i.key.name(), "bpm": i.bpm, "bars": bars, "part": part.name, "engine": "music::generate" }))
}

/// Instrument candidates for a role: detected, hostable instruments first; the user's remembered choice wins.
pub fn instrument_candidates(c: &Connection, role: &str) -> Vec<Value> {
    let preferred = memory::list(c, Some("plugin")).unwrap_or_default().into_iter().find(|f| f.key == format!("preferred_instrument:{role}")).and_then(|f| f.value["product_id"].as_str().map(String::from));
    let mut out: Vec<(i32, Value)> = vec![];
    for p in registry::list(c, "", Some("instrument"), None, false) {
        let pid = p["id"].as_str().unwrap_or("");
        let installs = registry::installs_of(c, pid);
        let hostable = installs.iter().find(|i| i["au_code"].is_string() && i["status"] == "available" && host::available());
        let name = p["name"].as_str().unwrap_or("");
        let known = sound::known_synth(name);
        let cats: Vec<String> = p["categories"].as_array().unwrap().iter().filter_map(|x| x.as_str().map(String::from)).collect();
        let mut score = 0;
        let mut why = vec![];
        if preferred.as_deref() == Some(pid) {
            score += 100;
            why.push("your usual instrument for this role (memory)".to_string());
        }
        if let Some(k) = known {
            score += 20;
            why.push(format!("{} synth (knowledge base)", k.synthesis));
        }
        if cats.iter().any(|c| c == "synth") {
            score += 10;
            why.push("synthesizer".into());
        }
        if (role == "bass" && cats.iter().any(|c| c == "bass")) || (role == "keys" && cats.iter().any(|c| c == "piano")) {
            score += 10;
            why.push(format!("{role} instrument"));
        }
        if hostable.is_some() {
            score += 5;
        } else {
            why.push("COSOUND cannot load this format yet: suggestions only, no parameter control".into());
        }
        if installs.iter().any(|i| i["au_code"] == "aumu:dls :appl") {
            score += 3;
            why.push("General MIDI sound set: the sound is chosen by program change".into());
        }
        if installs.iter().any(|i| i["scope"] == "apple") {
            score -= 15;
            why.push("Apple built-in General MIDI / sampler instrument (basic sounds)".into());
        }
        out.push((score, json!({ "product_id": pid, "name": name, "vendor": p["vendor"], "categories": cats, "install_id": hostable.map(|i| i["id"].clone()), "controllable": hostable.is_some(), "why": why })));
    }
    out.sort_by(|a, b| b.0.cmp(&a.0));
    out.into_iter().map(|x| x.1).collect()
}

/// Installed, hostable effect for a category (first by confidence).
fn effect_for(c: &Connection, category: &str) -> Option<(String, String, String)> {
    c.query_row(
        "select i.id, i.name, i.au_code from plugin_installs i join plugin_categories pc on pc.product_id = i.product_id
         where pc.category = ?1 and i.status = 'available' and i.au_code is not null and i.au_code like 'aufx:%' order by pc.confidence desc, i.scope = 'apple' limit 1",
        [category],
        |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
    )
    .ok()
}

fn unit_spec(c: &Connection, install_id: &str) -> Value {
    let i = registry::install(c, install_id).unwrap_or(json!({}));
    let code: Vec<String> = i["au_code"].as_str().unwrap_or("").split(':').map(String::from).collect();
    json!({ "install_id": install_id, "name": i["name"], "vendor": i["vendor"], "type": code.first(), "subtype": code.get(1), "manufacturer": code.get(2), "params": [] })
}

/// Make sure parameters of every unit are known (discovery is run once per install).
fn ensure_params(c: &Connection, install_id: &str) -> Result<(), String> {
    if pd::stored(c, install_id).is_empty() {
        pd::discover(c, install_id, 30)?;
    }
    Ok(())
}

/// Starting values per role for a recipe (normalized 0–1 of each parameter's range), from the generic recipes in
/// docs/knowledge/synthesizers §3; the effects a recipe needs when the instrument lacks the control.
fn recipe_targets(role: &str) -> (Vec<(&'static str, f64)>, Vec<&'static str>) {
    match role {
        "bass" | "sub" | "acid" => (vec![("cutoff", 0.55), ("resonance", 0.35), ("attack", 0.0), ("drive", 0.25), ("drive_mix", 0.20)], vec!["filter", "distortion"]),
        "lead" => (vec![("cutoff", 0.75), ("resonance", 0.30), ("drive", 0.20), ("drive_mix", 0.15), ("space_mix", 0.20)], vec!["distortion", "delay", "reverb"]),
        "pad" | "atmosphere" | "texture" | "chords" => (vec![("cutoff", 0.50), ("resonance", 0.25), ("attack", 0.60), ("release", 0.70), ("space_mix", 0.35), ("reverb_decay", 0.70)], vec!["filter", "reverb"]),
        "pluck" | "arp" => (vec![("cutoff", 0.55), ("resonance", 0.35), ("attack", 0.0), ("decay", 0.25), ("space_mix", 0.20)], vec!["filter", "delay"]),
        _ => (vec![("cutoff", 0.6)], vec!["reverb"]),
    }
}

/// Create a sound design: instrument choice, effects the recipe needs, starting values, notes from the music
/// engine, then the request's qualities as parameter changes. Returns the stored design (status "proposed").
#[allow(clippy::too_many_arguments)]
pub fn create(c: &Connection, request: &str, role: &str, genre: Option<&str>, key: Option<&str>, bpm: Option<f64>, product_id: Option<&str>) -> Result<Value, String> {
    let cands = instrument_candidates(c, role);
    let chosen = match product_id {
        Some(p) => cands.iter().find(|x| x["product_id"] == p).cloned().ok_or("that instrument is not detected on this computer")?,
        None => cands.iter().find(|x| x["controllable"] == true).cloned().ok_or("no instrument that COSOUND can load was detected (Audio Units on macOS)")?,
    };
    let iid = chosen["install_id"].as_str().ok_or("this instrument cannot be loaded by COSOUND (format not hostable): use it from the DAW")?.to_string();
    ensure_params(c, &iid)?;
    let mut spec = json!({ "instrument": unit_spec(c, &iid), "chain": [], "seconds": 6.0 });
    let is_dls = registry::install(c, &iid).is_some_and(|i| i["au_code"] == "aumu:dls :appl");
    if is_dls {
        spec["program"] = json!(gm_program(role));
    }
    let (targets, fx_needed) = recipe_targets(role);
    let inst_roles: Vec<String> = pd::stored(c, &iid).iter().filter_map(|p| p["role"].as_str().map(String::from)).collect();
    let mut notes_used = vec![];
    for fx in fx_needed {
        let covered = match fx {
            "filter" => inst_roles.iter().any(|r| r == "cutoff"),
            "distortion" => inst_roles.iter().any(|r| r == "drive"),
            _ => false,
        };
        if covered {
            continue;
        }
        let cat = if fx == "filter" { "filter" } else { fx };
        let found = effect_for(c, cat).filter(|(_, name, _)| fx != "filter" || name.to_lowercase().contains("low") || !name.contains("AU"));
        let found = found.or_else(|| if fx == "filter" { c.query_row("select id, name, au_code from plugin_installs where au_code = 'aufx:lpas:appl' and status = 'available'", [], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?))).ok() } else { None });
        match found {
            Some((id, name, _)) => {
                ensure_params(c, &id)?;
                spec["chain"].as_array_mut().unwrap().push(unit_spec(c, &id));
                notes_used.push(format!("{fx}: {name}"));
            }
            None => notes_used.push(format!("{fx}: no installed effect that COSOUND can load")),
        }
    }
    // starting values from the recipe
    let us = units(c, &spec);
    for (slot, _, ps, cats) in &us {
        for (role_t, n) in &targets {
            for p in ps.iter().filter(|p| p["writable"] == true && effective_role(p["role"].as_str(), cats).as_deref() == Some(*role_t)) {
                let (min, max, unit) = (p["min"].as_f64().unwrap_or(0.0), p["max"].as_f64().unwrap_or(1.0), p["unit"].as_str().unwrap_or(""));
                let u = unit_mut(&mut spec, *slot);
                let mut l = u["params"].as_array().cloned().unwrap_or_default();
                l.push(json!({ "address": p["address"], "value": from_norm(*n, min, max, unit) }));
                u["params"] = json!(l);
            }
        }
    }
    // progressive filter opening ("apertura progressiva del filtro", "filter opening", slow movement)
    let lower = request.to_lowercase();
    let (notes, seconds, music_info) = notes_for(role, genre, key, bpm, 2);
    spec["notes"] = json!(notes);
    spec["seconds"] = json!(seconds.min(20.0));
    let mut automation_note = None;
    if ["apertura", "opening", "si apre", "open up", "sweep", "progressiv"].iter().any(|w| lower.contains(w)) {
        for (slot, _, ps, cats) in &units(c, &spec) {
            if let Some(p) = ps.iter().find(|p| p["writable"] == true && effective_role(p["role"].as_str(), cats).as_deref() == Some("cutoff")) {
                let (min, max, unit) = (p["min"].as_f64().unwrap_or(0.0), p["max"].as_f64().unwrap_or(1.0), p["unit"].as_str().unwrap_or(""));
                let u = unit_mut(&mut spec, *slot);
                u["automation"] = json!([{ "address": p["address"], "from": from_norm(0.2, min, max, unit), "to": from_norm(0.65, min, max, unit), "start_s": 0.0, "end_s": seconds.min(20.0) - 1.0 }]);
                automation_note = Some(format!("cutoff of {} opens from {:.0} to {:.0} over the phrase", u["name"].as_str().unwrap_or(""), from_norm(0.2, min, max, unit), from_norm(0.65, min, max, unit)));
                break;
            }
        }
    }
    let base = spec.clone();
    let (designed, changes, missing) = propose(c, &spec, request);
    let id = uuid::Uuid::new_v4().to_string();
    let title = format!("{} – {}", role, chosen["name"].as_str().unwrap_or(""));
    c.execute(
        "insert into plugin_designs(id, kind, title, request, spec, base_spec, changes, status, files) values (?1, 'sound', ?2, ?3, ?4, ?5, ?6, 'proposed', '[]')",
        params![id, title, request, designed.to_string(), base.to_string(), json!(changes).to_string()],
    )
    .map_err(|e| e.to_string())?;
    let mut d = get(c, &id).unwrap();
    d["instrument_choice"] = chosen;
    d["alternatives"] = json!(cands.into_iter().skip(1).take(5).collect::<Vec<_>>());
    d["effects_added"] = json!(notes_used);
    d["automation"] = json!(automation_note);
    d["not_available"] = json!(missing);
    d["music"] = music_info;
    d["recipe"] = json!(sound::recipes_for(role, genre).first().map(|r| r.id));
    Ok(d)
}

/// A further change on an existing design: the current spec becomes the "before".
pub fn modify(c: &Connection, id: &str, request: &str) -> Result<Value, String> {
    let d = get(c, id).ok_or("unknown design")?;
    let (spec, changes, missing) = propose(c, &d["spec"], request);
    c.execute(
        "update plugin_designs set base_spec = spec, spec = ?2, changes = ?3, request = ?4, status = 'proposed', updated_at = datetime('now') where id = ?1",
        params![id, spec.to_string(), json!(changes).to_string(), request],
    )
    .map_err(|e| e.to_string())?;
    let mut d = get(c, id).unwrap();
    d["not_available"] = json!(missing);
    Ok(d)
}

pub fn get(c: &Connection, id: &str) -> Option<Value> {
    c.query_row("select id, kind, title, request, spec, base_spec, changes, status, files, created_at, updated_at from plugin_designs where id = ?1", [id], |r| {
        let j = |i: usize| -> rusqlite::Result<Value> { Ok(r.get::<_, Option<String>>(i)?.and_then(|s| serde_json::from_str(&s).ok()).unwrap_or(Value::Null)) };
        Ok(json!({ "id": r.get::<_, String>(0)?, "kind": r.get::<_, String>(1)?, "title": r.get::<_, String>(2)?, "request": r.get::<_, Option<String>>(3)?,
                   "spec": j(4)?, "base_spec": j(5)?, "changes": j(6)?, "status": r.get::<_, String>(7)?, "files": j(8)?, "created_at": r.get::<_, String>(9)?, "updated_at": r.get::<_, String>(10)? }))
    })
    .ok()
}

pub fn list(c: &Connection) -> Vec<Value> {
    let ids: Vec<String> = c.prepare("select id from plugin_designs order by updated_at desc limit 100").unwrap().query_map([], |r| r.get(0)).map(|m| m.filter_map(Result::ok).collect()).unwrap_or_default();
    ids.iter().filter_map(|i| get(c, i)).map(|mut d| {
        d["spec"] = Value::Null;
        d["base_spec"] = Value::Null;
        d
    }).collect()
}

/// Host request for a spec.
pub fn render_request(spec: &Value, out: &Path) -> Value {
    let strip = |u: &Value| json!({ "type": u["type"], "subtype": u["subtype"], "manufacturer": u["manufacturer"], "params": u["params"], "automation": u["automation"], "state_file": u["state_file"], "factory_preset": u["factory_preset"] });
    let mut r = json!({ "seconds": spec["seconds"].as_f64().unwrap_or(4.0), "sample_rate": 48000, "out": out, "notes": spec["notes"], "chain": spec["chain"].as_array().map(|a| a.iter().map(strip).collect::<Vec<_>>()).unwrap_or_default() });
    if spec["instrument"].is_object() {
        r["instrument"] = strip(&spec["instrument"]);
    }
    if let Some(p) = spec["program"].as_u64() {
        r["program"] = json!(p);
    }
    if let Some(f) = spec["input_file"].as_str() {
        r["input_file"] = json!(f);
    }
    r
}

fn measure(path: &Path) -> Result<Value, String> {
    let a = crate::smartlib::analyze::analyze_file(path, "audio", "render", vec![], &AtomicBool::new(false))?;
    Ok(json!({ "descriptors": a.details["features"]["descriptors"], "centroid_hz": a.details["features"]["centroid_hz"], "lufs": a.lufs, "peak_dbfs": a.peak_dbfs, "rms_dbfs": a.rms_dbfs, "true_peak_dbtp": a.details["measurements"]["true_peak_dbtp"] }))
}

pub fn design_dir(data_dir: &Path, id: &str) -> PathBuf {
    data_dir.join("generated").join("plugin-designs").join(id)
}

/// Render before and after in the isolated host, measure both, and check the requested qualities and the level.
pub fn audition(c: &Connection, data_dir: &Path, id: &str) -> Result<Value, String> {
    let d = get(c, id).ok_or("unknown design")?;
    let dir = design_dir(data_dir, id);
    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    let mut out = json!({ "id": id });
    for (which, spec) in [("before", &d["base_spec"]), ("after", &d["spec"])] {
        if spec.is_null() {
            continue;
        }
        let path = dir.join(format!("{which}.wav"));
        let _ = std::fs::remove_file(&path);
        let r = host::run("render", &render_request(spec, &path), 90).map_err(|e| {
            registry::log_error(c, spec["instrument"]["install_id"].as_str(), "audition", &e);
            e
        })?;
        out[which] = json!({ "file": path, "measured": measure(&path)?, "warnings": r["warnings"] });
    }
    // did the requested qualities move the right way? Each quality is checked on the measure that reflects it
    // (estimates): harmonics/edge by the spectral centroid, the others by the Smart Library's sound-character values.
    // Qualities to keep must not drop by more than 0.25.
    let mut checks = vec![];
    for (desc, weight, keep) in parse_request(d["request"].as_str().unwrap_or("")) {
        let (metric, path): (&str, Vec<&str>) = match desc.as_str() {
            "aggressive" | "bright" | "dark" => ("spectral centroid (Hz): more harmonics/edge = higher", vec!["centroid_hz"]),
            "warm" | "deep" | "soft" | "punchy" | "wide" | "atmospheric" => ("sound-character estimate 0–1", vec!["descriptors", desc.as_str()]),
            _ => continue,
        };
        let get = |w: &str| path.iter().fold(&out[w]["measured"], |v, k| &v[*k]).as_f64();
        let (Some(b), Some(a)) = (get("before"), get("after")) else { continue };
        let up = if desc == "dark" { -weight } else { weight };
        let ok = if keep {
            if path.len() == 1 { true } else { a >= b - 0.25 }
        } else if path.len() == 1 {
            if up >= 0.0 { a > b * 1.05 } else { a < b * 0.95 }
        } else if up >= 0.0 {
            a >= b - 0.02
        } else {
            a <= b + 0.02
        };
        checks.push(json!({ "quality": desc, "requested": if keep { "keep" } else if up >= 0.0 { "more" } else { "less" }, "metric": metric, "before": b, "after": a, "as_requested": ok }));
    }
    let peak = out["after"]["measured"]["true_peak_dbtp"].as_f64().or(out["after"]["measured"]["peak_dbfs"].as_f64());
    let mut warnings = vec![];
    if peak.is_some_and(|p| p > -1.0) {
        warnings.push(format!("level after the change reaches {:.1} dB(TP): risk of clipping, lower the output before using it", peak.unwrap()));
    }
    out["checks"] = json!(checks);
    out["warnings"] = json!(warnings);
    out["basis"] = json!("rendered in COSOUND's isolated host and measured; sound-character values are estimates");
    c.execute("update plugin_designs set status = case when status = 'saved' then status else 'auditioned' end, updated_at = datetime('now') where id = ?1", [id]).map_err(|e| e.to_string())?;
    Ok(out)
}

fn safe(s: &str) -> String {
    s.chars().map(|c| if c.is_alphanumeric() || " -_".contains(c) { c } else { '_' }).collect::<String>().trim().to_string()
}

/// A new file name that does not exist yet (never overwrite).
pub fn unique(dir: &Path, name: &str, ext: &str) -> PathBuf {
    let base = safe(name);
    let mut p = dir.join(format!("{base}.{ext}"));
    let mut n = 2;
    while p.exists() {
        p = dir.join(format!("{base} {n}.{ext}"));
        n += 1;
    }
    p
}

/// Save the design as new preset files (one .aupreset per unit) inside COSOUND's folder.
pub fn save(c: &Connection, data_dir: &Path, id: &str, name: &str) -> Result<Value, String> {
    let d = get(c, id).ok_or("unknown design")?;
    let spec = &d["spec"];
    let us = units(c, spec);
    let mut saves = vec![];
    let mut files: Vec<Value> = d["files"].as_array().cloned().unwrap_or_default();
    for (slot, u, _, _) in &us {
        let dir = data_dir.join("generated").join("presets").join(safe(u["vendor"].as_str().unwrap_or("Unknown"))).join(safe(u["name"].as_str().unwrap_or("Plug-in")));
        std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
        let p = unique(&dir, name, "aupreset");
        saves.push(json!({ "slot": slot, "path": p, "name": name }));
        files.push(json!({ "path": p, "kind": "preset", "unit": u["name"], "vendor": u["vendor"], "slot": slot, "location": "cosound" }));
    }
    let wav = design_dir(data_dir, id).join("saved.wav");
    std::fs::create_dir_all(wav.parent().unwrap()).map_err(|e| e.to_string())?;
    let _ = std::fs::remove_file(&wav);
    let mut req = render_request(spec, &wav);
    req["save_states"] = json!(saves);
    host::run("render", &req, 90)?;
    let chain_file = unique(&data_dir.join("generated").join("presets"), &format!("{name} (COSOUND chain)"), "json");
    std::fs::write(&chain_file, serde_json::to_vec_pretty(&json!({ "name": name, "request": d["request"], "spec": spec, "created_by": "COSOUND" })).unwrap()).map_err(|e| e.to_string())?;
    files.push(json!({ "path": chain_file, "kind": "chain", "location": "cosound" }));
    c.execute("update plugin_designs set status = 'saved', files = ?2, updated_at = datetime('now') where id = ?1", params![id, json!(files).to_string()]).map_err(|e| e.to_string())?;
    crate::db::log_op(c, None, "plugins.save_preset", "low", &json!({ "design": id, "files": files }), None).map_err(|e| e.to_string())?;
    Ok(json!({ "design": id, "files": files, "note": "new files only: no existing preset was changed" }))
}

/// Tests redirect the plug-in preset folder here so they never write into the real ~/Library.
#[cfg(test)]
pub static TEST_PRESETS_ROOT: std::sync::Mutex<Option<PathBuf>> = std::sync::Mutex::new(None);

/// Folder where Audio Unit hosts look for user presets (Apple convention ~/Library/Audio/Presets/<Manufacturer>/<Name>/).
pub fn au_user_presets_dir(vendor: &str, plugin: &str) -> Option<PathBuf> {
    #[cfg(test)]
    if let Some(r) = TEST_PRESETS_ROOT.lock().unwrap().clone() {
        return Some(r.join(safe(vendor)).join(safe(plugin)));
    }
    std::env::var("HOME").ok().map(|h| PathBuf::from(h).join("Library/Audio/Presets").join(safe(vendor)).join(safe(plugin)))
}

/// Copy a saved preset to the plug-in's user preset folder so DAWs can load it. New file, never overwrite.
pub fn install_preset(c: &Connection, id: &str, target_root: Option<&Path>) -> Result<Value, String> {
    let d = get(c, id).ok_or("unknown design")?;
    let mut files: Vec<Value> = d["files"].as_array().cloned().unwrap_or_default();
    let saved: Vec<Value> = files.iter().filter(|f| f["kind"] == "preset" && f["location"] == "cosound").cloned().collect();
    if saved.is_empty() {
        return Err("save the design first".into());
    }
    let mut done = vec![];
    for f in saved {
        let (vendor, unit) = (f["vendor"].as_str().unwrap_or("Unknown"), f["unit"].as_str().unwrap_or("Plug-in"));
        let dir = match target_root {
            Some(r) => r.join(safe(vendor)).join(safe(unit)),
            None => au_user_presets_dir(vendor, unit).ok_or("no home folder")?,
        };
        std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
        let src = PathBuf::from(f["path"].as_str().unwrap_or(""));
        let dst = unique(&dir, &src.file_stem().unwrap_or_default().to_string_lossy(), "aupreset");
        std::fs::copy(&src, &dst).map_err(|e| e.to_string())?;
        done.push(json!({ "path": dst, "kind": "preset", "unit": unit, "vendor": vendor, "location": "plugin_presets" }));
    }
    files.extend(done.iter().cloned());
    c.execute("update plugin_designs set files = ?2, updated_at = datetime('now') where id = ?1", params![id, json!(files).to_string()]).map_err(|e| e.to_string())?;
    crate::db::log_op(c, None, "plugins.install_preset", "medium", &json!({ "design": id, "files": done }), Some(&json!({ "op": "plugins.remove_installed_presets", "design": id }))).map_err(|e| e.to_string())?;
    Ok(json!({ "installed": done, "note": "new files in the plug-in preset folder; existing presets untouched; reversible with remove_installed_presets" }))
}

/// Undo of install_preset: deletes only files COSOUND itself wrote there for this design.
pub fn remove_installed(c: &Connection, id: &str) -> Result<Value, String> {
    let d = get(c, id).ok_or("unknown design")?;
    let mut files: Vec<Value> = d["files"].as_array().cloned().unwrap_or_default();
    let mut removed = vec![];
    files.retain(|f| {
        if f["location"] == "plugin_presets" {
            if let Some(p) = f["path"].as_str() {
                if std::fs::remove_file(p).is_ok() {
                    removed.push(p.to_string());
                }
            }
            false
        } else {
            true
        }
    });
    c.execute("update plugin_designs set files = ?2 where id = ?1", params![id, json!(files).to_string()]).map_err(|e| e.to_string())?;
    Ok(json!({ "removed": removed }))
}

/// Remember an instrument as the user's usual choice for a role (only through the shared memory, and only on the
/// user's explicit request).
pub fn remember_instrument(c: &Connection, role: &str, product_id: &str) -> Result<Value, String> {
    let key = format!("preferred_instrument:{role}");
    if let Some(old) = memory::list(c, Some("plugin"))?.into_iter().find(|f| f.key == key) {
        memory::delete(c, &old.id)?;
    }
    let f = memory::add(c, "plugin", &key, &json!({ "product_id": product_id }), "explicit", "custom", 1.0, Some("plugins.design"))?;
    Ok(json!(f))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn requests_in_italian_and_english() {
        let r = parse_request("Rendi questo basso più aggressivo, ma mantienilo caldo e profondo.");
        assert_eq!(r, vec![("aggressive".to_string(), 1.0, false), ("warm".to_string(), 1.0, true), ("deep".to_string(), 1.0, true)]);
        let r = parse_request("make it slightly brighter and less distorted, more wide");
        assert_eq!(r[0], ("bright".to_string(), 0.5, false));
        assert_eq!(r[1].0, "wide");
        assert_eq!(parse_request("meno scuro")[0], ("dark".to_string(), -1.0, false));
        assert!(parse_request("ciao").is_empty());
    }

    #[test]
    fn normalization_is_logarithmic_for_frequencies() {
        let n = to_norm(1000.0, 10.0, 22050.0, "hertz");
        assert!((from_norm(n, 10.0, 22050.0, "hertz") - 1000.0).abs() < 0.01);
        assert!(to_norm(469.0, 10.0, 22050.0, "hertz") > 0.45 && to_norm(469.0, 10.0, 22050.0, "hertz") < 0.55, "≈ geometric middle of the range");
        assert_eq!(to_norm(0.0, -20.0, 20.0, "decibels"), 0.5);
        assert_eq!(from_norm(0.25, -80.0, 20.0, "decibels"), -16.0, "dB moves use the top 48 dB of a huge range");
    }
}
