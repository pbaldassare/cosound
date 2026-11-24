# DAW Integration Capabilities – comparison matrix

Section 13 of the knowledge base. Synthesized on 2026-11-24 from the six DAW folders; every cell is
backed by a cited source in that DAW's `README.md` / `sources.md` (no new facts here).
Classes as defined in `../README.md`: `official`, `protocol`, `ui-automation`, `none`, `to-verify`.

## Versions verified

| DAW | Version (2026-11-24) | OS | Plugin formats | Main external route |
|---|---|---|---|---|
| Logic Pro | 13.0 | macOS | AU (v2, v3) | No public scripting API. Control surfaces (MCU/HUI/EuCon), OSC, MIDI file / MusicXML / AAF import, UI automation |
| Pro Tools | 2026.11 | macOS, Windows | AAX | Pro Tools Scripting SDK (PTSL, gRPC on localhost; SDK behind Avid account + license) |
| Cubase | 15.0.40 | macOS, Windows | VST3 (VST2 legacy, off by default) | MIDI Remote API (JavaScript, runs inside Cubase, reachable only via MIDI/SysEx ports); DAWproject import/export |
| Ableton Live | 12.4.16 | macOS, Windows | VST2, VST3, AU | Live Object Model via Max for Live (Suite or add-on); Python remote scripts (undocumented); Link (tempo/phase only) |
| FL Studio | 2026.1.7 (script API v45) | macOS, Windows | VST2, VST3, CLAP; AU on macOS | Python MIDI scripting (inside FL, triggered by MIDI); piano-roll scripts (user-launched); offline CLI render |
| REAPER | 7.86 | macOS, Windows, Linux | VST2, VST3, AU, CLAP, JSFX | ReaScript (Lua/EEL2/Python) + C++ extensions; OSC; web interface |

## Operation matrix

| Operation | Logic Pro | Pro Tools | Cubase | Ableton Live | FL Studio | REAPER |
|---|---|---|---|---|---|---|
| Create track | ui-automation | official | protocol (key command; name to-verify) | official | ui-automation | official |
| Insert MIDI clip/notes | protocol (MIDI file import) | ui-automation | protocol (MIDI file / DAWproject) | official | official, limited (piano-roll script / step seq) | official |
| Import audio | ui-automation | official | ui-automation | official | ui-automation | official |
| Set tempo / key | ui-automation | ui-automation | protocol (tempo); key to-verify | official | official (tempo) | official (tempo); key to-verify |
| Transport play/stop/locate | protocol | official | protocol | official | official | official |
| Fader / pan / mute | protocol | official (via automation commands) | protocol | official | official | official |
| Insert plugin | ui-automation | ui-automation | protocol (DirectAccess) | official for Live devices; ui-automation for third-party | ui-automation | official |
| Set plugin parameter | protocol | official (via automation commands) | protocol | official | official | official |
| Write automation | protocol | official | protocol | to-verify | to-verify | official |
| Load preset | ui-automation | ui-automation | to-verify | official (plugin's own presets); ui-automation (Live presets) | official, next/previous only | official |
| Export / render / bounce | ui-automation | official | ui-automation | ui-automation | official (offline CLI on saved project) | official |
| Read project state | to-verify | official | protocol | official | partial | official |

DAWproject (open interchange format): only Cubase supports it natively. Logic, Pro Tools, Ableton Live, FL Studio and REAPER have no native support documented (REAPER only through a third-party converter).

## Reading the matrix for COSOUND

1. **REAPER** is fully scriptable: the best target to prove the DAW Controller end to end.
2. **Ableton Live** is close to full control, but only through Max for Live (Suite or add-on), so this route excludes Live Intro/Standard users. Third-party plugin insertion and export need UI automation.
3. **Pro Tools** has a real official API (PTSL) for session, tracks, import, export and automation, but no MIDI-note, tempo or plugin-insert commands. SDK license terms must be read before use.
4. **Cubase** is controllable through a COSOUND MIDI Remote script plus virtual MIDI ports. It is the only DAW with official DAWproject support, which helps move whole arrangements in.
5. **Logic Pro** (the product's first target) has **no public API**. The realistic hybrid is control surface/OSC for mixer and transport, MIDI file import for content, and the macOS Accessibility API for everything else. It needs the Accessibility permission and must be re-verified on every Logic release. Logic 13's built-in AI assistant has no external API.
6. **FL Studio** scripting is reactive (MIDI-triggered) and cannot build arrangements; content creation relies on MIDI file drag/import or user-launched piano-roll scripts.

**Universal fallback for every DAW:** COSOUND generates a Standard MIDI File or rendered audio and the user (or UI automation) drops it into the project. This always works and is the safe baseline for the first release.

## Open items (to verify before Prompt 03 implementation)
- Pro Tools PTSL license terms; command list checked against an open mirror at 2025.10, not Avid's gated docs.
- Logic OSC message set and control-surface plug-in parameter mapping depth.
- Ableton: writing automation from the Live Object Model; status of Python remote scripts.
- FL Studio: automation recording from scripts; socket access from scripts.
- Cubase: key-command names for track creation; preset loading; Mackie Control status (source reliability C/U).
- REAPER: project key; render action ID.
