# Synthesizers – sound design and synthesis knowledge

Section 5 (Sound Design) and part of Section 11 (Plugin Intelligence). Verified 2026-11-24.
Sources: [`sources.md`](sources.md). Format/SDK details (how a host sees parameters, preset
containers) live in [`../integration-sdks/`](../integration-sdks/README.md); effects in
[`../plugins-effects/`](../plugins-effects/README.md).

How to read this file:
- Facts carry a citation `[Sn]`. Anything without a working source is marked **(to verify)**.
- **Recipes (section 3) are editorial starting points** built from the cited principles. They are
  not vendor measurements. Values are given as generic ranges or relative positions, so the agent
  has to map them onto each synth's own scaling. Treat them as heuristics to tune by ear.

---

## 1. Synthesis methods

### 1.1 Subtractive
- Starts from a harmonically rich oscillator and removes content with a filter, then shapes
  loudness over time with an amplifier and envelope [S1][S2]. Waveform names are shorthand for
  particular harmonic recipes [S2]:
  - **Sawtooth:** all harmonics at amplitude 1/n. Bright, the default for leads, basses and pads.
  - **Square:** odd harmonics only, giving a hollow sound.
  - **Triangle:** odd harmonics that fall away faster, so it sounds soft.
  - **Sine:** the fundamental only [S9].
- Changing pulse width changes the harmonic mix. PWM (pulse-width modulation) is the classic route
  to string and pad movement [S1] (series topic; detail **to verify** per article).

### 1.2 FM / phase modulation
- One oscillator (the modulator) varies the frequency, or the phase, of another (the carrier). At
  audio rates this produces sidebands instead of audible vibrato. Chowning discovered the effect by
  speeding vibrato up until it became a change of timbre [S12].
- **C:M ratio** sets where the sidebands fall. An integer ratio gives a harmonic spectrum:
  - 1:1 approximates a filtered sawtooth.
  - 1:2 gives odd harmonics only, like a filtered square.
  - A non-integer ratio gives inharmonic, bell-like or metallic spectra [S8].
- **Modulation index (β)** sets how much energy moves into the sidebands. In practice it plays the
  role of "brightness", so an envelope on the index works like a filter envelope [S8][S12].
- Most "FM" in software is phase modulation in practice. The two are closely related
  formulations [S12].

### 1.3 Wavetable
- A stored single-cycle waveform is read repeatedly to make a periodic tone. Crossfading or scanning
  between stored tables gives an evolving timbre. Vector synthesis is interpolation between several
  wavetables [S11].
- Frequency-domain wavetables are built from a target harmonic spectrum by inverse Fourier series.
  Interpolating between them is smoother [S11].
- Modern instruments add "warp" modes (FM, sync, bend, spectral warping) on top of table position.
  See the product profiles below.

### 1.4 Granular
- Sound is built from many short "grains", typically under 50 ms and often 10–30 ms, at densities
  of hundreds to thousands per second. The grain source can be a wavetable, FM or a sample
  [S14]. Xenakis and Roads proposed the technique, and Truax made it real-time in 1986 [S14].
- Typical controls [S13]:
  - **Position:** where in the source grains are taken.
  - **Size/length** of each grain.
  - **Density/rate**, or "Overlap" in some products.
  - Pitch, and randomisation ("spray"/jitter) of these values.
- How Size and Density interact [S13]:
  - Small grains at low density sound glitchy and discontinuous.
  - Higher density turns into a thin, pointillist haze.
  - Long grains at high density give smooth, evolving textures.

### 1.5 Additive
- Builds a sound as a sum of sinusoids, each with its own amplitude and frequency envelope. This is
  Fourier synthesis, now often called sinusoidal modelling [S9][S10].
- Implementations range from banks of sine oscillators to an inverse FFT [S10].
- Drawbar organs are a nine-partial additive instrument [S9].
- The cost is control: there are many parameters per note, so products expose macro-level
  controls on top. Harmor, for example, runs filters and phasers on additive data [S40].

---

## 2. Building blocks

### 2.1 Oscillators
- Choose the waveform for its harmonic content (see 1.1). Common oscillator-level controls:
  - Octave, semitone and fine tune.
  - Unison voice count and detune.
  - Hard sync.
  - Phase/retrigger.
  - Sub-oscillator.
- Example: Sylenth1 runs up to 8 unison voices per oscillator [S33].
- **Unison/detune**: several copies of the oscillator, slightly detuned and spread in stereo,
  thicken the sound. The Roland JP-8000 "Super Saw" is described as seven sawtooth waves detuned
  against each other [S15].

### 2.2 Filters
- **Types:**
  - Low-pass (LP), high-pass (HP) and band-pass (BP) [S4].
  - Notch/band-reject, made by combining LP and HP responses [S4].
  - Comb, formant and others are product-specific. For example, Albino has Comb-type filters [S26].
- **Slope (poles):** a single RC stage gives 6 dB/octave. Cascading stages gives 12, 18 or
  24 dB/octave [S3].
  - Steeper slopes sound darker and more "synth".
  - The TB-303 used an 18 dB/octave ladder low-pass [S16].
  - Sylenth1 offers 12 or 24 dB modes [S34].
- **Cutoff and resonance:** resonance boosts frequencies around the cutoff. Pushed high, the filter
  self-oscillates at the cutoff frequency, and the resonant low-pass sweep is the signature of
  subtractive synthesis [S4].
- **Drive:** saturation before or inside the filter. Examples:
  - Sylenth1 has a drive knob per filter stage [S34].
  - Albino has a saturation knob before each filter [S26].
  - Predator-3 has pre-filter distortion [S23].
- **Key tracking:** the cutoff follows the played note, so high notes don't sound duller than low
  ones. Many synths offer keyboard tracking as a modulation source (Sylenth1 does [S33]); Massive XL
  uses "Trackers" [S19]. Typical settings: about 0–50 % for basses, up to 100 % for leads, so
  timbre stays even across the keyboard (editorial heuristic).

### 2.3 Envelopes (ADSR and beyond)
- **ADSR** [S5]:
  - **Attack** sets how percussive the onset is.
  - **Decay** is the fall time to the sustain level.
  - **Sustain** is the level held while the key is down.
  - **Release** is the fade after key-up.
- Extended forms in the profiled products:
  - Sustain-fade in Predator-3 [S23].
  - Multi-stage looping envelopes: Predator-3 Multi-ENV, up to 15 stages [S23]; Albino 5-stage
    looping [S26].
  - Tempo-synced envelopes [S23].
- Typical routing: an amp envelope on the VCA, and a second envelope on filter cutoff or FM index.

### 2.4 LFOs
- Sub-audio periodic modulators. Typical parameters [S6]:
  - Waveform and rate (free or tempo-synced).
  - Depth, which is usually set at the destination.
  - Phase, delay/fade-in, and retrigger mode (poly/mono/free).
- Example: Predator-3 LFOs have Poly, Mono and Free modes and phase control [S23].
- Audio-rate and keytracked LFOs blur into FM (Vital supports both [S29]).

### 2.5 Modulation matrix
- A list of source → destination slots, each with an amount (sometimes a "via" scaler too).
- Examples:
  - Sylenth1: 16 slots [S34].
  - Albino-3: 16 routings across 27 sources and 36 destinations [S26].
  - Predator-3: 20 slots [S23].
- Modern UIs use drag-and-drop modulation instead of an explicit matrix: Vital [S29], Massive XL
  [S19].
- **Macros** map one control to many parameters. These are often the only controls exposed to the
  DAW or to hardware. See Massive/Massive XL below.

---

## 3. Genre recipes (generic parameter terms)

Editorial starting points. Each references the principle it relies on. Values are deliberately
relative, because every synth scales its knobs differently.

| Sound | Oscillators | Filter | Envelopes | Modulation / FX | Principle |
|---|---|---|---|---|---|
| **Sub bass** | One sine, or a triangle; mono, no unison; legato glide optional | LP fully open or off; key tracking irrelevant for a sine | Amp: attack ≈0, sustain full, short release (avoid clicks without smearing) | Keep mono, no stereo FX; light saturation adds audible harmonics on small speakers | A sine has only the fundamental [S2][S9]; distortion adds harmonics (see plugins-effects) |
| **Supersaw lead** | Sawtooth(s) with high unison count (≈7 voices), moderate detune, wide stereo spread | LP fairly open, mild resonance; HP to clean lows | Amp: fast attack, high sustain, medium release | Chorus/reverb/delay for width | JP-8000 Super Saw = 7 detuned saws [S15]; detuned copies thicken [S11] |
| **Plucky arp** | Saw or square, optional light unison | LP low-to-mid cutoff, moderate resonance, positive filter-env amount, key tracking partial | Filter env: attack 0, short decay, sustain 0; amp env: attack 0, short-medium decay, low sustain | Tempo-synced delay; arpeggiator | The decay of the filter envelope defines the "pluck" [S5][S4] |
| **Pad** | Two or more saws/wavetables detuned, or PWM; table position slowly modulated | LP mid cutoff, low resonance | Amp: slow attack, high sustain, long release; filter env slow | Slow LFO on cutoff/table/pan; chorus + long reverb | Movement from slow modulation and detune [S6][S11]; reverb in plugins-effects |
| **Acid bass (303-style)** | One saw (biting) or square (rounder); mono with glide/slide on tied notes | LP 18–24 dB/oct, low cutoff, high resonance; heavy env amount; cutoff automated open over time | Filter env: short decay, sustain 0 (snappy); accents raise volume and env depth | Distortion after the synth is common | 303 = saw/square into −18 dB/oct ladder LPF [S16]; Roland's programming tips: low cutoff, high resonance, slides, accents [S17]; resonance [S4] |
| **FM bell / e-piano** | 2-operator: carrier + modulator at a non-integer ratio (bell) or integer ratio (EP) | Often none | Index envelope: fast attack, medium decay (brightness falls) | Reverb | Non-integer C:M ratio gives inharmonic spectra; the index envelope controls brightness [S8][S12] |
| **Granular texture** | Sample source; large grain size, high density, slowly moving position, a little random pitch/position | LP/HP to taste | Slow amp envelope | Long reverb | Size and density interaction [S13][S14] |

---

## 4. Plugin profiles

Legend for the host-automation column:
- **documented**: the vendor states which parameters are exposed to the host.
- **standard**: the plug-in exposes host parameters as its format requires, but the scope is not
  documented (to verify by enumerating with a host, see integration-sdks).

### 4.1 Native Instruments Massive (classic)
- **Developer:** Native Instruments [S18]. Version: 1.x (exact build not stated on the page)
  **(to verify)**.
- **Synthesis:**
  - "Wave Scanning" wavetable synthesis in three parallel oscillators.
  - A modulation oscillator for ring, phase and other modulation.
  - Noise, comb filters, and a subtractive-style signal flow.
  - Stepper and Performer step-modulators.
  - 1,300 presets [S18].
- **Formats:**
  - Windows 64-bit: stand-alone, VST, AAX.
  - Mac 64-bit: stand-alone, VST, AU, AAX.
  - No VST3 is listed [S18].
  - Systems: macOS 15/26/27, Windows 10/11 [S18].
- **Macros:** eight macro controls, auto-mapped to Komplete Kontrol S-series and Maschine
  hardware [S18].
- **Preset file format/location:** not officially documented. The commonly cited `.nmsv` extension
  is **(to verify)**.
- **Host automation:** standard (scope not documented). **(to verify)**.

### 4.2 Native Instruments Massive X → Massive XL
- On 2026-11-24 the Massive X product URL redirects to **Massive XL**. The user guide calls it
  software version **2.0 (10/2026)** and says Massive X presets and Expansions are fully compatible
  [S19][S20]. Treat Massive XL as the current version of Massive X.
- **Synthesis** [S19][S20]:
  - Two wavetable oscillators with ten oscillator modes that change how tables are read.
  - Hundreds of tables, including OG Massive, PPG-style and user imports.
  - Noise, filters, inserts, phase modulation and feedback, in an open routing architecture.
  - Performers (drawn rhythmic modulation) and Trackers (per-note response).
- **New in XL** [S19]:
  - Preset Generator (audio file → four editable presets, with stem separation).
  - Mutator and Randomizer.
  - Morpher and Animator.
  - Built-in effects including Raum, Replika Shimmer and Tape Wobble.
- **Formats:**
  - macOS 64-bit: VST, VST3, AU, AAX.
  - Windows 64-bit: VST, VST3, AAX.
  - No stand-alone.
  - macOS 15/26/27 or Windows 10/11 [S19].
- **Host automation (documented):**
  - Macros 1–8 and the Morpher X/Y position are available for host automation.
  - A setting chooses whether Morpher moves report X/Y, Macros, both, or nothing.
  - The full version has 16 Macros [S20].
  - Whether every internal parameter is exposed is **(to verify)**. Design COSOUND around the
    Macros.
- **Presets:** stored in a "User Content" folder, managed from Settings → Show User Content
  Folder [S20]. The file format is not documented.

### 4.3 Rob Papen Predator-3
- **Developer:** Rob Papen Inspiration Soundware [S21]. The page doesn't give a build number.
- **Synthesis:** a hybrid synth with analog-style and hybrid wavetable sounds [S21][S23]:
  - Three oscillators with sub-oscillators, analog-modelled waves and additive "spectrum" waves,
    with 8 user spectrum waves per preset.
  - Two analog-modelled filters, 50 filter types including Albino-3 filters.
  - 4 LFOs, 4 envelopes and 2 Multi-ENVs.
  - An audio follower (sidechain).
  - A 20-slot mod matrix, 4 arpeggiators and an XY pad.
  - More than 7,000 presets. Bank folders starting with 0 hold Predator-2/3 presets, and those
    starting with 1 hold Predator-1 presets.
- **Formats** [S22]:
  - Windows: 64-bit VST3, AAX and VST, Windows 7+ (AAX for Pro Tools 12+).
  - macOS: AU, VST3, AAX and VST, 10.15+, ARM compatible.
  - NI NKS compatible.
- **Preset format:** not documented publicly **(to verify)**. NKS compatibility implies NKS preset
  files exist for Komplete Kontrol (to verify).
- **Host automation:** standard **(to verify)**.

### 4.4 Rob Papen Albino-3 Legend
- **Developer:** Rob Papen. Originally released in 2006 (with Peter Linsener) and re-released as
  "Legend", adding VST3 and current OS/CPU support [S24].
- **Synthesis** [S26]:
  - Four layers per preset, each with 4 oscillators. Oscillator types: Analog, Digital, Noise and
    Audio-Input, with FM, AM and sync.
  - 4 stereo filter types: Silk, Cream, Scream and Comb.
  - 8 envelopes and 4 LFOs.
  - A 16-routing mod matrix with 27 sources and 36 destinations.
  - 12 effect types and a 32-step arpeggiator.
  - Scala microtuning, 32 voices, and 2,100+ presets.
- **Formats:** same matrix as Predator-3 (VST3/AAX/VST on Windows; AU/VST3/AAX/VST on macOS 10.15+;
  ARM), NKS compatible [S25].
- **Presets:** "stored directly on hard disk", sorted into style maps [S26]. The file format is not
  documented.
- **Host automation:** standard **(to verify)**. Controller (faderbox) mappings can be saved [S26].

### 4.5 Xfer Records Serum 2 (and Serum 1)
- **Developer:** Xfer Records. Version: demo installers are named 2.1.5 (inferred) [S27].
- **Synthesis** [S27]:
  - Multiple oscillator types: wavetable (refined, with filtering, FM, PD, ring mod, distortion
    and dual warps), multisample (SFZ import), sample (looping, slicing), granular, and spectral
    (real-time harmonic resynthesis).
  - Flexible effect routing, an arpeggiator and a clip sequencer.
  - 626+ presets and 288 wavetables.
- **Formats:** 64-bit **VST3, AU, AAX**. VST2 and CLAP are not listed. Windows 10+; macOS High
  Sierra+ (Intel) or Big Sur+ (Apple Silicon) [S27]. Free upgrade for Serum 1 owners [S27].
- **Preset format:**
  - Serum 1 uses `.fxp`, a VST2-style preset container (see integration-sdks).
  - Serum 2 introduced `.SerumPreset`.
  - The developer states the Serum preset format "is complex/programmatic and isn't public", while
    the Serum *wavetable* format (a `clm ` chunk in WAV) is public [S28].
  - Community posts report `.SerumPreset` has a JSON header followed by a zlib stream, and a
    community tool converts it to and from JSON. This is reverse engineering (C quality). Do not
    rely on it for writing presets [S28].
- **Host automation:** standard **(to verify)**.

### 4.6 Vital (Vital Audio / Matt Tytel)
- **Synthesis:** "spectral warping wavetable synthesizer" [S29][S30]. Features [S29]:
  - Wavetable creation from samples, a built-in editor and text-to-wavetable.
  - Drag-and-drop modulation and custom LFO shapes.
  - Audio-rate modulation and keytracked LFOs.
  - MPE, and microtuning via `.tun`/`.scl`/`.kbd` files.
- **Platforms:** Windows 10+, macOS 10.15+ (Intel and Apple Silicon), Ubuntu 18.04+; 64-bit and
  OpenGL 3 required [S29].
- **Tiers:** Basic, Plus and Pro differ only in content [S29].
- **Formats:**
  - The open-source project file builds VST, VST3, AU, AUv3 and stand-alone [S30].
  - Vital Audio is listed as a CLAP vendor [S31].
  - Current binary version (1.5.x reported) and its exact formats are **(to verify)** [S32].
- **Source:** GPLv3. Closed-source use requires a separate licence. Free-tier presets may not be
  redistributed [S30].
- **Preset format (verified in source):**
  - Presets use the extension `.vital`. Saving writes the synth state serialized as **JSON text**
    (`saveToJson().dump()` → `replaceWithText`).
  - Related extensions: `.vitaltable` (wavetable), `.vitalskin`, `.vitallfo`, `.vitalbank`
    [S30].
  - This is the most COSOUND-friendly preset format of the set. It is readable and writable, but
    field semantics must be inferred from the source, and audio/wavetable data can be embedded as
    encoded buffers [S30].
- **Host automation:** standard, built on JUCE (to verify scope).

### 4.7 LennarDigital Sylenth1
- **Developer:** LennarDigital (NL). Current: demo **v3.075 (Windows) / v3.073 (macOS)** [S35].
- **Synthesis:** virtual-analog subtractive [S33][S34]:
  - 4 band-limited unison oscillators with up to 8 voices each.
  - 2 analog-style 4-stage filters per voice, in LP/BP/HP at 12 or 24 dB, with drive.
  - 4 ADSRs (2 for modulation and 2 for amplitude), 2 LFOs with 11 waveforms, and 16 mod slots.
  - An arpeggiator and 7 effects: distortion (5 types, 4× oversampled), phaser, chorus/flanger, EQ,
    delay, reverb and compressor.
  - 16-note polyphony and 2,500+ presets.
- **Formats:** requirements list "VSTi, AU or AAX compatible host". Windows 7/8/10/11 (32/64-bit),
  macOS 10.9+ (ARM/Intel) [S34]. The manual mentions the VST3 trademark [S36], but VST3 availability
  is **(to verify)**.
- **Preset format:** single presets save as **`.fxp`**, and banks of 512 presets as **`.fxb`**
  [S36] (manual v2.5). Both are VST2-era containers whose inner chunk is plug-in-private (see
  integration-sdks).
- **MIDI:** fully MIDI-controllable, with MIDI learn to CC/RPN/NRPN [S34].
- **Host automation:** standard **(to verify)**.

### 4.8 Summary matrix

| Plugin | Synthesis | Formats (vendor) | Preset file | Format public? | Automation scope |
|---|---|---|---|---|---|
| Massive | Wavetable ("wave scanning") VA | VST, AU, AAX, SA | (to verify) | No | to verify |
| Massive XL (ex-Massive X) v2.0 | Wavetable, open routing | VST, VST3, AU, AAX | User Content folder | No | Macros + Morpher documented |
| Predator-3 | Hybrid VA/additive/wavetable | VST, VST3, AU, AAX; NKS | (to verify) | No | to verify |
| Albino-3 Legend | 4-layer VA/digital/FM | VST, VST3, AU, AAX; NKS | (to verify) | No | to verify |
| Serum 2 (2.1.5) | Wavetable, sample, multisample, granular, spectral | VST3, AU, AAX | `.SerumPreset` (S1: `.fxp`) | **No** (vendor statement) | to verify |
| Vital | Spectral-warping wavetable | VST/VST3/AU (+CLAP listed) | `.vital` = JSON | Readable (GPL source) | to verify |
| Sylenth1 3.07x | VA subtractive | VST, AU, AAX (VST3 to verify) | `.fxp` / `.fxb` | Container yes, inner chunk no | to verify |

---

## 5. DAW stock instruments (brief)

| DAW | Instrument | What it is |
|---|---|---|
| Logic Pro 13 | **Alchemy** | Additive, spectral and granular synthesis/resynthesis, sampling and VA engines; up to four sources per preset, morphable; Perform controls mappable to almost any parameter [S37] |
| | **Retro Synth** | One synth with four oscillator engines: Analog, Sync, Table (wavetable) and FM [S37] (from guide section titles) |
| | **ES2** | Three-oscillator hybrid synth with PWM, FM, ring mod, "Digiwaves", a mod router, Vector Envelope and Planar Pad, and filter self-oscillation/overdrive [S37] (from guide section titles) |
| Ableton Live 12 | **Wavetable** | Two wavetable oscillators, two analog-modelled filters and a modulation system [S38] |
| | **Operator** | FM combined with subtractive and additive synthesis; four multi-waveform oscillators [S38] |
| | **Drift** | Subtractive, MPE-capable, low-CPU synth [S38] |
| FL Studio | **Sytrus** | Six operators; FM, RM, subtractive and additive; 3 filters; FX [S39] |
| | **Harmor** | Additive engine that applies "subtractive" modules (filters, phasers) to additive data, not to audio [S40] |
| Cubase | **Retrologue** | VA synth **(to verify)** [S41] |
| | **HALion Sonic** | Sample- and synth-based workstation player **(to verify)** [S42] |

Why this matters for COSOUND: stock instruments are always present and have DAW-native presets.
They are the safest first target for recipe execution. Third-party synths are the second target.
