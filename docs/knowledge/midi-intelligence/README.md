# MIDI Intelligence (Section 6)

What COSOUND must know about MIDI to generate correct data and insert it in a DAW. Citations `[Sn]` refer to
[`sources.md`](sources.md); retrieved 2026-11-24. The practical generation rules are in
[composition-guide.md](composition-guide.md). Pitch/octave-naming and theory live in [`../music-theory/`](../music-theory/README.md).

## 1. MIDI 1.0 channel voice messages

Status byte = high nibble (type) + low nibble n = channel 0–15 (shown to users as 1–16) [S2]. Data bytes are 7-bit (0–127).

| Message | Status | Data bytes | Notes |
|---|---|---|---|
| Note Off | 8n | key, release velocity | |
| Note On | 9n | key, velocity | Note On with velocity 0 = Note Off (receivers must accept both forms and treat them identically) [S1] |
| Polyphonic Key Pressure (poly aftertouch) | An | key, pressure | |
| Control Change | Bn | controller 0–119, value | 120–127 = Channel Mode messages [S2] |
| Program Change | Cn | program 0–127 | one data byte (users often see 1–128) |
| Channel Pressure (channel aftertouch) | Dn | pressure | one data byte |
| Pitch Bend | En | LSB, MSB (14-bit, 0–16383) | centre = 2000H (8192) = no bend; range set by receiver or RPN 0 [S2] |

- **Note numbers**: 0–127, middle C = 60 [S1]. Key transpose may remap 60 to another physical key [S1].
- **Velocity**: interpretation is up to the receiver; larger = stronger; **64 = mezzo-forte** and is the value devices
  without velocity sensing send; velocity→volume should preferably be exponential [S1]. Note-off release velocity
  default is 64 [S1].
- **Running status**: a repeated status byte may be omitted for voice/mode messages; this is why Note On vel 0 is the
  common way to end notes [S1].
- **Channel Mode (CC 120–127)** [S2]: 120 All Sound Off, 121 Reset All Controllers, 122 Local Control, 123 All Notes Off,
  124/125 Omni Off/On, 126 Mono On, 127 Poly On (124–127 also imply All Notes Off). COSOUND should send CC 123 (and/or
  120) on every channel it used when the user cancels playback ("panic").

## 2. Commonly used Control Change numbers [S3]

| CC | Function | Typical COSOUND use |
|---|---|---|
| 0 / 32 | Bank Select MSB / LSB | choosing sound banks on hardware/GM synths |
| 1 | Modulation wheel | vibrato depth / synth macro, filter sweeps |
| 2 | Breath controller | wind-style expression |
| 4 | Foot controller | |
| 5 | Portamento time | glide length for 303-style / legato leads |
| 6 / 38 | Data Entry MSB / LSB | RPN/NRPN values |
| 7 | Channel Volume | static level (mix) |
| 8 | Balance | |
| 10 | Pan | |
| 11 | Expression | dynamic swells within a part (keeps CC 7 as the mix level) |
| 64 | Damper/Sustain pedal (≤63 off, ≥64 on) | pianos, pads |
| 65 | Portamento on/off | |
| 66 / 67 | Sostenuto / Soft pedal | |
| 68 | Legato footswitch | |
| 71 | Sound Controller 2 – default Timbre/Harmonic Intensity (often resonance) | |
| 72 / 73 | Release time / Attack time | |
| 74 | Sound Controller 5 – default **Brightness** (often filter cutoff; MPE "slide") | filter sweeps |
| 75–79 | Decay, Vibrato rate/depth/delay, undefined | |
| 91 / 93 | Effects 1 (reverb send) / Effects 3 (chorus send) | |
| 98/99 | NRPN LSB/MSB | |
| 100/101 | RPN LSB/MSB (RPN 0 = pitch-bend sensitivity, 1 = fine tune, 2 = coarse tune [S5]) | |

The "≤63 off / ≥64 on" switch threshold for CC 64–69 is a widely used convention (to verify against [S1] section on
switches). In DAWs most soft synths ignore CC numbers unless mapped; prefer automating plugin parameters directly when
the DAW integration allows (see `../daw-integration/`, future).

## 3. General MIDI Level 1 essentials [S5]

- 16 channels; ≥ 24 voices; **key-based percussion is always channel 10**; 128 melodic programs + ≥ 47 percussion sounds.
- Required controllers: 1, 7, 10, 11, 64, 121, 123; RPN 0/1/2; channel pressure; pitch bend **default range ±2
  semitones**; defaults: bend centred, volume 100.
- GM percussion map (channel 10), full list [S5] (note names in Yamaha/Ableton C3 = 60 naming from [S20]):

| Key | Name (C3=60) | Sound | Key | Name | Sound | Key | Name | Sound |
|---|---|---|---|---|---|---|---|---|
| 35 | B0 | Acoustic Bass Drum | 51 | D♯2 | Ride Cymbal 1 | 67 | G3 | High Agogo |
| 36 | C1 | **Bass Drum 1 (kick)** | 52 | E2 | Chinese Cymbal | 68 | G♯3 | Low Agogo |
| 37 | C♯1 | Side Stick (rim) | 53 | F2 | Ride Bell | 69 | A3 | Cabasa |
| 38 | D1 | **Acoustic Snare** | 54 | F♯2 | Tambourine | 70 | A♯3 | Maracas |
| 39 | D♯1 | **Hand Clap** | 55 | G2 | Splash Cymbal | 71 | B3 | Short Whistle |
| 40 | E1 | Electric Snare | 56 | G♯2 | Cowbell | 72 | C4 | Long Whistle |
| 41 | F1 | Low Floor Tom | 57 | A2 | Crash Cymbal 2 | 73 | C♯4 | Short Guiro |
| 42 | F♯1 | **Closed Hi-Hat** | 58 | A♯2 | Vibraslap | 74 | D4 | Long Guiro |
| 43 | G1 | High Floor Tom | 59 | B2 | Ride Cymbal 2 | 75 | D♯4 | Claves |
| 44 | G♯1 | Pedal Hi-Hat | 60 | C3 | Hi Bongo | 76 | E4 | Hi Wood Block |
| 45 | A1 | Low Tom | 61 | C♯3 | Low Bongo | 77 | F4 | Low Wood Block |
| 46 | A♯1 | **Open Hi-Hat** | 62 | D3 | Mute Hi Conga | 78 | F♯4 | Mute Cuica |
| 47 | B1 | Low-Mid Tom | 63 | D♯3 | Open Hi Conga | 79 | G4 | Open Cuica |
| 48 | C2 | Hi-Mid Tom | 64 | E3 | Low Conga | 80 | G♯4 | Mute Triangle |
| 49 | C♯2 | **Crash Cymbal 1** | 65 | F3 | High Timbale | 81 | A4 | Open Triangle |
| 50 | D2 | High Tom | 66 | F♯3 | Low Timbale | | | |

**Drum racks:** Ableton's Drum Rack has 128 pads, one per MIDI note, each chain choosing the note it receives; the
chooser lists GM drum equivalents [S18]. Producers' "kick on C1" is MIDI 36 in C3 = 60 naming [S19][S20], which matches
GM Bass Drum 1. That default Drum Rack / FL / Logic Drum Machine Designer pad layouts start at 36 is common practice
(to verify per DAW). COSOUND should **read the user's actual kit mapping** when available and fall back to the GM map.

## 4. Standard MIDI Files (SMF 1.0) [S4]

- File = chunks: 4-char ASCII type + 32-bit big-endian length + data. One header chunk **MThd** (length 6) followed by
  one or more track chunks **MTrk**. Not IFF (no nesting, odd lengths allowed).
- Header data = three 16-bit words: `<format> <ntrks> <division>`.
  - **Format 0**: one multi-channel track (ntrks = 1); most interchangeable.
  - **Format 1**: one or more simultaneous tracks; the tempo map must be in the **first** track.
  - **Format 2**: independent single-track patterns (rarely supported).
  - **Division** bit 15 = 0 → bits 14–0 = **ticks per quarter note (PPQ)** (e.g. 96 → an 8th note = 48 ticks);
    bit 15 = 1 → SMPTE frames (−24/−25/−29/−30) × ticks per frame.
- Track events = `<delta-time> <event>`; delta-time is a **variable-length quantity** (7 bits per byte, MSB = "more
  bytes follow"; max 0x0FFFFFFF, i.e. ≤ 4 bytes). Running status allowed within tracks; sysex/meta events cancel it.
- If tempo/time signature are missing, assume **4/4 and 120 BPM**.
- Key meta events:

| Bytes | Meaning |
|---|---|
| FF 03 len text | Sequence/track name |
| FF 2F 00 | End of Track – mandatory |
| FF 51 03 tttttt | Set Tempo, µs per quarter note (24-bit) |
| FF 54 05 hr mn se fr ff | SMPTE offset |
| FF 58 04 nn dd cc bb | Time signature: nn numerator, dd = log2(denominator) (2 = quarter, 3 = eighth), cc MIDI clocks per metronome click, bb notated 32nds per MIDI quarter (normally 8). 6/8 example: FF 58 04 06 03 24 08 |
| FF 59 02 sf mi | Key signature: sf −7…+7 (flats negative, sharps positive), mi 0 = major, 1 = minor |
| FF 7F len data | Sequencer-specific |

- **Tempo math**: tempo (µs/qn) = 60 000 000 / BPM. 120 BPM = 500 000 (mido default) [S10][S11]; 124 BPM =
  483 871 (rounded). seconds = ticks × tempo / (PPQ × 10⁶) – mido's `tick2second`, `bpm2tempo` implement this [S10].
- **PPQ choice**: typical values 96–480; larger reduces rounding error [S10]. COSOUND default: **480 PPQ** (1/16 = 120
  ticks, 1/16 triplet = 80, 1/32 = 60) – all grid divisions used in dance music are integers.
- mido meta defaults: time_signature 4/4, clocks_per_click 24, notated_32nd_notes_per_beat 8; key_signature 'C' [S11].
- A key signature meta event (FF 59) for A minor = `sf 0, mi 1`; COSOUND should write it so DAWs/notation show the key.

## 5. MPE – MIDI Polyphonic Expression

- MMA/AMEI Recommended Practice (current **M1-100-UM v1.1**, editorial update of RP-053 v1.0, no technical change)
  for per-note expression using MIDI 1.0 messages: each sounding note gets its **own channel** [S6].
- Zones [S7]: a **Lower Zone** with master/manager channel 1 and member channels from 2 upward, and/or an **Upper Zone**
  with master channel 16 and members from 15 downward. Configured with the **MPE Configuration Message (RPN 6)** on the
  master channel; 0 member channels disables the zone.
- Per-note dimensions [S7]: pitch bend (default **±48 semitones** on member channels, ±2 on the master), pressure via
  **Channel Pressure**, third dimension ("slide"/timbre) via **CC 74**.
- COSOUND relevance: when generating for MPE instruments, write per-note bends/pressure/CC74 on rotating channels; for
  ordinary instruments, use plain channel messages.

## 6. MIDI 2.0 and the Universal MIDI Packet (UMP)

- Foundational MIDI 2.0 specs released **February 2020**; notable updates adopted by The MIDI Association and AMEI on
  **29 May 2023** [S9]. MIDI 2.0 extends, not replaces, MIDI 1.0; translation to/from MIDI 1.0 is a design priority [S8].
- **UMP**: packets of 32, 64, 96 or 128 bits; **16 Groups × 16 channels**; a UMP stream can carry MIDI 1.0 protocol
  messages or MIDI 2.0 protocol messages [S9].
- MIDI 2.0 channel voice messages: **16-bit velocity**; **32-bit** CC (128), Registered (16 384) and Assignable (16 384)
  controllers, poly/channel pressure and pitch bend; per-note pitch and per-note controllers; Attribute Type/Data on
  notes (e.g. articulation, tuning); optional Jitter-Reduction timestamps; Function Blocks [S9].
- **MIDI-CI** (Capability Inquiry) negotiates extended features between bidirectional devices, including **Profiles**
  (defined behaviours for an application) and **Property Exchange** (get/set device data "Resources") [S8].
- COSOUND relevance (2026): DAW insertion still goes overwhelmingly through MIDI 1.0 / SMF or DAW-native APIs; design the
  internal note model with float velocity (0–1) and float controller values so MIDI 2.0 resolution can be emitted later
  without loss (design decision).

## 7. Libraries for MIDI file I/O and theory

| Library | Purpose | License | Latest release (checked 2026-11-24) | Repo |
|---|---|---|---|---|
| mido | Low-level MIDI messages, ports, SMF read/write (ticks, meta messages) [S10][S11] | MIT [S12] | 1.3.3 (2024-10-25) | https://github.com/mido/mido |
| pretty_midi | High-level SMF handling in seconds: `PrettyMIDI`, `Instrument` (`program`, `is_drum`), `Note` (pitch, velocity, start, end), `write`, `note_name_to_number` (C4 = 60), `note_number_to_hz` [S13][S14] | MIT [S13] | 0.2.11 (2025-10-08) | https://github.com/craffel/pretty-midi |
| music21 | Computer-aided musicology toolkit: pitches, intervals, scales, chords, Roman numerals, keys, analysis [S15][S16]; MIDI/MusicXML import-export (to verify in the I/O docs; C4 = 60 confirmed in `pitch.py`) | BSD-3-Clause [S15] | v10.5.2 (2026-10-28) | https://github.com/cuthbertLab/music21 |

Suggested split (design decision): music21 for theory validation (is this note in key? chord → Roman numeral),
mido or pretty_midi for writing the .mid file. All three licenses permit commercial use with attribution.
