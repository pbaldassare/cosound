# Integrating the partner's central Agent with the COSOUND agents

For the project partner. COSOUND does **not** build a central Agent: this guide explains how yours connects to the specialized
agents without changing them and without us changing your code.

## 1. What exists on our side
- 13 specialized agents (`docs/agents/AGENTS.md`), each declared in `packages/agents/<id>.json` with operations, JSON Schema inputs,
  permissions and limits. 36 operations run today; 17 are declared as planned and answer `NOT_IMPLEMENTED`.
- An agent runtime that executes the tasks it receives and nothing else: validation, permissions, user approvals, sync/async
  execution, cancellation, status, events, activity log. It does not plan, choose agents or merge results: that is your Agent's job.
- One entry point, the **Agent API**: `Runtime::handle(json) -> json` (`docs/agents/AGENT-PROTOCOL.md`, schemas in `packages/contracts/`).
- An adapter slot in the app: the `AgentPort` trait in `desktop/src-tauri/src/ail/mod.rs`. The UI sends user turns to the active port;
  the port talks to the agents only through the Agent API.
- A temporary **MOCK** port (`ail/temp_test_connector.rs`, development builds only) that shows the whole path with explicit commands
  and one fixed scenario. It is a simulation and will be deleted.

## 2. What your Agent can do through the API
| Need | Message |
|---|---|
| Discover agents, competencies, operations, input schemas, limits | `registry.get` |
| Assign a task / send data | `task.submit` with a TaskRequest (`async: true` for long work) |
| Receive results | the `task.submit` answer, or `task.status` / `events.poll` for async tasks |
| Check status of operations | `task.status`, `task.list`, `events.poll` |
| Handle errors | `errors[].code` (table in AGENT-PROTOCOL §6), `limitations[]`, `retryable` |
| Handle approvals | `approval.list`; tell the user; the user approves in the app; you may reject or cancel |
| Stop work | `task.cancel` |
| Plan the next step | read `needs[]` (suggestions from an agent) and `proposed_daw_ops[]`; you decide |
| Understand a user turn (text or voice transcript) | `prompt.interpret`: intents, targets, parameters said/deduced, constraints, ambiguities, approval needs and instructions per component. Every turn typed in the app arrives at your port already interpreted (`AgentPort::send_interpreted`). Details: `docs/prompt-engine/` |

## 3. Writing the adapter (only new code on our side)
1. Analyse your Agent (language, runtime, tool-calling mechanism, memory, multi-agent support) and record it in
   `docs/architecture/AGENT-ANALYSIS.md`. Resolve decisions D-1…D-6 (`AGENT-INTEGRATION-LAYER.md` §8) in `DECISIONS.md`.
2. Implement `AgentPort` for your Agent in `desktop/src-tauri/src/ail/` (e.g. `partner_agent.rs`):
   - `send(text, api)`: forward the user turn to your Agent; when your Agent wants a tool, convert its call into a `task.submit`
     message and pass the TaskResult back; return `AgentEvent::AssistantText` for its replies.
   - `health()`: `is_temporary: false`, `connected` from the real link.
3. Map the catalog into your Agent's tool format. Suggested mapping (works for MCP and for function-calling APIs):
   tool name `<agent>__<operation>` (e.g. `music_theory__generate_bassline`), description `summary.en`, parameters = the operation's
   `input` schema; skip `planned` operations or expose them as unavailable.
4. Make `default_port()` return your port, remove the `temp-connector` feature, the MOCK file and `agents/scenario.rs`.
5. Run the contract tests (`cargo test`) and the end-to-end checks of §5.

No agent changes: agents only see TaskRequests. This is tested: `agents::tests::mock_is_replaceable_by_a_json_connector` runs the same
scenario in-process and through JSON messages only, with identical results.

## 4. Rules your Agent must respect (enforced on our side anyway)
- Medium/high-risk operations need the **user's** approval: the API refuses `approve` from the Agent.
- Agents never call each other: route `needs[]` yourself.
- Values carry their `basis`: do not present `estimated`, `convention`, `web` or `file_name` values as measurements.
- Files are referenced by path and SHA-256 (`files[]`), never sent inline except small MIDI as base64 in results.
- Memory: COSOUND's Music Memory (`memory_facts`) is the local store; the Adaptive Workflow agent is the way to read/write workflow
  items, and only when the user enabled it (decision D-4 still to confirm with you).

## 5. Acceptance checks after integration
1. `registry.get` lists 13 agents and their operations.
2. One task per agent through your Agent (the list of 36 inputs used in `agents::tests::every_available_operation_runs` is a ready set).
3. The six-step collaboration ("Melodic Techno at 124 BPM"): your Agent plans and assigns the steps itself; the MOCK script is the reference
   of expected outputs (`agents/scenario.rs`).
4. Error paths: unknown operation, invalid input, planned operation, disabled agent, permission off.
5. Approval: `adaptive_workflow.clear` waits; your Agent cannot approve; the user approves in the app.
6. Cancellation of an async `audio_analysis.analyze_file` on a long WAV.
