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
cd src-tauri && cargo test --features temp-connector   # Rust core: library, projects, imports, DAW, plugins, AIL
cd .. && npx tsc -p .                                    # UI type check
```

## Build installers
```bash
npm run app:build         # = tauri build → .dmg/.app (macOS), .msi/.exe (Windows); never includes the test connector
```
The resulting installers are **unsigned**:
- On macOS, Gatekeeper blocks them until they are signed with a Developer ID and notarized.
- On Windows, SmartScreen warns until they are signed (Authenticode).

They are not for distribution. CI (`docs/desktop/ci-desktop.yml`, to be moved to `.github/workflows/`) builds them on macOS, Windows and Linux runners as test artifacts.

## Structure
```
desktop/
  src/                       UI (React). Talks to the core only through src/lib/api.ts
    pages/                   Chat, Production, Library, Imports, Plugins, Daws, Agents, Settings, Account, Onboarding
    components/              Shell (sidebar/topbar), Voice (mic + takes), OpList (history/undo), ui
    i18n/                    it.ts / en.ts (every string bilingual)
    lib/                     api.ts (commands), store.tsx (app state), supabase.ts (account)
  src-tauri/
    src/lib.rs               Tauri commands only (thin layer)
    src/db.rs                local SQLite, settings, operation register, undo
    src/library.rs           Smart Library v1: volumes, authorized folders, scan, search
    src/projects.rs          projects, imported material (references + SHA-256)
    src/daw.rs               DAW detection (no connection)
    src/plugins.rs           plugin inventory (files + metadata, plugins never loaded)
    src/ail/                 Agent Integration Layer + temporary_test_connector (dev only)
    migrations/              local DB schema
packages/agents/registry.json  capability catalog of the 12 sub-agents (available / planned)
```
Every engine is its own Rust module with its own tests and no dependency on the UI. Future engines plug in the same way: MIDI, audio analysis, voice, reference, DAW connectors.
1. Add a module.
2. Add its commands.
3. Mark its capabilities `available` in `registry.json`.

A test fails if the registry and the Capability Bus disagree.

## Local data
The SQLite file `cosound.db` and the `takes/` folder (voice recordings) live in the app data folder:
- macOS: `~/Library/Application Support/com.cosound.desktop/`
- Windows: `%APPDATA%\com.cosound.desktop\`

User libraries, imported files and DAW projects are **never copied or modified**: only their paths and hashes are stored.
