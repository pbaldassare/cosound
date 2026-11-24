# COSOUND – Multi-agent architecture (proposal, v0.2)

Status: **PROPOSAL** (2026-11-24). Written under `docs/ARCHITECTURE-DIRECTIVE.md`.
Part of the Prompt 03 architecture: overview in `ARCHITECTURE.md`, contract and adapter in
`AGENT-INTEGRATION-LAYER.md` (which supersedes §5 details where they differ, e.g. `basis` field).
The central Agent is being built by the project partner and has **not been analyzed yet**: every
point marked ⏳ depends on its real code and will be revised when it arrives. Nothing here
describes or assumes how the central Agent works internally.

## 1. Analysis of the existing central Agent
⏳ Not possible yet: the code does not exist (partner is building it). On delivery we will document:
language, framework, LLM provider/model, request/response loop, tool-calling mechanism, memory,
inter-agent communication, limits. Then we adapt the sub-agents' side, never the Agent.

## 2. Integration principle
One brain, many specialists. The central Agent is the **only** component that talks to the user,
plans, delegates, evaluates results and decides. Sub-agents are **callable capabilities** with a
documented contract. They never call each other directly and never talk to the user: if a sub-agent
needs another's output, it returns `needs` and the central Agent decides.

```
                 User (chat / voice)
                         │
              ┌──────────▼──────────┐
              │   CENTRAL AGENT     │  (partner's code – unchanged)
              │ plan · delegate ·   │
              │ evaluate · confirm  │
              └──────────┬──────────┘
                         │  sub-agent contract (§5)
     ┌──────────┬────────┼─────────┬──────────────┐
     ▼          ▼        ▼         ▼              ▼
 Creative    Analysis   Library   Engineering   DAW Controller ──► Safety gate ──► DAW connectors
 Producer    Voice→Music Smart Lib Mixing                         (confirm, backup,   (Logic, Live, PT,
 Rhythm      Audio Anal. Plugin    Mastering                       op log, undo)       Cubase, FL, REAPER)
 Theory      Reference   Intel.
 Sound Des.
                         │
           Shared services (not agents): Knowledge Base · Project Context Store ·
           Music Memory · Local Library Index · Audio/MIDI processing engine
```

## 3. Sub-agents: roles and responsibilities

| # | Sub-agent | Owns (does) | Does NOT do | Knowledge | Main tools | Permissions |
|---|---|---|---|---|---|---|
| 1 | Music Producer | arrangement plans, section maps, energy curve, which parts to create; combines outputs of 2–4 into a production plan | writing notes itself, DAW ops | music-production/arrangement, genres | arrangement templates | none (pure reasoning) |
| 2 | Rhythm Engineer | BPM, groove, swing, drum patterns, percussion MIDI | harmony | music-production, midi-intelligence | MIDI writer, groove templates | none |
| 3 | Sound Designer | patch recipes, preset choice/edit, timbre, synth automation curves | plugin discovery (→9) | synthesizers, plugins-effects | preset read/write (documented formats), param maps from 9 | read preset folders |
| 4 | Music Theory | keys, scales, chords, progressions, melodies, basslines, voicing; validates musical correctness of any MIDI | rhythm feel, sound | music-theory, midi-intelligence | theory engine, MIDI writer | none |
| 5 | Voice-to-Music | mic capture → transcription (speech) or notes/rhythm (singing, humming, beatbox) → MIDI draft with confidence | deciding harmony (→4 refines) | voice-to-music | whisper.cpp, SwiftF0/pYIN, basic-pitch, VAD | microphone |
| 6 | Audio Analysis | analyze files/stems: tempo, key, loudness, spectrum, transients; stem separation; audio→MIDI where feasible; mix measurements | judging the mix (→10) | audio-stem-analysis | decoder, Demucs/RoFormer, Beat This!, pyloudnorm | read user-chosen files |
| 7 | Reference Track | web research on a track + audio reference analysis (via 6) → "reference profile" for original work | copying; downloading from streaming | reference-track | MusicBrainz API, web search, 6 | network |
| 8 | Smart Library | index/search authorized folders and drives, duplicates, offline drives | analyzing new audio in depth (→6) | smart-library | SQLite+FTS5+sqlite-vec, notify, volume IDs | authorized folders, removable/network volumes |
| 9 | Plugin Intelligence | inventory of installed plugins/presets, formats, parameter maps per plugin | choosing sounds (→3) | integration-sdks, synthesizers | plugin scan caches, DAW param APIs | read plugin folders |
| 10 | Mixing Engineer | mix diagnosis from **measured** data, mix moves proposal (EQ/comp/sidechain/levels) | measuring (→6) | audio-engineering/mixing, measurement-analysis | — | none |
| 11 | Mastering Engineer | loudness/true-peak targets per destination, master chain proposal, final QC | — | audio-engineering/mastering | — | none |
| 12 | DAW Controller | **the only component that touches a DAW**: translates abstract ops into connector calls, reports exact results | musical decisions | daw-integration + DAW folders | connectors per DAW | Accessibility/Automation (macOS), MIDI ports, OSC/local network |

Proposed additions (only if useful, no overlap): none required now. A **Project Context Store** and
**Safety gate** are services, not agents (no reasoning, deterministic), so they don't duplicate the
central Agent's coordination role.

## 4. Shared services (deterministic components, available to the central Agent)
- **Knowledge Base**: `docs/knowledge/` → later indexed per fact/version, retrieved per sub-agent.
- **Project Context Store**: current song state (DAW, tempo, key, sections, tracks, generated parts, decisions). Read by all; written only by the central Agent and DAW Controller results.
- **Music Memory**: user preferences (see `knowledge/music-memory`). Local-first, user-editable.
- **Library Index**: local SQLite (see `knowledge/smart-library`). Never uploaded by default.
- **Processing engine**: audio/MIDI libraries shared by 5, 6, 8 (one install, not one per agent).

## 5. Communication contract (central Agent ↔ sub-agents)
Transport is ⏳ (depends on the Agent's stack). Proposal: expose each sub-agent as a set of **tools**
over the **Model Context Protocol (MCP)**, an open, vendor-neutral standard that most agent frameworks
can call; fallback: the same JSON contract over local HTTP or stdio. Every call is structured and logged.

Request:
```json
{
  "task_id": "uuid", "agent": "music_theory", "action": "generate_bassline",
  "input": { "key": "A minor", "bpm": 124, "bars": 8, "style": "melodic techno" },
  "context_ref": "project:current@rev42",
  "constraints": { "max_duration_s": 30 },
  "requested_by": "central_agent"
}
```
Result:
```json
{
  "task_id": "uuid", "status": "ok | partial | needs_input | error",
  "output": { "midi_ref": "artifact://…", "notes": 64 },
  "explanation": "Root-fifth pattern on rolling 16ths, i–VI–VII movement…",
  "confidence": 0.8,
  "measured": true,
  "needs": [ { "agent": "rhythm_engineer", "why": "groove template for 124 BPM" } ],
  "proposed_daw_ops": [ { "op": "insert_midi_clip", "track": "Bass", "at_bar": 17, "risk": "low" } ],
  "sources": ["knowledge/music-theory/harmony.md"],
  "error": null
}
```
Rules: sub-agents **propose** DAW operations; only the DAW Controller executes them, after the central
Agent approves and the safety gate passes. `explanation` lets the user understand and edit choices.
`measured:false` is mandatory whenever a number was not actually computed.

## 6. Safety (directive §6)
| Risk | Examples | Policy |
|---|---|---|
| read | read project state, analyze file, search library | allowed |
| low | add new track/clip, new automation lane, set tempo of an empty project | allowed, logged, undoable |
| medium | change existing parts, mixer values, plugin params, tempo of an existing song | preview + user confirm |
| high | delete tracks/clips, overwrite files, replace plugins, render over existing file | confirm + **project backup first** |
- **Backup**: before medium/high ops, copy/snapshot the project (Save As versioned copy) when the DAW allows it.
- **Operation log**: every op with request, result, timestamp, DAW, before/after values (local).
- **Undo**: DAW native undo where the connector can trigger it; otherwise inverse op from the log; otherwise restore backup. Stated honestly per DAW.
- **Errors**: sub-agent returns `status:error` with cause; central Agent decides retry/alternative/ask user. No silent fallbacks.
- **Permissions**: each sub-agent declares the OS permissions it needs (table §3); requested only when that feature is first used.

## 7. Known incompatibilities / technical issues
1. ⏳ Central Agent stack unknown: MCP fit, process model, local vs cloud execution.
2. If the central Agent runs in the cloud, DAW Controller, Smart Library, Voice and audio analysis **must still run locally** (DAW access, mic, local files, privacy). Needs a local bridge.
3. Logic Pro (first target) has no public API: DAW Controller relies on control surface/OSC + MIDI import + Accessibility automation. Fragile across Logic versions.
4. Heavy ML (stem separation, transcription) needs local GPU/ANE; must run as background jobs with progress, not blocking chat.
5. Licensing: several analysis libraries are AGPL/GPL or have non-commercial models (Essentia, madmom models, ADTOF, Chordino). Excluded from the product.
6. Latency: voice→MIDI and DAW ops should run without an LLM round-trip where possible (deterministic tools).

## 7b. "Human hand" principle (product requirement, 2026-11-24)
COSOUND must operate instruments and plugins **the way the user's hand would**: open Massive (or any
installed synth) inside the DAW, turn its knobs, choose its presets, record its automations. Closed
preset formats are irrelevant on this route: we never edit the plugin's files, we operate the real
plugin that the user owns, on the user's machine, as the user would. Three layers, used in order:

1. **Host parameters** (precise, fast): every VST3/AU plugin publishes automatable parameters to the
   DAW. COSOUND sets them through the DAW route for that DAW (control surface/OSC/MIDI learn in Logic,
   LOM in Live, ReaScript in REAPER…) and writes automation lanes on them. How many parameters each
   synth exposes is plugin-specific (e.g. Massive XL documents only Macros + Morpher) → measured per plugin.
2. **On-screen hand** (everything else): screenshot of the plugin window → vision model locates
   controls (plugin GUIs are custom-drawn, so there is no accessibility tree) → real mouse/keyboard
   events (drag a knob, click a menu, browse the plugin's own preset browser) → new screenshot to
   verify the value changed. Automations can be recorded exactly like a human: DAW in Latch/Touch
   record while the hand moves the knob.
3. **Ear check**: render a short test note/phrase and analyze it (spectrum, envelope, loudness) to
   confirm the sound matches the request; iterate.

- **Plugin maps**: the first time a plugin is used, Plugin Intelligence builds a map (host parameter
  list + GUI layout: where each control is, its range, how it reads on screen) and caches it per
  plugin version. Next uses are fast; map rebuilt when the plugin updates.
- **Ownership** (no new agent): Sound Designer decides *what* to set; Plugin Intelligence owns the
  maps; DAW Controller is the only one that moves the hand. Safety gate applies (§6).
- **Permissions**: macOS Accessibility (send input) + Screen Recording (see the plugin window);
  Windows: SendInput + screen capture (elevated DAW requires COSOUND elevated too).
- **UX**: while the hand works, the user must not use mouse/keyboard → visible "COSOUND is operating"
  overlay with an instant stop key.
- **Limits**: slower than an API (seconds per action); sensitive to window size/HiDPI scaling and
  plugin GUI updates; vision mistakes are caught by the verify step, not assumed away.

## 8. Integration plan without duplication
1. Prompt 03+: build sub-agents and services behind the §5 contract, each testable alone with a throwaway harness (**not** an orchestrator).
2. When the central Agent arrives: analyze it (§1), map its tool mechanism to the contract, adapt our adapter layer, delete the harness.
3. Any new feature: assign to an existing sub-agent or add a tool; a new sub-agent only if no existing one owns that competence.
