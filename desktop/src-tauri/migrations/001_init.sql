-- COSOUND local database, schema v1 (subset of docs/architecture/DATABASE.md §2 used by the desktop app today).
-- Original files are never copied: every table stores references (paths + hashes).

create table settings (
  key   text primary key,
  value text not null
);

create table volumes (
  id           text primary key,
  fingerprint  text not null unique,   -- see library.rs `fingerprint`
  label        text not null,
  kind         text not null,          -- internal | external | network | folder
  disk_type    text,                   -- SSD | HDD | unknown
  fs_type      text,
  mount_point  text,                   -- last known mount point
  total_bytes  integer,
  online       integer not null default 0,
  last_seen_at text
);

create table library_roots (
  id           text primary key,
  volume_id    text not null references volumes(id),
  rel_path     text not null,          -- relative to the volume mount point
  scan_state   text not null default 'idle',  -- idle | scanning | done | offline | error
  file_count   integer not null default 0,
  last_scan_at text,
  error        text,
  created_at   text not null default (datetime('now')),
  unique (volume_id, rel_path)
);

create table files (
  id       integer primary key,
  root_id  text not null references library_roots(id) on delete cascade,
  rel_path text not null,              -- relative to the root
  name     text not null,
  ext      text not null,
  kind     text not null,              -- audio | midi | project | preset
  size     integer not null,
  mtime    integer,
  unique (root_id, rel_path)
);
create index files_name on files(name);

create table projects (
  id          text primary key,
  name        text not null,
  daw         text,
  bpm         real,
  musical_key text,
  notes       text not null default '',
  created_at  text not null default (datetime('now')),
  updated_at  text not null default (datetime('now')),
  opened_at   text,
  archived_at text
);

-- Libraries a project uses (links, not copies).
create table project_roots (
  project_id text not null references projects(id) on delete cascade,
  root_id    text not null references library_roots(id) on delete cascade,
  primary key (project_id, root_id)
);

-- Imported material (audio, MIDI, stems, loops, voice takes, DAW projects): references only.
create table imports (
  id          text primary key,
  project_id  text references projects(id) on delete set null,
  path        text not null,
  name        text not null,
  kind        text not null,           -- audio | midi | project | preset | other
  role        text not null,           -- track | stem | loop | sample | voice | midi | project
  provenance  text not null,           -- own | suno | udio | other_ai | unknown
  size        integer not null,
  sha256      text not null,
  imported_at text not null default (datetime('now')),
  unique (project_id, path)
);

-- Register of important operations (DATABASE.md op_log, reduced to what the app does today).
create table op_log (
  id          integer primary key,
  ts          text not null default (datetime('now')),
  project_id  text,
  op          text not null,           -- e.g. project.rename, library.root_add
  risk        text not null,           -- read | low | medium | high
  params_json text not null default '{}',
  undo_json   text,                    -- present when the op can be undone
  undone_at   text
);
create index op_log_project on op_log(project_id, id);
