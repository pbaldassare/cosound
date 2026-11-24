# Melody

Citations refer to [sources.md](sources.md). Retrieved 2026-11-24.

## 1. Phrase construction

- **Period** (typically 8 bars) = **antecedent** (bars 1–4: basic idea + contrasting idea ending on a *weak* cadence, HC
  or IAC) + **consequent** (bars 5–8: basic idea restated, then a contrasting idea ending on a *strong* cadence, PAC).
  Two cadences: question → answer [S10].
- **Sentence** (prototype 8 bars) = **presentation** (bars 1–4: a 2-bar basic idea stated twice, possibly varied, over
  tonic prolongation) + **continuation** (bars 5–8: fragmentation, sequence, faster harmonic/surface rhythm) ending in a
  cadence [S11].
- Electronic-music mapping (convention – to verify): 4- and 8-bar loops are typically built as "call (bars 1–2) / response
  (bars 3–4)" = a compressed period; 8-bar builds often follow the sentence shape (idea, repeat, then fragmenting/
  accelerating toward the drop).

## 2. Motifs and development

A motif (basic idea) is a short, recognizable pitch-and-rhythm cell (1–2 bars). Variation techniques COSOUND can apply
(definitions standard; only *sequence* is covered by a fetched source):

| Technique | What changes | Example (motif A–C–E) |
|---|---|---|
| Repetition | nothing | A–C–E, A–C–E |
| Sequence | the model is repeated starting on different scale degrees; diatonic sequences keep scale-degree steps so interval sizes may change [S12] | A–C–E → B–D–F → C–E–G |
| Inversion | intervals mirrored (up ↔ down) | A–C–E → A–F♯–D (real) or A–F–D (diatonic in A minor) |
| Retrograde | order reversed | E–C–A |
| Rhythmic displacement | same notes moved by an offbeat amount (e.g. one 16th) | onset 0 → onset 1/16 |
| Augmentation / diminution | durations ×2 / ÷2 | 8ths → quarters |
| Fragmentation | only part of the motif repeated | A–C, A–C [S11] |
| Ornamentation | non-chord tones added (see §5) | A–B–C–D–E |
| Octave displacement | one note moved ±12 | A–C5–E |

## 3. Arpeggios

An arpeggio plays chord tones successively. Common patterns (convention): up (1-3-5-8), down, up-down, "Alberti"
(1-5-3-5), broken with octave (1-5-8-10), and trance-style 16th-note arps cycling chord tones over 1–2 octaves. Keep the
arp on chord tones of the *current* chord; change pitches at chord changes, keep rhythm constant.

## 4. Countermelody and bass–melody interaction

- Two lines are independent when they differ in motion type; the four motion types are parallel, similar, contrary and
  oblique [S14]. Contrary motion between bass and melody gives the strongest independence (common-practice convention –
  to verify); oblique motion (one line holds, e.g. pedal bass, while the other moves) is idiomatic in electronic music.
- Countermelody rules (convention – to verify): fill rhythmic gaps of the main melody (call-and-response), use a different
  register (≥ an octave apart, or between bass and lead), prefer 3rds/6ths against the melody on strong beats, avoid
  doubling the melody in parallel octaves for long spans.

## 5. Melody vs chords: chord tones and non-chord tones

- Default rule (convention): strong beats → chord tones (root, 3rd, 5th, plus 7th/9th when the chord contains them);
  weak beats → chord tones or embellishing tones.
- Embellishing (non-chord) tones [S7]:

| Type | Approach | Leave | Metric position |
|---|---|---|---|
| Passing tone | step from chord tone | step, same direction, to another chord tone (fills a 3rd) | usually weak |
| Neighbor tone | step from chord tone | step back to the same chord tone | usually weak |
| Suspension | held over (common tone) from the previous chord into a new harmony | resolves **down** by step | strong |
| Retardation | like suspension | resolves **up** by step | strong |
| Anticipation | arrives early | repeated as chord tone of the next harmony | weak, before change |
| Appoggiatura | leap (usually up) | step in opposite direction | accented |
| Escape tone | step (usually up) | leap in opposite direction | unaccented |

- Tendency tones: leading tone (ti) → do; le → sol; chord sevenths resolve down by step [S20]. Ending a phrase on scale
  degree 1 over the tonic = PAC-like closure; ending on 2 or 7 over V = open/half-cadence feeling [S6].

## 6. Practical melody heuristics for generation (convention – to verify)

- Range: a hook usually spans ≤ an octave + a 3rd; mostly stepwise motion with occasional leaps; after a leap > P4,
  move by step in the opposite direction ("gap fill").
- Repetition is a feature: repeat the motif ≥ 2× before varying (sentence logic [S11]).
- One climax (highest note) per phrase, typically in the second half.
- In minor/dark material, the ♭6 → 5 and ♭2 → 1 (Phrygian) semitone moves give "darkness"; the raised 7 → 1 gives drive.
