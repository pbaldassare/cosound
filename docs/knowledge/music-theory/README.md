# Music Theory (Section 2)

Verified theory COSOUND uses to make coherent musical choices. Theory content is tool-independent; where a fact depends
on software (octave naming in DAWs) the version is stated. Citations `[Sn]` refer to [`sources.md`](sources.md).
Retrieved 2026-11-24.

Conventions used in every file:
- Pitch classes are integers 0–11 with C = 0 (C, C#/Db = 1, … B = 11).
- Formulas are given as **semitones above the root** (e.g. minor triad = 0-3-7) and as **step patterns** (W = 2, H = 1).
- Octave numbers use **scientific / ISO pitch notation (middle C = C4 = MIDI 60)** unless a DAW convention is named.
- Items marked **(derived)** were computed from sourced definitions (e.g. rotating a sourced scale); they were checked
  programmatically. Items marked **(convention – to verify)** are common practitioner usage without an authoritative
  source in our registry yet.

## Files

| File | Content |
|---|---|
| [notes-intervals.md](notes-intervals.md) | 12 notes, enharmonics, intervals, 12-TET frequency formula, MIDI numbering, DAW octave-naming table, note→MIDI→Hz table |
| [scales.md](scales.md) | Major, minor forms, pentatonics, blues, 7 modes, harmonic/melodic-minor modes, symmetric & "exotic" scales, mood/genre usage |
| [harmony.md](harmony.md) | Triads, sus, sevenths, extensions, add chords, inversions/voicings, diatonic chords per scale, function, progressions, cadences, tension, borrowing, pedals |
| [melody.md](melody.md) | Phrase construction, motifs, variation techniques, arpeggios, countermelody, bass–melody motion, chord tones vs non-chord tones |

## Quick reference (the facts COSOUND must never get wrong)

- Middle C is **MIDI note 60** in every system [S23][S24]; only its *name* varies (C3, C4 or C5) by vendor
  ([notes-intervals.md](notes-intervals.md#octave-naming-by-daw--vendor)).
- A4 = **MIDI 69 = 440 Hz**; f = 440 · 2^((m − 69)/12) [S32]; ISO 16 fixes A4 = 440 Hz [S22] (to verify – page 403).
- Major scale = 0 2 4 5 7 9 11 (W W H W W W H); natural minor = 0 2 3 5 7 8 10 (W H W W H W W) [S2].
- Major triad 0-4-7, minor 0-3-7, diminished 0-3-6, augmented 0-4-8 [S4].
- Diatonic major-key triads: I ii iii IV V vi vii°; natural minor: i ii° III iv v VI VII (derived from [S2][S4]).
