# Composition guide – generating musical MIDI

How COSOUND turns musical intent into MIDI. Spec facts cite [sources.md](sources.md); theory cites
[`../music-theory/`](../music-theory/README.md). Everything labelled **Default** is a COSOUND design default (practitioner
heuristic, not from an authoritative source) – it must be tunable per user (Music Memory) and is a candidate for
validation against reference MIDI. Grid values assume **480 PPQ** (quarter = 480, 8th = 240, 16th = 120, 16th-triplet =
80, 32nd = 60). Note numbers are MIDI numbers; names use scientific pitch (C4 = 60) unless stated.

## 1. Register map (Default)

| Part | MIDI range | SPN | Notes |
|---|---|---|---|
| Sub / 808 bass | 24–43 | C1–G2 | root-only or root+octave; keep mono |
| Bassline (mid-bass, synth bass) | 28–55 | E1–G3 | most techno/house basslines sit 33–48 (A1–C3) |
| Chords / pads | 48–79 | C3–G5 | avoid close 3rds below ~48 (muddy); keep voicing span ≤ 2 octaves |
| Lead / topline | 60–88 | C4–E6 | |
| Arps / plucks | 55–91 | G3–G6 | |
| Bells / sparkles | 72–100 | C5–E7 | |
| Drums | per kit map | | GM: kick 36, snare 38, clap 39, closed hat 42, open hat 46, crash 49, ride 51 [S5] |

Low-interval limit rule (Default): below ~MIDI 48 use only octaves and fifths in the same voicing; 3rds added an octave
higher.

## 2. Melodies

1. Pick scale + tonic (see `music-theory/scales.md`); pick phrase length (2/4/8 bars).
2. Build a 1–2-bar motif: 3–6 notes, mostly steps, ≤ 1 leap > P4 (melody.md §6).
3. Place chord tones on strong beats (beat 1 and 3 in 4/4, or the strong 16ths of the groove); use passing/neighbor tones
   on weak positions (melody.md §5).
4. Develop with repetition → variation (sequence, rhythmic displacement, fragmentation) and finish phrases on stable
   degrees (1, 3, 5) for "closed", on 2/5/7 for "open".
5. Validate: every note in scale (unless an explicit chromatic tone is planned), range within the part's register,
   no semitone clashes with the sustained chord on strong beats (m2/M7 against chord root/3rd).

## 3. Basslines

- Root-anchored: bass plays chord roots on downbeats; approaches next root by step, octave, or 5th (Default).
- Genre patterns (Default; conventions to verify):
  - **Techno (rolling):** 16th-note pattern avoiding the kick (offbeat 16ths: positions 2, 3, 4 of each beat), root pedal
    with occasional ♭2/♭7/5 or octave jumps.
  - **House:** offbeat 8ths ("and" of each beat) or syncopated root–octave figures.
  - **Trance:** offbeat 8ths or "galloping" 16ths (x . x x).
  - **Acid/303:** 16ths with accents (velocity 110+), slides = overlapping notes (legato → portamento), octave jumps.
  - **Drum & bass / dubstep:** long sub notes + pitch-bend glides; Reese bass held across bars.
- Sidechain-aware rhythm: avoid bass note onsets exactly on kick hits when the user asks for "clean low end" (Default).
- Bass vs melody: favour contrary or oblique motion ([S14] in music-theory).

## 4. Chords and voicings

- Pick progression from harmony.md §6 matched to mood; one chord per bar is the dance-music default, 2 per bar for
  pop/house, 1 per 2 bars for techno/ambient (Default).
- Voicing (Default): bass note in bass part; chord part uses close position 3–5 notes in C3–G5 (48–79); apply **voice
  leading** – choose the inversion of each next chord that minimises total semitone movement while keeping common tones
  (music-theory/harmony.md §2).
- Colour: add 7ths/9ths for deep house/lo-fi, sus2/sus4 for trance/progressive "open" sound, power-chord stacks (root+5+8)
  for aggressive styles.
- Stabs: short (1/16–1/8) chord hits on offbeats (deep house, dub techno).

## 5. Arpeggios and sequences

- Arp = chord tones cycled at a fixed rate (1/16 typical, 1/8 for slower, 1/16-triplet for trance gates); modes up,
  down, up-down, random-in-chord, with octave range 1–3 (Default).
- Step sequences: 16-step patterns per bar, each step on/off + pitch + velocity + optional tie/slide; polymeter (e.g.
  3-step motif over a 16-step bar, repeating every 3 bars) for hypnotic techno (Default).

## 6. Drum patterns (4/4, 16 steps per bar; Default)

| Style | Kick | Snare/Clap | Closed hat | Open hat |
|---|---|---|---|---|
| House / techno (four-on-the-floor) | 1, 5, 9, 13 | 5, 13 (clap) | 3, 7, 11, 15 (offbeat 8ths) or all 16ths | 3, 7, 11, 15 |
| Trap / hip-hop (half-time feel) | 1, (8/11 variants) | 9 | 8ths/16ths with 1/32 & triplet rolls | — |
| Breakbeat | 1, 11 | 5, 13 | 8ths | — |
| Drum & bass (≈170–175 BPM) | 1, 11 | 5, 13 | 8ths | — |

Use GM numbers [S5] unless the user's kit mapping is known (Ableton Drum Rack: one note per pad, any of 128 [S18]).

## 7. Velocity shaping (Default)

- Range 1–127; 64 ≈ mezzo-forte [S1]. Never write velocity 0 in Note On (it means Note Off [S1]).
- Accent hierarchy in 4/4: downbeat 100–120, backbeat 95–115, offbeat 8ths 80–100, 16th ghosts 40–70.
- Hats: alternate strong/weak (e.g. 100/70) for groove; ghost snares 30–50.
- Melodic phrases: slight crescendo toward the phrase peak, decrescendo at the end (±10–20).
- Sampled drums with velocity layers respond strongly; synth patches may ignore velocity – explain this to the user.

## 8. Note lengths, legato, staccato (Default)

- Legato: note end = next note start (or +1–20 ticks overlap); overlapping notes trigger glide on mono synths with
  legato/portamento (CC 65 / CC 5 [S3] where supported).
- Staccato: 25–50 % of the step length (e.g. 16th step 120 ticks → 30–60 ticks).
- Default gate: 80–90 % of step for plucks/basslines, 100 % for pads, drums 1/32–1/16 (drum samples ignore length).
- Leave ≥ 1 tick between repeated same-pitch notes so Note Off precedes the next Note On.

## 9. Quantization and swing

- Quantize = move note starts toward a grid; **strength/amount** 0–100 % (Ableton groove "Quantize": 100 % snaps to the
  Base grid, 0 % leaves positions) [S17]. Default grid: 1/16 for dance music; 1/16-triplet for trap hats.
- Swing (Default): delay every second grid subdivision. For 16th swing at 480 PPQ, offset = (swing% − 50 %) × 240 ticks
  where 50 % = straight and 66.7 % ≈ triplet feel; typical house swing 54–58 %, hip-hop 58–62 %.
- Ableton grooves add Timing, **Random** (random timing variation per voice, for humanising tight loops), and Velocity
  (−100…+100) parameters, scaled by a Global Amount up to 130 % [S17].

## 10. Humanization (Default)

| Parameter | Tight electronic | Loose / live feel |
|---|---|---|
| Timing randomisation | ±0–5 ms (≈ ±0–5 ticks at 124 BPM, 480 PPQ) | ±10–20 ms |
| Velocity randomisation | ±3–6 | ±8–15 |
| Note-length variation | ±0–5 % | ±10 % |

Never humanise kick on the grid in four-on-the-floor styles; humanise hats/percussion/keys first. Conversion: 1 tick =
60 / (BPM × PPQ) s → at 124 BPM, 480 PPQ ≈ 1.008 ms.

## 11. MIDI CC automation (Default + [S3])

- Filter sweep: CC 74 (brightness) ramps over 4–16 bars for builds; CC 1 for modulation/vibrato; CC 11 for swells (keep CC
  7 for static level); CC 64 for sustain on piano/pad parts [S3].
- Write CC points at ≤ 1/32 resolution for smooth ramps (or use DAW automation curves); reset controllers at the clip
  start (e.g. CC 74 = start value) so loops restart consistently.
- Pitch bend: 14-bit, centre 8192 [S2]; GM default range ±2 semitones [S5]; to bend +1 semitone at ±2 range send
  8192 + 4096 = 12288.

## 12. Algorithmic checklist: text request → MIDI

Example request: **"dark melodic techno bassline in A minor, 124 BPM"**.

1. **Parse intent** → part = bassline; genre = melodic techno; mood = dark; key = A minor; tempo = 124; length (not
   given) → default 4 or 8 bars, 4/4.
2. **Resolve defaults & ask only if critical** → if key/tempo missing, take from DAW project; otherwise use genre
   defaults (melodic techno: 120–126 BPM, minor/Phrygian).
3. **Choose scale** → A natural minor (A B C D E F G); "dark" → allow ♭2 (B♭) only as a Phrygian passing colour if
   the user asked "darker" (scales.md §6).
4. **Choose harmony context** → if the project has chords, read them; else imply i–VI–III–VII (Am–F–C–G) or a i–iv pedal
   vamp; the bassline follows chord roots (harmony.md §6).
5. **Register** → bass roots around A1 (MIDI 33, 55 Hz) to A2 (45, 110 Hz); keep ≤ C3 (48).
6. **Rhythm template** → rolling 16ths avoiding kick positions (steps 2–4, 6–8, …), 1-bar motif × 4 with variation in
   bar 4 (fill/octave jump).
7. **Pitch assignment** → root on most notes; 5th (E) and octave (A2) for movement; at chord changes move to new root
   (F1 = 29 → in range E1–G3 choose F2 = 41 if F1 too low for the sound); approach notes by step on the last 16th.
8. **Durations** → 16th gate 70–90 % (84–108 ticks at 480 PPQ); occasional legato slides if the synth is mono.
9. **Velocity** → accents 105–115 on the first note after each kick, others 80–95; ±4 random.
10. **Timing** → quantized 100 % to 1/16; swing 50–54 %; no humanisation on bass (tight techno).
11. **Automation (optional)** → CC 74 slow rise over 8 bars for build.
12. **Validate** → all notes in scale; range check; no overlapping same-pitch notes; no Note On velocity 0; bar count
    exact; End of Track present; tempo meta = 483 871 µs/qn; time-signature 4/4 (FF 58 04 04 02 18 08); key signature
    sf 0 mi 1 [S4].
13. **Write** → SMF format 1, 480 PPQ, track 0 = tempo map, track 1 = "Bass – A minor" on channel 1 (or insert via DAW
    API into the selected track).
14. **Explain** (see §13) and offer 2–3 one-click variations ("darker: Phrygian B♭", "more movement: octave jumps",
    "groovier: 56 % swing").

## 13. How to explain choices to the user

Template (short, musical, verifiable; in the user's language and DAW naming):

> **Key & scale:** A natural minor – dark but not harsh; I avoided the raised G♯ so it doesn't sound "classical".
> **Notes:** mostly the root A (A1 in Ableton naming = MIDI 45 = 110 Hz) with E and the upper octave for movement;
> it follows Am–F–C–G so it stays in key with your pads.
> **Rhythm:** rolling 16ths that leave space for the kick – cleaner low end with sidechain.
> **Feel:** tight 16th grid, small velocity accents after each kick.
> **Try:** "darker" (adds Phrygian B♭), "simpler" (offbeat 8ths), "longer" (8 bars with a variation).

Rules: name the scale/chords with Roman numerals *and* letter names; state octave naming once when mentioning note names
(DAW conventions differ – see `../music-theory/notes-intervals.md`); never claim a rule is universal when it is a genre
convention; flag any deliberate out-of-key note and why.

**Note on the example:** in Ableton/Logic (C3 = 60) naming MIDI 33 = A0 and MIDI 45 = A1; in scientific pitch they are
A1 and A2. COSOUND must translate before displaying.
