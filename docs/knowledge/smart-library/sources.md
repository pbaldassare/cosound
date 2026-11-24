# Sources – smart-library

All retrieved 2026-11-24. Apple pages were read through the developer.apple.com documentation JSON
of the same URL; GitHub licenses were read from the repository LICENSE/README and the GitHub API.

| ID | Source | URL | Retrieved | Software version | Reliability | Limitations |
|---|---|---|---|---|---|---|
| S1 | Apple – `URLResourceKey.volumeUUIDStringKey` | https://developer.apple.com/documentation/foundation/urlresourcekey/volumeuuidstringkey | 2026-11-24 | Foundation, current docs | A | Says only "nil if a persistent UUID is not available"; does not list which file systems lack one |
| S2 | Apple – Disk Arbitration framework | https://developer.apple.com/documentation/diskarbitration | 2026-11-24 | current docs | A | Overview page only; disk-description keys not checked individually |
| S3 | Apple – `NSWorkspace.didMountNotification` | https://developer.apple.com/documentation/appkit/nsworkspace/didmountnotification | 2026-11-24 | AppKit, current docs | A | – |
| S4 | Apple – `NSWorkspace.didUnmountNotification` | https://developer.apple.com/documentation/appkit/nsworkspace/didunmountnotification | 2026-11-24 | AppKit, current docs | A | – |
| S5 | Apple – File System Events (CoreServices) | https://developer.apple.com/documentation/coreservices/file_system_events | 2026-11-24 | current docs | A | Overview only |
| S6 | Apple – FSEvents Programming Guide, "Using the File System Events API" | https://developer.apple.com/library/archive/documentation/Darwin/Conceptual/FSEvents_ProgGuide/UsingtheFSEventsFramework/UsingtheFSEventsFramework.html | 2026-11-24 | archive (not updated recently) | A | Archived guide; behaviour on network/exFAT volumes not stated |
| S7 | Apple – FSEvents Programming Guide, "File System Event Security" | https://developer.apple.com/library/archive/documentation/Darwin/Conceptual/FSEvents_ProgGuide/FileSystemEventSecurity/FileSystemEventSecurity.html | 2026-11-24 | archive | A | Archived |
| S8 | Microsoft – Naming a Volume | https://learn.microsoft.com/en-us/windows/win32/fileio/naming-a-volume | 2026-11-24 | Win32, current | A | – |
| S9 | Microsoft – `GetVolumeInformationW` | https://learn.microsoft.com/en-us/windows/win32/api/fileapi/nf-fileapi-getvolumeinformationw | 2026-11-24 | Win32, current | A | – |
| S10 | Microsoft – `WM_DEVICECHANGE` | https://learn.microsoft.com/en-us/windows/win32/devio/wm-devicechange | 2026-11-24 | Win32, current | A | – |
| S11 | Microsoft – `RegisterDeviceNotificationW` | https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-registerdevicenotificationw | 2026-11-24 | Win32, current | A | – |
| S12 | Microsoft – `DEV_BROADCAST_VOLUME` | https://learn.microsoft.com/en-us/windows/win32/api/dbt/ns-dbt-dev_broadcast_volume | 2026-11-24 | Win32, current | A | – |
| S13 | Microsoft – `ReadDirectoryChangesW` | https://learn.microsoft.com/en-us/windows/win32/api/winbase/nf-winbase-readdirectorychangesw | 2026-11-24 | Win32, current | A | – |
| S14 | Microsoft – Change Journals | https://learn.microsoft.com/en-us/windows/win32/fileio/change-journals | 2026-11-24 | Win32, current | A | – |
| S15 | Microsoft – `FSCTL_QUERY_USN_JOURNAL` | https://learn.microsoft.com/en-us/windows/win32/api/winioctl/ni-winioctl-fsctl_query_usn_journal | 2026-11-24 | Win32, current | A | ReFS support not confirmed on this page |
| S16 | Microsoft – Creating, Modifying, and Deleting a Change Journal | https://learn.microsoft.com/en-us/windows/win32/fileio/creating-modifying-and-deleting-a-change-journal | 2026-11-24 | Win32, current | A | Exact privilege needed to *read* the journal not confirmed |
| S17 | notify-rs/notify – README | https://github.com/notify-rs/notify | 2026-11-24 | `main` (docs reference notify 9.0.0) | B | – |
| S18 | notify-rs/notify – crate docs "Known Problems" (`notify/src/lib.rs`) | https://github.com/notify-rs/notify/blob/main/notify/src/lib.rs | 2026-11-24 | `main` | B | – |
| S19 | Cyan4973/xxHash – README and LICENSE | https://github.com/Cyan4973/xxHash | 2026-11-24 | `dev` branch | B | Library BSD-2-Clause; CLI tool has a separate licence (not checked) |
| S20 | BLAKE3-team/BLAKE3 – README | https://github.com/BLAKE3-team/BLAKE3 | 2026-11-24 | `master` | B | – |
| S21 | libsndfile – `docs/command.md` + repo licence | https://github.com/libsndfile/libsndfile/blob/master/docs/command.md | 2026-11-24 | `master` | B | Licence (LGPL-2.1) from GitHub metadata |
| S22 | pdeljanov/Symphonia – README | https://github.com/pdeljanov/Symphonia | 2026-11-24 | `main` | B | – |
| S23 | FFmpeg – License and Legal Considerations | https://ffmpeg.org/legal.html | 2026-11-24 | current | B | "Not legal advice" per the page |
| S24 | FFmpeg – ffprobe documentation | https://ffmpeg.org/ffprobe.html | 2026-11-24 | current | B | – |
| S25 | Serial-ATA/lofty-rs – README | https://github.com/Serial-ATA/lofty-rs | 2026-11-24 | `main` | B | Licence from GitHub metadata (Apache-2.0); dual MIT not checked |
| S26 | SQLite – FTS5 Extension | https://www.sqlite.org/fts5.html | 2026-11-24 | current SQLite docs | A | – |
| S27 | asg017/sqlite-vec – README + LICENSE files | https://github.com/asg017/sqlite-vec | 2026-11-24 | `main` (pre-v1) | B | Explicitly pre-v1, breaking changes expected |
| S28 | LAION-AI/CLAP – README; HF model card `lukewys/laion_clap` | https://github.com/LAION-AI/CLAP ; https://huggingface.co/lukewys/laion_clap | 2026-11-24 | `main`; HF metadata | B | Licence of the training data itself not assessed |
| S29 | qiuqiangkong/audioset_tagging_cnn (PANNs) – README; Zenodo weights record 3987831 | https://github.com/qiuqiangkong/audioset_tagging_cnn ; https://zenodo.org/records/3987831 | 2026-11-24 | repo last push 2024-07 | B | Code MIT (GitHub); weights CC-BY-4.0 (Zenodo metadata) |
| S30 | tensorflow/models – YAMNet README | https://github.com/tensorflow/models/tree/master/research/audioset/yamnet | 2026-11-24 | `master` | B | Repo licence reported as NOASSERTION by GitHub API; Apache-2.0 (to verify) |
| S31 | Steinberg VST 3 Developer Portal – Preset Format | https://steinbergmedia.github.io/vst3_dev_portal/pages/Technical+Documentation/Locations+Format/Preset+Format.html | 2026-11-24 | VST 3 SDK docs | A | – |
| S32 | Steinberg VST 3 Developer Portal – Preset Locations | https://steinbergmedia.github.io/vst3_dev_portal/pages/Technical+Documentation/Locations+Format/Preset+Locations.html | 2026-11-24 | VST 3 SDK docs | A | – |
| S33 | Splice Help Center – "Where is my Splice folder?" | https://support.splice.com/en/articles/8652662-where-is-my-splice-folder | 2026-11-24 | Splice desktop app, current article | A | Windows path looks dated; inner folder layout not documented |
| S34 | Native Instruments Support – Native Access install/content locations (articles 211577305, 69000879242) | https://support.native-instruments.com/hc/articles/211577305 | 2026-11-24 | Native Access 2 | U | Page returned 404 to our fetch; facts taken from search snippets only (to verify) |
| S35 | Apple – `NSRemovableVolumesUsageDescription` | https://developer.apple.com/documentation/bundleresources/information-property-list/nsremovablevolumesusagedescription | 2026-11-24 | current docs | A | – |
| S36 | Apple – `NSNetworkVolumesUsageDescription` | https://developer.apple.com/documentation/bundleresources/information-property-list/nsnetworkvolumesusagedescription | 2026-11-24 | current docs | A | – |
| S37 | Apple – `NSURL.BookmarkCreationOptions.withSecurityScope` | https://developer.apple.com/documentation/foundation/nsurl/bookmarkcreationoptions/withsecurityscope | 2026-11-24 | current docs | A | – |
| S38 | Apple Platform Security – Controlling app access to files in macOS | https://support.apple.com/guide/security/controlling-app-access-to-files-secddd1d86a6/web | 2026-11-24 | current guide | A | – |
| S39 | Apple Loops metadata (CAF/AIFF tags: tempo, key, beats, category) | – | 2026-11-24 | – | U | No public Apple specification found (to verify) |
| S40 | ACID chunk in WAV (loop tempo/root note/beats) | – | 2026-11-24 | – | U | No official public spec found; reading supported via libsndfile [S21] |
| S41 | GitHub licence metadata: MTG/essentia (AGPL-3.0), librosa (ISC), aubio (GPL-3.0), Spotify/basic-pitch (Apache-2.0) | https://github.com/MTG/essentia ; https://github.com/librosa/librosa ; https://github.com/aubio/aubio ; https://github.com/spotify/basic-pitch | 2026-11-24 | default branches | B | Licence fields only, not full licence review |
| S42 | Proprietary preset formats: Serum (`.fxp`/`.SerumPreset`), Vital (`.vital`), NKS (`.nksf`), Audio Unit (`.aupreset`) | – | 2026-11-24 | – | U | No official public format spec verified (to verify) |
