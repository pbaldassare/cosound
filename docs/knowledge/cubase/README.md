# Cubase – DAW knowledge

Research date: 2026-11-24. Current version: **Cubase 15.0.40** (maintenance update, 2026-10-14) [S1].
MIDI Remote API **v1.3** ships with Cubase/Nuendo 15 (since 15.0.20, April 2026) [S1][S3].
macOS and Windows.

## Integration routes (ranked for COSOUND)

1. **MIDI Remote API (JavaScript ES5)** – official, documented. A script (driver) declares MIDI
   ports, a surface layout and host mapping pages [S2]. COSOUND would ship a custom script plus a
   virtual MIDI port pair and use MIDI/SysEx as its wire protocol (scripts can `sendMidi` and receive
   `mOnSysex`) [S6]. Host access includes [S6]:
   - `mTransport` (values, time display: `setTime` for locate, **`setTempoBPM`** and tempo change
     callback; cycle locators), `mMixConsole`, `mTrackSelection` (selected channel: volume, pan,
     mute, solo, sends, insert/strip effects, channel EQ, pre-filter, Quick Controls),
     `mControlRoom`, `mFocusedQuickControls`, `mMouseCursor`;
   - insert slots expose on/bypass/edit, **automation read/write**, plug-in parameter bank zones and
     plug-in identity callbacks [S6];
   - `makeCommandBinding(surfaceValue, category, command)` triggers Key Commands by category/name;
     `canPerform` check added in v1.3 [S4][S6];
   - **DirectAccess** (v1.2+): traverse runtime host objects by ObjectID, enumerate and write any
     parameter (`setParameterProcessValue`), object titles/types, change callbacks [S5]; v1.3 adds a
     **Plugin Manager** to browse plug-in collections and assign a plug-in to a slot
     (`trySetSlotPlugin`) [S4][S5].
   Limits: everything is driven through controller semantics (selected track, mixer banks); there is
   no documented call to create tracks or write MIDI notes except via Key Command bindings (to verify).
2. **Key Commands, Macros and Project Logical Editor** – PLE is a filter+action search-and-replace for
   the Project window and can chain macros [S12]; macros/PLE presets can be bound to key commands,
   which MIDI Remote command bindings can trigger (combination to verify).
3. **Remote Devices** – Studio Setup > Add Device; Generic Remote (legacy) for any MIDI controller [S8].
   Mackie Control is reported selectable under Add Device in Cubase 15 [S16] (to verify on official doc).
4. **File interchange** – DAWproject import/export, MIDI file import, AAF, audio mixdown [S9]–[S14].
5. **UI automation** – Accessibility (macOS) [S18] / UI Automation (Windows) [S17].

## Feature facts

- **Plug-in formats**: VST3; VST 2 disabled by default in 14/15, enable in VST Plug-in Manager, not on
  native Apple silicon/ARM Windows, no official support (to verify) [S15]. DAWproject carries VST3
  inserts only [S9]. ARA extensions supported (e.g. WaveLab Go in 15) [S1].
- **DAWproject**: File > Export > DAWproject; import/export audio, instrument, group, effect, marker,
  folder tracks, output channels, volume/pan/mute/sends/colour, routing, automation of volume, pan,
  mute, sends and insert/instrument parameters [S9][S10]. Community reports describe gaps (MIDI CC
  loss) in 15 (C, not verified).
- **MIDI**: File > Import > MIDI File into new or current project; options in Preferences > MIDI >
  MIDI File [S11]. MIDI file export exists (Export Options preference) – page not opened (to verify).
- **AAF**: File > Import > AAF with options dialog [S14]; AAF/OMF export not verified.
- **Export**: Export Audio Mixdown (locator range, cycle markers, arranger chains) [S13]; multi-channel
  (stem) export and job queues per dialog page (to verify) [S13].

## Capability table

| Operation | Class | How | Source ID |
|---|---|---|---|
| Create track | protocol | MIDI Remote command binding to an "Add Track" key command (to verify command names); or DAWproject import | S6, S9 |
| Insert MIDI clip/region | protocol | Import COSOUND-generated MIDI file / DAWproject; or record from virtual MIDI port | S10, S11 |
| Import audio | ui-automation | File > Import via UI (command binding to import dialog still needs file selection) | S17, S18 |
| Set tempo / key | protocol | Tempo: `TransportTimeDisplay.setTempoBPM` (MIDI Remote); key: to-verify | S6 |
| Transport play/stop/locate | protocol | MIDI Remote `mTransport` values, `setTime`; command bindings | S4, S6 |
| Set fader/pan/mute | protocol | MIDI Remote `mMixConsole` / selected channel values, DirectAccess | S5, S6 |
| Insert plugin | protocol | DirectAccess Plugin Manager `trySetSlotPlugin` (API 1.3) | S4, S5 |
| Set plugin parameter | protocol | DirectAccess `setParameterProcessValue`; parameter bank zones | S5, S6 |
| Write automation | protocol | Toggle insert/channel automation write + move values via MIDI Remote | S6 |
| Load preset | to-verify | Not found in API reference | S6 |
| Export / bounce | ui-automation | Export Audio Mixdown dialog (DAWproject export is file-level) | S13 |
| Read project state | protocol | DirectAccess object traversal + callbacks, streamed back over MIDI/SysEx; DAWproject export | S5, S9 |

## Limitations

- MIDI Remote runs only inside Cubase and talks through MIDI ports; COSOUND needs a bridge script and
  virtual MIDI ports, with SysEx framing for state.
- API is controller-oriented: selection/bank-relative, no documented track creation or note editing.
- DirectAccess/Plugin Manager need API 1.2/1.3 (Cubase 13.0.50+/15.0.20+) [S3].
- VST2 legacy only; DAWproject keeps VST3 inserts only.
