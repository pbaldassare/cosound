# Scales as interval formulas

Citations refer to [sources.md](sources.md). Formulas are semitones above the tonic; step patterns use semitone
counts (2 = W, 1 = H, 3 = augmented 2nd). All rotations were computed programmatically from the sourced parent scales
and double-checked (2026-11-24). Examples use C as tonic unless noted.

## 1. Core scales

| Scale | Semitones | Steps | Degrees vs major | Example (C) | Source |
|---|---|---|---|---|---|
| Major (Ionian) | 0 2 4 5 7 9 11 | 2 2 1 2 2 2 1 (W W H W W W H) | 1 2 3 4 5 6 7 | C D E F G A B | [S2] |
| Natural minor (Aeolian) | 0 2 3 5 7 8 10 | 2 1 2 2 1 2 2 (W H W W H W W) | 1 2 ♭3 4 5 ♭6 ♭7 | C D E♭ F G A♭ B♭ | [S2] |
| Harmonic minor | 0 2 3 5 7 8 11 | 2 1 2 2 1 3 1 | 1 2 ♭3 4 5 ♭6 7 | C D E♭ F G A♭ B | natural minor with raised 7th [S2]; steps derived |
| Melodic minor (ascending; "jazz minor") | 0 2 3 5 7 9 11 | 2 1 2 2 2 2 1 | 1 2 ♭3 4 5 6 7 | C D E♭ F G A B | raised 6th and 7th ascending [S2]; steps derived |
| Melodic minor (descending, classical) | = natural minor | | | | [S2] |
| Major pentatonic | 0 2 4 7 9 | 2 2 3 2 3 | 1 2 3 5 6 | C D E G A | major scale minus degrees 4 and 7 [S3] |
| Minor pentatonic | 0 3 5 7 10 | 3 2 2 3 2 | 1 ♭3 4 5 ♭7 | C E♭ F G B♭ | 5th rotation of major pentatonic (derived from [S3]) |
| Blues (minor hexatonic) | 0 3 5 6 7 10 | 3 2 1 1 3 2 | 1 ♭3 4 ♭5 5 ♭7 | C E♭ F G♭ G B♭ | minor pentatonic + ♭5 (convention – to verify; OMT blues page has no scale [S19]) |
| Major blues | 0 2 3 4 7 9 | 2 1 1 3 2 3 | 1 2 ♭3 3 5 6 | C D E♭ E G A | (convention – to verify) |
| Chromatic | 0–11 | all 1 | | | [S2] |

Classical practice treats the three minor forms as one minor key whose 6th/7th degrees vary by context [S2]. COSOUND
should do the same: natural minor for lines and loops, raised 7th when the V chord (major dominant) or a leading-tone
pull to the tonic is wanted.

## 2. The seven diatonic modes (rotations of the major scale)

Modes defined by starting the white-key collection on each note [S3]; formulas/steps derived. Brightness order (most
"bright"/raised to most "dark"/lowered): Lydian > Ionian > Mixolydian > Dorian > Aeolian > Phrygian > Locrian
(derived: each step lowers exactly one degree).

| Mode | White-key tonic | Semitones | Steps | Characteristic degree(s) | Tonic triad |
|---|---|---|---|---|---|
| Lydian | F | 0 2 4 6 7 9 11 | 2 2 2 1 2 2 1 | ♯4 | major |
| Ionian | C | 0 2 4 5 7 9 11 | 2 2 1 2 2 2 1 | (reference major) | major |
| Mixolydian | G | 0 2 4 5 7 9 10 | 2 2 1 2 2 1 2 | ♭7 | major |
| Dorian | D | 0 2 3 5 7 9 10 | 2 1 2 2 2 1 2 | ♮6 (vs minor) | minor |
| Aeolian | A | 0 2 3 5 7 8 10 | 2 1 2 2 1 2 2 | (reference minor) | minor |
| Phrygian | E | 0 1 3 5 7 8 10 | 1 2 2 2 1 2 2 | ♭2 | minor |
| Locrian | B | 0 1 3 5 6 8 10 | 1 2 2 1 2 2 2 | ♭2, ♭5 | diminished |

Solfège per mode in [S3] matches these formulas (e.g. Phrygian do ra me fa sol le te; Locrian do ra me fa se le te).
Locrian is uncommon outside jazz [S3] because its tonic triad is diminished.

## 3. Modes of harmonic minor (derived by rotation of 2 1 2 2 1 3 1)

| # | Common name | Semitones | Steps | Notes |
|---|---|---|---|---|
| 1 | Harmonic minor | 0 2 3 5 7 8 11 | 2 1 2 2 1 3 1 | |
| 2 | Locrian ♮6 | 0 1 3 5 6 9 10 | 1 2 2 1 3 1 2 | |
| 3 | Ionian ♯5 | 0 2 4 5 8 9 11 | 2 2 1 3 1 2 1 | |
| 4 | Dorian ♯4 ("Ukrainian Dorian") | 0 2 3 6 7 9 10 | 2 1 3 1 2 1 2 | |
| 5 | **Phrygian dominant** (a.k.a. Freygish, Spanish Phrygian) | 0 1 4 5 7 8 10 | 1 3 1 2 1 2 2 | Phrygian with major 3rd; tonic chord major/dom7 |
| 6 | Lydian ♯2 | 0 3 4 6 7 9 11 | 3 1 2 1 2 2 1 | |
| 7 | Ultralocrian (altered ♭♭7) | 0 1 3 4 6 8 9 | 1 2 1 2 2 1 3 | |

Names are conventional (to verify against a jazz-theory reference); the formulas are exact.

## 4. Modes of melodic minor (derived by rotation of 2 1 2 2 2 2 1)

| # | Common name | Semitones |
|---|---|---|
| 1 | Melodic minor | 0 2 3 5 7 9 11 |
| 2 | Dorian ♭2 | 0 1 3 5 7 9 10 |
| 3 | Lydian augmented | 0 2 4 6 8 9 11 |
| 4 | Lydian dominant (acoustic scale) | 0 2 4 6 7 9 10 |
| 5 | Mixolydian ♭6 | 0 2 4 5 7 8 10 |
| 6 | Locrian ♮2 | 0 2 3 5 6 8 10 |
| 7 | Altered (super-Locrian) | 0 1 3 4 6 8 10 |

The acoustic scale is described in [S3] as derived from the overtone series; its identity with Lydian dominant is
derived (C D E F♯ G A B♭).

## 5. Symmetric and other scales

| Scale | Semitones | Steps | Source |
|---|---|---|---|
| Whole tone | 0 2 4 6 8 10 (only 2 distinct collections: WT0, WT1) | all 2 | [S3] |
| Octatonic half–whole | 0 1 3 4 6 7 9 10 | 1 2 1 2 1 2 1 2 | [S3] (3 distinct collections) |
| Octatonic whole–half | 0 2 3 5 6 8 9 11 | 2 1 2 1 2 1 2 1 | rotation of [S3] (derived) |
| Double harmonic major ("Byzantine", "Arabic") | 0 1 4 5 7 8 11 | 1 3 1 2 1 3 1 | (convention – to verify) |
| Hungarian minor | 0 2 3 6 7 8 11 | 2 1 3 1 1 3 1 | (convention – to verify) |

## 6. Mood / genre usage (convention – to verify)

These associations are practitioner consensus, not from a source in our registry; COSOUND may use them as defaults but
should present them as tendencies, never rules.

| Scale / mode | Typical character | Common electronic contexts |
|---|---|---|
| Natural minor / Aeolian | dark, serious, emotional | techno, trance, deep/melodic house, drum & bass, most "dark" requests |
| Dorian | minor but warmer/"cool" (raised 6th) | deep house, nu-disco, lo-fi, funk-influenced house |
| Phrygian | very dark, tense (♭2 against tonic) | dark/industrial techno, hard techno, dubstep, psytrance basslines |
| Phrygian dominant | exotic, Middle-Eastern/flamenco colour | psytrance, Middle-Eastern-flavoured house, trap melodies |
| Harmonic minor | dramatic, "epic", strong V→i | trance leads, hardstyle, cinematic electronica, trap |
| Melodic minor | sophisticated minor, jazzy | neo-soul, jazzy house, future beats |
| Mixolydian | bright but laid-back (♭7) | funky house, disco, rock-influenced EDM |
| Lydian | dreamy, floating (♯4) | ambient, cinematic, chill/future bass pads |
| Ionian / major | happy, open | pop-EDM, progressive house, future bass, tropical house |
| Major / minor pentatonic | simple, safe, no semitone clashes | hooks, toplines, plucks, chill; minor pent for riffs |
| Blues scale | gritty, vocal | breaks, funky basslines, ghetto house |
| Whole tone | ambiguous, dreamlike | transitions, risers, FX |
| Locrian | unstable, rarely a home key | used for tension passages, rarely as a full-song tonality |

**Safe default for "dark" requests:** Aeolian; escalate to Phrygian for "darker/industrial", Dorian for "dark but
groovy/warm", harmonic minor when the user wants a strong pull back to the tonic.
