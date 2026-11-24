# COSOUND – Risks, limitations and technical roadmap (deliverables 14–15)

Version 0.3 · 2026-11-24

## 1. Risks and limitations

| # | Risk / limitation | Impact | Likelihood | Mitigation |
|---|---|---|---|---|
| R1 | **Central Agent unknown** (stack, location, memory) | high | certain (today) | AIL with swappable adapter; decisions D-1…D-6 tracked; no Agent features assumed |
| R2 | **Logic Pro has no public API** (first target DAW) | high | certain | hybrid connector: MIDI file import + control surface/OSC + human hand; health checks per Logic version |
| R3 | Human hand fragility (plugin GUI updates, HiDPI, window layout, user touching mouse) | medium | high | verify-after-every-action, GUI maps per plugin version, operating overlay + stop key, host params preferred when available |
| R4 | Vision mis-reads a control | medium | medium | read value back from screen and/or host param; render-and-listen check; undo |
| R5 | Windows has no native virtual MIDI ports in classic APIs | medium (Cubase, FL connectors) | high | Windows MIDI Services loopback (to verify) or a loopback driver; document setup |
| R6 | Engine size (PyTorch models) | medium | high | ONNX/Core ML first; heavy models as optional downloads |
| R7 | Model/library licenses | high (legal) | medium | ADR-011 policy; license check before bundling each weight file |
| R8 | Analysis accuracy (chords, beatbox, hummed tempo, structure) | medium | high | confidence on every result, user confirmation steps, `basis` field, in-house models later |
| R9 | DAW updates break connectors | medium | medium | connector version pinning, compatibility matrix in app, quick-fix releases via updater |
| R10 | Copyright/terms: reference tracks, Suno/Udio material, streaming | high (legal) | medium | analysis only of user-supplied files; output is original work; respect generator terms (e.g. Suno: no training on outputs); legal review before launch |
| R11 | AI-disclosure requirements of distribution platforms | medium | medium | provenance log of what COSOUND did (op log) available to the user; never features to hide AI use |
| R12 | macOS permissions UX (Accessibility, Screen Recording) scare users | medium | medium | just-in-time requests with clear explanation; features degrade gracefully without them |
| R13 | Performance on low-end laptops (separation, transcription) | medium | medium | background jobs with progress, GPU/ANE when available, quality presets |
| R14 | Development host is Linux; product targets macOS/Windows | high (verification) | certain | CI builds on macOS and Windows runners; a real Mac with Logic + plugins for DAW/hand testing (needed from milestone M4) |
| R15 | Signing costs/accounts (Apple Developer Program, Windows code-signing certificate) | low cost, blocking for release | certain | obtain before first external beta |
| R16 | Trial auto-conversion under EU consumer law | high (legal) | medium | explicit consent stored with terms version; reminder before conversion; legal review before payments |
| R17 | LLM cost per user | medium | medium | per-plan quotas in `ai_usage`, deterministic tools first, small models for simple tasks |

## 2. Technical roadmap

Principle: each milestone ships something testable on its own, behind the contract, so it plugs into
the central Agent later (parallel-development directive). Milestones are ordered by dependency, not dated.

| Milestone | Content | Verifiable result | Needs |
|---|---|---|---|
| **M0 Foundations** | monorepo layout (§12 of ARCHITECTURE), `packages/contracts` + codegen, Tauri app skeleton with IT/EN, Python engine sidecar with JSON-RPC ping, local SQLite migrations, CI building macOS + Windows | app opens on both OS, engine responds, CI green | — |
| **M1 Smart Library** | volumes & devices, authorized roots, incremental scan, metadata, duplicates, offline drives, search UI | index 10k-file test library, unplug/replug drive keeps records | — |
| **M2 Music Theory + MIDI Generation** | theory engine, MIDI generation (bass, chords, melody, drums), SMF export, piano-roll preview | generated MIDI passes theory validation tests; opens in any DAW | — |
| **M3 AIL + Bus + Safety Gate + Chat UI** | capability registry from `agent.yaml`, temp test connector (dev only), op log, backups, undo model, chat & op cards | explicit capability calls run end to end; destructive op blocked without confirmation | — |
| **M4 DAW Controller v1** | REAPER connector (full), Logic connector v1 (MIDI import, transport/mixer via control surface/OSC), human-hand proof of concept (Logic + Massive: open plugin, set a control, verify) | bassline from M2 lands in REAPER and Logic; hand changes a Massive control and verifies it | **Mac with Logic + Massive** |
| **M5 Voice-to-Music** | mic capture, push-to-talk, speech → brief, hum/whistle → MIDI with confidence, count-in workflow | hummed phrase → editable MIDI; spoken request → brief | mic-equipped machine |
| **M6 Audio Analysis & Stems** | decode, tempo/key/loudness/structure, stem separation job, keep/rebuild/replace classification, Suno stem import | Suno stems imported and classified; measurements match reference tools | — |
| **M7 Plugin Intelligence** | plugin scan (no loading where possible), host-parameter maps, GUI maps for the hand, preset inventory | top synths (Massive, Serum, Vital, Sylenth1, Predator, Albino) inventoried with control level | Mac/PC with plugins |
| **M8 Reference Track Analyzer** | web research mode + audio analysis mode, Reference Profile, library matching | profile clearly labelled web vs audio | — |
| **M9 Mixing & Mastering** | measurement suite (LUFS, true peak, spectrum, stereo), proposals, mastering targets | numbers match a reference meter within tolerance | — |
| **M10 Accounts in app & subscriptions** | login, entitlements token, trial with consent, plan gating; then Stripe Checkout + webhook | feature gating works offline within grace period | Stripe account (later) |
| **M-Agent** | analyze partner's Agent, write adapter, resolve D-1…D-6, remove temp connector | Agent drives sub-agents end to end | **Agent code** |
| **Later** | Ableton Live, FL Studio, Pro Tools, Cubase connectors; mobile; local LLM offline mode | — | DAW licenses / machines |

M0–M3 and M6, M8 can be built and tested on the current Linux server (engine and logic are
cross-platform; CI covers macOS/Windows builds). M4, M5, M7 require real macOS/Windows hardware.
