# COSOUND – Agent Registry, Tool Registry, permissions

Generated from `packages/agents/` (`<agent>.json`, `tools.json`, `permissions.json`). At runtime the same data, with live state
(agent enabled, permission granted, which agents use a tool), is returned by the Agent API message `registry.get`.

## 1. Agent Registry

One JSON manifest per agent, schema `packages/contracts/agent-manifest.schema.json`. Each declares: id, name, version, role,
competencies, operations (status, risk, approval, input JSON Schema, output, tools, permissions, basis, requires), tools, permissions,
limits, input/output formats, the agents whose outputs it can use. A test fails if a manifest and the code disagree
(available operations = implemented operations; tools and permissions exist; available operations use only available tools).

| Agent | Version | State | Operations (available/total) | Tools |
|---|---|---|---|---|
| `music_producer` | 1.0.0 | partial | 3/4 | ai.core, music.arrangement, music.theory |
| `rhythm_engineer` | 1.0.0 | partial | 3/4 | music.generator, music.smf_writer |
| `sound_designer` | 1.0.0 | partial | 2/3 | daw.connectors, library.index, plugins.catalog, plugins.param_maps, plugins.scanner, sound.recipes |
| `music_theory` | 1.0.0 | active | 7/7 | music.generator, music.key_detection, music.smf_writer, music.theory |
| `voice_to_music` | 1.0.0 | planned | 0/4 | music.smf_writer, voice.pitch_tracker, voice.speech_to_text |
| `audio_analysis` | 1.0.0 | partial | 2/5 | audio.stem_separator, audio.structure, audio.tempo_key, audio.wav_meter |
| `reference_track` | 1.0.0 | partial | 1/2 | audio.wav_meter, music.arrangement, web.research |
| `smart_library` | 1.0.0 | partial | 4/5 | audio.tempo_key, library.filename_meta, library.index |
| `plugin_intelligence` | 1.0.0 | partial | 3/4 | daw.connectors, plugins.catalog, plugins.param_maps, plugins.scanner, sound.recipes |
| `mixing_engineer` | 1.0.0 | partial | 1/2 | audio.wav_meter, daw.connectors |
| `mastering_engineer` | 1.0.0 | partial | 1/2 | audio.processing_chain, audio.wav_meter |
| `daw_controller` | 1.0.0 | partial | 2/3 | daw.connectors, daw.detector, daw.routes, files.writer |
| `adaptive_workflow` | 1.0.0 | partial | 7/8 | daw.connectors, memory.facts |

Enable/disable: per agent, by the user (setting `agent_enabled.<id>`, logged). A disabled agent answers `AGENT_DISABLED`.

## 2. Tool Registry (level 3: engines and tools)

Agents never call each other: they only use these tools, through the execution context (`Ctx`), which records `tools_used` in every result.

| Tool | Status | Module | What it does | Used by |
|---|---|---|---|---|
| `music.theory` | available | `desktop/src-tauri/src/music/theory.rs` | Scales, keys, chords, roman numerals, functions and cadences | music_producer, music_theory |
| `music.key_detection` | available | `desktop/src-tauri/src/music/theory.rs` | Key estimate from pitch classes | music_theory |
| `music.generator` | available | `desktop/src-tauri/src/music/generate.rs` | Chords, bass, melody, drums, transitions, musical checks | rhythm_engineer, music_theory |
| `music.smf_writer` | available | `desktop/src-tauri/src/music/smf.rs` | Writes format-1 MIDI files | rhythm_engineer, music_theory, voice_to_music |
| `music.arrangement` | available | `desktop/src-tauri/src/music/arrangement.rs` | Per-genre templates from the knowledge base | music_producer, reference_track |
| `audio.wav_meter` | available | `desktop/src-tauri/src/audio/wav.rs` | Loudness, true peak, RMS, correlation, bands | audio_analysis, reference_track, mixing_engineer, mastering_engineer |
| `library.index` | available | `desktop/src-tauri/src/library.rs` | Local index of authorized folders | sound_designer, smart_library |
| `library.filename_meta` | available | `desktop/src-tauri/src/library.rs` | BPM and key read from file names | smart_library |
| `plugins.scanner` | available | `desktop/src-tauri/src/plugins.rs` | Plugin inventory without loading them | sound_designer, plugin_intelligence |
| `plugins.catalog` | available | `desktop/src-tauri/src/sound.rs` | Profiles of well-known synths from the knowledge base | sound_designer, plugin_intelligence |
| `sound.recipes` | available | `desktop/src-tauri/src/sound.rs` | Synthesis recipes in generic terms | sound_designer, plugin_intelligence |
| `daw.detector` | available | `desktop/src-tauri/src/daw.rs` | Installed DAWs and versions | daw_controller |
| `daw.routes` | available | `desktop/src-tauri/src/agents/daw_controller.rs` | Verified technical route per DAW | daw_controller |
| `files.writer` | available | `desktop/src-tauri/src/agents/mod.rs` | Writes only into COSOUND's generated/ folder, with SHA-256 | daw_controller |
| `memory.facts` | available | `desktop/src-tauri/src/memory.rs` | COSOUND's single local memory (memory_facts) | adaptive_workflow |
| `daw.connectors` | planned – milestone M4 (Logic Pro first) | — | Connectors that execute operations in DAWs | sound_designer, plugin_intelligence, mixing_engineer, daw_controller, adaptive_workflow |
| `plugins.param_maps` | planned – milestone M7 | — | Plugin parameters through the DAW | sound_designer, plugin_intelligence |
| `voice.pitch_tracker` | planned – milestone M5 | — | Pitch and notes from voice | voice_to_music |
| `voice.speech_to_text` | planned – milestone M5 | — | Speech transcription | voice_to_music |
| `audio.tempo_key` | planned – milestone M6 | — | BPM and key from audio | audio_analysis, smart_library |
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

File access is enforced in code, not only declared: `Ctx::authorized_file` accepts only files inside online authorized library folders,
imported files, or COSOUND's own data folder; `Ctx::write_file` writes only inside `generated/` of COSOUND's data folder and returns the SHA-256.
