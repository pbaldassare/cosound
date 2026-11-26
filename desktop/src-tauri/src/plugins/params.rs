//! Parameter Discovery Engine: loads a plug-in in the isolated host, reads the parameters it really exposes
//! (address, name, unit, range, current value, writable) and its factory presets, and maps each parameter to a
//! sound role (cutoff, resonance, drive, attack …) from its name and unit. Roles drive sound design; a parameter
//! with no recognizable role stays available by name but is never moved automatically.
use super::{host, registry};
use rusqlite::{params, Connection};
use serde_json::{json, Value};

/// Sound roles and the words that identify them in parameter names (checked in order; first match wins).
const ROLES: [(&str, &[&str]); 30] = [
    ("dry_wet", &["dry/wet", "wet/dry", "dry wet", "wet dry", "blend"]),
    ("cutoff", &["cutoff", "cut off", "filter freq", "cutoff frequency", "flt freq", "fc"]),
    ("resonance", &["resonance", "reso", "res", "emphasis", "q"]),
    ("drive", &["drive", "distortion", "dist", "saturation", "satur", "overdrive", "soft clip gain", "squared term", "cubic term", "pre gain", "pregain", "warmth"]),
    ("attack", &["attack", "atk"]),
    ("decay", &["decay"]),
    ("sustain", &["sustain"]),
    ("release", &["release", "rel"]),
    ("lfo_rate", &["lfo rate", "lfo speed", "lfo freq", "mod rate", "rate"]),
    ("lfo_depth", &["lfo depth", "lfo amount", "mod depth", "depth"]),
    ("unison", &["unison", "voices"]),
    ("detune", &["detune", "spread"]),
    ("waveform", &["waveform", "wave", "shape", "wt pos", "wavetable"]),
    ("octave", &["octave", "coarse tuning", "coarse", "transpose"]),
    ("fine_tune", &["fine tuning", "fine", "tuning"]),
    ("glide", &["glide", "portamento"]),
    ("sub_level", &["sub level", "sub osc", "sub"]),
    ("delay_time", &["delay time", "time"]),
    ("feedback", &["feedback"]),
    ("reverb_decay", &["decay time", "reverb time", "rt60", "high freq decay", "low freq decay"]),
    ("reverb_size", &["size", "room"]),
    ("threshold", &["threshold"]),
    ("ratio", &["ratio"]),
    ("makeup", &["makeup", "make-up", "master gain", "output gain"]),
    ("pan", &["pan", "balance"]),
    ("width", &["width", "stereo"]),
    ("volume", &["volume", "level", "gain", "output", "master"]),
    ("filter_env", &["env amount", "filter env", "env depth"]),
    ("frequency", &["frequency", "freq", "center"]),
    ("bandwidth", &["bandwidth"]),
];

/// Role of a parameter from its name and unit. Unit refines ambiguous words ("Decay" in a reverb vs an envelope).
pub fn role(name: &str, unit: &str, plugin_categories: &[String]) -> Option<&'static str> {
    let n = name.to_lowercase();
    let words: Vec<&str> = n.split(|c: char| !c.is_alphanumeric() && c != '/' && c != '-').filter(|w| !w.is_empty()).collect();
    let has = |k: &str| if k.contains(' ') || k.contains('/') { n.contains(k) } else { words.contains(&k) };
    let is_reverb = plugin_categories.iter().any(|c| c == "reverb");
    let is_delay = plugin_categories.iter().any(|c| c == "delay");
    let is_filter = plugin_categories.iter().any(|c| c == "filter" || c == "eq");
    // special cases first
    if is_reverb && (n.contains("decay") || n.contains("time")) && unit == "seconds" {
        return Some("reverb_decay");
    }
    if is_delay && has("time") {
        return Some("delay_time");
    }
    if is_filter && (n.contains("cutoff") || (unit == "hertz" && (has("frequency") || has("freq")))) {
        return Some("cutoff");
    }
    if n.contains("ring mod") || n.contains("decimation") || n.contains("rounding") || n.contains("delay mix") || n.contains("polynomial mix") {
        return None; // special-purpose stages (e.g. inside AUDistortion): shown, never moved automatically
    }
    if n.trim() == "mix" || n.trim() == "mix level" {
        return Some("dry_wet");
    }
    for (r, keys) in ROLES {
        if keys.iter().any(|k| has(k)) {
            // "Delay Mix", "Ring Mod Mix", "Decimation Mix": a mix is dry/wet only if nothing more specific matched
            return Some(r);
        }
    }
    None
}

/// Discover parameters and factory presets of one install (Audio Units on macOS) and store them.
pub fn discover(c: &Connection, install_id: &str, timeout_s: u64) -> Result<Value, String> {
    let inst = registry::install(c, install_id).ok_or("unknown plug-in install")?;
    let code = inst["au_code"].as_str().ok_or_else(|| {
        format!("{} {}: parameter discovery needs hosting this format, which COSOUND does not do yet (Audio Units only, on macOS)", inst["name"].as_str().unwrap_or(""), inst["format"].as_str().unwrap_or(""))
    })?;
    if inst["status"] != "available" {
        return Err(format!("not loadable: {}", inst["status_reason"].as_str().unwrap_or(inst["status"].as_str().unwrap_or(""))));
    }
    let parts: Vec<&str> = code.split(':').collect();
    let req = json!({ "type": parts[0], "subtype": parts.get(1).unwrap_or(&""), "manufacturer": parts.get(2).unwrap_or(&"") });
    let res = match host::run("describe", &req, timeout_s) {
        Ok(v) => v,
        Err(e) => {
            registry::log_error(c, Some(install_id), "discover_parameters", &e);
            return Err(e);
        }
    };
    let pid = inst["product_id"].as_str().unwrap_or("").to_string();
    let cats: Vec<String> = registry::categories_of(c, &pid).iter().filter_map(|x| x["category"].as_str().map(String::from)).collect();
    let version = inst["version"].as_str().map(String::from);
    let tx = c.unchecked_transaction().map_err(|e| e.to_string())?;
    tx.execute("delete from plugin_params where install_id = ?1", [install_id]).map_err(|e| e.to_string())?;
    let mut out = vec![];
    for p in res["parameters"].as_array().into_iter().flatten() {
        let name = p["name"].as_str().unwrap_or("");
        let unit = p["unit"].as_str().unwrap_or("");
        let r = role(name, unit, &cats);
        tx.execute(
            "insert or replace into plugin_params(install_id, address, identifier, name, unit, min_value, max_value, default_value, writable, role, plugin_version) values (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
            params![install_id, p["address"].as_i64(), p["identifier"].as_str(), name, unit, p["min"].as_f64(), p["max"].as_f64(), p["value"].as_f64(), p["writable"].as_bool().unwrap_or(false), r, version],
        )
        .map_err(|e| e.to_string())?;
        let mut q = p.clone();
        q["role"] = json!(r);
        out.push(q);
    }
    tx.execute("delete from plugin_presets where product_id = ?1 and source = 'factory' and install_id = ?2", params![pid, install_id]).map_err(|e| e.to_string())?;
    for fp in res["factory_presets"].as_array().into_iter().flatten() {
        let name = fp["name"].as_str().unwrap_or("");
        tx.execute(
            "insert or ignore into plugin_presets(product_id, install_id, source, name, number, format, category, readable, details) values (?1, ?2, 'factory', ?3, ?4, 'factory', ?5, 1, ?6)",
            params![pid, install_id, name, fp["number"].as_i64(), super::presets::category_of(name, &[]), json!({ "loadable_by": "cosound-auhost" }).to_string()],
        )
        .map_err(|e| e.to_string())?;
    }
    tx.commit().map_err(|e| e.to_string())?;
    Ok(json!({
        "install_id": install_id, "instrument": res["instrument"], "parameters": out,
        "factory_presets": res["factory_presets"], "supports_user_presets": res["supports_user_presets"],
        "latency_s": res["latency_s"], "tail_s": res["tail_s"],
        "mapped_roles": stored(c, install_id).iter().filter(|p| !p["role"].is_null()).count(),
    }))
}

/// Parameters stored for an install.
pub fn stored(c: &Connection, install_id: &str) -> Vec<Value> {
    let mut st = c.prepare("select address, identifier, name, unit, min_value, max_value, default_value, writable, role, plugin_version, discovered_at from plugin_params where install_id = ?1 order by address").unwrap();
    st.query_map([install_id], |r| {
        Ok(json!({ "address": r.get::<_, i64>(0)?, "identifier": r.get::<_, Option<String>>(1)?, "name": r.get::<_, String>(2)?, "unit": r.get::<_, Option<String>>(3)?,
                   "min": r.get::<_, Option<f64>>(4)?, "max": r.get::<_, Option<f64>>(5)?, "default": r.get::<_, Option<f64>>(6)?, "writable": r.get::<_, bool>(7)?,
                   "role": r.get::<_, Option<String>>(8)?, "plugin_version": r.get::<_, Option<String>>(9)?, "discovered_at": r.get::<_, String>(10)? }))
    })
    .map(|m| m.filter_map(Result::ok).collect())
    .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roles_from_real_parameter_names() {
        let fx = |c: &str| vec![c.to_string()];
        // names exactly as Apple's units report them (cosound-auhost describe)
        assert_eq!(role("Cutoff Frequency", "hertz", &fx("filter")), Some("cutoff"));
        assert_eq!(role("Resonance", "decibels", &fx("filter")), Some("resonance"));
        assert_eq!(role("Dry/Wet Mix", "crossfade", &fx("delay")), Some("dry_wet"));
        assert_eq!(role("Delay Time", "seconds", &fx("delay")), Some("delay_time"));
        assert_eq!(role("Feedback", "percent", &fx("delay")), Some("feedback"));
        assert_eq!(role("High Freq Decay Time", "seconds", &fx("reverb")), Some("reverb_decay"));
        assert_eq!(role("Soft Clip Gain", "decibels", &fx("distortion")), Some("drive"));
        assert_eq!(role("Coarse Tuning", "semitones", &fx("sampler")), Some("octave"));
        assert_eq!(role("Stereo Pan", "pan", &fx("sampler")), Some("pan"));
        assert_eq!(role("Threshold", "decibels", &fx("compressor")), Some("threshold"));
        // third-party style names
        assert_eq!(role("Filter 1 Cutoff", "generic", &fx("synth")), Some("cutoff"));
        assert_eq!(role("Amp Env Attack", "seconds", &fx("synth")), Some("attack"));
        assert_eq!(role("Osc A Unison", "generic", &fx("synth")), Some("unison"));
        assert_eq!(role("Performance Parameter 1", "generic", &fx("sampler")), None, "unknown purpose: never moved automatically");
        assert_eq!(role("Delay Mix", "percent", &fx("distortion")), None, "a stage mix is not the plug-in's dry/wet");
        assert_eq!(role("Mix", "percent", &fx("chorus")), Some("dry_wet"));
    }
}
