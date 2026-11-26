//! F. Audio Analysis: measurements on WAV files and stems (tool audio.wav_meter). Measured values
//! only; what is not measured yet (BPM, key, structure, instruments) is said explicitly.
use super::{s, unknown, Agent, AgentError, Ctx, OpResult, Outcome};
use serde_json::{json, Value};

pub struct AudioAnalysis;

pub const NOT_MEASURED: &str = "BPM, key, structure and instruments are not measured yet (engine M6)";

impl Agent for AudioAnalysis {
    fn id(&self) -> &'static str {
        "audio_analysis"
    }
    fn operations(&self) -> &'static [&'static str] {
        &["analyze_file", "analyze_stems"]
    }
    fn run(&self, op: &str, input: &Value, cx: &mut Ctx) -> OpResult {
        match op {
            "analyze_file" => {
                let (path, m) = cx.measure_wav(s(input, "path").unwrap_or(""))?;
                cx.limit(NOT_MEASURED);
                Ok(Outcome::new(json!({ "path": path, "measurements": m, "not_measured": ["bpm", "key", "structure", "instruments"] }), "measured"))
            }
            "analyze_stems" => {
                let mut stems = Vec::new();
                let mut failed = 0;
                for p in input["paths"].as_array().into_iter().flatten().filter_map(|p| p.as_str()) {
                    if cx.cancelled() {
                        return Err(AgentError::new("CANCELLED", "analysis stopped"));
                    }
                    match cx.measure_wav(p) {
                        Ok((path, m)) => stems.push(json!({ "path": path, "measurements": m })),
                        Err(e) if e.code == "CANCELLED" => return Err(e),
                        Err(e) => {
                            failed += 1;
                            stems.push(json!({ "path": p, "error": { "code": e.code, "message": e.message } }));
                        }
                    }
                }
                if failed == stems.len() {
                    return Err(AgentError::new("UNSUPPORTED_FORMAT", "none of the stems could be measured"));
                }
                if failed > 0 {
                    cx.limit(format!("{failed} stem(s) could not be measured"));
                }
                let max = stems.iter().filter_map(|s| s["measurements"]["integrated_lufs"].as_f64()).fold(f64::MIN, f64::max);
                for st in &mut stems {
                    if let Some(l) = st["measurements"]["integrated_lufs"].as_f64() {
                        st["relative_lufs"] = json!(((l - max) * 100.0).round() / 100.0);
                    }
                }
                cx.limit(NOT_MEASURED);
                Ok(Outcome::new(json!({ "stems": stems, "relative_to": "loudest stem (integrated loudness)" }), "measured"))
            }
            _ => Err(unknown(self.id(), op)),
        }
    }
}
