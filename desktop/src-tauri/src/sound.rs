//! Tools `sound.recipes` and `plugins.catalog`: patch recipes in generic synth terms and facts about
//! well-known synthesizers. Source: docs/knowledge/synthesizers/README.md (§3 recipes, §4 profiles).
//! Recipes are starting points written in generic parameter terms (no plugin-specific values).
use serde::Serialize;

pub const SOURCE_RECIPES: &str = "docs/knowledge/synthesizers/README.md §3";
pub const SOURCE_SYNTHS: &str = "docs/knowledge/synthesizers/README.md §4";

#[derive(Serialize, Clone, Copy)]
pub struct Recipe {
    pub id: &'static str,
    pub label: &'static str,
    pub roles: &'static [&'static str],
    pub oscillators: &'static str,
    pub filter: &'static str,
    pub envelopes: &'static str,
    pub modulation_fx: &'static str,
    pub principle: &'static str,
    /// Keep the sound mono (sub/bass): no stereo-widening variants are offered.
    pub mono: bool,
}

pub const RECIPES: [Recipe; 7] = [
    Recipe { id: "sub_bass", label: "Sub bass", roles: &["bass", "sub"], mono: true,
        oscillators: "One sine, or a triangle; mono, no unison; legato glide optional",
        filter: "Low-pass fully open or off; key tracking irrelevant for a sine",
        envelopes: "Amp: attack ≈0, full sustain, short release (no clicks, no smearing)",
        modulation_fx: "Keep mono, no stereo FX; light saturation adds harmonics audible on small speakers",
        principle: "A sine has only the fundamental; distortion adds harmonics" },
    Recipe { id: "acid_bass", label: "Acid bass (303-style)", roles: &["bass", "acid"], mono: true,
        oscillators: "One saw (biting) or square (rounder); mono with glide on tied notes",
        filter: "Low-pass 18–24 dB/oct, low cutoff, high resonance, heavy envelope amount; cutoff automated open over time",
        envelopes: "Filter env: short decay, sustain 0 (snappy); accents raise volume and envelope depth",
        modulation_fx: "Distortion after the synth is common",
        principle: "Saw/square into a resonant ladder low-pass; slides and accents" },
    Recipe { id: "supersaw_lead", label: "Supersaw lead", roles: &["lead"], mono: false,
        oscillators: "Sawtooth(s) with high unison (≈7 voices), moderate detune, wide stereo spread",
        filter: "Low-pass fairly open, mild resonance; high-pass to clean the lows",
        envelopes: "Amp: fast attack, high sustain, medium release",
        modulation_fx: "Chorus, reverb and delay for width",
        principle: "Detuned saw copies thicken the sound (JP-8000 Super Saw = 7 saws)" },
    Recipe { id: "pluck_arp", label: "Plucky arp", roles: &["pluck", "arp", "lead"], mono: false,
        oscillators: "Saw or square, optional light unison",
        filter: "Low-pass low-to-mid cutoff, moderate resonance, positive filter-envelope amount, partial key tracking",
        envelopes: "Filter env: attack 0, short decay, sustain 0; amp env: attack 0, short-medium decay, low sustain",
        modulation_fx: "Tempo-synced delay; arpeggiator",
        principle: "The filter envelope decay defines the pluck" },
    Recipe { id: "pad", label: "Pad", roles: &["pad", "chords"], mono: false,
        oscillators: "Two or more detuned saws/wavetables, or PWM; wavetable position slowly modulated",
        filter: "Low-pass mid cutoff, low resonance",
        envelopes: "Amp: slow attack, high sustain, long release; filter env slow",
        modulation_fx: "Slow LFO on cutoff/table/pan; chorus + long reverb",
        principle: "Movement from slow modulation and detune" },
    Recipe { id: "fm_bell", label: "FM bell / e-piano", roles: &["keys", "bell"], mono: false,
        oscillators: "2-operator FM: carrier + modulator at a non-integer ratio (bell) or integer ratio (e-piano)",
        filter: "Often none",
        envelopes: "Index envelope: fast attack, medium decay (brightness falls)",
        modulation_fx: "Reverb",
        principle: "Non-integer C:M ratios give inharmonic spectra; the index envelope controls brightness" },
    Recipe { id: "granular_texture", label: "Granular texture", roles: &["texture", "atmosphere", "fx"], mono: false,
        oscillators: "Sample source; large grains, high density, slowly moving position, a little random pitch/position",
        filter: "Low-pass/high-pass to taste",
        envelopes: "Slow amp envelope",
        modulation_fx: "Long reverb",
        principle: "Grain size and density interact" },
];

#[cfg(test)]
pub const ROLES: [&str; 11] = ["bass", "sub", "acid", "lead", "pluck", "arp", "pad", "chords", "keys", "texture", "atmosphere"];

/// Recipes for a role, best fit for the genre first.
pub fn recipes_for(role: &str, genre: Option<&str>) -> Vec<&'static Recipe> {
    let mut v: Vec<&Recipe> = RECIPES.iter().filter(|r| r.roles.contains(&role)).collect();
    if role == "bass" && matches!(genre, Some("techno" | "minimal")) {
        v.sort_by_key(|r| r.id != "acid_bass");
    }
    v
}

/// Rule-based variations of a recipe, in the same generic terms.
pub fn variants(r: &Recipe) -> Vec<(&'static str, Vec<&'static str>)> {
    let mut v = vec![
        ("brighter", vec!["Raise filter cutoff about a quarter of its range", "Increase filter-envelope amount", "Add a little resonance"]),
        ("darker", vec!["Lower filter cutoff", "Reduce resonance", "Shorten or lower the filter envelope"]),
        ("punchier", vec!["Shorten amp attack to 0", "Shorter filter-envelope decay", "Light saturation after the synth"]),
    ];
    if !r.mono {
        v.push(("wider", vec!["More unison voices or detune", "Add chorus", "Pan modulation or stereo delay"]));
    }
    v
}

#[derive(Serialize, Clone, Copy)]
pub struct Synth {
    pub id: &'static str,
    pub name: &'static str,
    pub vendor: &'static str,
    pub synthesis: &'static str,
    /// Lower-case substrings that identify the plugin in a scan (more specific first).
    #[serde(skip)]
    pub patterns: &'static [&'static str],
}

pub const SYNTHS: [Synth; 7] = [
    Synth { id: "massive_xl", name: "Massive XL (ex-Massive X)", vendor: "Native Instruments", synthesis: "Wavetable, open routing", patterns: &["massive x"] },
    Synth { id: "massive", name: "Massive", vendor: "Native Instruments", synthesis: "Wavetable (wave scanning), three oscillators, modulation oscillator", patterns: &["massive"] },
    Synth { id: "predator", name: "Predator-3", vendor: "Rob Papen", synthesis: "Hybrid: analog-modelled, additive spectrum and wavetable oscillators", patterns: &["predator"] },
    Synth { id: "albino", name: "Albino-3 Legend", vendor: "Rob Papen", synthesis: "Four layers of 4 oscillators (analog, digital, noise), FM/AM/sync", patterns: &["albino"] },
    Synth { id: "serum", name: "Serum / Serum 2", vendor: "Xfer Records", synthesis: "Wavetable", patterns: &["serum"] },
    Synth { id: "vital", name: "Vital", vendor: "Vital Audio", synthesis: "Spectral-warping wavetable", patterns: &["vital"] },
    Synth { id: "sylenth1", name: "Sylenth1", vendor: "LennarDigital", synthesis: "Virtual-analog subtractive, 4 unison oscillators", patterns: &["sylenth"] },
];

/// Known synth for a scanned plugin name, if any.
pub fn known_synth(plugin_name: &str) -> Option<&'static Synth> {
    let n = plugin_name.to_lowercase();
    SYNTHS.iter().find(|s| s.patterns.iter().any(|p| n.contains(p)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn catalog_lookups() {
        assert_eq!(known_synth("Massive X").unwrap().id, "massive_xl");
        assert_eq!(known_synth("Massive").unwrap().id, "massive");
        assert_eq!(known_synth("Serum2").unwrap().vendor, "Xfer Records");
        assert!(known_synth("Pro-Q 3").is_none());
        assert_eq!(recipes_for("bass", Some("techno"))[0].id, "acid_bass");
        assert_eq!(recipes_for("bass", Some("edm"))[0].id, "sub_bass");
        assert!(variants(&RECIPES[0]).iter().all(|v| v.0 != "wider"), "sub bass stays mono");
        assert!(recipes_for("kazoo", None).is_empty());
        for r in ROLES {
            assert!(!recipes_for(r, None).is_empty(), "{r}");
        }
    }
}
