# COSOUND – Agent Registry, Tool Registry, permissions

Generated from `packages/agents/` (`<agent>.json`, `tools.json`, `permissions.json`) by `packages/agents/render_docs.py`.
At runtime the same data, with live state (agent enabled, permission granted, which agents use a tool), is returned by the
Agent API message `registry.get`.

## 1. Agent Registry

One JSON manifest per agent, schema `packages/contracts/agent-manifest.schema.json`. Each declares: id, name, version, role,
competencies, operations (status, risk, approval, input JSON Schema, output, tools, permissions, basis, requires), tools, permissions,
limits, input/output formats, the agents whose outputs it can use. A test fails if a manifest and the code disagree
(available operations = implemented operations; tools and permissions exist; available operations use only available tools).

| Agent | Version | State | Operations (available/total) | Tools |
|---|---|---|---|---|
| `music_producer` | 1.0.0 | partial | 3/4 | ai.core, music.arrangement, music.theory |
| `rhythm_engineer` | 1.0.0 | active | 5/5 | music.generator, music.midi_editor, music.rhythm, music.smf_writer |
| `sound_designer` | 1.0.0 | partial | 9/10 | daw.connectors, library.audio_analyzer, library.index, music.generator, plugins.catalog, plugins.design, plugins.host, plugins.param_maps, plugins.parameters, plugins.registry, sound.recipes |
| `music_theory` | 1.0.0 | active | 21/21 | memory.facts, music.arpeggio, music.bass, music.chords, music.composer, music.generator, music.harmony, music.key_detection, music.keys, music.melody, music.midi_analysis, music.midi_editor, music.notes, music.scales, music.smf_writer, music.styles, music.theory, music.validation |
| `voice_to_music` | 1.0.0 | partial | 1/5 | music.smf_writer, music.voice_bridge, voice.pitch_tracker, voice.speech_to_text |
| `audio_analysis` | 1.0.0 | partial | 2/5 | audio.decoder, audio.stem_separator, audio.structure, audio.tempo_key, audio.wav_meter |
| `reference_track` | 1.0.0 | partial | 2/3 | audio.wav_meter, music.arrangement, music.composer, music.harmony, music.melody, music.reference_bridge, music.validation, web.research |
| `smart_library` | 1.0.0 | active | 13/13 | library.audio_analyzer, library.classifier, library.duplicates, library.filename_meta, library.index, library.midi_reader, library.preset_reader, library.scanner, library.search |
| `plugin_intelligence` | 1.0.0 | partial | 12/13 | daw.connectors, library.audio_analyzer, library.index, library.search, plugins.catalog, plugins.chains, plugins.daw_adapter, plugins.host, plugins.knowledge, plugins.param_maps, plugins.parameters, plugins.presets, plugins.registry, plugins.scanner, sound.recipes |
| `mixing_engineer` | 1.0.0 | partial | 1/2 | audio.decoder, audio.wav_meter, daw.connectors |
| `mastering_engineer` | 1.0.0 | partial | 1/2 | audio.decoder, audio.processing_chain, audio.wav_meter |
| `daw_controller` | 1.0.0 | partial | 3/4 | daw.connectors, daw.detector, daw.routes, files.writer, library.index, plugins.daw_adapter |
| `adaptive_workflow` | 1.0.0 | partial | 9/10 | daw.connectors, memory.facts |

Total: 82 of 97 operations available.

Enable/disable: per agent, by the user (setting `agent_enabled.<id>`, logged). A disabled agent answers `AGENT_DISABLED`.

## 2. Tool Registry (level 3: engines and tools)

Agents never call each other: they only use these tools, through the execution context (`Ctx`), which records `tools_used` in every result.

| Tool | Status | Module | What it does | Used by |
|---|---|---|---|---|
| `music.theory` | available | `desktop/src-tauri/src/music/theory.rs` | Scales, keys, chords, roman numerals, functions and cadences | music_producer, music_theory |
| `music.key_detection` | available | `desktop/src-tauri/src/music/theory.rs` | Key estimate from pitch classes | music_theory |
| `music.generator` | available | `desktop/src-tauri/src/music/generate.rs` | Chords, bass, melody, drums, transitions, musical checks | rhythm_engineer, sound_designer, music_theory |
| `music.smf_writer` | available | `desktop/src-tauri/src/music/smf.rs` | Writes format-1 MIDI files | rhythm_engineer, music_theory, voice_to_music |
| `music.arrangement` | available | `desktop/src-tauri/src/music/arrangement.rs` | Per-genre templates from the knowledge base | music_producer, reference_track |
| `audio.wav_meter` | available | `desktop/src-tauri/src/audio/wav.rs` | Loudness, true peak, RMS, correlation, bands | audio_analysis, reference_track, mixing_engineer, mastering_engineer |
| `library.index` | available | `desktop/src-tauri/src/library.rs` | Local index of authorized folders | sound_designer, smart_library, plugin_intelligence, daw_controller |
| `library.scanner` | available | `desktop/src-tauri/src/smartlib/scan.rs` | Incremental scan: new, changed, moved, missing files | smart_library |
| `library.audio_analyzer` | available | `desktop/src-tauri/src/smartlib/features.rs` | Spectrum, transients, BPM, key, sound character, similarity vector | sound_designer, smart_library, plugin_intelligence |
| `library.classifier` | available | `desktop/src-tauri/src/smartlib/classify.rs` | Multiple categories from name, folders, metadata, audio, MIDI, presets | smart_library |
| `library.search` | available | `desktop/src-tauri/src/smartlib/search.rs` | Natural-language search, scoring, similarity, fitting loops | smart_library, plugin_intelligence |
| `library.midi_reader` | available | `desktop/src-tauri/src/smartlib/midi.rs` | Tracks, notes, intervals, key, BPM, pattern type | smart_library |
| `library.preset_reader` | available | `desktop/src-tauri/src/smartlib/presets.rs` | Vital, VST3, AU, FXP/FXB presets | smart_library |
| `library.duplicates` | available | `desktop/src-tauri/src/smartlib/dupes.rs` | Full hash, audio hash, similar sounds | smart_library |
| `audio.decoder` | available | `desktop/src-tauri/src/smartlib/decode.rs` | WAV, AIFF, CAF, MP3, AAC/ALAC, FLAC, OGG Vorbis | audio_analysis, mixing_engineer, mastering_engineer |
| `library.filename_meta` | available | `desktop/src-tauri/src/library.rs` | BPM and key read from file names | smart_library |
| `plugins.scanner` | available | `desktop/src-tauri/src/plugins/scanner.rs` | Plug-ins in the standard folders and macOS's Audio Unit registry: format, version, architecture | plugin_intelligence |
| `plugins.catalog` | available | `desktop/src-tauri/src/sound.rs` | Profiles of well-known synths from the knowledge base | sound_designer, plugin_intelligence |
| `sound.recipes` | available | `desktop/src-tauri/src/sound.rs` | Synthesis recipes in generic terms | sound_designer, plugin_intelligence |
| `daw.detector` | available | `desktop/src-tauri/src/daw.rs` | Installed DAWs and versions | daw_controller |
| `daw.routes` | available | `desktop/src-tauri/src/agents/daw_controller.rs` | Verified technical route per DAW | daw_controller |
| `files.writer` | available | `desktop/src-tauri/src/agents/mod.rs` | Writes only into COSOUND's generated/ folder, with SHA-256 | daw_controller |
| `memory.facts` | available | `desktop/src-tauri/src/memory.rs` | COSOUND's single local memory (memory_facts) | music_theory, adaptive_workflow |
| `plugins.registry` | available | `desktop/src-tauri/src/plugins/registry.rs` | Local database of detected plug-ins, one product across formats, documented DAW compatibility | sound_designer, plugin_intelligence |
| `plugins.knowledge` | available | `desktop/src-tauri/src/plugins/knowledge.rs` | Official documentation with source and version (Apple header), knowledge base | plugin_intelligence |
| `plugins.host` | available | `desktop/src-tauri/auhost/main.swift` | Loads Audio Units in a separate process with a time limit: parameters, rendering, state | sound_designer, plugin_intelligence |
| `plugins.parameters` | available | `desktop/src-tauri/src/plugins/params.rs` | The plug-in's real parameters and the sound role of each | sound_designer, plugin_intelligence |
| `plugins.presets` | available | `desktop/src-tauri/src/plugins/presets.rs` | Factory presets, preset folders, Smart Library; semantic search; measured timbre | plugin_intelligence |
| `plugins.design` | available | `desktop/src-tauri/src/plugins/design.rs` | Parameter-change proposals, before/after audition, new presets, undo | sound_designer |
| `plugins.chains` | available | `desktop/src-tauri/src/plugins/chains.rs` | Effect chains per goal, order, measured material, level check | plugin_intelligence |
| `plugins.daw_adapter` | available | `desktop/src-tauri/src/plugins/daw.rs` | What each DAW allows with plug-ins, and the plan for the DAW Controller | plugin_intelligence, daw_controller |
| `music.notes` | available | `desktop/src-tauri/src/music/notes.rs` | Notes IT/EN, octaves, MIDI, frequencies, simple and compound intervals | music_theory |
| `music.scales` | available | `desktop/src-tauri/src/music/scales.rs` | 33 scales with formula, degrees, character and source | music_theory |
| `music.keys` | available | `desktop/src-tauri/src/music/keys.rs` | Signatures, circle of fifths, related keys, modulation, key estimation | music_theory |
| `music.chords` | available | `desktop/src-tauri/src/music/chords.rs` | 36 chord types, symbols, inversions, voicings, recognition | music_theory |
| `music.harmony` | available | `desktop/src-tauri/src/music/harmony.rs` | Progression analysis, substitutions, generation by style and mood | music_theory, reference_track |
| `music.melody` | available | `desktop/src-tauri/src/music/melody.rs` | Motif-based melodies, question/answer phrases, non-chord tones, variations | music_theory, reference_track |
| `music.bass` | available | `desktop/src-tauri/src/music/bass.rs` | 11 bass styles, kick-aware | music_theory |
| `music.arpeggio` | available | `desktop/src-tauri/src/music/arp.rs` | 9 arpeggio patterns, rates, octaves, accents | music_theory |
| `music.rhythm` | available | `desktop/src-tauri/src/music/rhythm.rs` | Grid, position and duration, Euclidean, polyrhythm, swing, quantize | rhythm_engineer |
| `music.styles` | available | `desktop/src-tauri/src/music/styles.rs` | 11 editable, blendable style profiles | music_theory |
| `music.composer` | available | `desktop/src-tauri/src/music/compose.rs` | From a request in words to a validated composition and MIDI | music_theory, reference_track |
| `music.midi_editor` | available | `desktop/src-tauri/src/music/edit.rs` | Edits in words, versions, compare, undo | rhythm_engineer, music_theory |
| `music.midi_analysis` | available | `desktop/src-tauri/src/music/analysis.rs` | Key, chords, progression, phrases, repetitions, out-of-key notes | music_theory |
| `music.validation` | available | `desktop/src-tauri/src/music/validate.rs` | Technical error, stylistic choice, compositional freedom | music_theory, reference_track |
| `music.voice_bridge` | available | `desktop/src-tauri/src/music/voice.rs` | Notes estimated from the voice → rhythm, key, MIDI, chords | voice_to_music |
| `music.reference_bridge` | available | `desktop/src-tauri/src/music/reference.rs` | Reference reading with sources and checked original composition | reference_track |
| `daw.connectors` | planned – milestone M4 (Logic Pro first) | — | Connectors that execute operations in DAWs | sound_designer, plugin_intelligence, mixing_engineer, daw_controller, adaptive_workflow |
| `plugins.param_maps` | planned – DAW connectors (milestone M4) and per-DAW parameter maps (M7) | — | Parameter control of the instance inside the DAW | sound_designer, plugin_intelligence |
| `voice.pitch_tracker` | planned – milestone M5 | — | Pitch and notes from voice | voice_to_music |
| `voice.speech_to_text` | planned – milestone M5 | — | Speech transcription | voice_to_music |
| `audio.tempo_key` | planned – milestone M6 | — | BPM and key from audio | audio_analysis |
| `audio.structure` | planned – milestone M6 | — | Sections from audio | audio_analysis |
| `audio.stem_separator` | planned – milestone M6 | — | Stem separation | audio_analysis |
| `audio.processing_chain` | planned – milestone M9 | — | EQ, compression, limiter and render | mastering_engineer |
| `web.research` | planned – the central Agent or the AI Core (decision D-2) | — | Online research | reference_track |
| `ai.core` | planned – ADR-005, decisions D-2/D-3 | — | Language reasoning | music_producer |

## 3. Permissions

Checked by the runtime before an operation runs (`PERMISSION_DENIED` otherwise). User toggles live in Settings; non-grantable permissions
belong to components that do not exist yet.

| Permission | Setting | Default | Grantable | Meaning |
|---|---|---|---|---|
| `files.authorized_read` | — | on | yes | Read only files in authorized folders, imported files and COSOUND's folder |
| `files.app_write` | — | on | yes | Write only inside COSOUND's generated/ folder |
| `microphone` | `perm_microphone` | off | yes | Record the voice, only while the user presses Record |
| `external_devices` | `perm_external_devices` | on | yes | See external, USB and network drives |
| `daw.detect` | `perm_daw_detection` | on | yes | Look for installed DAWs |
| `plugins.scan` | `perm_plugin_scan` | on | yes | Read the list of installed plugins |
| `memory.workflow` | `perm_workflow_memory` | off | yes | Remember shortcuts and procedures declared by the user |
| `daw.control` | — | off | no | Execute operations in DAWs (connectors not built yet, M4) |
| `network` | — | off | no | Agents have no network access in this version |
| `plugins.host` | `perm_plugin_host` | on | yes | Load plug-ins in COSOUND's isolated process to read parameters and render previews |
| `presets.write` | `perm_preset_write` | off | yes | Save new presets into the plug-ins' preset folders, never overwriting |
| `memory.music` | `perm_music_memory` | off | yes | Use the musical preferences saved in the memory (genres, keys, BPM) to fill in requests, never to impose them |

File access is enforced in code, not only declared: `Ctx::authorized_file` accepts only files inside online authorized library folders,
imported files, or COSOUND's own data folder; `Ctx::write_file` writes only inside `generated/` of COSOUND's data folder and returns the SHA-256.
