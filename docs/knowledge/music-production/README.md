# Music Production – rhythm, genres, structure (Section 3)

Knowledge COSOUND uses to reason like an electronic-music producer. Verified 2026-11-24. Sources: [sources.md](sources.md).

**Read this first – status of the numbers.** BPM ranges, bar-maps and "typical sounds" in this section are
**genre conventions**, reported by secondary sources (reliability C). They describe what is common, not what is
correct. COSOUND must present them as conventions ("tech house is usually around…"), never as rules, and should
prefer measuring the user's reference tracks (see `../audio-engineering/measurement-analysis.md`) over quoting a table.
Lines marked **(to verify)** have no checked source; lines marked **(derived)** are arithmetic from cited facts.

## Files

| File | Content |
|---|---|
| this README | Rhythm fundamentals: BPM, meter, grid, swing, syncopation, triplets, accents, quantization, polymeter, core electronic drum patterns as 16-step grids |
| [genres.md](genres.md) | 11 genre profiles (Techno, Melodic Techno, Progressive House, Progressive Techno, Deep House, Tech House, Afro House, Melodic House, Trance, Minimal, EDM incl. big room / electro / future house) |
| [arrangement.md](arrangement.md) | Sections, transitions, energy, phrase lengths, anti-loop techniques, bar-timeline templates per genre |

---

## 1. Tempo and meter

- **BPM** = quarter-note beats per minute. All genres in scope are in **4/4** with a four-on-the-floor or
  near-four-on-the-floor kick [S8][S9][S11][S12][S13].
- Broad tempo bands taught by Ableton: house ~115–130, techno/trance ~120–140, dubstep 135–145, drum & bass
  160–180 BPM [S1]. Subgenre ranges are in [genres.md](genres.md).
- Useful timing arithmetic **(derived)**: one 4/4 bar lasts `240 / BPM` seconds.

| BPM | 1 bar | 8 bars | 16 bars | 32 bars | 1/16 note |
|---|---|---|---|---|---|
| 120 | 2.000 s | 16.0 s | 32.0 s | 64.0 s | 125.0 ms |
| 124 | 1.935 s | 15.5 s | 31.0 s | 61.9 s | 121.0 ms |
| 128 | 1.875 s | 15.0 s | 30.0 s | 60.0 s | 117.2 ms |
| 132 | 1.818 s | 14.5 s | 29.1 s | 58.2 s | 113.6 ms |
| 138 | 1.739 s | 13.9 s | 27.8 s | 55.7 s | 108.7 ms |

  So a 7-minute club track at 128 BPM is ≈ 224 bars, at 124 BPM ≈ 217 bars **(derived)**.

## 2. The 16-step grid (notation used everywhere in this KB)

One 4/4 bar = 16 sixteenth-note steps. Beat positions: **1, 5, 9, 13** (beats 1–4). Off-beat 8ths ("&"):
**3, 7, 11, 15**. The remaining steps (2, 4, 6, 8, 10, 12, 14, 16) are the "e" and "a" 16ths.

```
step:   1 2 3 4 5 6 7 8 9 10 11 12 13 14 15 16
count:  1 e & a 2 e & a 3 e  &  a  4  e  &  a
```

DAW mapping: NI and MusicRadar tutorials write positions as `bar.beat.sixteenth`, e.g. `1.1.3` = step 3,
`1.4.3` = step 15 [S4][S8][S12]. Legend: `X` hit, `x` soft/ghost hit, `O` open hat, `.` rest.

## 3. Core electronic drum patterns

**Four-on-the-floor kick** – kick on every beat (steps 1, 5, 9, 13) [S8][S9][S11][S12][S13].
**Clap/snare on 2 and 4** – steps 5 and 13 [S4][S8][S9][S11][S12].
**Off-beat hats** – hat on the "&" of every beat (steps 3, 7, 11, 15); the classic house/tech-house
closed or open hat placement [S4][S11][S12]. In techno, open 909-style hats sit on the off-beats while closed hats
run 16ths with the first 16th (the kick position) left empty [S5].

```
              1 2 3 4 5 6 7 8 9 10 11 12 13 14 15 16
Kick          X . . . X . . . X .  .  .  X  .  .  .     four-on-the-floor
Clap          . . . . X . . . . .  .  .  X  .  .  .     2 and 4
Open hat      . . O . . . O . . .  O  .  .  .  O  .     off-beat
Closed hat    . X X X . X X X . X  X  X  . X  X  X      16ths minus kick steps (techno) [S5]
```

**Rolling 16th bassline** – bass on 16ths with the first 16th of each beat rested (so the kick owns the
downbeat); typically saw/square, sidechained to the kick [S5]:

```
Bass (rolling) . X X X . X X X . X  X  X  .  X  X  X
```

**Off-beat "oompa" sub** – sine sub one octave down on the off-beat 8ths against the kick [S5]; trance uses
short off-beat bass notes "mostly between kicks", leaving some out for bounce [S9]:

```
Bass (offbeat) . . X . . . X . . .  X  .  .  .  X  .
```

**Tech-house bass example** (NI tutorial, 130 BPM): notes on 16ths 1, 5, 9, 12, 15 [S14] – note the 12 and 15
anticipations that create syncopation against the straight kick:

```
Bass (NI TH)   X . . . X . . . X .  .  X  .  .  X  .
```

**Shuffled hats / shakers** – produced by swing (section 4) or by hand-placing hits late: the NI deep-house
example nudges extra shaker hits "nearly halfway through the 16th note" [S11]; the NI progressive-house example
swings a string hit on a 1/32-triplet grid [S12].

## 4. Swing and groove

- **Definition (Linn/MPC)**: swing delays only the **second 16th of each 8th-note pair** (the even steps 2, 4,
  6…16). The percentage is the share of the 8th-note duration given to the first 16th [S2].
  - **50 %** = straight (no swing) [S2][S3].
  - **54 %** loosens straight 16ths "without it sounding like swing" [S2].
  - **66 %** = perfect triplet swing (first 16th gets 2/3 of the 8th) [S2].
  - Linn describes the 50–70 % region as full of useful settings; 62 % at 90 BPM felt looser than 66 % [S2].
  - MPC-style settings up to **75 %** are commonly cited by users; the 75 % maximum was **not** confirmed from
    Linn's own words (only a reader comment) **(to verify)** [S3].
- **Different DAW conventions** (as of 2013): Logic, MPC and Ableton's groove pool use 50 % = straight;
  Cubase and FL Studio use 0 % = straight. Attack's editors state 0 % FL/Cubase = 50 % Logic/MPC and 100 % FL =
  66.6 % Logic, implying roughly `Logic% ≈ 50 + FL%/6` [S3]. **COSOUND rule:** before recommending a swing
  number, name the DAW and its convention; current DAW behaviour must be checked in that DAW's manual folder
  (`../logic-pro/`, `../ableton-live/`, …) **(to verify per DAW version)**.
- **Swing in milliseconds (derived)**: delay of each even 16th = `(p − 0.5) × (30 / BPM)` s. At 124 BPM:
  54 % → 9.7 ms, 58 % → 19.4 ms, 62 % → 29.0 ms, 66 % → 38.7 ms.
- Swing is most commonly applied at **16th-note** resolution in dance music [S3].
- **Groove** more broadly = swing + micro-timing + velocity accents. Tutorials vary hat/shaker velocities
  (e.g. 89/100/90/93) to avoid a machine-gun feel [S12]; MusicRadar's techno guide says to vary closed-hat
  velocities "for groove" [S5].

## 5. Syncopation, triplets, accents

- **Syncopation** = emphasis on weak positions (even 16ths, "&"). In four-on-the-floor music it lives mainly
  in bass, percussion and stabs, because kick and clap stay on the beat: e.g. bass on 16th 12 and 15 [S14],
  syncopated shaker patterns in melodic techno [S8], syncopated grooves as a defining trait of afro house [S10].
- **Triplets** divide a beat into 3 (1/8T = 12 per bar, 1/16T = 24 per bar). Trance example: clap triplet
  flourish at the end of a loop [S9]. A 66 % swing places the even 16th on the 1/8-triplet grid [S2].
- **Accents** = velocity / level emphasis; see groove above. An accent bass hit on `1.4.3` (step 15) in a
  two-bar pattern is used in the melodic-techno example [S8].

## 6. Quantization

- Quantizing snaps notes to the grid; swing quantize (section 4) moves the even 16ths. Live-played parts keep
  timing/velocity variation deliberately: the NI afro-house tutorial records keys live for that reason [S10].
- **COSOUND default (convention, to verify per genre):** drums tight-quantized with genre-appropriate swing;
  percussion and keys either swung or partially quantized; never quantize away a user's intentional feel
  without asking.

## 7. Polyrhythm and polymeter

- **Polymeter** = layers of different loop lengths at the same tempo; e.g. kick lane 16 steps (4/4), clap lane
  12 steps (3/4), hat lane 14 steps (7/8), snare lane 20 steps (5/4). The pattern realigns only after the least
  common multiple of the lengths (here 1680 sixteenths = 105 bars) [S6].
- **Techno "3-over-4" device**: a 3-step (3/16) or 6-step motif looped against the 16-step bar so its accent
  lands on a different 16th each time; it realigns after 3 bars (48 steps) **(derived from [S6] method; genre
  usage to verify)**:

```
              1 2 3 4 5 6 7 8 9 10 11 12 13 14 15 16
Kick          X . . . X . . . X .  .  .  X  .  .  .
Perc 3-step   X . . X . . X . . X  .  .  X  .  .  X     (bar 1; bar 2 starts on step 3 of the motif)
```

- **Polyrhythm** (in strict terms) = different equal divisions of the same span (e.g. 3 against 4 per bar);
  dotted-8th delays (3/16 spacing) against a 4/4 kick produce a similar rolling effect – NI tutorials use
  dotted-8th delay on chords/organs [S11][S12].

## 8. COSOUND rules for rhythm advice

1. State BPM and swing advice **with its convention** (DAW swing scale, genre convention) and never as an absolute.
2. When the user provides a reference, measure its tempo/onset grid rather than relying on the genre table.
3. Describe patterns on the 16-step grid above so they can be turned into MIDI unambiguously.
