# REAPER – DAW knowledge

Research date: 2026-11-24. Current version: **REAPER 7.86** (2026-11-13) [S1][S2].

## Integration routes

1. **ReaScript (EEL2, Lua, Python)** – official. EEL2 and Lua are embedded; Python must be installed
   separately and enabled in preferences, and some UI features are unavailable from Python [S3].
   Scripts run inside REAPER as actions. The same API is callable from **C/C++ extension plug-ins**
   loaded by REAPER [S3]. This is the most complete programmatic surface of the three DAWs here.
2. **OSC** – official. Add an OSC control surface in Preferences > Control Surfaces; messages are
   defined by a `.ReaperOSC` pattern config (copy `Default.ReaperOSC`); OSC can also be bound to
   REAPER actions and FX parameter learn [S4]. Since ReaScripts are actions, an OSC message can launch
   a COSOUND script (to verify end-to-end).
3. **Web interface** – built-in HTTP control surface (index.html / fancier.html, touch volume/pan)
   [S2]. Its HTTP command set is shipped with REAPER, not documented online (to verify).
4. **Command line** – `-renderproject` renders a project headless-ish (hides main window) [S2].
5. **MIDI / control surfaces** – standard MIDI learn and surfaces (generic; details not researched here).
6. **SWS/S&M extension** – **third-party**, open source, v2.14.0 #7 (2025-09-07); adds actions and
   extra API functions [S6]. Optional dependency only.
7. **Ableton Link**: no native support found in changelog (to verify) [S2].

## Feature facts
- **Plug-in formats**: VST, VST3, LV2, AU, CLAP, DX, JS; LV2 and CLAP on Windows, macOS and Linux [S5].
- **MIDI export**: Export project MIDI as single or multitrack .MID with tempo map [S2].
- **Render settings** are scriptable: `GetSetProjectInfo` keys `RENDER_SETTINGS` (master, stems +
  master, stems only, multichannel, render matrix…), `RENDER_BOUNDSFLAG`, `RENDER_CHANNELS`,
  `RENDER_SRATE` [S3].
- **DAWproject**: not in changelog; REAPER listed only under a third-party converter
  (ProjectConverter) [S7][S2] → none natively (to verify).
- **FX containers** addressable via index offsets; `TrackFX_GetNamedConfigParm` navigation since 7.06 [S3].

## Capability table

| Operation | Class | How | Source ID |
|---|---|---|---|
| Create track | official | `InsertTrackAtIndex(idx, wantDefaults)` | S3 |
| Insert MIDI clip | official | `CreateNewMIDIItemInProj(track, start, end)` + `MIDI_InsertNote(...)` (+ `MIDI_Sort`) | S3 |
| Import audio | official | `InsertMedia(file, mode)` (current/new track, tempo-match, ReaSamplOmatic…) | S3 |
| Set tempo / key | official (tempo) / to-verify (key) | `SetCurrentBPM`, `SetTempoTimeSigMarker`; no project-key API found | S3 |
| Transport play/stop/locate | official / protocol | `OnPlayButton`, `OnStopButton`, `SetEditCurPos(time, moveview, seekplay)`; also OSC/web | S3 S4 |
| Set fader/pan/mute | official / protocol | `SetMediaTrackInfo_Value` (`D_VOL`, `D_PAN`, `B_MUTE`); also OSC | S3 S4 |
| Insert plugin | official | `TrackFX_AddByName(track, "VST3:…"/"CLAP:"/"AU:"/"JS:", recFX, instantiate)` | S3 |
| Set plugin parameter | official / protocol | `TrackFX_SetParam`; OSC FX learn | S3 S4 |
| Write automation | official | `GetTrackEnvelopeByName` / `GetFXEnvelope(…, create=true)` + `InsertEnvelopePoint` (+ `Envelope_SortPoints`) | S3 |
| Load preset | official | `TrackFX_SetPreset(name or .vstpreset path)`, `TrackFX_SetPresetByIndex` | S3 |
| Export / render | official | Configure via `GetSetProjectInfo` `RENDER_*`, trigger render action via `Main_OnCommand` (action ID to verify); CLI `-renderproject` | S3 S2 |
| Read project state | official | `CountTracks`, `GetSetProjectInfo(_String)`, `GetSetMediaTrackInfo_String`, FX/param getters; `Main_SaveProject` / `Main_openProject` | S3 |

## Limitations
- ReaScript runs inside REAPER; an external app needs a bridge (OSC → action, web interface, or a
  C++ extension / deferred script polling) (to verify chosen pattern).
- Python ReaScript needs a user-installed Python and loses some UI features [S3].
- Rendering through actions uses action IDs not documented in the API reference (to verify).
- Web-interface API and `Default.ReaperOSC` message set not documented online [S4].
- SWS is third-party; don't depend on it for core features [S6].
- No native DAWproject or Ableton Link found [S7][S2].
