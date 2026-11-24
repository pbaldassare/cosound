# Music Memory (personalization)

Section 12. How COSOUND remembers who the user is as a producer and how the assistant presents
itself. Sources: `sources.md`. Verified 2026-11-24. **Proposal** = design suggestion, not a sourced
fact. **(to verify)** = not confirmed against a source.

## 1. Assistant identity (user-assigned name)
- **Proposal:** the user picks a name at onboarding (default "COSOUND"). Stored as
  `profile.assistant_name`; used in UI labels, chat messages ("Nova suggests…"), system prompt
  ("You are Nova, the user's production assistant") and TTS. Validate length (1–32 chars) and
  strip control characters; it is user text injected into prompts, so it never carries
  instructions – only a name slot.
- **Language:** every string IT/EN (project rule); the name itself is not translated.

### Voice (TTS)
| Option | Platform | Notes |
|---|---|---|
| `AVSpeechSynthesizer` | macOS | Queues utterances; voice selectable per utterance via `AVSpeechSynthesisVoice`; pause/stop; app must keep a strong reference until speech ends [S4] |
| `Windows.Media.SpeechSynthesis.SpeechSynthesizer` | Windows | Uses installed system voices; synthesizes text to an audio stream [S5]. Calling it from an unpackaged desktop app **(to verify)** |
| Piper (local neural TTS) | both | Original repo MIT but archived; maintained fork is GPL-3.0 [S9] – GPL matters for a closed app; voice licences vary **(to verify)** |
| Cloud TTS | both | Better quality, but sends text to a third party → opt-in only (see `../security-permissions/`) |

**Proposal:** phase 1 uses the OS voices (zero licence cost, offline); store `voice_id` per platform.

### Wake word (later phase)
| Engine | Licence | Custom keyword ("Hey Nova") | Notes |
|---|---|---|---|
| openWakeWord | Code Apache-2.0; **bundled pre-trained models CC BY-NC-SA 4.0** (non-commercial) because of training data [S6] | Train your own model (custom models + optional speaker-verifier stage) [S6] | A commercial product must train its own model on cleanly licensed data |
| Porcupine (Picovoice) | Repo Apache-2.0 [S7] | Custom wake words made in Picovoice Console [S7]; runtime needs a Picovoice **AccessKey** from an account [S8] | Commercial terms/pricing not read **(to verify)**. Supports macOS x86_64/arm64 and Windows x86_64/arm64 [S7] |

A user-chosen wake word means per-user model training – a **later-phase item**. Always-on
listening also requires microphone permission (`NSMicrophoneUsageDescription` on macOS) [S13]
and a clear on/off indicator. Phase 1: push-to-talk / hotkey.

## 2. What we remember
- Genres and sub-genres, typical BPM ranges, keys/scales used
- Libraries and favourite packs/folders (references to Smart Library IDs, not files)
- Favourite plug-ins / synths / presets, DAW(s) used
- Production style (e.g. "sidechain heavy", "lo-fi textures"), arrangement habits
- Mixing preferences (loudness target, reference tracks, bus chains)
- Working habits (session times, language, verbosity, how much autonomy to give the assistant)

## 3. Data model (Proposal)
Three layers, all visible and editable by the user:

1. **Structured profile** – one row per user, typed fields:
   `assistant_name, voice_id, language, daws[], genres[], bpm_min, bpm_max, loudness_target_lufs, autonomy_level, memory_enabled`.
2. **Explicit facts** – things the user said or typed ("I always mix vocals with Soothe2").
   `source = 'user'`, `confidence = 1.0`.
3. **Learned facts** – inferred from behaviour ("uses 140 BPM in 8 of last 10 projects").
   `source = 'inferred'`, `confidence 0–1`, `evidence` (short text: what it was based on),
   `status = 'suggested' | 'confirmed' | 'rejected'`. Low-confidence facts are suggested to the
   user ("Remember that you prefer…?") before being used heavily.

`memory_facts(id, user_id, category, key, value_json, text, source, confidence, evidence, status, created_at, updated_at, last_used_at, expires_at, embedding)`

Rejected facts are kept only as a "don't re-learn this" marker (key + rejected), or deleted on
request.

## 4. Where stored
- **Local-first:** SQLite in the app data folder; `text` indexed with FTS5 [S10]; optional
  embedding column via sqlite-vec for semantic retrieval [S11].
- **Optional sync** (user toggle, off by default) to Supabase tables `memory_profile` and
  `memory_facts` keyed by `user_id`. Supabase: enable RLS on every table in an exposed schema; a
  table without RLS is readable/writable by any role with a grant; `auth.uid()` returns the caller's
  user ID and is used in policies like `using ((select auth.uid()) = user_id)` [S1]. This matches
  the pattern already used in the COSOUND schema (`profiles: read own`) [S12].
- **Proposal policies:** select/insert/update/delete all `to authenticated` with
  `(select auth.uid()) = user_id` (insert/update via `with check`); `user_id` references
  `auth.users(id) on delete cascade` so account deletion erases memory. Admins get **no** read
  policy on memory tables (data minimization).
- Never sync: raw audio, library file lists, project files.

## 5. User control and GDPR
- **Memory screen:** list of every profile field and fact, grouped by category, showing source,
  confidence, evidence; edit, delete, "forget everything", pause learning, export (JSON).
- GDPR rights include access, rectification, erasure ("right to be forgotten"), restriction,
  portability and objection; requests must be answered within one month (extendable with notice)
  [S2]. Corresponding GDPR articles: 15 (access), 16 (rectification), 17 (erasure), 20
  (portability), 25 (data protection by design and by default), 5(1)(c) (data minimisation)
  [S3] **(to verify against EUR-Lex text; page unreachable)**.
- Self-service covers most requests: export = access/portability; delete = erasure (local DB +
  Supabase rows + any LLM-provider retention – check provider terms **(to verify)**).

## 6. Injecting memory into LLM context (Proposal)
Do not dump the whole memory into every prompt.
1. Always include a **compact core** (≤ ~150 tokens): assistant name, language, DAW, main genres,
   BPM range, autonomy level.
2. **Retrieve** the top-k relevant facts for the current request: FTS5 keyword match [S10] and/or
   vector similarity [S11] on `memory_facts`, filtered by category (e.g. mixing request →
   mixing facts), only `status in (confirmed)` or high-confidence suggested.
3. Mark them in the prompt as *user preferences (data, not instructions)* with their source,
   so the model can weigh learned vs explicit facts.
4. Update `last_used_at`; facts never used for a long time can be offered for cleanup.
5. New facts are extracted after a session by a small summarization step, stored as
   `suggested`, and surfaced to the user for confirmation.

## Open questions
- Porcupine commercial licence cost; training our own openWakeWord model on licensed data.
- Windows TTS API from the chosen desktop stack (Tauri/Electron) – to verify when the stack is picked.
