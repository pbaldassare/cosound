-- Prompt 08: Plugin Intelligence System. Only plug-ins actually detected on this computer are stored.

-- One row per product: the same instrument in AU, VST3 and CLAP is one product with three installs.
create table plugin_products (
  id            text primary key,          -- normalized "vendor/name"
  name          text not null,
  vendor        text,
  kind          text not null,             -- instrument | effect | midi_effect | generator | other
  available     integer not null default 1,-- 1 when at least one install is present and usable
  first_seen    text not null default (datetime('now')),
  last_verified text
);

create table plugin_installs (
  id            text primary key,          -- format + path (or AU codes for units without a bundle path)
  product_id    text not null references plugin_products(id) on delete cascade,
  format        text not null,             -- AU | AUv3 | VST3 | VST2 | CLAP | AAX
  name          text not null,
  vendor        text,
  version       text,
  path          text,
  scope         text not null,             -- system | user | apple | app_extension
  os            text not null,             -- macos | windows | linux
  archs         text,                      -- JSON array: arm64, x86_64 …
  au_code       text,                      -- "type:subtype:manufacturer" for Audio Units
  bundle_id     text,
  registered    integer,                   -- AU: 1 when macOS lists it (component registry)
  status        text not null,             -- available | missing | incompatible | error
  status_reason text,
  details       text,                      -- JSON: tags, VST3 sub-categories, SDK, flags
  last_verified text not null default (datetime('now')),
  missing_since text
);
create index plugin_installs_product on plugin_installs(product_id);

create table plugin_categories (
  product_id text not null references plugin_products(id) on delete cascade,
  category   text not null,
  source     text not null,                -- au_tags | vst3_categories | au_type | name | knowledge | user
  confidence real not null,
  primary key (product_id, category)
);

-- Parameters read from the plug-in itself (isolated host), per install and plug-in version.
create table plugin_params (
  install_id     text not null references plugin_installs(id) on delete cascade,
  address        integer not null,
  identifier     text,
  name           text not null,
  unit           text,
  min_value      real,
  max_value      real,
  default_value  real,
  writable       integer not null,
  role           text,                     -- cutoff, resonance, drive, attack … (from the name; null = not mapped)
  plugin_version text,
  discovered_at  text not null default (datetime('now')),
  primary key (install_id, address)
);

-- Presets found for plug-ins: factory presets reported by the plug-in, and files in the standard preset folders.
-- Presets in the user's authorized library folders stay in the Smart Library (files/file_tags) and are not copied here.
create table plugin_presets (
  id          integer primary key,
  product_id  text not null references plugin_products(id) on delete cascade,
  install_id  text,
  source      text not null,               -- factory | preset_folder | cosound
  name        text not null,
  number      integer,                     -- factory preset number
  path        text,
  format      text,                        -- aupreset | vstpreset | fxp …
  category    text,
  readable    integer not null default 0,
  details     text,                        -- JSON: what was read, timbre analysis
  last_seen   text not null default (datetime('now')),
  unique (product_id, source, name, path)
);

-- Diagnostics: load failures, timeouts, crashes of the isolated host.
create table plugin_errors (
  id         integer primary key,
  install_id text,
  operation  text not null,
  error      text not null,
  at         text not null default (datetime('now'))
);

-- Sound designs and effect chains prepared by COSOUND (proposals, auditions, saved presets).
create table plugin_designs (
  id          text primary key,
  kind        text not null,               -- sound | chain
  title       text not null,
  request     text,
  spec        text not null,               -- JSON render spec (instrument, chain, parameter values)
  base_spec   text,                        -- JSON spec before the proposed changes (for before/after)
  changes     text,                        -- JSON list of parameter changes with reasons
  status      text not null,               -- proposed | auditioned | saved
  files       text,                        -- JSON list of files written by COSOUND for this design
  created_at  text not null default (datetime('now')),
  updated_at  text not null default (datetime('now'))
);
