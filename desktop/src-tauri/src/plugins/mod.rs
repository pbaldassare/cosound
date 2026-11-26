//! Plugin Intelligence System (Prompt 08): a technical engine, separate from the Plugin Intelligence and Sound
//! Designer agents, which only call it. Modules:
//! - `scanner`    Plugin Scanner (folders + macOS Audio Unit registry, formats, versions, architectures)
//! - `registry`   Plugin Registry (local database, one product per plug-in across formats, DAW compatibility)
//! - `categories` multi-label catalog
//! - `knowledge`  Plugin Knowledge Engine (official documentation with source and version, knowledge base)
//! - `host`       isolated Audio Unit host (`cosound-auhost`, separate process with time limits)
//! - `params`     Parameter Discovery Engine (real parameters, sound roles, factory presets)
//! - `presets`    Preset Analyzer (preset folders, Smart Library presets, semantic search, timbre by rendering)
//! - `design`     Plugin Control Interface + Preset Management (proposals, audition, new presets, undo)
//! - `chains`     Effect Chain Engine (order, material, overload check)
//! - `daw`        DAW Integration Adapter (capabilities per DAW, plans for the DAW Controller)
//! - `api`        entry points for the app and the agents
pub mod api;
pub mod categories;
pub mod chains;
pub mod daw;
pub mod design;
pub mod host;
pub mod knowledge;
pub mod params;
pub mod presets;
pub mod registry;
pub mod scanner;

#[cfg(test)]
mod tests;
