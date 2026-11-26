//! Prompt Engine (Prompt 10): natural language (text or voice transcripts, Italian and English) → a structured,
//! versioned interpretation for the partner's Central Agent. It understands and structures; it does not plan,
//! orchestrate or execute (no second Central Agent). Docs: docs/prompt-engine/.
pub mod api;
pub mod constraints;
pub mod context;
pub mod enhance;
pub mod extract;
pub mod instructions;
pub mod intent;
pub mod lexicon;
pub mod model;
pub mod profile;
pub mod safety;
pub mod text;

/// Version of the interpretation contract (`packages/contracts/prompt-interpretation.schema.json`).
pub const SCHEMA_VERSION: &str = "1.0";

#[cfg(test)]
mod tests;
