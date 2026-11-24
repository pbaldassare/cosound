# Plugin formats, SDKs and Plugin Intelligence

Section 11 (Plugin Intelligence). Verified 2026-11-24. Sources: [`sources.md`](sources.md).
Per-synth facts are in [`../synthesizers/`](../synthesizers/README.md). DAW-specific scripting
detail belongs in each DAW folder and in `../daw-integration/`.

Capability classes (see the knowledge-base README) are given for each COSOUND route in section 3.

---

## 1. Plugin formats

### 1.1 VST2 (Steinberg, legacy)
- **Status:** replaced by VST3, which is "not compatible with the older VST (1 & 2)" [S11].
- **Licensing:**
  - Distributing a VST2 plug-in or host in binary form is allowed only for developers who signed
    the VST2 licence agreement **before October 2018**. Anyone else may not distribute.
  - The VST2 headers (`aeffect.h`, `aeffectx.h`) are under a licence that does **not** allow
    redistribution [S2].
  - → COSOUND must not ship its own VST2 hosting code built on those headers.
- **Host support is shrinking:**
  - In Cubase/Nuendo 14 and 15, VST2 is reported disabled by default, with discontinuation
    planned, and it can't be enabled natively on Apple Silicon [S38] (U, to verify).
  - Ableton lists 64-bit VST2 as still supported [S37] (U).
- **Presets:** `.fxp` (single program) and `.fxb` (bank) are the VST2-era containers. Sylenth1
  documents saving to them [S41]. Serum 1 also uses `.fxp` [S39]. The outer container is commonly
  documented, but **the inner chunk is opaque, plug-in-private data**. The public spec sits inside
  the non-redistributable VST2 SDK **(to verify)**.
- **VST2 "shell" plug-ins** (several plug-ins in one binary) are replaced in VST3 by a factory that
  can export several classes [S11].

### 1.2 VST3 (Steinberg)
- **SDK and licence:**
  - Current SDK **3.8.1 (2026-08-11)**; 3.8.0 was released 2025-10-20 [S3].
  - **Since 3.8 the SDK is MIT-licensed.** GPLv3 and the Steinberg proprietary licence are no
    longer offered [S1][S4].
  - No fees, no signed agreement, a perpetual licence. Closed-source binaries are allowed. The VST
    logo/trademark is optional but, if used, follows the usage guidelines [S1][S2].
  - MIT is stated to be compatible with GPLv3 projects (e.g. JUCE-based) [S2].
  - A C API exists at `steinbergmedia/vst3_c_api` [S9].
- **Platforms:** Windows, macOS, Linux (defined install paths for each) [S5].
- **Discovery** [S5]:
  - No registration step is required.
  - Hosts **recursively scan** fixed folders, User then Global then Application. The first plug-in
    found for a given Processor UID wins.
  - macOS: `~/Library/Audio/Plug-ins/VST3/`, `/Library/Audio/Plug-ins/VST3/`,
    `/Network/Library/Audio/Plug-ins/VST3/`, `$APP/Contents/VST3/`.
  - Windows: `%LOCALAPPDATA%\Programs\Common\VST3\`, `C:\Program Files\Common Files\VST3\`
    (+ `(x86)` for 32-bit), `$APP\VST3\`.
  - Linux: `~/.vst3/`, `/usr/lib/vst3/`, `/usr/local/lib/vst3/`, (lib64 variants), `$APP/vst3/`.
  - **`moduleinfo.json`** (optional, since 3.7.5, in `Contents/Resources` since 3.7.8) describes the
    factory and classes in JSON5. A host can learn what a bundle contains **without loading it**.
    The SDK includes `moduleinfotool` and a parser library [S10].
- **Parameters** [S8]:
  - The host calls `IEditController::getParameterCount` / `getParameterInfo`.
  - Each parameter has a stable 32-bit ID (0–0x7FFFFFFF; the upper range is reserved for hosts), a
    title, units, step count, default value and flags. `kCanAutomate` marks host-automatable
    parameters, and `kIsBypass` marks bypass.
  - Values are **normalized 0.0–1.0**. Plug-ins convert to and from display strings.
  - Plug-ins must not reassign IDs. They may only add or remove parameters across versions.
  - Some parameters are private or read-only (e.g. meters).
- **Processor/controller split:** `IComponent` + `IAudioProcessor` (DSP) are separate from
  `IEditController` (parameters/GUI) and may even run in different contexts [S9].
- **State** [S9]:
  - The host stores the processor state with `IComponent::getState/setState` and GUI-only state
    with `IEditController::getState/setState`.
  - On restore, the host passes the processor state to the processor first, then to the controller
    via `setComponentState`, and then rescans parameters.
  - The state blob is plug-in-defined (opaque).
- **Presets:**
  - `.vstpreset` [S7]: header `'VST3'`, version, a 32-byte ASCII class ID, and an offset to a chunk
    list (`'List'`, entries of id/offset/size). The SDK's `Vst::PresetFile` reads and writes it.
    The container is documented; **the component-state chunk inside is plug-in-private**.
  - Preset folders by OS [S6]:
    - macOS: `~/Library/Audio/Presets/$COMPANY/$PLUGIN/`, `/Library/Audio/Presets/...`.
    - Windows: `Documents\VST3 Presets\$COMPANY\$PLUGIN\`, `%APPDATA%\VST3 Presets\...`,
      `%PROGRAMDATA%\VST3 Presets\...`.
    - Linux: `~/.vst3/presets/...`, `/usr/share/vst3/presets/...`.
  - Preset types: User, User_Factory, Shared_Factory, App_Factory.
- **Hosts:** Cubase/Nuendo, Ableton Live (VST3 since 10.1 [S37]), FL Studio, REAPER, Bitwig, Studio
  One and many others. **Logic Pro does not host VST** (it hosts Audio Units [S43]) **(to verify
  per DAW)**.

### 1.3 Audio Units (Apple) – AUv2 and AUv3
- **AUv2 (legacy component bundles):**
  - Installed in `~/Library/Audio/Plug-Ins/Components/` or `/Library/Audio/Plug-Ins/Components/`
    [S18].
  - Types include effect (`aufx`), music effect (`aumf`) and others. Conformance is checked with
    Apple's **`auval`** tool [S18].
  - Factory presets are defined by the AU (`GetPresets` / `NewFactoryPresetSet`) [S17].
  - Hosts persist state through **`kAudioUnitProperty_ClassInfo`**, a `CFPropertyList` dictionary
    [S17]. A `.aupreset` file is that dictionary saved as a property list **(to verify: file
    naming/location conventions are not in the checked pages)**.
- **AUv3 (app extensions):**
  - Effects and instruments are delivered as Audio Unit v3 app extensions [S12].
  - API availability: macOS 10.11+, iOS/iPadOS 9+, visionOS 1+ [S13][S14].
- **Discovery:** `AVAudioUnitComponentManager` searches and queries the audio components registered
  with the system (macOS 10.10+) [S15].
- **Parameters:** `AUAudioUnit.parameterTree` is a hierarchical tree of `AUParameter`s [S14].
- **State and presets:**
  - `fullState` is "a persistable snapshot of the audio unit's properties and parameters, suitable
    for saving as a user preset" [S13].
  - `factoryPresets` holds developer presets, and `userPresets` holds user presets (macOS 10.15+)
    [S16].
- **Hosts:** Logic Pro, GarageBand, MainStage (macOS) [S43]; Ableton Live (AUv2, and AUv3 from 11.2)
  [S37] (U); also many others on macOS. **macOS/iOS only.**

### 1.4 AAX (Avid)
- **Purpose:** AAX is the plug-in format for **Pro Tools, VENUE | S6L and Media Composer** [S19].
- **SDK access** [S19]:
  - Through a click-through agreement in an Avid developer account. The agreement is a commercial
    agreement.
  - **Pro Tools Developer** runs unsigned plug-ins during development and requires an iLok licence
    request.
  - Shipping builds must be **signed with PACE tools**. These are free, need a physical iLok
    (2 or later), are issued per publisher, and do local signing only.
  - No fees. No Avid certification is required. iLok copy protection is optional.
  - Only the latest SDK/Pro Tools versions are supported.
- **Third-party hosting:** Avid's page describes AAX only as a format for its own products and
  offers no host programme [S19]. Ableton lists AAX as unsupported [S37]. **Conclusion: COSOUND
  cannot realistically host AAX plug-ins.** "No third-party AAX hosting" is an inference from the
  absence of any documentation (to verify with Avid).
- **Presets:** Pro Tools `.tfx` plug-in settings **(to verify)**.

### 1.5 CLAP (open standard)
- **Standard:**
  - CLever Audio Plug-in, an ABI with a stable 1.x line. A plug-in built for 1.x loads in any 1.y
    host. **MIT licence** [S20].
  - Current header version **1.2.10** [S21].
- **Discovery** [S21]:
  - Hosts recursively scan for `.clap` files or bundles.
  - Linux: `~/.clap`, `/usr/lib/clap`.
  - Windows: `%COMMONPROGRAMFILES%\CLAP`, `%LOCALAPPDATA%\Programs\Common\CLAP`.
  - macOS: `/Library/Audio/Plug-Ins/CLAP`, `~/Library/Audio/Plug-Ins/CLAP`.
  - Plus every directory in the **`CLAP_PATH`** environment variable.
- **Parameters** (`ext/params.h`) [S22]:
  - The host can read parameter values at any time on the main thread.
  - Changes travel as events (`CLAP_EVENT_PARAM_VALUE`, gesture begin/end), during `process()` or
    `flush()` when audio isn't running.
  - IDs are stable. MIDI-CC-driven changes can be flagged "don't record".
- **State and presets** [S20]:
  - `state` / `state-context` extensions.
  - A **preset-discovery factory** lets hosts index a plug-in's presets in *their native file
    formats*.
  - A `preset-load` extension asks the plug-in to load one.
- **Hosts (Nov 2024 list):** Bitwig Studio, REAPER, FL Studio, Studio One, MultitrackStudio, MuLab,
  Bidule, Qtractor, Zrythm, Carla, Anklang, and others. Vendors include u-he, FabFilter, Surge,
  TAL, Vital Audio and others [S23]. **Not** in Ableton Live's official format list [S37] (U).
  Logic/Cubase/Pro Tools are not listed [S23].

### 1.6 Comparison

| | VST2 | VST3 | AUv2 / AUv3 | AAX | CLAP |
|---|---|---|---|---|---|
| Owner / licence | Steinberg; closed, no new licences since 2018 [S2] | Steinberg; **MIT since 3.8** [S1] | Apple; free SDK | Avid; agreement + PACE signing [S19] | Open, MIT [S20] |
| Platforms | Win/mac (legacy) | Win/mac/Linux [S5] | macOS (AUv3 also iOS/visionOS) [S13] | Win/mac [S19] | Win/mac/Linux [S21] |
| Discovery | Host-specific folder scan | Fixed folders + `moduleinfo.json` [S5][S10] | System registry, `AVAudioUnitComponentManager` [S15] | Pro Tools only | Fixed folders + `CLAP_PATH` [S21] |
| Parameters | Index-based (to verify) | 32-bit IDs, normalized 0–1, flags [S8] | `AUParameterTree` [S14] | n/a for us | Stable IDs, event-based [S22] |
| State | Opaque chunk | `getState/setState` opaque streams [S9] | `ClassInfo` / `fullState` dictionary [S13][S17] | — | `state` extension [S20] |
| Preset file | `.fxp` / `.fxb` | `.vstpreset` (documented container) [S7] | `.aupreset` (plist, to verify) | `.tfx` (to verify) | Plug-in native + discovery factory [S20] |
| COSOUND can host? | No (licence) | **Yes** | **Yes** (macOS) | **No** | Yes (by implementing the host side; no JUCE support) |

---

## 2. Hosting toolkits

### 2.1 JUCE
- **Version:** 9.0.6 (2026-11-19) [S27].
- **Licence:** dual, JUCE commercial licence or **AGPLv3** [S28].
- **Hosting API** [S24][S25][S26]:
  - `AudioPluginFormatManager` holds the list of formats and creates `AudioPluginInstance`s.
  - `KnownPluginList` is the scan result list, and it can be saved and loaded. Use it as a
    plugin-cache model.
  - `AudioProcessor::getStateInformation` / `setStateInformation` read and write a plug-in's state
    blob.
  - Each instance exposes its parameters as `AudioProcessorParameter` objects.
- **Formats JUCE can host** (from the `format_types` source tree) [S27]:
  - VST3, AudioUnit (macOS), LV2, LADSPA.
  - VST (VST2), which needs the non-redistributable VST2 SDK [S2].
  - **No CLAP or AAX hosting.**
- **AudioPluginHost:** the JUCE example host, in `extras/AudioPluginHost` of the JUCE repo [S27].
  It's a reference for scanning, a parameter-list UI and state handling.

### 2.2 Spotify pedalboard (Python)
- **Version:** 0.9.26 [S29]. **Licence: GPLv3** [S29].
- **Formats:** loads **VST3** (macOS, Windows, Linux) and **Audio Units** (macOS only). VST2, CLAP
  and AAX are not mentioned (not supported) [S29][S30].
- **API** [S30]:
  - `load_plugin(path, plugin_name=…, parameter_values=…)`.
  - `get_plugin_names_for_file()` lists the plug-ins in a multi-plugin `.vst3`.
  - Each plug-in's `parameters` dict is keyed by normalized parameter names. A parameter is set as
    an attribute (e.g. `plugin.mix = 0.5`) through an `AudioProcessorParameter` that has
    `raw_value` in [0, 1], a label and units.
  - `is_instrument` / `is_effect`, `name`, `manufacturer_name`, `version`.
  - `show_editor()` opens the native UI, on the main thread.
  - **VST3 preset/state:**
    - `load_preset()` loads a **`.vstpreset`**.
    - `preset_data` gets or sets the state as `.vstpreset` bytes.
    - `raw_state` gets or sets the internal state bytes. For VST3 this is "usually" XML with an
      8-byte header. That is JUCE's wrapper format for JUCE-built plug-ins; others differ.
    - Invalid state data can crash the Python process.
- **Fit for COSOUND:**
  - A good prototyping/offline tool: render a synth with a candidate patch and analyse the audio.
  - It is a *separate* host, so it does not control the instance inside the user's DAW session.
  - **GPLv3 is incompatible with shipping a closed-source COSOUND binary** that links it.
    Prototyping or server use needs a licence review.

---

## 3. How COSOUND can "recognize and control plugin parameters"

### (a) Through the DAW's own API (controls the instance the user actually hears)

| DAW | Mechanism | Plugin parameter access | Class |
|---|---|---|---|
| REAPER | ReaScript (Lua/Python/EEL) | `TrackFX_GetNumParams`, `TrackFX_GetParamName`, `TrackFX_SetParam`, `TrackFX_GetNamedConfigParm` [S31] | official |
| Ableton Live | Live Object Model via Max for Live (and Remote Scripts) | `Device` → `DeviceParameter` objects (name, value, min/max) [S32]. Live shows up to 64 plug-in parameters automatically; for more, the user "configures" which ones appear [S36], which limits what is exposed | official (M4L) / protocol |
| FL Studio | MIDI scripting (Python) `plugins` module | `getParamValue`, `getParamName`, `getParamValueString`, `setParamValue` per mixer/channel slot [S33] | official (from a MIDI-script context) |
| Cubase / Nuendo | MIDI Remote API (JS) v1.3 | Built for controller mapping; reaches focused plug-in / Quick Controls (detailed scope **to verify**) [S34] | protocol |
| Logic Pro | No public plug-in API. Scripter (MIDI FX) `TargetEvent` can drive a plug-in parameter on the same channel strip, chosen via "Learn plug-in parameter" [S35]; otherwise control surfaces / MIDI learn / Accessibility | protocol (Scripter, MIDI) / ui-automation |
| Pro Tools | AAX only; external control via EUCON/HUI (to verify; see `../pro-tools/`) | — | to-verify |

This is the **preferred route**: it changes the instance the user actually hears, and the
changes are saved with the project. Its quality varies widely by DAW. Logic, COSOUND's first
target, is the weakest: no enumerating API, so control depends on MIDI learn, Scripter or UI
automation.

### (b) By hosting plug-ins itself (JUCE or pedalboard)
- COSOUND loads the user's own VST3/AU in a private host. It can then:
  - enumerate every parameter (ID, name, range, display string, automatable flag) [S8][S14];
  - set values and render audio to *audition and verify* a patch, for example by matching against
    a reference;
  - export the result as `.vstpreset` [S7][S30] or AU `fullState` [S13] for the user to load in the
    DAW.
- **Limits:**
  - It's a separate instance: changes don't reach the DAW's session automatically.
  - Licensing/activation: some plug-ins may refuse to run or show demo behaviour outside known hosts
    **(to verify per vendor)**.
  - AAX can't be hosted [S19]. VST2 can't be shipped [S2]. CLAP needs a custom host (not in JUCE)
    [S27].
  - Sandboxing and crash isolation are needed: a crashing plug-in takes the host down (pedalboard
    warns about this for bad state data) [S30].

### (c) Reading/writing preset files when the format is documented

| Format | Container documented? | Payload readable? | Practical verdict |
|---|---|---|---|
| `.vstpreset` | Yes [S7] | Plug-in-private state chunk | Can **copy, rename, index and load** presets. Editing values requires the plug-in (route b) |
| AU `ClassInfo` / `fullState` / `.aupreset` | Dictionary (plist) [S13][S17] | Often contains an opaque `data` blob (to verify) | Index and load; edit via the AU itself |
| `.fxp` / `.fxb` | Commonly known (to verify spec source) | Plug-in-private chunk [S41] | Index only |
| **Vital `.vital`** | Yes – plain JSON (source-verified) [S40] | **Yes** | **Read and write directly**: the best case |
| Serum `.fxp` / `.SerumPreset` | Not public (vendor) [S39] | Reverse-engineered only | Index/organize only; don't write |
| NI Massive / Massive XL | Not documented [S42] | — | Index only |
| CLAP | Plug-in native, with a discovery factory [S20] | Via plug-in | Index via factory when hosting |

### (d) Plugin scanning caches
- COSOUND can build its own inventory without loading anything:
  - Walk the standard folders: VST3 [S5], CLAP [S21] and AU Components [S18].
  - Read `moduleinfo.json` where present (name, vendor, version, class IDs, categories) [S10].
  - On macOS, query `AVAudioUnitComponentManager` [S15].
- A JUCE `KnownPluginList` is a ready model for a persistent scan cache [S25].
- Reading DAWs' *own* scan caches (e.g. Ableton's or REAPER's plug-in databases) is undocumented
  **(to verify per DAW; classify ui-automation/none until confirmed)**.
- Parameter lists need the plug-in to be loaded once (route b) or a DAW query (route a). Cache them
  per plug-in ID and version.

---

## 4. Known limits (summary)

1. **Closed preset formats:** Serum (vendor states not public [S39]), NI and Rob Papen (not
   documented), and the inner chunks of `.fxp`/`.vstpreset` [S7]. Only Vital is openly
   readable/writable [S40].
2. **Opaque parameter names:** hosts see only what the plug-in exports. Generic names, missing
   units, or no exposure beyond macros are all possible. Massive XL documents its Macros and Morpher
   as the host-automation surface [S42]. Ableton shows at most 64 parameters automatically [S36].
   COSOUND needs a per-plug-in semantic map (e.g. "Filter 1 Cutoff" → cutoff), curated or learned
   from audio.
3. **Normalized values:** VST3, CLAP and pedalboard work in 0–1. Mapping to Hz, dB or ms needs the
   plug-in's own string conversion or a calibration sweep [S8][S30].
4. **AAX is not hostable** by third parties (inferred) [S19]. **VST2 can't be newly distributed**
   [S2].
5. **Logic Pro** (first target) has no parameter-enumeration API [S35]. The realistic options:
   - Scripter `TargetEvent` / MIDI learn for a few mapped parameters;
   - AU hosting by COSOUND for design and audition, then a preset handed back to Logic;
   - UI automation as a last resort.
6. **Licences of hosting libraries:** JUCE is AGPLv3 or commercial [S28]; pedalboard is GPLv3
   [S29]; the VST3 SDK is MIT [S1].

---

## 5. Open items (to verify)
- Exact VST2 `.fxp`/`.fxb` spec source, given the VST2 SDK can't be redistributed.
- `.aupreset` file conventions (folder `~/Library/Audio/Presets/<Manufacturer>/<Name>/`?).
- Cubase 15 VST2 status and Ableton CLAP status (S37/S38 unreachable).
- How much of each synth is exposed to host automation (Serum 2, Vital, Sylenth1, Predator-3,
  Albino-3): measure by enumerating with a JUCE/pedalboard host.
- Whether plug-in copy protection blocks loading in a non-DAW host.
