# COSOUND – Technology decisions (ADR log)

Each decision: context → choice → alternatives rejected → consequences. Criteria from Prompt 03 §15:
compatibility (macOS + Windows), cost, performance, maintenance, future growth. Popularity is not a reason.
Status: **Accepted** (can be implemented), **Proposed** (needs confirmation), **Waiting** (depends on the central Agent).

---

### ADR-001 Desktop shell: Tauri 2 — Accepted
- **Context**: needs deep OS access (Accessibility/CGEvent/ScreenCaptureKit on macOS, UI Automation/SendInput on Windows, volumes, file watching, keychain, MIDI, mic) plus a modern chat UI.
- **Choice**: Tauri 2: Rust core + system WebView (WKWebView on macOS, WebView2 on Windows), React UI.
- **Alternatives**: *Electron*: mature but ships a full Chromium (large installer, high memory) and OS-level work would go through Node native modules anyway. *Native Swift + C#/WinUI*: best OS integration but two UIs to build and maintain. *JUCE (C++)*: great for audio plugins, weak for a chat/document UI; AGPL or paid license.
- **Consequences**: small installer; OS work in Rust with mature crates (`notify`, `symphonia`, `keyring`, `rusqlite`, `cpal`, `midir`, `objc2`, `windows`); WebView differences between macOS and Windows must be tested; team needs Rust.
- **Irreversibility**: medium (UI is React and portable; Core logic would need porting). Documented here per Prompt 03 §16.

### ADR-002 Music Engine: Python sidecar — Accepted
- **Context**: the libraries chosen in Prompt 02 (basic-pitch, Demucs, librosa, pyloudnorm, mido/pretty_midi, music21, Beat This!) are Python.
- **Choice**: separate Python process bundled with the app, started/stopped by Core, JSON-RPC over stdio. Inference via ONNX Runtime / Core ML where models allow, PyTorch only where unavoidable.
- **Alternatives**: rewrite in Rust (huge cost, no equivalents for most models); cloud processing (violates local-first, privacy, cost per minute of audio).
- **Consequences**: installer size → heavy models (Demucs/RoFormer, Whisper large) as **optional downloads** on first use; crash isolation (engine restart doesn't kill the app); Python version pinned (basic-pitch stops at 3.11 → verify or use its ONNX model directly).
- **Amendment 2026-11-25 (Prompt 05, M2)**: music theory and MIDI generation run **inside the Rust core** (`desktop/src-tauri/src/music/`), not in the Python sidecar. They need none of the Python libraries above (deterministic rules + a small SMF writer), and keeping them in Rust means the installed app works with no Python on the user's machine and the code is covered by the same `cargo test` and CI. The sidecar stays the plan for audio analysis, stems, voice transcription and reference analysis (M5, M6, M8, M9), where the Python models are needed. The capability contract does not change: the central Agent calls the same capability names whichever process serves them.

### ADR-003 Local database: SQLite + FTS5 + sqlite-vec — Accepted
- **Choice**: one SQLite file, single writer (Core). FTS5 for text search, sqlite-vec for audio-similarity.
- **Alternatives**: embedded Postgres (heavy), separate vector DB (extra service), cloud DB (forbidden for library data).
- **Consequences**: zero admin, fast for 1M+ rows; sqlite-vec is pre-1.0 → isolated behind a small module so it can be swapped.

### ADR-004 Cloud backend: keep Supabase — Accepted
- **Context**: Prompt 01 already runs Auth, Postgres+RLS, Storage on Supabase (`dhfdqmlbjmitpqpohgno`).
- **Choice**: extend it (new tables, Edge Functions); no second backend.
- **Consequences**: one auth system for site and app; Edge Functions (Deno/TypeScript) for gateway, entitlements, billing.

### ADR-005 AI Core: server-side gateway, provider-agnostic — Proposed (part Waiting: D-3)
- **Choice**: Edge Function `ai-gateway` holds provider keys, enforces per-plan quotas (`ai_usage`), selects model per task. Default models: Claude family (`claude-opus-5-5` for planning-heavy tasks, `claude-sonnet-5-5` for most sub-agent reasoning, `claude-haiku-5-5` for prompt cleanup/classification); swappable by config.
- **Alternatives**: keys in the app (leak risk); local LLM only (quality too low for production planning on typical laptops; may be added for offline mode).
- **Consequences**: if the central Agent brings its own LLM access, AI Core serves only sub-agent reasoning, or nothing (D-2/D-3).

### ADR-006 Agent communication: contract v1 + AIL adapters — Accepted (transport Waiting: D-1)
- **Choice**: JSON Schema contract in `packages/contracts`, code-generated for TS/Rust/Python; internal Capability Bus; AIL adapter per Agent transport. MCP is the first candidate adapter because it is an open protocol many agent frameworks already consume.
- **Consequences**: sub-agents are testable today; only the adapter changes on Agent delivery.

### ADR-007 DAW integration: one connector per DAW + human hand — Accepted
- **Choice**: connector interface in Core; per-DAW implementations using the routes verified in `knowledge/daw-integration`; human-hand executor (Accessibility/UIA + screen capture + vision) for anything without an API; universal fallback = export MIDI/audio file for import.
- **Order**: REAPER as first end-to-end test bed (fully scriptable) in parallel with Logic Pro (product priority), then Ableton Live, FL Studio, Pro Tools, Cubase.
- **Consequences**: Logic and parts of other DAWs depend on UI automation → health checks and version pinning per connector.

### ADR-008 Audio & MIDI I/O — Accepted (Windows virtual MIDI: open, decide after a Windows 11 test)
- `cpal` for mic capture (CoreAudio/WASAPI); `midir` for MIDI ports. Virtual MIDI ports: native on macOS (CoreMIDI); on Windows classic APIs have no virtual ports → use Windows MIDI Services loopback where available (to verify per Windows version) or a third-party loopback driver; affects Cubase (MIDI Remote) and FL Studio (MIDI scripting) connectors.

### ADR-009 Voice: local first — Accepted
- whisper.cpp for speech (MIT), SwiftF0/pYIN + basic-pitch for melody, Silero VAD; all local. Apple SpeechAnalyzer as optional macOS 26+ path.

### ADR-010 Security & distribution — Accepted
- OS keychains via `keyring`; Tauri capability allowlists; Developer ID + notarization (macOS), Authenticode (Windows); Tauri signed updater; installers in the existing private `releases` bucket via signed URLs.

### ADR-011 Licensing policy — Accepted
- Only permissive or LGPL (dynamically linked) dependencies in the shipped app. Excluded: Essentia (AGPL), madmom/ADTOF models (non-commercial), Chordino (GPL), aubio (GPL), pedalboard (GPL), JUCE without commercial license, openWakeWord bundled models. Every model's weights license is checked before bundling.

### ADR-012 Commercial gating by entitlements — Accepted
- Feature keys per plan in `plan_features`; trial length in `app_settings`; auto-conversion only with a recorded `consents` row. Payments postponed (Stripe planned).
