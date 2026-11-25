//! Music engine, milestone M2 (Prompt 05 Part B): theory, deterministic MIDI generation, SMF export.
//! Runs inside the Rust core (see ADR-002 note in docs/architecture/DECISIONS.md): no AI, no network,
//! no external runtime. Exposed to the central Agent only through the Capability Bus (`caps`).
pub mod caps;
pub mod generate;
pub mod smf;
pub mod theory;
