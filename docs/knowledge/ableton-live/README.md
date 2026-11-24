# Ableton Live – DAW knowledge

Research date: 2026-11-24. Current version: **Live 12.4.16** (released 2026-10-30) [S1]. The Live
Object Model (LOM) reference used here states it documents **Live 12.4.5** [S2].

## Integration routes (ranked for COSOUND)

1. **Max for Live + Live API (LOM)** – official, documented. A Max for Live device placed in the Set
   can read and modify the Live Set and extend control surfaces through the Live API [S10]. Max for
   Live is included in Live Suite, or Live Standard with the add-on [S10]. The LOM documents Song,
   Track, ClipSlot, Clip, Device, DeviceParameter, MixerDevice, PluginDevice, RackDevice, TakeLane,
   CuePoint, Scene, ControlSurface and device-specific classes [S2]. A COSOUND "bridge" M4L device
   would receive commands from the desktop app over the network and call the LOM.
2. **MIDI Remote Scripts (Python control-surface scripts)** – Live loads Python 3 scripts (Python 3
   since Live 11) from a "Remote Scripts" folder in the User Library and lists them in the Control
   Surface chooser (to verify, source unreadable) [S15]. They reach the same object model (the LOM
   has a ControlSurface class) [S2]. **Status: not officially documented as a public SDK** – Ableton's
   help articles on custom scripts could not be read (403) [S15][S16]; treat the Python API surface as
   unofficial/undocumented (to verify). Community project AbletonOSC uses this route to expose the
   LOM over OSC (listen 11000 / reply 11001, Live 11+, MIT) [S17].
3. **MIDI / control surfaces** – up to six control surfaces, MIDI Map mode, Key Map mode, takeover
   modes [S11]. No native OSC in the Remote Control chapter [S11].
4. **Ableton Link** – syncs tempo, beat/phase, start/stop (Link 3), optional arrangement position,
   and Link Audio (audio streaming between peers); C++ library, LinkKit for iOS [S14]. Link Audio
   arrived in Live 12.4 [S1]. Timing only – not a project-control API.
5. **UI automation** – for anything the APIs lack (render, browser loading of plug-ins/presets).

## Feature facts

- **Plug-in formats**: VST2, VST3 (Windows and macOS), AU v2 and AU v3 (macOS; AUv3 since 11.2);
  CLAP not mentioned [S12]. Plug-in parameters must be added to Live's panel via Configure mode to be
  exposed; some plug-ins don't publish all parameters [S12].
- **Plug-in presets**: VST presets/banks load/save from the device header; AU presets stored as .au
  in the user Library [S12]. LOM `PluginDevice` exposes `presets` (list) and settable
  `selected_preset_index` [S9].
- **Audio import**: WAV, AIFF, AIFF-C, FLAC, OGG Vorbis, MP3; M4A on Mac only [S13].
- **MIDI**: import via browser or Create > Import MIDI File; export single clips as Standard MIDI
  files (Export MIDI Clip) [S13].
- **Export / stems**: Export Audio/Video; "All Individual Tracks" renders one post-fader file per
  audio-producing track (to verify details) [S13]. No LOM function for export/render [S3].
- **DAWproject**: no mention in manual or release notes read [S1][S13]; Live is not in the
  DAWproject supporters list [S18] → none (to verify).
- **Stem separation**: added in 12.3/12.4 ("Separate Stems for Time Selection" in 12.4) [S1].
- **MIDI Tools**: transformations/generators can be built in Max for Live [S10].

## Capability table

| Operation | Class | How | Source ID |
|---|---|---|---|
| Create track | official | `Song.create_midi_track` / `create_audio_track` (LOM) | S3 |
| Insert MIDI clip | official | `ClipSlot.create_clip(length)` (Session, empty slot, MIDI track) or `Track.create_midi_clip(start, length)` (Arrangement); notes via `Clip.add_new_notes` / `apply_note_modifications` (Live 11+) | S4 S5 S6 |
| Import audio | official | `ClipSlot.create_audio_clip(path)` or `Track.create_audio_clip(path, position)` (Arrangement); fails on frozen/non-audio tracks | S4 S5 |
| Set tempo / key | official | `Song.tempo`; `Song.root_note`, `Song.scale_name` | S3 |
| Transport play/stop/locate | official | `Song.start_playing`, `stop_playing`, `continue_playing`, `current_song_time`, `jump_by`, `scrub_by`; also Link start/stop | S3 S14 |
| Set fader/pan/mute | official | `Track.mixer_device.volume` / `panning` / `sends` (DeviceParameter.value); `Track.mute`, `solo`, `arm` | S4 S7 S8 |
| Insert plugin | none (official for native devices) / ui-automation | `Track.insert_device(name, index)` since 12.3 – **only native Live devices; not plug-ins or M4L devices**. Plug-ins need UI/browser; Remote Script browser loading undocumented (to-verify) | S4 |
| Set plugin parameter | official | `DeviceParameter.value` on device parameters (plug-in params only those exposed via Configure) | S7 S12 |
| Write automation | to-verify | LOM Clip documents only `clear_envelope` / `clear_all_envelopes` and `has_envelopes`; DeviceParameter exposes `automation_state`, `re_enable_automation`. No documented envelope-write call; recording parameter changes while recording automation is plausible but unverified | S6 S7 |
| Load preset | official (plug-in programs) / ui-automation (Live presets) | `PluginDevice.selected_preset_index`; native .adv / rack presets via browser = UI. `SimplerDevice.replace_sample` added in 12.4 | S9 S1 |
| Export / render | ui-automation | Export Audio/Video dialog; no LOM export function. MIDI clip export also UI | S3 S13 |
| Read project state | official | Full LOM get/observe on Song/Tracks/Clips/Devices; `Song.file_path` gives set path | S2 S3 |

## Limitations
- Bridge code must run **inside Live** (M4L device or Remote Script); no out-of-process official API.
- Max for Live requires Suite or the add-on [S10] – Live Intro/Standard users without it lose route 1.
- Python Remote Script API is undocumented/unsupported publicly (to verify) [S15][S16].
- Third-party plug-ins cannot be inserted programmatically via LOM [S4]; their parameters are only
  reachable once instantiated (and, for many, once configured) [S12].
- No programmatic render/export, no DAWproject [S3][S18].
- No native OSC [S11]; OSC requires a third-party script (AbletonOSC) [S17].
