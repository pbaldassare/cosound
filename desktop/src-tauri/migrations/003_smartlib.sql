-- Prompt 07: Smart Library System. Extends `files` (DATABASE.md §2.1, specs/SMART-LIBRARY.md §5).
-- File rows are now updated in place (stable ids), so labels and user corrections survive rescans.

alter table files add column status text not null default 'present';   -- present | modified | moved | missing
alter table files add column missing_since text;
alter table files add column moved_from text;
alter table files add column quick_hash text;                          -- SHA-256 of size + first/last 64 KiB
alter table files add column full_hash text;                           -- SHA-256 of the whole file (computed for duplicate checks)
alter table files add column audio_hash text;                          -- SHA-256 of the decoded samples (same audio, any container)
alter table files add column indexed_at text;
alter table files add column analysis_state text not null default 'pending'; -- pending | done | error
alter table files add column analysis_version integer not null default 0;
alter table files add column analysis_error text;
alter table files add column analyzed_at text;
alter table files add column format text;
alter table files add column duration_s real;
alter table files add column sample_rate integer;
alter table files add column channels integer;
alter table files add column bit_depth integer;
alter table files add column peak_dbfs real;
alter table files add column rms_dbfs real;
alter table files add column lufs real;
alter table files add column bpm real;
alter table files add column bpm_src text;
alter table files add column bpm_conf real;
alter table files add column musical_key text;
alter table files add column key_src text;
alter table files add column key_conf real;
alter table files add column is_loop integer;
alter table files add column loop_bars real;
alter table files add column category text;
alter table files add column details text;                             -- JSON: features, metadata, MIDI or preset data
alter table files add column vec text;                                 -- JSON array: timbre vector for similarity
create index files_status on files(status);
create index files_category on files(category);
create index files_bpm on files(bpm);
create index files_size on files(size);
create index files_full_hash on files(full_hash);
create index files_audio_hash on files(audio_hash);
create index files_quick_hash on files(quick_hash);
create index files_analysis on files(analysis_state);

create table file_tags (
  file_id    integer not null references files(id) on delete cascade,
  tag        text not null,
  kind       text not null,                -- category | type | genre | descriptor | plugin
  source     text not null,                -- filename | folder | metadata | analysis | midi | preset | user
  confidence real not null default 1.0,
  primary key (file_id, tag, kind)
);
create index file_tags_tag on file_tags(tag);

-- User corrections, re-applied after every analysis: bpm, key, add:<kind>:<tag>, remove:<kind>:<tag>
create table file_corrections (
  file_id    integer not null references files(id) on delete cascade,
  field      text not null,
  value      text,
  updated_at text not null default (datetime('now')),
  primary key (file_id, field)
);

create table library_exclusions (
  root_id  text not null references library_roots(id) on delete cascade,
  rel_path text not null,                  -- folder inside the root that is never scanned
  primary key (root_id, rel_path)
);
alter table volumes add column excluded integer not null default 0;

create table scan_jobs (
  id            text primary key,
  root_id       text not null,
  kind          text not null,             -- incremental | full
  state         text not null,             -- running | done | cancelled | offline | unavailable | error | interrupted
  files_seen    integer not null default 0,
  files_new     integer not null default 0,
  files_changed integer not null default 0,
  files_moved   integer not null default 0,
  files_missing integer not null default 0,
  files_back    integer not null default 0,
  started_at    text not null default (datetime('now')),
  finished_at   text,
  error         text
);
