# FL Studio – DAW knowledge

Research date: 2026-11-24. Current version: **FL Studio 2026.1.7** maintenance update (2026-09-28);
FL Studio 2026 first released 7 July 2026 [S1]. MIDI scripting API tables go up to **API v45** [S2].

## Integration routes

1. **MIDI Controller Scripting (Python)** – official. A `device_<name>.py` script in
   `Documents\Image-Line\FL Studio\Settings\Hardware\<devicename>\` is run by FL Studio's built-in
   Python (no install needed); manual names Fruity and Producer Editions [S2]. It is
   **event-driven**: callbacks such as `OnInit`, `OnMidiMsg`, `OnNoteOn`, `OnControlChange`,
   `OnSysEx`, `OnIdle`, `OnRefresh`, `OnProjectLoad` [S2]. Modules: transport, mixer, channels,
   patterns, playlist, arrangement, plugins, ui, general, device, launchMapPages [S2].
   COSOUND would reach it by sending MIDI (CC/SysEx) to a virtual port bound to a COSOUND script,
   which translates messages into API calls and answers via MIDI/SysEx out. Python version, stdlib
   and socket availability are not documented (to verify) [S2].
   - **Can**: rename/colour/mute/solo/select and set volume/pan of channels and mixer tracks
     (`mixer.setTrackVolume`, `setTrackPan`, `muteTrack`, `setTrackName`, `setRouteTo`); transport
     start/record/song position (`transport.start`, `record`, `setSongPos`); tempo
     (`mixer.setCurrentTempo`, API 38; `mixer.getCurrentTempo`); get/set plugin parameters
     (`plugins.getParamValue` / `setParamValue`) and step through presets (`plugins.nextPreset`);
     pattern length/clone/clear/duplicate (`patterns.setPatternLength` API 39, duplicate data API 45);
     step-sequencer grid bits and step parameters (`channels.setGridBit`,
     `setStepParameterByIndex`); play a note on a channel (`channels.midiNoteOn`); markers
     (`arrangement.addAutoTimeMarker`, `jumpToMarker`); browser navigation; undo points [S2][S1].
   - **Cannot (no function documented)**: write piano-roll notes into a pattern, add channels or
     plug-ins, load samples, save or render a project [S2]. `ui.showPicker` only opens the plugin
     picker [S2]. Setters are only safe when `general.safeToEdit()` allows [S2].
2. **Piano roll scripting (Python, `.pyscript`)** – official. Scripts stored in
   `Documents\Image-Line\FL Studio\Settings\Piano roll scripts` and run **by the user from the Piano
   roll Scripts menu** [S3]. The `flpianoroll` module's `score` can add/get/delete notes and markers
   (incl. scale and time-signature markers); `ScriptDialog` builds the parameter UI [S3]. External
   triggering is not documented (to verify) – practical pattern: COSOUND writes data a script reads,
   user (or UI automation) runs the script.
3. **Command-line render** – official: `FL.exe /R /E<formats> /F<folder> /O<outdir>`, `/M` exports
   MIDI, `/Z` ZIP; macOS via `open ... --args -R -E...` [S5]. Offline, on saved .flp files.
4. **MIDI** – generic MIDI input/out and MIDI-clock sync (via scripts/settings) [S2]. **Ableton Link
   / OSC**: no official support found; forum threads only (to verify) [S7].
5. **UI automation** – adding plug-ins, importing audio, rendering in a running session.
6. FL 2026 adds the in-app assistant "Gopher" (experimental) able to generate Piano roll scripts on
   request [S1] – not an external API.

## Feature facts
- **Plug-in formats**: Windows – VST2, VST3 (32/64-bit), CLAP; macOS – VST2, VST3, AU (64-bit), CLAP [S4].
- **Export**: WAV (16/24/32f), MP3, OGG, M4A, FLAC, MIDI (notes + automation; use "Prepare for MIDI
  export") [S5]. **Stems**: "Split mixer tracks" writes one WAV per mixer track (WAV only) [S5].
- **DAWproject**: not mentioned in export docs [S5]; FL not in DAWproject supporters [S8] → none (to verify).
- **Stem separation**: "Remix a song" tool stem-separates a song into the Playlist [S1].
- **Audio/MIDI import**: not verified from the manual in this pass (to verify).

## Capability table

| Operation | Class | How | Source ID |
|---|---|---|---|
| Create track | ui-automation | No function to add channels/plugins; playlist & mixer tracks are pre-existing slots that scripts can rename/route | S2 |
| Insert MIDI clip | official (limited) / protocol | Piano-roll script `score.addNote` (user-invoked) ; MIDI script: step grid (`setGridBit`), pattern length/clone; or record MIDI sent by COSOUND | S2 S3 |
| Import audio | ui-automation | No API to load samples/audio | S2 |
| Set tempo / key | official (tempo) / none (key) | `mixer.setCurrentTempo`; no project key – only piano-roll scale markers via piano-roll script | S2 S3 |
| Transport play/stop/locate | official | `transport.start`, `stop`, `record`, `setSongPos` (via MIDI-triggered script) | S2 |
| Set fader/pan/mute | official | `mixer.setTrackVolume`, `setTrackPan`, `muteTrack`; channel equivalents | S2 |
| Insert plugin | ui-automation | No function; `ui.showPicker` only opens picker | S2 |
| Set plugin parameter | official | `plugins.setParamValue` / `getParamValue` | S2 |
| Write automation | to-verify | `mixer.automateEvent` and REC flags (`REC_InitStore`) exist but are barely documented | S2 |
| Load preset | official (next/prev only) | `plugins.nextPreset` / prev navigation; loading a named preset file = ui-automation | S2 |
| Export / render | official (offline CLI) / ui-automation (live session) | `FL.exe /R /E... /M`; `FPT_Save` is a global-transport constant (save, to-verify) | S5 S2 |
| Read project state | official (partial) | Getters for names, volumes, tempo, pattern/track counts; no note data from MIDI scripts (piano-roll scripts read notes of the open score) | S2 S3 |

## Limitations
- MIDI scripting runs inside FL and is triggered only by MIDI/callbacks; COSOUND needs a virtual
  MIDI port + custom SysEx/CC protocol [S2].
- No API to create channels, insert plug-ins, load samples, write piano-roll notes from MIDI scripts,
  or render in-session [S2].
- Piano-roll scripts need user invocation (to verify) [S3].
- Manual says MIDI scripting is available "only in Fruity Edition and Producer Edition"; whether
  that means "and above" (Signature/All Plugins) needs checking (to verify) [S2].
- No Link, OSC or DAWproject found [S7][S8].
- API stubs repo is marked unmaintained [S6]; rely on the manual [S2].
