# COSOUND desktop app

Desktop application for macOS and Windows. It is built with Tauri 2: a Rust core and a React + TypeScript + Tailwind interface (ADR-001).
It uses the same visual identity as the website.

> Status: **development version 0.1.0, not distributed.** There are no signed installers and no download links yet.
> The central Agent is **not connected** (it is being developed by the partner). See `docs/desktop/PROMPT-04-REPORT.md`.

## Requirements
| | macOS | Windows | Linux (development only) |
|---|---|---|---|
| Node.js | 22+ | 22+ | 22+ |
| Rust | stable (rustup) | stable (rustup, MSVC) | stable |
| System | Xcode Command Line Tools | Visual Studio Build Tools (C++), WebView2 (already in Windows 10/11) | `libwebkit2gtk-4.1-dev libayatana-appindicator3-dev librsvg2-dev libxdo-dev build-essential` |

## Setup
```bash
cd desktop
cp .env.example .env      # fill in the same values as web/.env (public Supabase URL + publishable key)
npm install
```
Without `.env` the app still works. Local features are unaffected; the Account section says that the online account is not configured.

## Run (development)
```bash
npm run app:dev           # tauri dev with the TEMPORARY test connector (feature temp-connector)
npx tauri dev             # same, without the test connector (as a release build behaves)
```

## Tests
```bash
cd src-tauri && cargo test --features temp-connector   # Rust core: Smart Library, projects, imports, DAW, plugins, music, agents, AIL
cd .. && npx tsc -p .                                    # UI type check
```

## Build installers
```bash
npm run app:build         # = tauri build → .dmg/.app (macOS), .msi/.exe (Windows); never includes the test connector
```
The resulting installers are **unsigned**:
- On macOS, Gatekeeper blocks them until they are signed with a Developer ID and notarized.
- On Windows, SmartScreen warns until they are signed (Authenticode).

They are not for distribution. CI (`.github/workflows/desktop.yml`) builds them on macOS, Windows and Linux runners as test artifacts on every push that touches `desktop/`. Publishing that workflow file to GitHub needs a login with the `workflow` permission.

## Structure
```
desktop/
  src/                       UI (React). Talks to the core only through src/lib/api.ts
    pages/                   Chat, Production, Sketch (music sketch + piano roll), Library, Imports, Plugins, Daws, Agents, Settings, Account, Onboarding
    components/              Shell (sidebar/topbar), Voice (mic + takes), OpList (history/undo), SmartLibrary (search, detail, loops, duplicates, index), ui
    i18n/                    it.ts / en.ts (every string bilingual)
    lib/                     api.ts (commands), store.tsx (app state), supabase.ts (account), preview.ts (Web Audio preview of a sketch)
  src-tauri/
    src/lib.rs               Tauri commands only (thin layer)
    src/db.rs                local SQLite, settings, operation register, undo
    src/library.rs           drives (identity, kind, online) and authorized folders
    src/smartlib/            Smart Library engine (Prompt 07): incremental scan, Symphonia decoding, audio features, MIDI and
                             preset readers, classification, IT/EN search, similarity, loops, duplicates, background service
                             (docs/smart-library/)
    src/projects.rs          projects, imported material (references + SHA-256)
    src/daw.rs               DAW detection (no connection)
    src/plugins/             Plugin Intelligence System (Prompt 08): scanner, registry, knowledge, parameters, presets,
                             sound design, effect chains, DAW adapter (docs/plugin-intelligence/)
    auhost/main.swift        cosound-auhost: isolated Audio Unit host (macOS), compiled by build.rs, bundled as a sidecar
    src/music/               Music Intelligence Engine (Prompt 09) on top of the M2 engine: notes, scales, keys, chords, harmony,
                             melody, bass, arp, rhythm, styles, request (IT/EN words), compose, edit, analysis, validate,
                             voice and reference bridges, smf (MIDI files), api (docs/music-intelligence/)
    src/audio/wav.rs         WAV meter: BS.1770 loudness, true peak, RMS, correlation, bands
    src/sound.rs             sound recipes and synth catalog (from docs/knowledge)
    src/memory.rs            Music Memory (memory_facts)
    src/agents/              Multi-Agent System: protocol, registry, runtime (Agent API), 13 agents, tests (docs/agents/)
    src/ail/                 Agent Integration Layer + MOCK central-agent connector (dev only)
    migrations/              local DB schema
packages/agents/*.json         agent manifests (13), tools.json, permissions.json – the Agent and Tool Registry
packages/agents/render_docs.py regenerates docs/agents/AGENTS.md and REGISTRIES.md from the manifests
packages/plugin-knowledge/     official plug-in documentation as JSON (Apple AU parameters from the SDK header)
packages/music-styles/         style profiles (11 genres) with sources, overridable from the app data folder
packages/contracts/*.json      JSON Schemas of the Agent Protocol
```
Every engine is its own Rust module with its own tests and no dependency on the UI. Future engines plug in as level-3 tools:
1. Add the tool module and list it in `packages/agents/tools.json`.
2. Use it from an agent operation and mark that operation `available` in the agent's manifest.

A test fails if a manifest and the code disagree. See `docs/agents/`.

## Local data
The SQLite file `cosound.db`, the `takes/` folder (voice recordings) and the `generated/` folder (exported MIDI sketches) live in the app data folder:
- macOS: `~/Library/Application Support/com.cosound.desktop/`
- Windows: `%APPDATA%\com.cosound.desktop\`

User libraries, imported files and DAW projects are **never copied or modified**: only their paths and hashes are stored.
