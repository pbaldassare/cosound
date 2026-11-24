# Smart Library (multidrive)

Section 10 of the knowledge base. Goal: find, catalog and index the user's music libraries
(samples, loops, one-shots, MIDI, presets, instrument libraries) across the internal disk,
external HDD/SSD, USB sticks, dedicated sample drives, custom folders and NAS/network shares,
and keep the index correct when drives come and go. Sources: `sources.md` in this folder.
Verified 2026-11-24. Anything marked **(to verify)** was not confirmed against a source.

Design notes (not sourced facts) are labelled **Proposal**.

## 1. Privacy principles (apply to everything below)
- **Scan only locations the user has explicitly added.** No silent whole-disk crawl.
- **The index stays local.** Library files and the index are never uploaded by default. Only
  short text/metadata the user chooses to send (e.g. "a 90 BPM kick list") may go to the LLM.
- On macOS, access to Desktop/Documents/Downloads, network volumes and removable volumes is
  gated by user consent, and full-storage access needs the user to add the app manually in
  System Settings [S38]. Accessing a file on a removable or network volume without implied
  consent (Open panel, drag-and-drop, Finder open) triggers a one-time system prompt, whose text
  comes from `NSRemovableVolumesUsageDescription` / `NSNetworkVolumesUsageDescription` [S35][S36].
- **Proposal:** let the user add folders through the system folder picker. This counts as implied
  consent [S35][S36] and, in a sandboxed build, yields a URL from which a security-scoped
  bookmark can be created so access survives relaunches [S37]. Do not ask for Full Disk Access.
  See `../security-permissions/` for the full permission matrix.

## 2. Detecting volumes and keeping a stable identity

### macOS
| Need | API | Notes |
|---|---|---|
| Mount / unmount events | `NSWorkspace.didMountNotification` / `didUnmountNotification` | Must be observed on NSWorkspace's own notification center; unmount is posted even when a drive is yanked out without ejecting; userInfo carries the mount path [S3][S4] |
| Lower-level disk events | Disk Arbitration framework | Can register for, and even block, mount/unmount [S2] |
| Stable volume ID | `URLResourceKey.volumeUUIDStringKey` | Persistent UUID, or **nil** when the volume has none [S1] |
| Device IDs | – | FSEvents device IDs may change across reboots, especially for removable devices; Apple says to compare the volume UUID instead [S6] |

### Windows
| Need | API | Notes |
|---|---|---|
| Arrival / removal | `WM_DEVICECHANGE` with `DBT_DEVICEARRIVAL`, `DBT_DEVICEREMOVECOMPLETE` … [S10]; `RegisterDeviceNotification` (window or service handle) for device-interface / handle notifications [S11] |
| Which drive changed | `DEV_BROADCAST_VOLUME.dbcv_unitmask` (bit 0 = A:, bit 1 = B: …); `DBTF_NET` flag marks a network volume, `DBTF_MEDIA` a media change [S12] |
| Stable volume ID | Volume GUID path `\\?\Volume{GUID}\` [S8] | Microsoft introduced it because drive letters change as volumes are added and removed; one volume can have **more than one** GUID path [S8] |
| Secondary ID | `GetVolumeInformationW` serial number | Assigned by the OS when the volume is formatted (not the hardware serial) [S9] |

**Drive letters are not identity.** Store paths as *(volume-id, path relative to volume root)*,
never as `E:\Samples\…` [S8].

**Proposal – volume fingerprint** (because no single ID is always present):
`volume_uuid` (macOS) or `volume_guid` + `serial` (Windows), plus label, file-system type and total
capacity as tie-breakers. Whether a Windows volume GUID stays the same when the same USB drive
is plugged into a *different* PC is **(to verify)**; the format serial travels with the volume [S9].
For network shares use the normalized server + share path (`smb://nas/Samples`, `\\nas\Samples`)
as the identity (no volume UUID guarantee) **(to verify)**.

## 3. Watching for changes

| Mechanism | Platform | Granularity / limits |
|---|---|---|
| FSEvents | macOS | Per-directory notifications, coalesced in time; when events collide or are dropped you get `kFSEventStreamEventFlagMustScanSubDirs` and must rescan that subtree [S6]. Can also answer "what changed since event ID X" across launches [S5]. Users only see events for directories they can reach [S7]. A volume with `.fseventsd/no_log` is not logged [S7] |
| `ReadDirectoryChangesW` | Windows | If the buffer overflows, the call still succeeds but all queued changes are discarded; fails with `ERROR_NOTIFY_ENUM_DIR` when the system couldn't record all changes (re-enumerate); over the network, buffers > 64 KB fail; fails with `ERROR_INVALID_FUNCTION` if the redirector/file system doesn't support it [S13] |
| USN change journal | Windows | Per-volume persistent log of changes kept by NTFS, so an indexer can catch up without a full rescan [S14]. `FSCTL_QUERY_USN_JOURNAL` requires an NTFS volume and is not supported over SMB 3.0 [S15]. Creating/deleting a journal is an administrator task [S16]; whether a normal user can *read* it is **(to verify)**. Not available on exFAT/FAT sample drives [S15] |
| `notify` crate (Rust/Tauri) | both | Wraps FSEvents (macOS) and ReadDirectoryChangesW (Windows), with a polling fallback; licence CC0-1.0 (debouncers MIT/Apache-2.0) [S17]. Its docs warn network filesystems such as NFS may emit no events; workaround is `PollWatcher` (default interval 30 s), which can miss changes reverted between scans [S18] |

**Rules for COSOUND (Proposal):**
1. Live watching is a hint, never the truth. On every mount, launch, overflow or "must scan"
   flag, run an incremental rescan of that root.
2. NAS/SMB shares: poll (e.g. every few minutes, user-configurable) or rescan on demand, because
   native events are unreliable over the network [S13][S18]. Whether FSEvents reports changes
   made *by other machines* on an SMB share is **(to verify)** – assume not.
3. exFAT/FAT USB drives: no USN journal on Windows [S15]; whether FSEvents history persists on
   exFAT on macOS is **(to verify)**. Treat them as "rescan on mount".

## 4. Incremental indexing
- **Change detection (Proposal):** key = (volume-id, relative path). If size and mtime are
  unchanged, skip. Otherwise re-read metadata and re-hash.
- **Duplicate detection:** content hash. xxHash (XXH3 64/128-bit) is a very fast
  *non-cryptographic* hash, output stable across releases; library is BSD-2-Clause [S19]. BLAKE3 is a
  fast *cryptographic* hash, parallelizable, public domain (CC0) or Apache-2.0 [S20]. XXH3 is enough
  for dedup; use BLAKE3 only if collision resistance matters (e.g. sync).
- **Partial hashing (Proposal):** for files above a threshold (e.g. > 64 MB), hash size + first and
  last N MB + a middle block for a quick fingerprint; confirm full hash only when two quick
  fingerprints collide. Ceiling: false "same" is possible before confirmation.
- Run the scanner at low priority, batch DB writes, and make it resumable.

## 5. Metadata extraction

| Tool | What it gives | Licence | Notes |
|---|---|---|---|
| Symphonia (Rust) | Decode/demux AAC, ADPCM, AIFF, ALAC, CAF, FLAC, MKV, MP1-3, MP4, OGG, Vorbis, WAV, WebM; reads most tag formats | MPL-2.0 [S22] | Pure safe Rust, good fit for Tauri |
| libsndfile (C) | Format, channels, sample rate, frames; `SFC_GET_LOOP_INFO` returns loop info incl. time signature, length in beats and MIDI root note; `SFC_GET_INSTRUMENT` | LGPL-2.1 [S21] | Practical way to read ACID-style loop data [S40] |
| FFmpeg / ffprobe | Stream/format info for almost anything, JSON output (`-output_format json`) [S24] | LGPL-2.1+; GPL if optional GPL parts are enabled [S23] | Ship an LGPL build as a separate process to avoid GPL; heavy binary |
| lofty (Rust) | Tags: ID3v1/v2 (MP3, AAC, AIFF, WAV…), AIFF text chunks, Vorbis comments, MP4 ilst | Apache-2.0 (GitHub) [S25] | Tags only, no decoding |

Embedded musical metadata:
- **ACID chunk (WAV)**: loop tempo, root note, beats. No official public spec found [S40]; read via
  libsndfile loop info [S21].
- **Apple Loops** (AIFF/CAF with tempo, key, scale, category, descriptors): no public Apple spec
  found [S39] **(to verify)**. Treat as best-effort.
- **ID3/RIFF INFO**: often only title/artist/genre/BPM (`TBPM`) for full tracks [S25].

**BPM/key from filename (Proposal):** sample packs commonly embed tempo and key in names, e.g.
`Kick_01.wav`, `Loop_Dark_128bpm_Fm.wav`, `120_Am_Guitar.wav`. Regex candidates:
`(\d{2,3})\s?bpm`, a bare 2–3-digit number in 60–200 when the file is a loop, and keys like
`[A-G](#|b)?\s?(m|min|maj|minor|major)?`. Rank sources: embedded loop chunk > filename >
audio analysis, and store the source + confidence with each value. Common naming conventions are
observed practice, not a documented standard **(to verify per vendor)**.

**Audio analysis (later phase):** tempo/key estimators exist in librosa (ISC), aubio (GPL-3.0),
Essentia (AGPL-3.0) [S41] – licence matters for a closed-source desktop app: prefer
permissive (librosa ISC is Python; for native code a Rust/C implementation must be chosen and
checked).

## 6. Categorization
- **One-shot vs loop (Proposal heuristic):** loop if embedded loop info exists, or filename contains
  "loop"/BPM, or duration ≈ whole number of bars at the detected/declared tempo; one-shot if
  short (e.g. < ~2 s) with one main onset. Folder names (`Kicks`, `Loops`, `One Shots`) are a
  strong signal.
- **Instrument / sound class via embeddings:**

| Model | Use | Code licence | Weights licence |
|---|---|---|---|
| LAION CLAP | Joint audio–text embeddings → text-to-audio search ("dark plucky synth") and tagging; music+speech checkpoints exist; also in HF Transformers | CC0-1.0 [S28] | CC0-1.0 (HF card) [S28] |
| PANNs (CNN14 etc.) | AudioSet tagging, 527 classes | MIT [S29] | CC-BY-4.0 (Zenodo) – attribution required [S29] |
| YAMNet | AudioSet event classifier, 521 classes, MobileNet v1 | tensorflow/models (Apache-2.0 **to verify**) [S30] | weights download from Google storage, licence **(to verify)** [S30] |

AudioSet classes are general sound events, not producer categories (kick/snare/pad…): a small
local classifier on top of embeddings, trained on user-confirmed labels, is the likely path
**(Proposal)**. Training-data licences of these models were not assessed.

## 7. Other content types
- **MIDI files** (`.mid`): index name, tracks, length; parse with a MIDI library (see
  `../midi-intelligence/` when written).
- **VST3 presets**: `.vstpreset` binary with header id `VST3`, version, 32-byte ASCII class ID
  (identifies the plug-in) and a chunk list [S31]. Standard folders: macOS
  `~/Library/Audio/Presets/<Company>/<Plugin>/` (user) and `/Library/Audio/Presets/…` (shared);
  Windows `<Documents>/VST3 Presets/<Company>/<Plugin>/` (user) [S32]. The class ID lets us map
  presets to installed plug-ins without loading them.
- **`.fxp`/`.fxb`** (VST2 program/bank, also used by Serum 1), **`.SerumPreset`**, **`.vital`**,
  **`.nksf`** (NI NKS), **`.aupreset`** (Audio Unit, believed to be a property list): no official
  spec verified [S42] **(to verify)**. Index by filename, folder and extension; read internals
  only where a spec is confirmed.
- **Splice**: the desktop app creates a Splice folder on first download – macOS
  `/Users/<user>/Splice`, Windows listed as `C:\Documents\Splice`; user can move/rename it and must
  update the path in Splice preferences [S33]. Proposal: offer it as a suggested root, never
  assume it.
- **Native Instruments**: Native Access installs library content to a user-chosen "Content"
  location (default on macOS reportedly `/Users/Shared`); libraries are recognised by a folder
  containing a `.nicnt` file [S34] **(to verify)**.

## 8. Local index storage
- **SQLite** file in the app data folder. Text search with **FTS5** (virtual table; `unicode61`,
  `trigram` tokenizers for partial-name matches; `bm25()` ranking; external-content tables so the
  FTS index points at the main `files` table) [S26].
- **Similarity search:** `sqlite-vec` – `vec0` virtual tables storing float/int8/binary vectors,
  pure C, no dependencies, MIT or Apache-2.0; **pre-v1, breaking changes expected** [S27]. Pin the
  version and keep embeddings re-computable.
- **Proposal – minimal schema:**
  - `volumes(id, kind[internal|external|usb|network], fingerprint, label, last_seen_at, online)`
  - `roots(id, volume_id, rel_path, bookmark_blob, added_by_user_at, watch_mode[events|poll|manual])`
  - `files(id, volume_id, rel_path, size, mtime, quick_hash, full_hash, kind[audio|midi|preset|other], duration_ms, sample_rate, channels, bpm, bpm_src, key, key_src, is_loop, missing)`
  - `tags(file_id, tag, source[user|filename|embedded|model], confidence)`
  - `files_fts` (FTS5 over name, path, tags); `file_vec` (vec0 over CLAP embedding).

## 9. Offline / disconnected devices (Proposal)
- On unmount, mark the volume `online=0`; **keep** all its records and tags. Search results show
  them greyed with "drive offline: <label>".
- On mount, match the fingerprint (§2) → set online, run incremental rescan of its roots.
- If a fingerprint is unknown but many (size, quick_hash, rel_path) match an offline volume
  (e.g. cloned drive, changed UUID after reformat), propose a merge to the user instead of
  auto-merging.
- Files missing after a rescan are flagged `missing`, purged only after a grace period or user
  action (user tags are valuable).

## 10. Smart search (Proposal)
Combine: FTS5 text (name/path/tags) + structured filters (BPM range, key, loop/one-shot,
type, volume online) + optional audio similarity (sqlite-vec nearest neighbours on CLAP
embeddings, or CLAP text embedding for natural-language queries) [S26][S27][S28]. The LLM turns
the user request into filters + query text; it receives only result metadata, never audio.

## Open questions
- Volume GUID stability across different Windows PCs; FSEvents behaviour on SMB and exFAT.
- Apple Loops and preset internals (no public spec).
- Windows default paths for Splice and NI need checking on a current install.
