# Music Intelligence Engine – known limitations (Prompt 09)

2026-11-27. What the engine does not do, or does only partly. It declares these limits in its answers instead of
inventing results.

## Composition
1. **Rules, not a trained model.**
   - Melodies, basslines, arpeggios and progressions come from music theory rules, style profiles and seeds.
   - They are coherent and checked (key, chords, range, timing, melody/bass relations). Whether they are *beautiful*
     or *catchy* is not measured: that needs listening by the user.
   - A trained model could be added later, but only behind the same deterministic checks (Prompt 09 §21).
2. **Eleven style profiles**, all electronic or cinematic: techno, melodic techno, progressive house, deep house,
   house, tech house, minimal, EDM, trance, ambient, cinematic.
   - The theory engine knows many more scales and chords (jazz, modal, exotic), but there is no jazz, pop, hip-hop or
     classical profile yet. A user JSON file in `<app data>/music-styles/` can add one.
3. **Generation is in 4/4.**
   - Analysis reads any time signature from a file; generated files are always 4/4.
   - Odd meters (7/8, 5/4) and tempo changes inside a composition are not generated.
4. **Twelve-tone equal temperament only.** Microtonal scales are not supported; sung pitches are rounded to the
   nearest semitone (the deviation in cents is reported).

## Language
5. **The request parser works on keywords (IT/EN), not a language model.**
   - It understands parts, moods, keys, tempo, bars, styles and blends ("con elementi di…").
   - Negations and complex sentences ("senza basso", "come nella seconda strofa") are not understood.
   - Words it does not understand are **shown** to the user, never ignored silently. When the partner's Central
     Agent arrives it can pass structured input (parts, key, BPM, styles) and skip the parser.
6. **Engine messages are in English.** Labels in the app are in Italian and English; the detailed findings and
   choices written by the engine (for example "100% of the notes on beats are chord tones") are in English for now.

## Analysis
7. **One chord per bar.** MIDI analysis looks for the chord of each bar; faster harmonic rhythm (two chords per
   bar) is reduced to the strongest one.
8. **Tempo only from the file.** A MIDI file without a tempo event is reported as "not stated in the file"; the
   tempo is not guessed from note timing. Only the first tempo of the file is used.
9. **Chords from audio are not recognised** (engine M6, not built). The Reference Track bridge uses key and BPM
   measured by the Smart Library, and chords given by the user or by external sources, labelled as such.

## Voice
10. **No pitch tracking from audio yet** (engine M5, not built).
    - The Voice-to-Music bridge works on notes already estimated (frequency or MIDI, onset, length), for example
      from the partner's Agent or a future tracker.
    - The operations `transcribe_melody`, `transcribe_rhythm`, `transcribe_speech` and `classify_take` stay planned.

## DAW and playback
11. **The DAW receives a file, not live notes.**
    - COSOUND exports a Standard MIDI File (format 1) and the DAW Controller plans its import (`import_midi`).
    - Writing notes directly into a DAW project needs the DAW connectors (milestone M4).
12. **The preview in the app is simple.** The Composer page plays the parts with basic Web Audio sounds, not with
    the user's instruments. The real sound is in the DAW (or in the Plugin Intelligence host, Prompt 08).

## Memory
13. **Preferences are read, not learned here.**
    - With the "Preferenze musicali" permission, the engine reads key, BPM and style preferences from the shared
      memory (`memory_facts`), and only to fill what the request leaves open.
    - It does not write to memory by itself: learning stays with the Adaptive Workflow agent, as for Prompt 08.

## Platforms
14. The engine is pure Rust with no system dependencies; its tests run on macOS, Windows and Linux in CI. The
    Composer page was checked in a browser with real engine data, not yet inside the packaged app on Windows.
