# Prompt Engine – known limitations (Prompt 10)

2026-11-27. What the engine does not do, or does only partly. It declares these limits in its output (issues,
ambiguities, `operation_available`) instead of guessing.

## Understanding
1. **Rules, not a language model.**
   - Understanding comes from IT/EN vocabularies, patterns and the meaning of the whole request (constraints masked,
     comparatives, creation vs change, search vs sound design). It covers the requests of Prompt 10 and the
     producer vocabulary of the knowledge base.
   - Free paraphrases far from that vocabulary ("dagli più cattiveria nella parte bassa"), irony, long stories and
     unusual word orders can be missed. Then the engine says it did not understand and proposes an example; it does
     not guess.
   - A language model can be connected through `ModelAdapter` (see 6) to improve wording and inferred details, always
     checked by the rules.
2. **References to the project depend on the context received.** COSOUND does not read the DAW project yet (DAW
   connectors, milestone M4). Tracks, selection and plug-ins come from the Central Agent or the caller; without
   them, "il basso" is a part, not a specific track, and "questo elemento" is resolved only from the conversation.
3. **One question at a time.** When several things are unclear, the first essential one is asked.
4. **Artist references are kept as said** ("nello stile di …"): never looked up, never completed.

## Languages and voice
5. **Italian and English only.** Spanish, French, German and Portuguese are recognised and answered with
   "language not supported yet"; other languages may be read as one of the two with low confidence.
6. **Voice:** the engine receives transcripts (`source: "voice"`, `transcript_confidence`); it does not record or
   transcribe. Speech transcription belongs to the Voice-to-Music engine (milestone M5), not built yet, so in the app
   the chat is typed. A transcript with confidence below 0.5 is confirmed with the user before anything else.

## Models
7. **No model is connected.** The adapter interface, templates, validation, fallback and budgets are implemented and
   tested with a scripted test model; real local or cloud adapters (HTTP client, credentials in the OS keychain) are
   to be developed. The Settings page shows that understanding runs on COSOUND's rules.

## Execution
8. **The engine executes nothing**, by design. Several suggested operations are still planned in the agents
   (for example `mixing_engineer.apply_moves`, `mastering_engineer.master_track`, DAW `execute_plan`): the
   interpretation marks them `operation_available: false` and tells the user.
9. **The MOCK Central Agent** runs only four safe operations from an interpretation, as a simulation. Real planning
   and coordination are the partner's Central Agent (*in attesa dell'Agent*).

## Replies
10. **Replies are templates**, in the chosen style and level of detail. They are precise but not conversational
    prose; free conversation is the Central Agent's job.
11. Some labels in the interpretation details are internal ids (for example `delete_tracks`).
