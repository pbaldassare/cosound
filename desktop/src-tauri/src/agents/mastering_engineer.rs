//! K. Mastering Engineer: measures the master and compares it with release destinations.
//! It never reports a master as done: processing and export do not exist yet (M9).
use super::{s, unknown, Agent, Ctx, OpResult, Outcome};
use serde_json::{json, Value};

pub struct MasteringEngineer;

/// (id, label, target LUFS, max true peak, reduce_only, reliability, note)
/// Source: docs/knowledge/audio-engineering/README.md. Only Spotify is official.
const DESTINATIONS: [(&str, &str, Option<f64>, f64, bool, &str, &str); 7] = [
    ("spotify", "Spotify", Some(-14.0), -1.0, false, "official", "≤ −2 dBTP advised when the master is louder than −14 LUFS"),
    ("apple_music", "Apple Music (Sound Check)", Some(-16.0), -1.0, false, "secondary", "Apple publishes no LUFS target; ≈ −16 LUFS comes from secondary sources"),
    ("youtube", "YouTube", Some(-14.0), -1.0, true, "secondary_unverified", "reported as reduce-only: quieter masters are not turned up"),
    ("tidal", "TIDAL", Some(-14.0), -1.0, false, "secondary_unverified", "album-style normalisation reported"),
    ("amazon_music", "Amazon Music", Some(-14.0), -1.0, false, "secondary", "official page only shows a user toggle"),
    ("club", "Club / DJ (no normalisation)", None, -1.0, false, "secondary", "no loudness normalisation; dance chart PLR ≈ 6–11 dB"),
    ("broadcast_ebu", "Broadcast (EBU R 128)", Some(-23.0), -1.0, false, "standard", "−23 LUFS ±1 LU (live), max −1 dBTP"),
];

impl Agent for MasteringEngineer {
    fn id(&self) -> &'static str {
        "mastering_engineer"
    }
    fn operations(&self) -> &'static [&'static str] {
        &["analyze_master"]
    }
    fn run(&self, op: &str, input: &Value, cx: &mut Ctx) -> OpResult {
        if op != "analyze_master" {
            return Err(unknown(self.id(), op));
        }
        let (path, m) = cx.measure_wav(s(input, "path").unwrap_or(""))?;
        let wanted: Vec<&str> = input["destinations"].as_array().map(|a| a.iter().filter_map(|v| v.as_str()).collect()).unwrap_or_default();
        let r1 = |x: f64| (x * 10.0).round() / 10.0;
        let lufs = m.integrated_lufs;
        let dest: Vec<Value> = DESTINATIONS
            .iter()
            .filter(|d| wanted.is_empty() || wanted.contains(&d.0))
            .map(|&(id, label, target, tp_max, reduce_only, reliability, note)| {
                let change = match (target, lufs) {
                    (Some(t), Some(l)) => Some(if reduce_only { (t - l).min(0.0) } else { t - l }),
                    _ => None,
                };
                let tp_limit = if id == "spotify" && lufs.is_some_and(|l| l > -14.0) { -2.0 } else { tp_max };
                json!({
                    "id": id, "label": label, "target_lufs": target, "max_true_peak_dbtp": tp_limit, "reliability": reliability, "note": note,
                    "expected_gain_change_db": change.map(r1),
                    "true_peak_ok": m.true_peak_dbtp <= tp_limit,
                })
            })
            .collect();
        let mut sugg = Vec::new();
        let mut add = |action: &str, reason: String| sugg.push(json!({ "action": action, "reason": reason, "executed": false }));
        if m.clipped_samples > 0 {
            add("Remove clipping (lower the limiter input or ceiling)", format!("{} clipped samples", m.clipped_samples));
        }
        if m.true_peak_dbtp > -1.0 {
            add("Lower the limiter ceiling so true peak stays ≤ −1 dBTP", format!("true peak {} dBTP", m.true_peak_dbtp));
        }
        if let Some(p) = m.plr_db.filter(|p| *p < 6.0) {
            add("Consider less limiting: very dense master", format!("PLR {p} dB, below the 6–11 dB reported for dance charts (secondary source)"));
        }
        if let Some(c) = m.low_band_correlation.filter(|c| *c < 0.8) {
            add("Make the low end more mono", format!("low-band correlation {c}"));
        }
        if m.stereo_correlation.is_some_and(|c| c < 0.0) {
            add("Fix phase: the master loses content in mono", format!("stereo correlation {:?}", m.stereo_correlation));
        }
        if m.leading_silence_s > 1.0 || m.trailing_silence_s > 2.0 {
            add("Trim silence at the start or end", format!("{} s at start, {} s at end", m.leading_silence_s, m.trailing_silence_s));
        }
        if lufs.is_none() {
            cx.limit("integrated loudness not available (file shorter than 400 ms, silent, or more than 2 channels)");
        }
        cx.limit("no mastering was performed: this is an analysis (processing and export are milestone M9)");
        cx.limit("only Spotify publishes its target officially; other platform values come from secondary sources");
        Ok(Outcome::new(json!({ "path": path, "measurements": m, "destinations": dest, "suggestions": sugg, "mastered": false, "source": "docs/knowledge/audio-engineering/README.md" }), "measured"))
    }
}
