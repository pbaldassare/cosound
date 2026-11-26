//! J. Mixing Engineer: per-track measurements and mix proposals. Proposals only: every suggestion
//! carries `executed: false`; applying them needs a DAW connector (M4).
use super::{s, unknown, Agent, AgentError, Ctx, OpResult, Outcome};
use crate::audio::wav::Measurements;
use serde_json::{json, Value};

pub struct MixingEngineer;

fn is(role: &str, name: &str, words: &[&str]) -> bool {
    let t = format!("{role} {name}").to_lowercase();
    words.iter().any(|w| t.contains(w))
}

fn tip(target: &str, action: &str, reason: String, basis: &str) -> Value {
    json!({ "target": target, "action": action, "reason": reason, "basis": basis, "executed": false })
}

/// Rule-based proposals from measured values. Conventions come from
/// docs/knowledge/audio-engineering/mixing.md (gain staging, sub mono, sidechain).
pub fn suggestions(tracks: &[(String, String, Measurements)]) -> Vec<Value> {
    let mut out = Vec::new();
    for (name, role, m) in tracks {
        if m.clipped_samples > 0 {
            out.push(tip(name, "Lower the level or fix clipping at the source", format!("{} clipped samples measured", m.clipped_samples), "measured"));
        } else if m.true_peak_dbtp > -1.0 {
            out.push(tip(name, "Leave headroom: lower the track so peaks sit around -6 dBFS", format!("true peak {} dBTP; convention: peak meters at -6 dBFS", m.true_peak_dbtp), "measured+convention"));
        }
        if m.sample_peak_dbfs < -30.0 {
            out.push(tip(name, "Raise the gain at the start of the chain", format!("peak only {} dBFS", m.sample_peak_dbfs), "measured"));
        }
        if is(role, name, &["bass", "kick", "sub", "808"]) {
            if let Some(c) = m.low_band_correlation.filter(|c| *c < 0.9) {
                out.push(tip(name, "Keep the low end mono below about 100–150 Hz", format!("low-band correlation {c} (1 = mono)"), "measured+convention"));
            }
        }
        if let Some(c) = m.stereo_correlation.filter(|c| *c < 0.0) {
            out.push(tip(name, "Check polarity and mono compatibility", format!("stereo correlation {c}: content cancels in mono"), "measured"));
        }
        if m.dc_offset.iter().any(|d| d.abs() > 0.001) {
            out.push(tip(name, "Remove DC offset (high-pass filter)", format!("DC offset {:?}", m.dc_offset), "measured"));
        }
    }
    let find = |w: &[&str]| tracks.iter().find(|(n, r, _)| is(r, n, w));
    if let (Some(k), Some(b)) = (find(&["kick"]), find(&["bass", "sub"])) {
        if let (Some(kl), Some(bl)) = (k.2.integrated_lufs, b.2.integrated_lufs) {
            if bl - kl > 3.0 {
                out.push(tip(&b.0, "Duck the bass from the kick (sidechain, 2–3 dB) or lower the bass", format!("bass {:.1} LU louder than the kick", bl - kl), "measured+convention"));
            }
        }
        if let (Some(kb), Some(bb)) = (&k.2.band_energy, &b.2.band_energy) {
            if kb.low_pct > 50.0 && bb.low_pct > 50.0 {
                out.push(tip("kick + bass", "Make room between kick and bass (EQ or sidechain)", format!("both carry most energy below 120 Hz (kick {}%, bass {}%)", kb.low_pct, bb.low_pct), "measured+convention"));
            }
        }
    }
    out
}

impl Agent for MixingEngineer {
    fn id(&self) -> &'static str {
        "mixing_engineer"
    }
    fn operations(&self) -> &'static [&'static str] {
        &["analyze_mix"]
    }
    fn run(&self, op: &str, input: &Value, cx: &mut Ctx) -> OpResult {
        if op != "analyze_mix" {
            return Err(unknown(self.id(), op));
        }
        let mut tracks = Vec::new();
        for t in input["tracks"].as_array().into_iter().flatten() {
            let path = s(t, "path").ok_or_else(|| AgentError::input("every track needs a path"))?;
            let (p, m) = cx.measure_wav(path)?;
            let name = s(t, "name").map(String::from).unwrap_or_else(|| p.file_stem().map(|x| x.to_string_lossy().to_string()).unwrap_or_default());
            tracks.push((name, s(t, "role").unwrap_or("").to_string(), m));
        }
        let sugg = suggestions(&tracks);
        let louds: Vec<f64> = tracks.iter().filter_map(|t| t.2.integrated_lufs).collect();
        let spread = if louds.len() > 1 { Some(louds.iter().cloned().fold(f64::MIN, f64::max) - louds.iter().cloned().fold(f64::MAX, f64::min)) } else { None };
        cx.limit("proposals only: nothing was changed in the mix (applying moves needs a DAW connector, milestone M4)");
        cx.limit("frequency bands are coarse (2nd-order filters); no fine spectral or masking analysis yet");
        Ok(Outcome::new(
            json!({
                "tracks": tracks.iter().map(|(n, r, m)| json!({ "name": n, "role": r, "measurements": m })).collect::<Vec<_>>(),
                "loudness_spread_lu": spread.map(|x| (x * 10.0).round() / 10.0),
                "suggestions": sugg,
                "source": "docs/knowledge/audio-engineering/mixing.md",
            }),
            "measured",
        ))
    }
}
