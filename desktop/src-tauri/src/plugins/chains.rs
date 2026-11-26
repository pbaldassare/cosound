//! Effect Chain Engine: chains for a goal, in a justified order, adapted to the measured material, mapped to
//! effects actually installed, and checked for overload by rendering the material through the chain.
//! A chain is never applied as a fixed template: without audio it is marked "not adapted".
//! Sources: docs/knowledge/audio-engineering/mixing.md (§2 EQ, §3 compression, §6 low end, §8 reverb/delay,
//! §9 saturation), docs/knowledge/plugins-effects/README.md, docs/knowledge/audio-engineering/mastering.md.
use super::{design, host, registry};
use rusqlite::Connection;
use serde_json::{json, Value};
use std::path::Path;
use std::sync::atomic::AtomicBool;

pub struct Stage {
    pub category: &'static str,
    pub purpose: &'static str,
    pub purpose_it: &'static str,
    /// Words in the plug-in name that fit this stage best (e.g. a high-pass for "remove rumble").
    pub prefer: &'static [&'static str],
    /// Starting settings in plain units (role, value, unit): applied only to parameters with that role.
    pub settings: &'static [(&'static str, f64, &'static str)],
    pub source: &'static str,
}

pub struct Goal {
    pub id: &'static str,
    pub label: &'static str,
    pub stages: &'static [Stage],
}

pub const GOALS: [Goal; 4] = [
    Goal { id: "techno_bass", label: "Techno bass", stages: &[
        Stage { category: "eq", purpose: "remove rumble under the fundamental; keep the sub clean", purpose_it: "togliere il rombo sotto la fondamentale; sub pulito", prefer: &["hipass", "highpass", "high pass", "nband", "parametric"], settings: &[("cutoff", 30.0, "hertz")], source: "mixing.md §2" },
        Stage { category: "saturation", purpose: "harmonics so the bass is heard on small speakers", purpose_it: "armoniche perché il basso si senta sulle casse piccole", prefer: &[], settings: &[("drive_mix", 20.0, "percent")], source: "mixing.md §9, §7" },
        Stage { category: "compressor", purpose: "steady level; slow attack keeps the transient", purpose_it: "livello costante; attacco lento per conservare il transiente", prefer: &["dynamics", "comp"], settings: &[("threshold", -20.0, "decibels"), ("attack", 0.03, "seconds")], source: "mixing.md §3" },
        Stage { category: "limiter", purpose: "control of the low-frequency peaks before the bus", purpose_it: "controllo dei picchi in basso prima del bus", prefer: &["limiter"], settings: &[], source: "mixing.md §1, mastering.md §6" },
    ]},
    Goal { id: "melodic_lead", label: "Melodic techno lead", stages: &[
        Stage { category: "eq", purpose: "high-pass to leave the lows to kick and bass", purpose_it: "passa-alto per lasciare i bassi a kick e basso", prefer: &["hipass", "highpass", "high pass", "nband", "parametric"], settings: &[("cutoff", 150.0, "hertz")], source: "mixing.md §2" },
        Stage { category: "distortion", purpose: "controlled distortion: presence and edge", purpose_it: "distorsione controllata: presenza e mordente", prefer: &[], settings: &[("drive_mix", 15.0, "percent")], source: "plugins-effects §1" },
        Stage { category: "chorus", purpose: "width and movement", purpose_it: "ampiezza e movimento", prefer: &[], settings: &[("space_mix", 25.0, "percent")], source: "plugins-effects §4" },
        Stage { category: "delay", purpose: "tempo-synced echoes (dotted 1/8)", purpose_it: "echi a tempo (1/8 puntato)", prefer: &[], settings: &[("space_mix", 20.0, "percent"), ("feedback", 30.0, "percent")], source: "mixing.md §8" },
        Stage { category: "reverb", purpose: "space; high-passed return", purpose_it: "spazio; ritorno filtrato passa-alto", prefer: &["reverb2", "reverb"], settings: &[("space_mix", 20.0, "percent")], source: "mixing.md §8" },
    ]},
    Goal { id: "atmospheric_pad", label: "Atmospheric pad", stages: &[
        Stage { category: "filter", purpose: "high-pass ≈140 Hz against mud; low-pass for softness", purpose_it: "passa-alto ≈140 Hz contro il fango; passa-basso per morbidezza", prefer: &["lowpass", "low pass"], settings: &[("cutoff", 6000.0, "hertz")], source: "mixing.md §2" },
        Stage { category: "modulation", purpose: "slow movement", purpose_it: "movimento lento", prefer: &[], settings: &[], source: "synthesizers §3 (pad)" },
        Stage { category: "chorus", purpose: "width", purpose_it: "ampiezza", prefer: &[], settings: &[("space_mix", 30.0, "percent")], source: "plugins-effects §4" },
        Stage { category: "reverb", purpose: "long space", purpose_it: "spazio lungo", prefer: &["reverb2", "reverb"], settings: &[("space_mix", 40.0, "percent"), ("reverb_decay", 4.0, "seconds")], source: "plugins-effects §2" },
        Stage { category: "stereo", purpose: "width above ≈150 Hz, mono below", purpose_it: "ampiezza sopra ≈150 Hz, mono sotto", prefer: &[], settings: &[], source: "mixing.md §6" },
    ]},
    Goal { id: "mix_bus", label: "Mix bus glue", stages: &[
        Stage { category: "eq", purpose: "gentle corrective EQ", purpose_it: "EQ correttiva leggera", prefer: &["nband", "parametric", "graphic"], settings: &[], source: "mixing.md §2" },
        Stage { category: "compressor", purpose: "bus glue, low ratio, slow attack", purpose_it: "coesione del bus, rapporto basso, attacco lento", prefer: &["dynamics", "comp"], settings: &[("attack", 0.03, "seconds")], source: "mixing.md §3" },
        Stage { category: "saturation", purpose: "light bus saturation", purpose_it: "saturazione leggera del bus", prefer: &[], settings: &[("drive_mix", 10.0, "percent")], source: "mixing.md §9" },
        Stage { category: "limiter", purpose: "ceiling against true-peak overs", purpose_it: "tetto contro i superamenti di true peak", prefer: &["limiter"], settings: &[], source: "mastering.md §2, §6" },
    ]},
];

/// Order rules for any chain (by category). Returns problems found.
pub fn check_order(cats: &[String]) -> Vec<String> {
    let pos = |c: &str| cats.iter().position(|x| x == c);
    let mut issues = vec![];
    if let Some(l) = pos("limiter") {
        if l + 1 != cats.len() {
            issues.push("the limiter should be the last processor: anything after it can raise peaks again (mastering.md §6)".into());
        }
    }
    for space in ["reverb", "delay"] {
        for drive in ["distortion", "saturation"] {
            if let (Some(s), Some(d)) = (pos(space), pos(drive)) {
                if s < d {
                    issues.push(format!("{drive} after {space} distorts the reverb/echo tails too: usually {drive} goes first"));
                }
            }
        }
    }
    if let (Some(r), Some(cmp)) = (pos("reverb"), pos("compressor")) {
        if r < cmp {
            issues.push("compressing after the reverb pumps the tail: usually dynamics before time-based effects".into());
        }
    }
    issues
}

/// Measurements of the material a chain will process.
pub fn material(path: &Path) -> Result<Value, String> {
    let a = crate::smartlib::analyze::analyze_file(path, "audio", "material", vec![], &AtomicBool::new(false))?;
    let m = &a.details["measurements"];
    let crest = match (a.peak_dbfs, a.rms_dbfs) {
        (Some(p), Some(r)) => Some(p - r),
        _ => None,
    };
    Ok(json!({ "lufs": a.lufs, "peak_dbfs": a.peak_dbfs, "rms_dbfs": a.rms_dbfs, "crest_db": crest, "true_peak_dbtp": m["true_peak_dbtp"],
               "correlation": m["correlation"], "bands": a.details["features"]["bands"], "centroid_hz": a.details["features"]["centroid_hz"],
               "descriptors": a.details["features"]["descriptors"] }))
}

fn installed_for(c: &Connection, category: &str, prefer: &[&str]) -> Option<Value> {
    let mut st = c
        .prepare(
            "select i.id, i.name, i.au_code from plugin_installs i join plugin_categories pc on pc.product_id = i.product_id
             where pc.category = ?1 and i.status = 'available' order by (i.au_code is null), pc.confidence desc, i.scope = 'apple'",
        )
        .ok()?;
    let rows: Vec<Value> = st
        .query_map([category], |r| Ok(json!({ "install_id": r.get::<_, String>(0)?, "name": r.get::<_, String>(1)?, "hostable": r.get::<_, Option<String>>(2)?.is_some() })))
        .ok()?
        .filter_map(Result::ok)
        .collect();
    // the plug-in whose name fits the stage's function best (e.g. a high-pass, not a band-pass, to remove rumble)
    let rank = |v: &Value| {
        let n = v["name"].as_str().unwrap_or("").to_lowercase().replace(['-', ' '], "");
        prefer.iter().position(|p| n.contains(&p.replace(' ', ""))).unwrap_or(prefer.len())
    };
    rows.into_iter().enumerate().min_by_key(|(i, v)| (rank(v), *i)).map(|(_, v)| v)
}

/// Propose a chain for a goal. With `material_path`, stages are adapted to the measurements.
pub fn propose(c: &Connection, goal: &str, material_path: Option<&Path>, bpm: Option<f64>) -> Result<Value, String> {
    let g = GOALS.iter().find(|g| g.id == goal).ok_or_else(|| format!("unknown goal {goal} (use {})", GOALS.map(|g| g.id).join(", ")))?;
    let m = material_path.map(material).transpose()?;
    let mut stages = vec![];
    let mut notes = vec![];
    for s in g.stages {
        let mut include = true;
        let mut why: Vec<String> = vec![];
        let mut settings: Vec<Value> = s.settings.iter().map(|(r, v, u)| json!({ "role": r, "value": v, "unit": u })).collect();
        if let Some(m) = &m {
            let crest = m["crest_db"].as_f64();
            let bright = m["descriptors"]["bright"].as_f64().unwrap_or(0.0);
            let low = m["bands"][0].as_f64().unwrap_or(0.0);
            match s.category {
                "compressor" if crest.is_some_and(|c| c < 8.0) => {
                    include = false;
                    why.push(format!("skipped: crest factor {:.1} dB, the material is already dense (mixing.md §3)", crest.unwrap()));
                }
                "distortion" | "saturation" if bright > 0.7 => {
                    for x in settings.iter_mut() {
                        x["value"] = json!(x["value"].as_f64().unwrap_or(0.0) / 2.0);
                    }
                    why.push(format!("halved: the material is already bright ({:.0}%)", bright * 100.0));
                }
                "filter" | "eq" if goal == "atmospheric_pad" && low < 0.1 => why.push(format!("little low end in the material ({:.0}% of energy): the high-pass is a safety, not a fix", low * 100.0)),
                "stereo" if m["correlation"].as_f64().is_some_and(|c| c < 0.5) => why.push("the material is already wide (correlation < 0.5): check mono compatibility before widening (mixing.md §5)".into()),
                _ => {}
            }
            if matches!(s.category, "saturation" | "distortion") && m["peak_dbfs"].as_f64().is_some_and(|p| p > -3.0) {
                notes.push(format!("input peaks at {:.1} dBFS: lower the input of the {} stage, drive stages react to level (mixing.md §1)", m["peak_dbfs"].as_f64().unwrap(), s.category));
            }
        }
        if s.category == "delay" {
            if let Some(b) = bpm {
                settings.push(json!({ "role": "delay_time", "value": (60.0 / b) * 0.75, "unit": "seconds", "note": format!("dotted 1/8 at {b} BPM") }));
            }
        }
        let plugin = installed_for(c, s.category, s.prefer);
        if plugin.is_none() {
            why.push(format!("no installed plug-in in category {}", s.category));
        }
        stages.push(json!({ "category": s.category, "purpose": s.purpose, "purpose_it": s.purpose_it, "include": include, "why": why, "settings": settings, "plugin": plugin, "source": s.source }));
    }
    let cats: Vec<String> = stages.iter().filter(|s| s["include"] == true).filter_map(|s| s["category"].as_str().map(String::from)).collect();
    Ok(json!({
        "goal": g.id, "label": g.label, "stages": stages, "order_issues": check_order(&cats), "material": m, "notes": notes,
        "adapted_to_material": m.is_some(),
        "status": if m.is_some() { "adapted to the measured material" } else { "generic starting point: give an audio file to adapt it and check the level" },
    }))
}

/// Render the material through the hostable stages of a proposal and measure the result: overload check.
pub fn check(c: &Connection, proposal: &Value, material_path: &Path, work: &Path) -> Result<Value, String> {
    let mut chain = vec![];
    let mut skipped = vec![];
    for s in proposal["stages"].as_array().into_iter().flatten().filter(|s| s["include"] == true) {
        let Some(iid) = s["plugin"]["install_id"].as_str().filter(|_| s["plugin"]["hostable"] == true) else {
            skipped.push(s["category"].clone());
            continue;
        };
        let mut u = design_unit(c, iid);
        let ps = super::params::stored(c, iid);
        let ps = if ps.is_empty() { super::params::discover(c, iid, 30).map(|_| super::params::stored(c, iid)).unwrap_or_default() } else { ps };
        let cats: Vec<String> = registry::categories_of(c, registry::install(c, iid).and_then(|i| i["product_id"].as_str().map(String::from)).as_deref().unwrap_or("")).iter().filter_map(|x| x["category"].as_str().map(String::from)).collect();
        let mut vals = vec![];
        for st in s["settings"].as_array().into_iter().flatten() {
            let role = st["role"].as_str().unwrap_or("");
            let target = ps.iter().find(|p| {
                let r = p["role"].as_str();
                let eff = if r == Some("dry_wet") { if cats.iter().any(|c| c == "distortion" || c == "saturation") { Some("drive_mix") } else { Some("space_mix") } } else { r };
                eff == Some(role) && p["writable"] == true
            });
            if let (Some(p), Some(v)) = (target, st["value"].as_f64()) {
                let (min, max) = (p["min"].as_f64().unwrap_or(f64::MIN), p["max"].as_f64().unwrap_or(f64::MAX));
                vals.push(json!({ "address": p["address"], "value": v.clamp(min, max) }));
            }
        }
        u["params"] = json!(vals);
        chain.push(u);
    }
    if chain.is_empty() {
        return Err("none of the chain's plug-ins can be loaded by COSOUND: the level cannot be checked here".into());
    }
    std::fs::create_dir_all(work).map_err(|e| e.to_string())?;
    let out = work.join("chain-check.wav");
    let _ = std::fs::remove_file(&out);
    let input = material(material_path)?;
    let dur = input["lufs"].is_number().then_some(()).map(|_| 12.0).unwrap_or(8.0);
    let spec = json!({ "chain": chain, "input_file": material_path, "seconds": dur, "notes": [] });
    host::run("render", &design::render_request(&spec, &out), 120)?;
    let output = material(&out)?;
    let gain = match (output["rms_dbfs"].as_f64(), input["rms_dbfs"].as_f64()) {
        (Some(a), Some(b)) => Some(a - b),
        _ => None,
    };
    let tp = output["true_peak_dbtp"].as_f64().or(output["peak_dbfs"].as_f64());
    let mut risks = vec![];
    if tp.is_some_and(|p| p > -1.0) {
        risks.push(format!("output reaches {:.1} dB(TP): overload risk, lower the chain's output by at least {:.1} dB", tp.unwrap(), tp.unwrap() + 1.0));
    }
    if gain.is_some_and(|g| g < -6.0) {
        risks.push(format!("the chain removes {:.1} dB RMS: a stage is cutting the useful signal (check filters and their frequencies)", -gain.unwrap()));
    }
    if gain.is_some_and(|g| g > 6.0) {
        risks.push(format!("the chain adds {:.1} dB RMS: compare at equal loudness, and watch the next stages' headroom", gain.unwrap()));
    }
    let _ = std::fs::remove_file(&out);
    Ok(json!({ "rendered_stages": chain.len(), "not_rendered": skipped, "input": input, "output": output, "rms_change_db": gain, "risks": risks,
               "basis": "material rendered through the installed plug-ins in COSOUND's isolated host and measured" }))
}

fn design_unit(c: &Connection, install_id: &str) -> Value {
    let i = registry::install(c, install_id).unwrap_or(json!({}));
    let code: Vec<String> = i["au_code"].as_str().unwrap_or("").split(':').map(String::from).collect();
    json!({ "install_id": install_id, "name": i["name"], "type": code.first(), "subtype": code.get(1), "manufacturer": code.get(2) })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn order_rules() {
        let v = |x: &[&str]| x.iter().map(|s| s.to_string()).collect::<Vec<_>>();
        assert!(check_order(&v(&["eq", "distortion", "chorus", "delay", "reverb"])).is_empty());
        assert_eq!(check_order(&v(&["limiter", "eq"])).len(), 1);
        assert_eq!(check_order(&v(&["reverb", "distortion"])).len(), 1);
        assert_eq!(check_order(&v(&["reverb", "compressor"])).len(), 1);
        for g in GOALS {
            assert!(check_order(&g.stages.iter().map(|s| s.category.to_string()).collect::<Vec<_>>()).is_empty(), "template {} breaks its own rules", g.id);
        }
    }
}
