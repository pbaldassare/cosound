# Pro Tools – DAW knowledge

Research date: 2026-11-24. Current version: **Pro Tools 2026.11** (November 2026) [S1][S3].
macOS and Windows.

## Integration routes (ranked for COSOUND)

1. **Pro Tools Scripting SDK ("PTSL", Pro Tools Scripting Library)** – official. A free,
   language-independent API for Mac and Windows giving access to session open/close/save, exports,
   timeline operations, "get" commands to inspect clips and "set" commands for writeable actions [S6].
   Distributed from developer.avid.com behind an Avid account and a license agreement that must be
   accepted before download [S4][S5]. Transport is **gRPC + Protocol Buffers**; the client connects by
   default to `localhost:31416` and registers with company name + application name
   (`RegisterConnection`, available since Pro Tools 2023.3); older versions used a certificate
   (`AuthorizeConnection`) [S7]. Avid's press release says it works with SoundFlow and that it is an
   access layer outside Pro Tools [S6]. License terms (redistribution, commercial use) were **not read**
   → to verify before shipping.
   - Command set (from the 2025.10 proto, 159 command IDs) [S7] includes: CreateSession, OpenSession,
     SaveSession(As), Import (types: Audio, Session data), ImportAudioToClipList, SpotClipsByID,
     CreateNewTracks, DeleteTracks, GetTrackList, SelectTracksByName, RenameTargetTrack, SetTrackColor,
     SetTrack{Mute,Solo,RecordEnable,InputMonitor,Hidden,Inactive,Frozen,…}State,
     SetTimelineSelection/GetTimelineSelection, TogglePlayState, ToggleRecordEnable,
     GetTransportState, memory locations (Create/Edit/Select/GetMemoryLocations), Cut/Copy/Paste,
     ExportMix, BounceTrack, ExportClipsAsFiles, ExportSelectedTracksAsAAFOMF,
     ExportSessionInfoAsText, CreateBatchJob, Subscribe/PollEvents, CreateSignalPath,
     SetTrackMainOutputAssignments, **GetTrackControlInfo / GetTrackControlBreakpoints /
     SetTrackControlBreakpoints** with control types Volume, Pan, Mute, Lfe, **PluginParameter** [S7].
   - **Not in the proto** (as of 2025.10): insert plug-in, load plug-in preset, set tempo/meter/key,
     write MIDI notes, import MIDI files (Import types are only Audio and Session) [S7].
2. **Control surfaces** – EUCON (Avid S1/S6, Avid Control app; EUCON surfaces can even insert default
   EQ/Dynamics plug-ins) [S3]; HUI-personality MIDI surfaces [S3]. No public EUCON SDK is listed on
   Avid's developer portal [S5] → EUCON emulation is not an available route (to verify). HUI
   emulation over a virtual MIDI port is possible (protocol).
3. **MIDI Machine Control** – Pro Tools can follow MMC from an external master (transport) [S3].
4. **Pro Tools Extensions (new in 2026.11)** – a new AAX-based format for session-wide tools that live
   in a side panel and work without a session open; Stem Separator, Splice and SoundFlow ship as
   Extensions [S1][S2]. Whether third parties can build Extensions with a public SDK is **not stated**
   on the developer portal [S5] → to verify. Potentially an in-app COSOUND panel.
5. **UI automation** – Accessibility (macOS) [S9] / UI Automation (Windows) [S8] for the gaps.

## Feature facts

- **Plug-in formats**: AAX only (host-based AAX Native and AAX DSP on HDX) [S3]; no VST/AU documented.
  AAX SDK is available via Avid developer account [S5].
- **MIDI**: import MIDI files (File > Import > MIDI, Import MIDI Settings dialog) and export
  (File > Export > MIDI, MIDI/Instrument tracks) [S3]. MPE support added in 2026.11 [S1].
- **Interchange**: AAF/OMF/MXF import/export; Import Session Data [S3]. **DAWproject**: no mention in
  the 2026.11 Reference Guide [S3] → none.
- **Export / bounce**: Bounce Mix, Track Bounce (per-track stems) [S3]; PTSL ExportMix/BounceTrack [S7].
- **Stem separation**: Stem Separator Extension, local AI processing, in all tiers [S1][S2].

## Capability table

| Operation | Class | How | Source ID |
|---|---|---|---|
| Create track | official | PTSL `CreateNewTracks` (audio, MIDI, instrument, aux… track types) | S7 |
| Insert MIDI clip/region | ui-automation | PTSL import is audio/session only; MIDI file import via File > Import > MIDI (UI) | S3, S7 |
| Import audio | official | PTSL `Import` (Audio) / `ImportAudioToClipList` + `SpotClipsByID` | S7 |
| Set tempo / key | ui-automation | Not in PTSL; tempo/meter via Import Session Data (official, indirect) or UI | S3, S7 |
| Transport play/stop/locate | official | PTSL `TogglePlayState`, `SetTimelineSelection`, memory locations; also HUI/MMC (protocol) | S3, S7 |
| Set fader/pan/mute | official | Mute: `SetTrackMuteState`; volume/pan via `SetTrackControlBreakpoints` (static value semantics to verify); HUI (protocol) | S7 |
| Insert plugin | ui-automation | No PTSL command; EUCON can insert default plug-ins but no public SDK | S3, S5, S7 |
| Set plugin parameter | official | `SetTrackControlBreakpoints` with `TCType_PluginParameter` (automation-based, to verify); HUI pages (protocol) | S7 |
| Write automation | official | `SetTrackControlBreakpoints` (volume, pan, mute, plug-in params) | S7 |
| Load preset | ui-automation | No PTSL command | S7 |
| Export / bounce | official | PTSL `ExportMix`, `BounceTrack`, `ExportSelectedTracksAsAAFOMF` | S7 |
| Read project state | official | `GetTrackList`, session getters, `GetClipList`, `GetMemoryLocations`, `GetTransportState`, events | S7 |

## Limitations

- PTSL requires downloading Avid's SDK under a license agreement; terms not reviewed (to verify).
- No PTSL commands for plug-in insertion, presets, tempo/key, or MIDI note editing (as of 2025.10 proto).
- Command list verified via a community mirror of the proto (B), not Avid's own docs (login-gated).
- EUCON is the richest surface protocol but has no public SDK listed.
- AAX-only: third-party VST/AU instruments the user owns may be unavailable in Pro Tools.
