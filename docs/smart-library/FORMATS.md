# Smart Library – formats actually supported (Prompt 07)

Checked 2026-11-26. Each row says what the engine really does with the format and how that was verified. Three levels:
- **Read and analyzed**: contents decoded or parsed, data extracted.
- **Metadata read**: container header only.
- **Recognized only**: indexed by extension, searchable by name, folder and drive; contents **not read** (shown as "contents
  not read").

## Audio (read and analyzed)

Decoder: Symphonia 0.6.1 (pure Rust, MPL-2.0), streamed in packets. Analysis covers the first 90 s.

| Extension | Codecs | Verified with |
|---|---|---|
| `.wav`, `.wave` | PCM 8/16/24/32-bit integer, 32/64-bit float | Generated test files + afconvert output |
| `.aif`, `.aiff` | PCM big-endian | Test files written by the engine's test helper; macOS system sound (24-bit); afconvert output |
| `.aifc` | Uncompressed (NONE/sowt) | afconvert output |
| `.caf` | PCM (16/24-bit) | afconvert output (24-bit) |
| `.flac` | FLAC | afconvert output |
| `.m4a`, `.mp4` | AAC-LC, ALAC | afconvert output (both codecs) |
| `.aac` | AAC-LC (ADTS) | Decoder supports it; **no real ADTS file tested** |
| `.mp3` | MPEG-1/2 Layer III | A real MP3 from the user's Music folder (7 min, 48 kHz); analyzed locally, not copied anywhere |
| `.ogg` | Vorbis only (Opus not decoded) | Decoder supports it; **no real file tested** |

Extracted:
- duration, sample rate, channels, bit depth (when the codec has one);
- peak, RMS, integrated loudness (LUFS) and true peak;
- tags (title, artist, genre, BPM, key, comment, when present);
- WAV loop chunks: ACID tempo, beats and root, and the `smpl` root note.

Analysis: tempo, key, loop/one-shot, bars, spectrum, transients, sound character and timbre vector (ARCHITECTURE.md §1).

## MIDI (read and analyzed)
| Extension | What is read | Verified with |
|---|---|---|
| `.mid`, `.midi` | Standard MIDI File, format 0 and 1: tracks, notes, tempo, time and key signatures, programs (GM instrument names), range, intervals, polyphony, pattern type (drums / chords / bass / arpeggio / melody), rhythm, bars | Files written by COSOUND's own SMF writer (Prompt 05, checked earlier with `file` and Python `mido`) |

## Presets
| Extension | Level | What is read |
|---|---|---|
| `.vital` | Read | JSON: preset name, author, style/category, comments |
| `.vstpreset` | Read | VST3 header, class ID, `Info` XML chunk: plugin name, category |
| `.aupreset` | Read | Property list: name, manufacturer, AU type and subtype |
| `.fxp`, `.fxb` | Metadata read | VST2 container header: plugin ID (four-character code), program name, parameter count |
| `.nksf`, `.nmsv`, `.serumpreset`, `.h2p`, `.adg`, `.adv`, `.pst`, `.patch` | Recognized only | Layout not public or not verified. The plugin is inferred from folder names when they match a known synth (source "path") |

Verified with fixture files written by the tests from the published layouts. No third-party preset file was available on
this Mac.

## Instruments and projects (recognized only)
- Instrument libraries: `.nki`, `.nkm`, `.nkc`, `.exs`, `.sfz`, `.sf2`.
- DAW projects: `.logicx` (package, counted as one file), `.als`, `.flp`, `.ptx`, `.cpr`, `.rpp`, `.dawproject`.

These are searchable by name, folder and drive. Their contents are not read.

## Not supported
Opus, WMA, APE, DSD, Monkey's Audio, Kontakt and Komplete internals, proprietary sample-library containers, encrypted files.
Files with these extensions are not indexed, or are indexed as "contents not read" when their extension is in the lists
above.
