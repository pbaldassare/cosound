-- Prompt 09: Music Intelligence Engine. Compositions and their versions (every MIDI edit is a new version: compare
-- and undo). Musical preferences are NOT stored here: they live in the shared memory (memory_facts).
create table music_clips (
  id         text primary key,
  title      text not null,
  request    text,
  key_name   text not null,
  bpm        real not null,
  bars       integer not null,
  style      text,
  timeline   text,                       -- JSON: chord symbols with start/len ticks
  report     text,                       -- JSON: the composition report (progression, validation …)
  created_at text not null default (datetime('now')),
  updated_at text not null default (datetime('now'))
);

create table music_versions (
  clip_id     text not null references music_clips(id) on delete cascade,
  version     integer not null,
  operation   text not null,             -- compose | edit | variation | import
  description text,
  key_name    text not null,
  tracks      text not null,             -- JSON: [{name, channel, program, notes, cc}]
  created_at  text not null default (datetime('now')),
  primary key (clip_id, version)
);
