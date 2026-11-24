# Audio Engineering – mixing and mastering (Section 4)

Knowledge COSOUND uses to act as a mix/mastering engineer for electronic music. Verified 2026-11-24.
Sources: [sources.md](sources.md).

**Reliability split.** Measurement definitions (LUFS, true peak, LRA, gating) come from published standards
(ITU-R BS.1770-5, EBU R 128 / Tech 3341 / Tech 3342 – reliability A). Platform targets come from official help
pages where they exist (Spotify – A) and otherwise from secondary sources (C) or are unverified (U). Mixing
technique comes from Sound On Sound, FabFilter and iZotope (C, some A for product manuals) and is **practice,
not law**. Numbers marked *convention* are typical values, not targets COSOUND may impose.

## Files

| File | Content |
|---|---|
| [mixing.md](mixing.md) | Gain staging, EQ, compression, sidechain, phase/polarity, stereo & mono, low end, kick–bass, reverb/delay, saturation, transients, balance |
| [mastering.md](mastering.md) | Loudness metrics (BS.1770-5, EBU), true peak, PLR, limiting, mastering EQ/compression, export formats, dither, platform normalisation targets, club loudness in practice |
| [measurement-analysis.md](measurement-analysis.md) | What COSOUND must measure before advising, how, and the "never claim unmeasured values" rule |

## Key numbers at a glance

| Item | Value | Status |
|---|---|---|
| Spotify normalisation | −14 LUFS integrated (Normal); Loud −11, Quiet −19; ITU 1770 | Official [S5] |
| Spotify true-peak advice | ≤ −1 dBTP; ≤ −2 dBTP if master louder than −14 LUFS | Official [S5] |
| Apple Music Sound Check | ≈ −16 LUFS | **Not published by Apple** [S6][S7]; secondary [S10] |
| YouTube | ≈ −14 LUFS (reduce-only) | Secondary/unverified [S10][S16][S25] |
| TIDAL | ≈ −14 LUFS, album-style normalisation | Secondary/unverified [S10][S24] |
| Amazon Music | ≈ −14 LUFS | Secondary [S10]; official page only shows a user toggle [S26] |
| AES TD1008 | music −16 LUFS (album: loudest track ≈ −14); speech −18 | Secondary summary of AES doc [S9][S10]; original not fetched [S8] |
| EBU R 128 (broadcast) | −23.0 LUFS (±1.0 LU for live); max −1 dBTP | Standard [S2] |
| DAW reference level | 0 VU ≈ −18 dBFS (also −20/−16 used) | Convention [S12] |
| Sub-bass mono | below ≈ 100 Hz mono; widening above ≈ 100–150 Hz if mono-safe | Convention [S21][S14] |
| Loud dance/electronic chart masters | −11.3 … −6.2 LUFS-I, avg −8.3 | Measured sample of 10 [S11] |
