-- Prompt 10: Prompt Engine. One row per interpreted user turn: conversation continuity (follow-ups, corrections,
-- undo/restore refer to earlier turns) and traceability. Local only; the user can clear a session.
create table prompt_turns (
  request_id     text primary key,
  session_id     text not null,
  turn           integer not null,
  source         text not null,            -- text | voice
  language       text not null,
  act            text not null,            -- new_request | modification | correction | undo | restore | confirm | cancel | question | smalltalk
  primary_intent text,
  text           text not null,
  interpretation text not null,            -- JSON, packages/contracts/prompt-interpretation.schema.json
  created_at     text not null default (strftime('%Y-%m-%dT%H:%M:%SZ', 'now'))
);
create index prompt_turns_session on prompt_turns(session_id, turn);
