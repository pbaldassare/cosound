//! Multi-Agent System, level 2: the specialized agents (Prompt 06).
//! Docs: docs/agents/. Every agent is an independent module that implements `Agent`, declares
//! itself in packages/agents/<id>.json and uses only Level-3 tools through `Ctx`. Agents never
//! call each other and never talk to the user: the central Agent (partner's code) coordinates.
pub mod protocol;
pub mod registry;
pub mod runtime;
pub mod schema;
#[cfg(any(test, feature = "temp-connector"))]
pub mod scenario;

mod adaptive_workflow;
mod audio_analysis;
mod daw_controller;
mod mastering_engineer;
mod mixing_engineer;
mod music_producer;
mod music_theory;
mod plugin_intelligence;
mod reference_track;
mod rhythm_engineer;
mod smart_library;
mod sound_designer;
mod voice_to_music;

#[cfg(test)]
mod tests;

use crate::db::Db;
pub use protocol::{AgentError, FileRef};
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};

/// What an operation returns when it succeeds.
pub struct Outcome {
    pub result: Value,
    /// measured | rule_based | convention | estimated | web | file_name | mixed | none
    pub basis: &'static str,
    pub needs: Vec<Value>,
    pub proposed_daw_ops: Vec<Value>,
}

impl Outcome {
    pub fn new(result: Value, basis: &'static str) -> Outcome {
        Outcome { result, basis, needs: vec![], proposed_daw_ops: vec![] }
    }
}

pub type OpResult = Result<Outcome, AgentError>;

pub trait Agent: Send + Sync {
    fn id(&self) -> &'static str;
    /// Operations this implementation executes; must equal the manifest's "available" ones.
    fn operations(&self) -> &'static [&'static str];
    fn run(&self, op: &str, input: &Value, cx: &mut Ctx) -> OpResult;
}

pub fn all_agents() -> Vec<Box<dyn Agent>> {
    vec![
        Box::new(music_producer::MusicProducer),
        Box::new(rhythm_engineer::RhythmEngineer),
        Box::new(sound_designer::SoundDesigner),
        Box::new(music_theory::MusicTheory),
        Box::new(voice_to_music::VoiceToMusic),
        Box::new(audio_analysis::AudioAnalysis),
        Box::new(reference_track::ReferenceTrack),
        Box::new(smart_library::SmartLibrary),
        Box::new(plugin_intelligence::PluginIntelligence),
        Box::new(mixing_engineer::MixingEngineer),
        Box::new(mastering_engineer::MasteringEngineer),
        Box::new(daw_controller::DawController),
        Box::new(adaptive_workflow::AdaptiveWorkflow),
    ]
}

/// Execution context: the only door from an agent to the computer. It checks permissions,
/// allows reading only authorized files, writing only inside COSOUND's own folder, and records
/// the tools used, files produced and limitations met.
pub struct Ctx<'a> {
    pub db: &'a Db,
    pub data_dir: &'a Path,
    pub project_id: Option<String>,
    granted: &'a HashMap<String, bool>,
    cancel: &'a AtomicBool,
    pub tools_used: Vec<String>,
    pub files: Vec<FileRef>,
    pub limitations: Vec<String>,
}

impl<'a> Ctx<'a> {
    pub fn new(db: &'a Db, data_dir: &'a Path, project_id: Option<String>, granted: &'a HashMap<String, bool>, cancel: &'a AtomicBool) -> Ctx<'a> {
        Ctx { db, data_dir, project_id, granted, cancel, tools_used: vec![], files: vec![], limitations: vec![] }
    }

    pub fn tool(&mut self, id: &str) {
        if !self.tools_used.iter().any(|t| t == id) {
            self.tools_used.push(id.into());
        }
    }

    pub fn limit(&mut self, text: impl Into<String>) {
        let t = text.into();
        if !self.limitations.contains(&t) {
            self.limitations.push(t);
        }
    }

    pub fn granted(&self, perm: &str) -> bool {
        self.granted.get(perm).copied().unwrap_or(false)
    }

    pub fn cancelled(&self) -> bool {
        self.cancel.load(Ordering::SeqCst)
    }

    /// A file the user authorized: inside an online authorized library folder, an imported file,
    /// or a file in COSOUND's own data folder. Anything else is refused.
    pub fn authorized_file(&self, raw: &str) -> Result<PathBuf, AgentError> {
        let p = std::fs::canonicalize(raw).map_err(|e| AgentError::new("FILE_NOT_FOUND", format!("{raw}: {e}")))?;
        if !p.is_file() {
            return Err(AgentError::new("FILE_NOT_FOUND", format!("{raw} is not a file")));
        }
        let c = self.db.lock().unwrap();
        let mut allowed: Vec<PathBuf> = crate::library::roots(&c).map_err(|e| AgentError::new("EXECUTION_FAILED", e.to_string()))?.into_iter().filter(|r| r.online).map(|r| PathBuf::from(r.path)).collect();
        allowed.push(self.data_dir.to_path_buf());
        let inside = allowed.iter().any(|a| p.starts_with(std::fs::canonicalize(a).unwrap_or_else(|_| a.clone())));
        let imported = {
            let mut st = c.prepare("select path from imports").map_err(|e| AgentError::new("EXECUTION_FAILED", e.to_string()))?;
            let paths: Vec<String> = st.query_map([], |r| r.get(0)).map(|m| m.filter_map(Result::ok).collect()).unwrap_or_default();
            paths.iter().any(|ip| std::fs::canonicalize(ip).is_ok_and(|ip| ip == p))
        };
        if inside || imported {
            Ok(p)
        } else {
            Err(AgentError::new("PERMISSION_DENIED", format!("{raw} is outside the folders the user authorized (Smart Library folders, imported files, COSOUND's folder)")))
        }
    }

    /// Tools `audio.wav_meter` (+ `audio.decoder` for non-WAV formats) on an authorized file
    /// (shared by every agent that measures audio).
    pub fn measure_wav(&mut self, raw: &str) -> Result<(PathBuf, crate::audio::wav::Measurements), AgentError> {
        let path = self.authorized_file(raw)?;
        let ext = path.extension().map(|e| e.to_string_lossy().to_lowercase()).unwrap_or_default();
        self.tool("audio.wav_meter");
        let cancel = self.cancel;
        if ext != "wav" && ext != "wave" {
            if !crate::smartlib::decode::AUDIO_EXTS.contains(&ext.as_str()) {
                return Err(AgentError::new("UNSUPPORTED_FORMAT", format!("{raw}: format not supported (WAV, AIFF, CAF, MP3, M4A, FLAC, OGG)")));
            }
            self.tool("audio.decoder");
            let mut meter: Option<crate::audio::wav::Meter> = None;
            let mut frame = Vec::new();
            let d = crate::smartlib::decode::decode(&path, None, &|| cancel.load(Ordering::SeqCst), &mut |buf, ch, rate| {
                let m = meter.get_or_insert_with(|| crate::audio::wav::Meter::new(rate, ch));
                frame.resize(ch, 0.0);
                for f in buf.chunks_exact(ch) {
                    for (o, v) in frame.iter_mut().zip(f) {
                        *o = *v as f64;
                    }
                    m.push(&frame);
                }
            });
            return match d {
                Ok(d) => meter.map(|m| (path, m.finish("decoded", d.bit_depth.unwrap_or(0) as u16))).ok_or_else(|| AgentError::new("UNSUPPORTED_FORMAT", format!("{raw}: no audio decoded"))),
                Err(crate::smartlib::decode::DecodeError::Cancelled) => Err(AgentError::new("CANCELLED", "analysis stopped")),
                Err(crate::smartlib::decode::DecodeError::Unsupported(e)) | Err(crate::smartlib::decode::DecodeError::Io(e)) => Err(AgentError::new("UNSUPPORTED_FORMAT", format!("{raw}: {e}"))),
            };
        }
        match crate::audio::wav::analyze(&path, &|| cancel.load(Ordering::SeqCst)) {
            Ok(m) => Ok((path, m)),
            Err(crate::audio::wav::MeterError::Cancelled) => Err(AgentError::new("CANCELLED", "analysis stopped")),
            Err(crate::audio::wav::MeterError::Invalid(e)) => Err(AgentError::new("UNSUPPORTED_FORMAT", format!("{raw}: {e}"))),
        }
    }

    /// Write a file inside COSOUND's data folder (`generated/<sub>/`) and record it.
    pub fn write_file(&mut self, sub: &str, name: &str, bytes: &[u8], kind: &str) -> Result<FileRef, AgentError> {
        let safe = |s: &str| s.chars().map(|c| if c.is_ascii_alphanumeric() || c == '-' || c == '_' || c == '.' { c } else { '-' }).collect::<String>();
        let dir = self.data_dir.join("generated").join(safe(sub));
        std::fs::create_dir_all(&dir).map_err(|e| AgentError::new("EXECUTION_FAILED", e.to_string()))?;
        let path = dir.join(safe(name));
        std::fs::write(&path, bytes).map_err(|e| AgentError::new("EXECUTION_FAILED", e.to_string()))?;
        let f = FileRef { path: path.to_string_lossy().to_string(), kind: kind.into(), bytes: bytes.len() as u64, sha256: Sha256::digest(bytes).iter().map(|b| format!("{b:02x}")).collect() };
        self.files.push(f.clone());
        Ok(f)
    }
}

/// Small input helpers shared by the agents.
pub fn s<'v>(input: &'v Value, k: &str) -> Option<&'v str> {
    input[k].as_str().filter(|v| !v.trim().is_empty())
}

pub fn unknown(agent: &str, op: &str) -> AgentError {
    AgentError::new("OPERATION_UNKNOWN", format!("{agent} has no operation {op:?}"))
}
