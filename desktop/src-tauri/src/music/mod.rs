//! Music Intelligence Engine (Prompt 05 M2, extended by Prompt 09). Pure, deterministic modules:
//! - Music Theory Engine: `theory` (keys, modes, diatonic chords, roman numerals), `notes` (names, octaves, MIDI,
//!   frequencies, intervals)
//! - Scale & Key Engine: `scales` (catalog), `keys` (signatures, circle, modulation, estimation)
//! - Chord Engine: `chords`; Harmony Engine: `harmony`
//! - generators: `generate` (M2 parts), `melody`, `bass`, `arp`, `rhythm`
//! - Style Intelligence: `styles`; Composition Engine: `compose`; request parser: `request`
//! - MIDI Generation: `smf`; MIDI editing: `edit`; MIDI Analysis: `analysis`; Validation: `validate`
//! - bridges: `voice` (Voice-to-Music), `reference` (Reference Track); `api` (entry points); `caps` (M2 API)
pub mod analysis;
pub mod api;
pub mod arp;
pub mod arrangement;
pub mod bass;
pub mod caps;
pub mod chords;
pub mod compose;
pub mod edit;
pub mod generate;
pub mod harmony;
pub mod keys;
pub mod melody;
pub mod notes;
pub mod reference;
pub mod request;
pub mod rhythm;
pub mod scales;
pub mod smf;
pub mod styles;
pub mod theory;
pub mod validate;
pub mod voice;
