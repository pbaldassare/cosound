# Harmony

Citations refer to [sources.md](sources.md). Chord formulas are semitones above the root. Extended/sus/add chords are
built by stacking intervals defined in [S1] on triads/sevenths from [S4]; those rows are marked (derived). Diatonic
chord tables were computed programmatically from the scale formulas in [scales.md](scales.md) (2026-11-24).

## 1. Chord formulas

### Triads [S4]
| Quality | Symbol (C) | Semitones | Intervals above root |
|---|---|---|---|
| Major | C | 0 4 7 | M3, P5 |
| Minor | Cm | 0 3 7 | m3, P5 |
| Diminished | C° / Cdim | 0 3 6 | m3, d5 |
| Augmented | C+ / Caug | 0 4 8 | M3, A5 |

### Suspended (derived; 3rd replaced)
| Chord | Semitones | Notes |
|---|---|---|
| sus2 | 0 2 7 | C D G – no 3rd, neither major nor minor |
| sus4 | 0 5 7 | C F G – the 4th traditionally resolves down to the 3rd |
| 7sus4 | 0 5 7 10 | common dominant substitute |

### Seventh chords [S4]
| Quality | Symbol (C) | Semitones | Construction |
|---|---|---|---|
| Major 7th | Cmaj7 | 0 4 7 11 | major triad + M7 |
| Dominant 7th | C7 | 0 4 7 10 | major triad + m7 |
| Minor 7th | Cm7 | 0 3 7 10 | minor triad + m7 |
| Half-diminished | Cm7♭5 / Cø7 | 0 3 6 10 | dim triad + m7 |
| Fully diminished | C°7 / Cdim7 | 0 3 6 9 | dim triad + d7 |
| Minor-major 7th | CmMaj7 | 0 3 7 11 | minor triad + M7 (derived) |
| Augmented major 7th | Cmaj7♯5 | 0 4 8 11 | aug triad + M7 (derived) |
| 6 / m6 | C6 / Cm6 | 0 4 7 9 / 0 3 7 9 | triad + M6 (derived) |

### Extensions and add chords (derived)
| Chord | Semitones (full) | Typical practical voicing |
|---|---|---|
| maj9 | 0 4 7 11 14 | |
| 9 (dominant) | 0 4 7 10 14 | 5th often omitted |
| m9 | 0 3 7 10 14 | very common in house/lo-fi |
| 7♭9 / 7♯9 | 0 4 7 10 13 / 0 4 7 10 15 | dominant tension |
| add9 | 0 4 7 14 | triad + 9, no 7th |
| m(add9) | 0 3 7 14 | |
| 6/9 | 0 4 7 9 14 | |
| 11 (dominant) | 0 4 7 10 14 17 | 3rd usually omitted (P11 clashes with M3) → ≈ 9sus4 |
| m11 | 0 3 7 10 14 17 | all tones usable (P11 vs m3 is not a semitone clash) |
| maj7♯11 | 0 4 7 11 18 | Lydian colour |
| add11 / add4 | 0 4 7 17 / 0 4 5 7 | |
| 13 (dominant) | 0 4 7 10 14 17 21 | usually 0 4 10 14 21 (omit 5th and 11th) |
| maj13 | 0 4 7 11 14 21 | omit 11 |
| m13 | 0 3 7 10 14 17 21 | often 0 3 10 14 21 |

Omission conventions in the right column are practitioner usage (convention – to verify); the formulas are exact.

## 2. Inversions and voicings

- A chord is inverted when a chord member other than the root is in the bass; quality is preserved [S4].
- Figured-bass labels [S15]: triads root position 5/3 (no figure), 1st inversion 6 (6/3), 2nd inversion 6/4; seventh
  chords 7, 6/5, 4/3, 4/2 (or 2).
- **Close voicing**: chord tones within one octave above the bass. **Open voicing**: spread over more than an octave
  (e.g. root – 5th – 3rd an octave up) (convention – to verify).
- **Voice leading**: four motion types between voices – parallel, similar, contrary, oblique; distinguishing them is
  essential to good voice leading [S14]. Practical COSOUND rules (common-practice convention – to verify): keep common
  tones, move other voices by the smallest step, avoid parallel P5/P8 between outer voices when a "classical/clean"
  sound is wanted (EDM power-chord stacks intentionally ignore this).
- Functional dissonances (chord sevenths, etc.) should be prepared by common tone or step and resolve **down by step**
  when the function changes [S20]. The leading tone (ti) tends to resolve up to do, especially from dominant chords [S20].
- Register guidance for MIDI voicings is in `../midi-intelligence/composition-guide.md`.

## 3. Diatonic chords per scale (derived)

Roman numerals: upper case = major, lower case = minor, ° = diminished, + = augmented, ø7 = half-diminished.

| Scale | 1 | 2 | 3 | 4 | 5 | 6 | 7 |
|---|---|---|---|---|---|---|---|
| Major – triads | I | ii | iii | IV | V | vi | vii° |
| Major – 7ths | Imaj7 | ii7 | iii7 | IVmaj7 | V7 | vi7 | viiø7 |
| Natural minor – triads | i | ii° | III | iv | v | VI | VII |
| Natural minor – 7ths | i7 | iiø7 | IIImaj7 | iv7 | v7 | VImaj7 | VII7 |
| Harmonic minor – triads | i | ii° | III+ | iv | V | VI | vii° |
| Harmonic minor – 7ths | i(maj7) | iiø7 | III+maj7 | iv7 | V7 | VImaj7 | vii°7 |
| Melodic minor (asc.) – triads | i | ii | III+ | IV | V | vi° | vii° |
| Melodic minor (asc.) – 7ths | i(maj7) | ii7 | III+maj7 | IV7 | V7 | viø7 | viiø7 |
| Dorian – triads | i | ii | III | IV | v | vi° | VII |
| Phrygian – triads | i | II | III | iv | v° | VI | vii |
| Lydian – triads | I | II | iii | iv° | V | vi | vii |
| Mixolydian – triads | I | ii | iii° | IV | v | vi | VII |
| Locrian – triads | i° | II | iii | iv | V | VI | vii |
| Phrygian dominant – triads | I | II | iii° | iv | v° | VI+ | vii |

In minor keys the flat-side chords are often written relative to the major-key numerals (♭III, ♭VI, ♭VII), as in [S9].
Example A natural minor: Am Bdim C Dm Em F G; A harmonic minor: Am Bdim C+ Dm E F G♯dim.

## 4. Functional harmony

- **Tonic (T)**: I (also III, VI share tonic function); **Subdominant / predominant (S)**: II, IV; **Dominant (D)**:
  V, VII [S5]. Groupings are simplified – function depends on the scale degrees present [S5].
- Phrase model: **T → (S) → D → T**, simplest I–V–I, expanded to I–IV–V–I, I–ii–V–I, I–vi–ii–V–I [S16]. Root-position
  I marks tonic arrival; V with the leading tone points back to tonic [S16].
- In minor-key pop/rock, **♭VI–♭VII–i** commonly replaces iv–V–i as S–D–T (A minor: F–G–Am) [S9].

## 5. Cadences

| Cadence | Progression | Definition | Source |
|---|---|---|---|
| Perfect authentic (PAC) | V–I (root position), melody ends on 1 | strongest close | [S6] |
| Imperfect authentic (IAC) | V–I, melody ends on 3 (or 5) | weaker close | [S6] |
| Half (HC) | … → V (phrase ends on V) | open, "question" | [S6] |
| Deceptive | V → vi (VI in minor) instead of I | "fails" to cadence, extends phrase | [S17] |
| Plagal | IV → I | S prolonging T; "amen" | [S17][S18] |
| Phrygian half | iv6 → V in minor | (convention – to verify) | — |
| Aeolian "cadence" | ♭VI → ♭VII → i | pop/rock minor | [S9] |
| Double plagal | ♭VII → IV → I | | [S18] |

## 6. Progressions common in electronic music

Examples in A minor / C major. The patterns are Roman-numeral facts; their genre association is convention (to verify)
unless a source is given.

| Progression | A minor / C major example | Notes |
|---|---|---|
| i–VI–III–VII | Am–F–C–G | the "epic minor" loop (trance, progressive house, future bass) |
| i–VII–VI–VII | Am–G–F–G | Aeolian loop, never touches V |
| i–VI–VII–i | Am–F–G–Am | contains the Aeolian ♭VI–♭VII–i cadence [S9] |
| i–iv–VII–III | Am–Dm–G–C | circle-of-fifths motion in minor |
| i–iv–v–i / i–iv–V–i | Am–Dm–Em–Am / …E–Am | V (major, from harmonic minor) for a stronger pull |
| i–VI–iv–V | Am–F–Dm–E | harmonic-minor drama (trance, hardstyle) |
| i–iv (two-chord vamp) | Am–Dm | minimal techno / deep house |
| i–VII | Am–G | hypnotic two-chord loop |
| i–♭II | Am–B♭ | Phrygian tension (dark techno, dubstep) |
| I–V–vi–IV | C–G–Am–F | pop-EDM "axis" loop |
| vi–IV–I–V | Am–F–C–G | same loop started on vi – same chords as i–VI–III–VII |
| ii7–V7–Imaj7 / i9–iv9 | Dm7–G7–Cmaj7 / Am9–Dm9 | jazzy/deep house with 7th/9th colour |
| I–IV alternation | C–F | plagal oscillation [S18] |
| 12-bar blues | I I I I / IV IV I I / V IV I I | [S19] |

## 7. Tension and resolution

- Dissonant intervals (m2, M7, tritone) and functional dissonances (chord sevenths, leading tone in V) create tension that
  expects resolution by step [S1][S20]. The tritone in V7 (3rd and 7th of the chord) resolves inward/outward to the
  tonic triad (convention – to verify; standard common-practice teaching).
- Sources of tension usable in electronic music: dominant (V/V7) before the drop, suspended chords resolving (sus4→3),
  chromatic ♭II (Phrygian), pedal points under changing chords, rising sequences, unresolved half cadences at breakdown
  ends (convention – to verify).

## 8. Borrowed chords / modal interchange

- **Modal mixture** = using chords from the parallel key (e.g. F-minor chords in F major) without modulating [S8].
  Notation: keep the numeral, adjust quality; prefix ♭/♯ if the root is altered (♭VI) [S8].
- Common borrowings in major (from parallel minor): iv, ♭VI, ♭VII, ♭III, ii°/iiø7, i (convention – to verify; [S8]
  discusses ♭VI and the iv-family). In minor, the major I at the end is the "Picardy third" [S8].
- Electronic usage: I–♭VII–IV (Mixolydian borrowing), iv minor in a major chorus for bittersweet colour, ♭VI–♭VII–I
  "victory" cadence (convention – to verify).

## 9. Pedal points

A pedal point is a sustained or repeated note (usually tonic or dominant, normally in the bass) held while harmonies
above it change, creating tension when chords clash with it and release when they return to agreement (definition is
standard theory; not covered by our fetched OMT pages – to verify). In electronic music this is the default for
techno/trance: a repeating root-note bass (pedal) under moving pads or arps.
