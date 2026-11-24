# COSOUND internal knowledge base (not public)

Verified technical and musical knowledge the COSOUND agents will use to understand music and
operate DAWs, synths and plugins. Lives in the repo only: never served by the website.

## Sections (Prompt 02 architecture)

| # | Section | Folder(s) |
|---|---|---|
| 1 | DAW Knowledge | `logic-pro/`, `ableton-live/`, `fl-studio/`, `pro-tools/`, `cubase/`, `reaper/` |
| 2 | Music Theory | `music-theory/` |
| 3 | Music Production (rhythm, genres, structure) | `music-production/` |
| 4 | Audio Engineering (mixing, mastering) | `audio-engineering/` |
| 5 | Sound Design (synthesis, effects) | `synthesizers/`, `plugins-effects/` |
| 6 | MIDI Intelligence | `midi-intelligence/` |
| 7 | Voice-to-Music | `voice-to-music/` |
| 8 | Audio and Stem Analysis | `audio-stem-analysis/` |
| 9 | Reference Track Analysis | `reference-track/` |
| 10 | Smart Library (multidrive) | `smart-library/` |
| 11 | Plugin Intelligence (formats, SDKs, parameter control) | `integration-sdks/`, `synthesizers/` |
| 12 | Music Memory (personalization) | `music-memory/` |
| 13 | DAW Integration Capabilities | `daw-integration/` (comparison matrix) |
| 14 | Security and Permissions | `security-permissions/` |

Cross-cutting deliverables: `PROMPT-02-SUMMARY.md` (recommended technologies, limitations,
indications for Prompt 03).

Each folder has a `README.md` (the knowledge) and a `sources.md` (the source registry).

## Rules for every entry
- **Prefer official sources** (vendor manuals, API/SDK docs, standards bodies). Authoritative
  technical sources (peer-reviewed papers, maintained open-source project docs) are allowed and
  marked as such. Never invent APIs, parameters or measurements.
- **Paraphrase, do not copy.** Manuals are copyrighted: summarize in our own words and link the
  source. No bulk reproduction of vendor documentation.
- **Every technical fact links to a source** in that folder's `sources.md`, cited inline as `[S1]`, `[S2]`…
- **Versioned**: tie facts to the software version they were verified on.
- If a source could not be opened and checked, the fact is marked **(to verify)**.

### Source registry format (`sources.md`)

| ID | Source | URL | Retrieved | Software version | Reliability | Limitations |
|---|---|---|---|---|---|---|

Reliability levels:
- **A** – official vendor documentation or published standard (MIDI Association, ITU, EBU, AES, Apple, Microsoft…)
- **B** – authoritative technical: peer-reviewed paper, official docs/README of a maintained open-source project
- **C** – secondary: reputable press, educational sites, community wikis
- **U** – not verified (page unreachable or claim not confirmed)

### DAW capability classes
Keep "what the DAW can do" separate from "what COSOUND can drive from outside". Each operation is one of:
- `official` – documented programmatic interface (scripting API, SDK, documented file format)
- `protocol` – possible through an external protocol or tool (MIDI, MCU/HUI, OSC, MIDI Remote, file import/export)
- `ui-automation` – only by driving the user interface (Accessibility API, key commands, UI Automation)
- `none` – not available
- `to-verify` – not yet confirmed by a source
