# COSOUND – Specialized agents: responsibilities (Prompt 06)

Generated from `packages/agents/<id>.json` (the source of truth, compiled into the app and checked by tests).
Status: **available** = runs today on real engines; **planned** = declared with the missing component, the runtime answers `NOT_IMPLEMENTED` (never simulated).
Fields marked * are required. All agents: contract version 1, agent version 1.0.0.

| | Agent | Role | Operations available | Permissions |
|---|---|---|---|---|
| A | [Music Producer](#a-music-producer) | Song structure, arrangement and production plan | 3/4 | — |
| B | [Rhythm Engineer](#b-rhythm-engineer) | Drums, groove, swing, transitions and timing | 3/4 | — |
| C | [Sound Designer](#c-sound-designer) | Choosing and designing sounds with the installed instruments | 2/3 | — |
| D | [Music Theory](#d-music-theory) | Music theory and coherent composition | 7/7 | — |
| E | [Voice-to-Music](#e-voice-to-music) | From voice to musical material (singing, whistling, beatbox, speech) | 0/4 | files.authorized_read |
| F | [Audio Analysis](#f-audio-analysis) | Measurements on audio files and stems | 2/5 | files.app_write, files.authorized_read |
| G | [Reference Track](#g-reference-track) | Reference profile to create an original track | 1/2 | files.authorized_read, network |
| H | [Smart Library](#h-smart-library) | Search in authorized libraries on internal and external drives | 4/5 | external_devices, files.authorized_read |
| I | [Plugin Intelligence](#i-plugin-intelligence) | Installed plugins, categories and features | 3/4 | plugins.scan |
| J | [Mixing Engineer](#j-mixing-engineer) | Mix diagnosis from measurements and proposed moves | 1/2 | daw.control, files.authorized_read |
| K | [Mastering Engineer](#k-mastering-engineer) | Technical check of the master and release destinations | 1/2 | files.app_write, files.authorized_read |
| L | [DAW Controller](#l-daw-controller) | The only component that operates DAWs, with verified methods only | 2/3 | daw.control, daw.detect, files.app_write |
| M | [Adaptive Workflow](#m-adaptive-workflow) | Memory of the user's working habits, under their control | 7/8 | memory.workflow |

## A. Music Producer

`music_producer` · v1.0.0 · Song structure, arrangement and production plan

**Competencies**

- Understanding structured musical requests (genre, BPM, key)
- Per-genre structures and arrangements
- Section layout and energy curve
- Choice of elements per section
- Creative variants of progression and style
- Production plan: which work to request from the other agents

**Operations**

| Operation | Status | Risk | Basis | Input | Output |
|---|---|---|---|---|---|
| `plan_arrangement` – Section map with bars, timing, energy and elements | available | read | convention | `genre`*: string (enum); `bpm`: number | sections[], total_bars, duration, source |
| `propose_variants` – Harmonic progression variants that fit the genre and key | available | read | rule_based | `genre`*: string (techno, melodic_techno, progressive_house, house, edm); `key`: string; `tonic`: string; `mode`: string (enum); `count`: integer; `seed`: integer | variants[] with chords, functions, cadence |
| `production_plan` – Production plan: structure + work suggested for the other agents (needs) | available | read | convention | `genre`*: string (enum); `bpm`: number; `key`: string | plan + needs[] for the central Agent |
| `interpret_request` – Turn a natural-language request into a structured brief | planned – requires language understanding: done by the central Agent or the AI Core (decisions D-2, D-3) | read | — | `text`*: string | brief |

**Tools (level 3):** `ai.core`, `music.arrangement`, `music.theory`

**Permissions:** none

**Input formats:** application/json · **Output formats:** application/json

**Works with (through the central Agent, never directly):** music_theory, rhythm_engineer, sound_designer, smart_library, daw_controller

**Limits**

- Does not write notes: asks Music Theory and Rhythm Engineer through the central Agent
- Arrangement templates are conventions to verify, not measurements of real tracks
- Genres with templates: techno, melodic techno/house, progressive house/techno, deep house, house, tech house, afro house, trance, minimal, EDM

## B. Rhythm Engineer

`rhythm_engineer` · v1.0.0 · Drums, groove, swing, transitions and timing

**Competencies**

- Kick, clap, snare, hi-hats and percussion
- Rhythm patterns per genre
- Groove and swing
- BPM and subdivisions (ms, dotted, triplet)
- General MIDI drum patterns
- Fills, snare builds and impacts for drops and transitions

**Operations**

| Operation | Status | Risk | Basis | Input | Output |
|---|---|---|---|---|---|
| `generate_drums` – MIDI drum pattern with checks and a MIDI file | available | read | rule_based | `genre`: string (techno, melodic_techno, progressive_house, house, edm); `key`: string; `tonic`: string; `mode`: string (enum); `progression`: string/array; `bpm`: number; `bars`: integer; `seed`: integer; `swing`: number; `sevenths`: boolean; `chord_rhythm`: string (sustain, stabs, pulse); `bass_style`: string (offbeat, rolling, pulse, syncopated, sustain); `drum_style`: string (techno, melodic_techno, progressive_house, house, edm); `fill`: boolean | part, checks, midi.base64 |
| `generate_transition` – Snare build, tom fill or drop impact | available | read | rule_based | `kind`*: string (snare_build, drum_fill, drop_impact); `bars`: integer; `bpm`: number; `seed`: integer | part, checks, midi.base64 |
| `groove_timing` – Note lengths in ms and swing offset at a tempo | available | read | rule_based | `bpm`*: number; `swing`: number | beat_ms, bar_ms, note_lengths[], swing_offset_ms |
| `humanize` – Humanize an existing MIDI part (timing and dynamics) | planned – requires MIDI file import (reading existing MIDI) is not built yet | read | — | `path`*: string | part |

**Tools (level 3):** `music.generator`, `music.smf_writer`

**Permissions:** none

**Input formats:** application/json · **Output formats:** application/json, audio/midi

**Works with (through the central Agent, never directly):** music_producer, daw_controller

**Limits**

- 4/4 only
- Swing on the off-16ths only
- Cannot read existing MIDI yet

## C. Sound Designer

`sound_designer` · v1.0.0 · Choosing and designing sounds with the installed instruments

**Competencies**

- Timbre analysis per role (bass, lead, pad, pluck, arp, texture)
- Synthesis recipes in generic terms (oscillators, filter, envelopes, modulation)
- Choice of synths and presets that are actually installed
- Sound variants (brighter, darker, punchier, wider)
- Works with Plugin Intelligence and Smart Library

**Operations**

| Operation | Status | Risk | Basis | Input | Output |
|---|---|---|---|---|---|
| `suggest_sounds` – Recipe for the role + installed synths + presets in the library | available | read | rule_based | `role`*: string (enum); `genre`: string | recipes[], installed_instruments[], presets[] |
| `patch_variants` – Variants of a recipe | available | read | rule_based | `recipe_id`*: string (sub_bass, acid_bass, supersaw_lead, pluck_arp, pad, fm_bell, granular_texture) | recipe, variants[] |
| `program_synth` – Set a synth's parameters inside the DAW | planned – requires plugin parameter maps (M7) and a DAW connector (M4) | medium · user confirmation | — | `plugin`*: string; `recipe_id`: string | applied parameters |

**Tools (level 3):** `daw.connectors`, `library.index`, `plugins.catalog`, `plugins.param_maps`, `plugins.scanner`, `sound.recipes`

**Permissions:** none

**Input formats:** application/json · **Output formats:** application/json

**Works with (through the central Agent, never directly):** plugin_intelligence, smart_library, daw_controller

**Limits**

- Proposes only synths detected on the computer; others are marked as not installed
- Recipes are in generic terms, not plugin-specific values
- Never edits presets or plugin files

## D. Music Theory

`music_theory` · v1.0.0 · Music theory and coherent composition

**Competencies**

- Notes, sharps, flats, tones and semitones
- Intervals and scales (major, minor, modal, pentatonic, blues)
- Keys, relative and parallel keys, key signatures
- Chords, inversions and roman-numeral progressions
- Melodies, basslines and arpeggios
- Tension, resolution and cadences
- Musical coherence between parts
- Key finding from a set of notes

**Operations**

| Operation | Status | Risk | Basis | Input | Output |
|---|---|---|---|---|---|
| `key_info` – Scale, key signature, relative keys and diatonic chords | available | read | rule_based | `key`: string; `tonic`: string; `mode`: string (enum) | scale, signature, triads[], sevenths[] |
| `progression` – Progression analysis: chords, functions, tension, cadences | available | read | rule_based | `genre`: string (techno, melodic_techno, progressive_house, house, edm); `key`: string; `tonic`: string; `mode`: string (enum); `progression`: string/array; `bpm`: number; `bars`: integer; `seed`: integer; `swing`: number; `sevenths`: boolean; `chord_rhythm`: string (sustain, stabs, pulse); `bass_style`: string (offbeat, rolling, pulse, syncopated, sustain); `drum_style`: string (techno, melodic_techno, progressive_house, house, edm); `fill`: boolean | chords[], cadences, tension_curve |
| `generate_chords` – Voice-led chords | available | read | rule_based | `genre`: string (techno, melodic_techno, progressive_house, house, edm); `key`: string; `tonic`: string; `mode`: string (enum); `progression`: string/array; `bpm`: number; `bars`: integer; `seed`: integer; `swing`: number; `sevenths`: boolean; `chord_rhythm`: string (sustain, stabs, pulse); `bass_style`: string (offbeat, rolling, pulse, syncopated, sustain); `drum_style`: string (techno, melodic_techno, progressive_house, house, edm); `fill`: boolean | part, checks, midi.base64 |
| `generate_bassline` – Bassline on chord tones | available | read | rule_based | `genre`: string (techno, melodic_techno, progressive_house, house, edm); `key`: string; `tonic`: string; `mode`: string (enum); `progression`: string/array; `bpm`: number; `bars`: integer; `seed`: integer; `swing`: number; `sevenths`: boolean; `chord_rhythm`: string (sustain, stabs, pulse); `bass_style`: string (offbeat, rolling, pulse, syncopated, sustain); `drum_style`: string (techno, melodic_techno, progressive_house, house, edm); `fill`: boolean | part, checks, midi.base64 |
| `generate_melody` – Melody with chord tones on strong beats | available | read | rule_based | `genre`: string (techno, melodic_techno, progressive_house, house, edm); `key`: string; `tonic`: string; `mode`: string (enum); `progression`: string/array; `bpm`: number; `bars`: integer; `seed`: integer; `swing`: number; `sevenths`: boolean; `chord_rhythm`: string (sustain, stabs, pulse); `bass_style`: string (offbeat, rolling, pulse, syncopated, sustain); `drum_style`: string (techno, melodic_techno, progressive_house, house, edm); `fill`: boolean | part, checks, midi.base64 |
| `detect_key` – Most likely key from notes or pitch-class weights | available | read | estimated | `notes`: array; `weights`: array | best, candidates[] with correlation |
| `check_coherence` – Check that given MIDI parts fit the key and progression | available | read | rule_based | `genre`: string (techno, melodic_techno, progressive_house, house, edm); `key`: string; `tonic`: string; `mode`: string (enum); `progression`: string/array; `bpm`: number; `bars`: integer; `seed`: integer; `swing`: number; `sevenths`: boolean; `chord_rhythm`: string (sustain, stabs, pulse); `bass_style`: string (offbeat, rolling, pulse, syncopated, sustain); `drum_style`: string (techno, melodic_techno, progressive_house, house, edm); `fill`: boolean; `ppq`: integer; `parts`*: array | checks per part |

**Tools (level 3):** `music.generator`, `music.key_detection`, `music.smf_writer`, `music.theory`

**Permissions:** none

**Input formats:** application/json · **Output formats:** application/json, audio/midi

**Works with (through the central Agent, never directly):** music_producer, rhythm_engineer, voice_to_music, daw_controller

**Limits**

- 4/4 only and one chord per bar when generating
- Rule-based melodies, no motif development
- Key finding is an estimate (Krumhansl profiles)

## E. Voice-to-Music

`voice_to_music` · v1.0.0 · From voice to musical material (singing, whistling, beatbox, speech)

**Competencies**

- Analysis of sung or whistled melodies
- Pitch, interval, duration and rhythm recognition
- BPM estimate when possible
- Conversion to MIDI
- Vocal rhythms to drum patterns
- Telling speech, singing, whistling and vocal percussion apart

**Operations**

| Operation | Status | Risk | Basis | Input | Output |
|---|---|---|---|---|---|
| `transcribe_melody` – Sung/whistled melody → MIDI with confidence | planned – requires pitch-tracking engine (milestone M5) | read | — | `path`*: string | part, confidence |
| `transcribe_rhythm` – Beatbox → drum pattern | planned – requires onset/drum-class engine (milestone M5) | read | — | `path`*: string | part |
| `transcribe_speech` – Speech → text for the central Agent | planned – requires speech-to-text engine (milestone M5) | read | — | `path`*: string | text |
| `classify_take` – Speech, singing, whistling or vocal percussion? | planned – requires voice classifier (milestone M5) | read | — | `path`*: string | class, confidence |

**Tools (level 3):** `music.smf_writer`, `voice.pitch_tracker`, `voice.speech_to_text`

**Permissions:** `files.authorized_read`

**Input formats:** audio/wav, audio/webm · **Output formats:** application/json, audio/midi

**Works with (through the central Agent, never directly):** music_theory, rhythm_engineer

**Limits**

- Transcription engine not built yet (M5): no operation can run
- The microphone is used only with the user's explicit consent (Settings) and only while recording

## F. Audio Analysis

`audio_analysis` · v1.0.0 · Measurements on audio files and stems

**Competencies**

- Integrated loudness (ITU-R BS.1770), true peak, peak, RMS, crest factor
- Stereo and low-band correlation, DC offset, clipping
- Coarse band balance
- Leading and trailing silence
- Comparison between stems
- Clear split between measured data and estimates

**Operations**

| Operation | Status | Risk | Basis | Input | Output |
|---|---|---|---|---|---|
| `analyze_file` – Measurements of an authorized WAV file | available | read | measured | `path`*: string | measurements |
| `analyze_stems` – Measurements of several stems and relative levels | available | read | measured | `paths`*: array | stems[], relative_lufs |
| `detect_tempo_key` – BPM and key of an audio file | planned – requires tempo/key engine (milestone M6) | read | — | `path`*: string | bpm, key, confidence |
| `detect_structure` – Sections, breakdowns and drops | planned – requires structure engine (milestone M6) | read | — | `path`*: string | sections[] |
| `separate_stems` – Stem separation | planned – requires stem separation engine (milestone M6) | low | — | `path`*: string | stems[] |

**Tools (level 3):** `audio.stem_separator`, `audio.structure`, `audio.tempo_key`, `audio.wav_meter`

**Permissions:** `files.app_write`, `files.authorized_read`

**Input formats:** application/json, audio/wav · **Output formats:** application/json

**Works with (through the central Agent, never directly):** mixing_engineer, mastering_engineer, reference_track

**Limits**

- WAV files only (PCM 16/24/32-bit, float 32/64-bit)
- Loudness for mono and stereo only
- BPM, key, structure and instruments not measured yet (M6)
- Reads only files in authorized folders, imported files or COSOUND's folder

## G. Reference Track

`reference_track` · v1.0.0 · Reference profile to create an original track

**Competencies**

- Analysis of tracks named by the user
- Use of information found online, always labelled as such
- Audio measurements when the file is legitimately provided
- BPM, key, structure, loudness
- Guidance for an original track with similar stylistic traits

**Operations**

| Operation | Status | Risk | Basis | Input | Output |
|---|---|---|---|---|---|
| `build_profile` – Profile with the source of every value (web or measured) and a brief for an original track | available | read | mixed | `title`: string; `artist`: string; `genre`: string (enum); `web_facts`: object; `audio_path`: string | profile{field: {value, basis, source}}, brief |
| `research_online` – Online research about a track | planned – requires web research tool; in this version research is done by the central Agent, which passes the facts in build_profile.web_facts | read | — | `title`*: string; `artist`: string | web_facts |

**Tools (level 3):** `audio.wav_meter`, `music.arrangement`, `web.research`

**Permissions:** `files.authorized_read`, `network`

**Input formats:** application/json, audio/wav · **Output formats:** application/json

**Works with (through the central Agent, never directly):** audio_analysis, music_producer

**Limits**

- Never copies melodies, recordings, arrangements or other expressive elements
- Never downloads audio from streaming services
- No network access: web facts are provided by the central Agent

## H. Smart Library

`smart_library` · v1.0.0 · Search in authorized libraries on internal and external drives

**Competencies**

- Search for samples, loops, MIDI, presets and projects
- Internal, external, USB and network drives already authorized
- Recognition of previously indexed drives
- Classification by type
- BPM and key written in the file name
- Probable duplicates
- Original file paths

**Operations**

| Operation | Status | Risk | Basis | Input | Output |
|---|---|---|---|---|---|
| `search` – Search by name, type, BPM and key (from the file name) | available | read | file_name | `query`*: string; `kind`: string (audio, midi, project, preset); `bpm_min`: number; `bpm_max`: number; `key`: string; `limit`: integer | hits[] with path (null when the drive is offline) |
| `stats` – Indexed file counts by type | available | read | measured | — | roots, files_by_kind |
| `find_duplicates` – Probable duplicates (same name and size) | available | read | estimated | `limit`: integer | groups[] |
| `volumes` – Drives and authorized folders, connected or not | available | read | measured | — | volumes[], roots[] |
| `analyze_audio_content` – Content indexing (measured BPM/key, similarity) | planned – requires tempo/key engine (M6) and embeddings | read | — | `root_id`*: string | updated index |

**Tools (level 3):** `audio.tempo_key`, `library.filename_meta`, `library.index`

**Permissions:** `external_devices`, `files.authorized_read`

**Input formats:** application/json · **Output formats:** application/json

**Works with (through the central Agent, never directly):** sound_designer, music_producer

**Limits**

- Never moves, modifies or deletes original files
- Cannot authorize new folders: only the user can, from the system dialog
- BPM and key come from file names, not measurements
- Duplicates are candidates (name and size); contents are not compared

## I. Plugin Intelligence

`plugin_intelligence` · v1.0.0 · Installed plugins, categories and features

**Competencies**

- Detection of installed plugins (VST3, AU, CLAP, VST2)
- Organization by type and vendor
- Features of well-known synths (Massive, Predator, Serum, Vital, Sylenth1…)
- Instruments and effects suited to a request
- Technical documentation from the knowledge base

**Operations**

| Operation | Status | Risk | Basis | Input | Output |
|---|---|---|---|---|---|
| `scan` – Inventory of installed plugins | available | read | measured | — | plugins[] by format, kind, vendor |
| `find_for_role` – Installed instruments suited to a role | available | read | rule_based | `role`*: string (enum) | candidates[] (detected only) |
| `plugin_info` – Profile of a known synth and whether it is installed | available | read | rule_based | `name`*: string | catalog entry, installed |
| `map_parameters` – Parameter map of a plugin | planned – requires parameter maps through a DAW (milestone M7) | read | — | `name`*: string | parameters[] |

**Tools (level 3):** `daw.connectors`, `plugins.catalog`, `plugins.param_maps`, `plugins.scanner`, `sound.recipes`

**Permissions:** `plugins.scan`

**Input formats:** application/json · **Output formats:** application/json

**Works with (through the central Agent, never directly):** sound_designer, daw_controller

**Limits**

- Never reports a plugin as available unless it was detected
- Never loads plugins: reads files and metadata only
- Plugin parameters and control not available yet (M7)

## J. Mixing Engineer

`mixing_engineer` · v1.0.0 · Mix diagnosis from measurements and proposed moves

**Competencies**

- Level balance and gain staging
- EQ, compression, saturation, reverb and delay (proposals)
- Stereo image and mono low end
- Frequency management and overlaps (indicative)
- Dynamics analysis
- Clear split between suggested and executed moves

**Operations**

| Operation | Status | Risk | Basis | Input | Output |
|---|---|---|---|---|---|
| `analyze_mix` – Per-track measurements and mix proposals (never executed) | available | read | measured | `tracks`*: array; `genre`: string | tracks[] with measurements, suggestions[] (executed: false) |
| `apply_moves` – Apply the moves in the DAW | planned – requires DAW connector (milestone M4) | medium · user confirmation | — | `moves`*: array | applied moves |

**Tools (level 3):** `audio.wav_meter`, `daw.connectors`

**Permissions:** `daw.control`, `files.authorized_read`

**Input formats:** application/json, audio/wav · **Output formats:** application/json

**Works with (through the central Agent, never directly):** audio_analysis, mastering_engineer, daw_controller

**Limits**

- Proposals only: no move is executed
- Measurements need WAV files of the tracks
- Coarse frequency bands (2nd-order filters), no fine spectral analysis

## K. Mastering Engineer

`mastering_engineer` · v1.0.0 · Technical check of the master and release destinations

**Competencies**

- Integrated loudness and true peak
- Dynamic range (PLR, crest factor)
- Tonal balance and stereo image (indicative)
- Clipping check
- Comparison with platform targets, with source reliability
- Proposals per destination

**Operations**

| Operation | Status | Risk | Basis | Input | Output |
|---|---|---|---|---|---|
| `analyze_master` – Master measurements and comparison with destinations | available | read | measured | `path`*: string; `destinations`: array | measurements, destinations[] with expected gain change, suggestions[] |
| `master_track` – Process and export the master | planned – requires processing chain and render (milestone M9) | high · user confirmation | — | `path`*: string; `destination`: string (spotify, apple_music, youtube, tidal, amazon_music, club, broadcast_ebu) | file |

**Tools (level 3):** `audio.processing_chain`, `audio.wav_meter`

**Permissions:** `files.app_write`, `files.authorized_read`

**Input formats:** application/json, audio/wav · **Output formats:** application/json

**Works with (through the central Agent, never directly):** audio_analysis, mixing_engineer

**Limits**

- Never reports a master as done: processing does not exist yet (M9)
- Only Spotify publishes its target officially (−14 LUFS, ≤ −1 dBTP); other values come from secondary sources and are labelled as such

## L. DAW Controller

`daw_controller` · v1.0.0 · The only component that operates DAWs, with verified methods only

**Competencies**

- Detection of installed DAWs
- Operation plan: tempo, tracks, MIDI/audio import, section markers
- Verified technical route per DAW
- Risk and confirmation for every operation
- First phase: Logic Pro; then Ableton Live, FL Studio, Pro Tools; then Cubase, REAPER

**Operations**

| Operation | Status | Risk | Basis | Input | Output |
|---|---|---|---|---|---|
| `detect` – Installed DAWs and versions | available | read | measured | — | daws[] |
| `plan_operations` – Prepare DAW operations and the MIDI files to import (executes nothing in the DAW) | available | low | rule_based | `daw`: string (logic, ableton, fl, protools, cubase, reaper); `bpm`: number; `key`: string; `project_name`: string; `sections`: array; `parts`: array | operations[] with route, risk, approval; manual_steps[]; files[] |
| `execute_plan` – Execute the plan in the DAW | planned – requires DAW connectors (milestone M4): Logic Pro first | medium · user confirmation | — | `plan_task_id`*: string | executed operations |

**Tools (level 3):** `daw.connectors`, `daw.detector`, `daw.routes`, `files.writer`

**Permissions:** `daw.control`, `daw.detect`, `files.app_write`

**Input formats:** application/json, audio/midi, audio/wav · **Output formats:** application/json, audio/midi

**Works with (through the central Agent, never directly):** music_theory, rhythm_engineer, music_producer, plugin_intelligence

**Limits**

- No DAW is connected yet: plans are not executed
- Uses only verified methods per DAW (see docs/knowledge/daw-integration)
- Every medium or high-risk operation needs the user's confirmation

## M. Adaptive Workflow

`adaptive_workflow` · v1.0.0 · Memory of the user's working habits, under their control

**Competencies**

- Personal keyboard shortcuts
- Quick commands and recurring operation sequences
- DAW workflows
- Working preferences
- Recall of relevant procedures
- Standard commands vs user customizations

**Operations**

| Operation | Status | Risk | Basis | Input | Output |
|---|---|---|---|---|---|
| `remember` – Store a declared shortcut, procedure or preference | available | low | none | `kind`*: string (shortcut, procedure, preference); `name`*: string; `daw`: string (logic, ableton, fl, protools, cubase, reaper); `keys`: string; `steps`: array; `value`: string; `scope`: string (standard, custom); `source`: string | fact |
| `recall` – Procedures relevant to a request | available | read | none | `query`: string; `daw`: string (logic, ableton, fl, protools, cubase, reaper); `kind`: string (shortcut, procedure, preference) | facts[] ranked |
| `list` – The whole workflow memory | available | read | none | — | facts[] |
| `update` – Edit an item | available | low | none | `id`*: string; `name`: string; `keys`: string; `steps`: array; `value`: string; `scope`: string (standard, custom) | fact |
| `forget` – Delete an item | available | low | none | `id`*: string | deleted |
| `clear` – Delete the whole workflow memory (needs the user's confirmation) | available | high · user confirmation | none | — | deleted count |
| `status` – Whether memory is on and how many items | available | read | none | — | enabled, count |
| `learn_from_history` – Propose procedures from repeated DAW operations | planned – requires a DAW event stream (milestone M4); keystrokes are never recorded | read | — | — | suggestions[] |

**Tools (level 3):** `daw.connectors`, `memory.facts`

**Permissions:** `memory.workflow`

**Input formats:** application/json · **Output formats:** application/json

**Works with (through the central Agent, never directly):** daw_controller

**Limits**

- Stores only what is declared explicitly: no keystroke logging
- Refuses passwords, codes, card or account numbers and keys
- Off until the user turns it on; viewable, editable and deletable
- Uses COSOUND's Music Memory (memory_facts): not a second memory
