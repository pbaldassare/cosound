# PromptInterpretation – schema 1.0

JSON Schema: `packages/contracts/prompt-interpretation.schema.json` (`schema: "cosound.prompt-interpretation"`,
`schema_version: "1.0"`). A real output of the engine: [example-interpretation.json](example-interpretation.json)
(the second turn of "Crea un basso profondo per il drop" → "Rendilo più aggressivo, ma non toccare la melodia", with
five tracks in the context).

Field names, ids and values are in English whatever language the user speaks. Only `goal.interpretation`,
`expected_output.description`, issue messages, the clarification question and `reply.text` are in the user's language.
The schema is independent of any AI model: the deterministic rules and any future model produce the same structure,
and every output is validated before it leaves the engine.

## Fields

| Field | Meaning |
|---|---|
| `request_id`, `session_id`, `created_at` | Identity of the turn; the session groups a conversation |
| `user` | `id` (`local` unless the Central Agent provides one), detected `language` |
| `project` | Open project (id, name, DAW) if known |
| `input` | The user's words **unchanged**, `source` text/voice, `language`, `transcript_confidence`, `external_content[]` (always `treated_as: "data"`, with warnings) |
| `conversation` | `act`: new_request, modification, correction, undo, restore, confirm, cancel, question, smalltalk; `refers_to` the earlier request; `turn` |
| `intents[]`, `primary_intent` | One or more of 19 intents with confidence and evidence words |
| `goal` | Machine id (e.g. `increase_aggressiveness`, `lower_level`, `compose`) and the interpretation in plain words |
| `targets[]` | Elements (bass, kick, pad, melody, drop…), kind (part, track, instrument, section, mix, project…), the track id/name when resolved, `resolved_by` explicit / context / selection / conversation / new, confidence |
| `parameters.explicit[]` | What the user said: group (general, rhythm, harmony, sound, structure, technical), name, value, unit, **evidence** (the user's words), confidence |
| `parameters.inferred[]` | What was deduced, with its source: `context` (project, earlier turn), `memory` (saved preference), `inferred` (a model, checked) |
| `constraints[]` | preserve / only / exclude / resource_restriction / no_delete / no_overwrite, subject, the user's words, scope request / session / permanent, source user / conversation / memory |
| `resources[]` | Library, plug-ins, DAW, files, references needed, and whether they are available |
| `expected_output` | Kind (midi_clip, modified_sound, mix_change, search_results, analysis_report, answer…) and description |
| `instructions[]` | Per competent component: `agent` (registry id), purpose, inputs, constraints (`kind:subject`), `suggested_operation` (only if it exists), `operation_available`, `requires_approval`, `destructive`. **Not a plan**: no order, nothing executed |
| `enhancement` | `user_words`, `semantic` interpretation, `suggested_technical[]` (aspect, direction, why) and `authorized_operations[]` (only what the user's words ask for) |
| `approval` | `required`, reasons, `destructive_operations[]` (kind, subject, reversible) |
| `ambiguities[]` | kind (missing_optional, missing_recoverable, missing_essential, unresolved_reference, contradiction, low_transcript_confidence), what about, severity (default_used, resolved_from_context, ask), options |
| `clarification` | `needed`, one short `question`, `options` (for example the tracks that could be meant) |
| `issues[]` | code (not_understood, incompatible_parameters, contradictory, unresolved_reference, missing_resource, unsupported_operation, impossible, language_not_supported, unsafe_input, external_instructions_ignored, input_too_long), severity, message, **suggestion** |
| `defaults_used[]` | Values COSOUND chose because they were not essential, with why |
| `confidence` | 0–1 |
| `interpreter` | `adapter` (`rules-1.0` or a model id), `template`, `fallback`, and the reason or rejected model values |
| `context` | Summary of the context used (counts, ids; no file contents) |
| `reply` | What the assistant says, in the user's language and chosen style |

## Rules a consumer can rely on
1. If `clarification.needed` is true, `instructions` is empty: ask the question, do not act.
2. If an issue has `severity: "error"`, `instructions` is empty.
3. Every instruction carries all constraints; none may be relaxed to improve a result.
4. `destructive: true` or `requires_approval: true` → the user approves in COSOUND before execution (Safety Gate).
5. `parameters.explicit` never contains a value the user did not say; deduced values are in `inferred` with their source.
6. `operation_available: false` means the operation is planned, not usable today.

## Versioning
- `schema_version` follows the contract: additive optional fields keep `1.x`; renamed or removed fields mean `2.0`,
  and the engine will answer both versions during a transition.
- Templates have their own version (`interpret.v1.json`, `version` 1.0.0, changelog).
- The Agent API returns the schema with `prompt.schema`.
