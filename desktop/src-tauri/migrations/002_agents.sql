-- Prompt 06: activity log of agent tasks and the Music Memory store.
-- Shapes follow docs/architecture/DATABASE.md §2.4 (ail_tasks) and §2.5 (memory_facts).

create table ail_tasks (
  task_id      text primary key,
  request_id   text not null,
  trace_id     text,
  session_id   text,
  project_id   text,
  agent        text not null,
  operation    text not null,
  capability   text not null,          -- agent.operation
  requested_by text,                   -- central_agent | user | mock | test
  request      text not null,          -- TaskRequest (JSON)
  status       text not null,          -- PENDING | RUNNING | WAITING_APPROVAL | COMPLETED | FAILED | CANCELLED
  result       text,                   -- TaskResult (JSON)
  error        text,
  created_at   text not null default (datetime('now')),
  updated_at   text not null default (datetime('now'))
);
create index ail_tasks_created on ail_tasks(created_at);

-- The single local memory of COSOUND (no second memory exists). Adaptive Workflow uses category 'workflow'.
create table memory_facts (
  id          text primary key,
  category    text not null,           -- workflow | genre | bpm | library | plugin | style | mix | habit | assistant
  key         text not null,
  value       text not null,           -- JSON
  origin      text not null check (origin in ('explicit', 'learned')),
  scope       text not null default 'custom' check (scope in ('standard', 'custom')),
  confidence  real not null default 1.0,
  source_ref  text,
  sync        integer not null default 0, -- never uploaded unless the user opts in (not implemented)
  created_at  text not null default (datetime('now')),
  updated_at  text not null default (datetime('now'))
);
create index memory_facts_category on memory_facts(category, key);
