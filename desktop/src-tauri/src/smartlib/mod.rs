//! Smart Library System (Prompt 07): the technical engine that scans, indexes, analyzes, classifies
//! and searches the user's music libraries. Local-first; never modifies, moves or deletes user files.
//! The Smart Library Agent (agents/smart_library.rs) interprets requests and calls this engine.
pub mod analyze;
pub mod api;
pub mod classify;
pub mod decode;
pub mod dupes;
pub mod features;
pub mod index;
pub mod midi;
pub mod presets;
pub mod query;
pub mod scan;
pub mod search;
pub mod service;

#[cfg(test)]
mod tests;
