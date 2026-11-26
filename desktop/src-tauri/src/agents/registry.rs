//! Agent Registry, Tool Registry and permission catalog. The source of truth is the JSON in
//! packages/agents/ (one manifest per agent, tools.json, permissions.json): language-neutral, so
//! the central Agent can read it whatever its stack. Compiled into the app and checked by tests.
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Order of the agents in the registry (A–M of Prompt 06).
#[cfg_attr(not(test), allow(dead_code))]
pub const AGENT_IDS: [&str; 13] = [
    "music_producer",
    "rhythm_engineer",
    "sound_designer",
    "music_theory",
    "voice_to_music",
    "audio_analysis",
    "reference_track",
    "smart_library",
    "plugin_intelligence",
    "mixing_engineer",
    "mastering_engineer",
    "daw_controller",
    "adaptive_workflow",
];

const MANIFESTS: [&str; 13] = [
    include_str!("../../../../packages/agents/music_producer.json"),
    include_str!("../../../../packages/agents/rhythm_engineer.json"),
    include_str!("../../../../packages/agents/sound_designer.json"),
    include_str!("../../../../packages/agents/music_theory.json"),
    include_str!("../../../../packages/agents/voice_to_music.json"),
    include_str!("../../../../packages/agents/audio_analysis.json"),
    include_str!("../../../../packages/agents/reference_track.json"),
    include_str!("../../../../packages/agents/smart_library.json"),
    include_str!("../../../../packages/agents/plugin_intelligence.json"),
    include_str!("../../../../packages/agents/mixing_engineer.json"),
    include_str!("../../../../packages/agents/mastering_engineer.json"),
    include_str!("../../../../packages/agents/daw_controller.json"),
    include_str!("../../../../packages/agents/adaptive_workflow.json"),
];
const TOOLS: &str = include_str!("../../../../packages/agents/tools.json");
const PERMISSIONS: &str = include_str!("../../../../packages/agents/permissions.json");
#[cfg(test)]
pub const MANIFEST_SCHEMA: &str = include_str!("../../../../packages/contracts/agent-manifest.schema.json");
#[cfg(test)]
pub const RESULT_SCHEMA: &str = include_str!("../../../../packages/contracts/task-result.schema.json");
#[cfg(test)]
pub const REQUEST_SCHEMA: &str = include_str!("../../../../packages/contracts/task-request.schema.json");

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct Bi {
    pub it: String,
    pub en: String,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct BiList {
    pub it: Vec<String>,
    pub en: Vec<String>,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct OpSpec {
    pub name: String,
    /// available | planned
    pub status: String,
    /// read | low | medium | high
    pub risk: String,
    #[serde(default)]
    pub requires_approval: bool,
    pub summary: Bi,
    /// JSON Schema of the input object.
    pub input: Value,
    pub output: String,
    #[serde(default)]
    pub tools: Vec<String>,
    #[serde(default)]
    pub permissions: Vec<String>,
    #[serde(default)]
    pub basis: Option<String>,
    /// For planned operations: what is missing before it can run.
    #[serde(default)]
    pub requires: Option<String>,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct Manifest {
    pub contract_version: String,
    pub id: String,
    pub name: String,
    pub version: String,
    pub role: Bi,
    pub competencies: BiList,
    pub operations: Vec<OpSpec>,
    pub tools: Vec<String>,
    pub permissions: Vec<String>,
    pub limits: BiList,
    pub input_formats: Vec<String>,
    pub output_formats: Vec<String>,
    #[serde(default)]
    pub collaborates_with: Vec<String>,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct Tool {
    pub id: String,
    pub name: String,
    pub layer: u8,
    /// available | planned
    pub status: String,
    #[serde(default)]
    pub module: Option<String>,
    pub description: Bi,
    #[serde(default)]
    pub requires: Option<String>,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct Permission {
    pub id: String,
    /// Local setting holding the user's choice ("1"/"0"); None = not a user toggle.
    #[serde(default)]
    pub setting: Option<String>,
    pub default: bool,
    /// false = cannot be granted in this version (the component does not exist yet).
    pub grantable: bool,
    pub label: Bi,
    pub description: Bi,
}

pub struct Registry {
    pub manifests: Vec<Manifest>,
    pub tools: Vec<Tool>,
    pub permissions: Vec<Permission>,
}

impl Registry {
    pub fn load() -> Result<Registry, String> {
        let manifests = MANIFESTS.iter().map(|s| serde_json::from_str::<Manifest>(s).map_err(|e| format!("agent manifest: {e}"))).collect::<Result<Vec<_>, _>>()?;
        Ok(Registry {
            manifests,
            tools: serde_json::from_str(TOOLS).map_err(|e| format!("tools.json: {e}"))?,
            permissions: serde_json::from_str(PERMISSIONS).map_err(|e| format!("permissions.json: {e}"))?,
        })
    }
    pub fn manifest(&self, id: &str) -> Option<&Manifest> {
        self.manifests.iter().find(|m| m.id == id)
    }
}
