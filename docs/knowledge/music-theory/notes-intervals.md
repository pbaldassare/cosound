# Notes, intervals, frequencies and MIDI numbering

Citations refer to [sources.md](sources.md). Retrieved 2026-11-24.

## 1. The 12 notes and enharmonics

- Seven letter names A–G; accidentals sharp (♯, +1 semitone) and flat (♭, −1 semitone) [S13]. Double sharp/flat
  (±2) exist for correct spelling (e.g. D♭♭ is a spelling of C) [S13].
- **Enharmonic equivalence**: two spellings that occupy the same key in 12-TET (C♯ = D♭) are enharmonically equivalent
  [S13]. Spelling still matters for theory: in A harmonic minor the 7th is **G♯**, not A♭, because each letter is used
  once per diatonic scale (derived from [S2]).
- **Semitone (half step)** = smallest step (adjacent keys); **tone (whole step)** = 2 semitones; **octave** = 12
  semitones (frequency ratio 2:1) [S1][S2].
- Octave numbers change at **C**: B3 is followed by C4; C♭4 sounds the same as B3 [S13].

| PC | Sharp name | Flat name |
|---|---|---|
| 0 | C (B♯) | C |
| 1 | C♯ | D♭ |
| 2 | D | D |
| 3 | D♯ | E♭ |
| 4 | E (F♭) | E |
| 5 | F (E♯) | F |
| 6 | F♯ | G♭ |
| 7 | G | G |
| 8 | G♯ | A♭ |
| 9 | A | A |
| 10 | A♯ | B♭ |
| 11 | B (C♭) | B |

**COSOUND spelling rule (convention):** spell notes from the key: sharp keys (G, D, A, E, B, F♯ major and their relative
minors) use sharps; flat keys (F, B♭, E♭, A♭, D♭ major / D, G, C, F, B♭ minor) use flats. Never mix ♯ and ♭ inside a
diatonic scale.

## 2. Intervals

Interval name = generic size (count letter names inclusively) + quality: perfect (P), major (M), minor (m), augmented
(A), diminished (d). Unisons, 4ths, 5ths, octaves are perfect-type; 2nds, 3rds, 6ths, 7ths are major/minor-type.
Augmented = one semitone wider than M or P; diminished = one semitone narrower than m or P [S1].
Inversion: semitone sizes sum to 12, generic sizes to 9; M↔m, A↔d, P↔P [S1].

Harmonic consonance classes (common-practice) [S1]: **perfect consonances** P1, P5, P8; **imperfect consonances** m3,
M3, m6, M6; **dissonances** m2, M2, all A/d intervals (incl. tritone), m7, M7, and the **P4** when it sounds against the
bass. Melodically, P4 and steps are consonant [S1].

| Semitones | Main name | Enharmonic alt. | Ratio idea (just, for reference) | Harmonic class [S1] |
|---|---|---|---|---|
| 0 | P1 unison | d2 | 1:1 | perfect consonance |
| 1 | m2 | A1 | 16:15 | dissonant (sharpest) |
| 2 | M2 | d3 | 9:8 | dissonant (mild) |
| 3 | m3 | A2 | 6:5 | imperfect consonance |
| 4 | M3 | d4 | 5:4 | imperfect consonance |
| 5 | P4 | A3 | 4:3 | consonant melodically; dissonant above bass |
| 6 | A4 / d5 (tritone) | — | 45:32 | dissonant (unstable, wants to resolve) |
| 7 | P5 | d6 | 3:2 | perfect consonance |
| 8 | m6 | A5 | 8:5 | imperfect consonance |
| 9 | M6 | d7 | 5:3 | imperfect consonance |
| 10 | m7 | A6 | 16:9 / 9:5 | dissonant (mild) |
| 11 | M7 | d8 | 15:8 | dissonant (sharp) |
| 12 | P8 octave | — | 2:1 | perfect consonance |

The "ratio idea" column is standard just-intonation approximation for intuition only (convention – to verify); 12-TET
intervals are exactly 2^(n/12).

Compound intervals (> octave) keep the simple interval's quality: C4–E5 is a M10 [S1]. Chord-extension names used in
[harmony.md](harmony.md) (derived): 13 = m9 (♭9), 14 = M9, 15 = A9 (♯9, = m10), 17 = P11, 18 = A11 (♯11),
20 = m13 (♭13), 21 = M13.

## 3. Frequencies (12-TET, A4 = 440 Hz)

- Reference pitch A4 = 440 Hz is fixed by **ISO 16:1975** [S22] (to verify: ISO page returned 403; content known from the
  ISO catalogue summary, which also says tuning instruments must be within ±0.5 Hz and the standard was confirmed 2022).
- MIDI note → frequency, as implemented in pretty_midi [S32]:

  **f(m) = 440 · 2^((m − 69) / 12)**   and inverse   **m = 69 + 12 · log2(f / 440)**

  Each semitone multiplies frequency by 2^(1/12) ≈ 1.059463; each cent is 1/100 semitone.
- Fractional m (e.g. after pitch bend) is valid in the formula [S32].

### Note → MIDI → Hz (one octave, middle C up; values computed with the formula, rounded to 0.01 Hz)

| Note (SPN) | Yamaha/Ableton/Logic-default name | MIDI | Hz |
|---|---|---|---|
| C4 (middle C) | C3 | 60 | 261.63 |
| C♯4/D♭4 | C♯3 | 61 | 277.18 |
| D4 | D3 | 62 | 293.66 |
| D♯4/E♭4 | D♯3 | 63 | 311.13 |
| E4 | E3 | 64 | 329.63 |
| F4 | F3 | 65 | 349.23 |
| F♯4/G♭4 | F♯3 | 66 | 369.99 |
| G4 | G3 | 67 | 392.00 |
| G♯4/A♭4 | G♯3 | 68 | 415.30 |
| A4 | A3 | 69 | 440.00 |
| A♯4/B♭4 | A♯3 | 70 | 466.16 |
| B4 | B3 | 71 | 493.88 |
| C5 | C4 | 72 | 523.25 |

Other anchors (computed): MIDI 0 = C−1 (SPN) = 8.18 Hz; MIDI 21 = A0 (lowest piano key) = 27.50 Hz; MIDI 24 = C1 =
32.70 Hz; MIDI 28 = E1 = 41.20 Hz; MIDI 33 = A1 = 55.00 Hz; MIDI 36 = C2 = 65.41 Hz; MIDI 48 = C3 = 130.81 Hz;
MIDI 108 = C8 = 4186.01 Hz; MIDI 127 = G9 = 12543.85 Hz.

Rule of thumb: MIDI number = 12 · (SPN octave + 1) + PC (C4 → 12·5 + 0 = 60) — same rule as pretty_midi's
`note_name_to_number` [S32] and music21's C4 = 60 [S33].

## 4. MIDI note numbering

- MIDI note numbers run 0–127 (7-bit data byte). The MIDI 1.0 Detailed Specification assigns **middle C the reference
  value 60** (the middle C of an 88-key piano, not necessarily the physical centre of a given keyboard) [S23]. General
  MIDI Level 1 repeats "Middle C = MIDI Key 60 (3CH)" [S24].
- The spec does **not** define an octave name for note 60; MIDI Association staff state middle C is always 60 but may be
  called C3, C4 or C5 [S25].
- Key transpose may map key 60 to a different physical key [S23]; COSOUND must reason in note numbers, not labels.

### Octave naming by DAW / vendor

| System | Name of MIDI 60 | Name of MIDI 0 | Evidence | Status |
|---|---|---|---|---|
| Scientific / ISO pitch notation, music textbooks | C4 | C−1 | OMT: middle C = C4 [S13] | verified (B) |
| pretty_midi, music21 (Python) | C4 | C−1 | source code [S32][S33] | verified (B) |
| Yamaha convention ("C3 = middle C") | C3 | C−2 | Logic manual names its C3 option "C3 (Yamaha)" [S26] | verified indirectly via Apple (A); no Yamaha doc fetched (to verify) |
| Logic Pro | **C3 by default**, switchable | C−2 (C3 mode) / C−1 (C4 mode) | "Display Middle C as: C3 (Yamaha) / C4 (Roland)" in Preferences > Display; manual calls C3 the standard mode [S26] | verified on Logic 10.2.1 (A); current Logic version (to verify) |
| Roland convention | C4 | C−1 | Logic's "C4 (Roland)" option [S26] | verified indirectly (A) |
| Ableton Live 12 | C3 | C−2 | Piano-roll ruler spans "C-2–C8" (0–127 ⇒ 60 = C3) [S27]; Ableton staff: "Live uses the C3 standard" [S28] | verified (A, inferred) |
| Cubase | C3 (default "MIDI" note-name style) | C−2 | Steinberg forum users; option at Preferences > Event Display > MIDI > Note Name Style [S30] | (to verify) – no official manual page found |
| FL Studio | C5 | C0 | Manual: default sampler Root Key is C5 [S29] (root key = 60 by design) | partially verified (A, inferred) – (to verify) explicit statement |
| REAPER | C4 by default; "MIDI octave name display offset" preference (−1 ⇒ C3) | — | user reports [S31] | (to verify) |
| Pro Tools | — | — | not researched | (to verify) |

**Implication for COSOUND:** store and transmit only MIDI numbers; when talking to the user, render names with the
convention of *their* DAW (Ableton/Logic/Cubase → C3 = 60, FL → C5 = 60, textbooks → C4 = 60) and say so once, e.g.
"A1 in Ableton's naming (MIDI 45, 110 Hz)". Note: the classic "kick on C1" producer convention is the Yamaha/Ableton
naming of MIDI 36, which is also GM "Bass Drum 1" (see `../midi-intelligence/`).
