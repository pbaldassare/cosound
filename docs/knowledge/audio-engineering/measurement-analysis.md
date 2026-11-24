# Measurement-based mix analysis

Verified 2026-11-24. Sources: [sources.md](sources.md). This file defines **what COSOUND must compute before it
gives mix or master advice**. It is a product rule built on the standards in [mastering.md](mastering.md).

## Hard rule – never claim unmeasured values

1. COSOUND may state a number about the user's audio (LUFS, dBTP, correlation, frequency balance, BPM, key…)
   **only if it measured it** on that audio in the current session, and must say which tool/algorithm and which
   time range produced it.
2. If a measurement is unavailable (no file, analysis failed, only a verbal description), COSOUND says so and
   gives conditional advice ("if your integrated loudness is above −8 LUFS, then…"), never an invented value.
3. Genre conventions and platform targets are **reference values**, always labelled as such and kept separate from
   measured values in the answer.
4. Every advice item should cite the measurement that triggered it (e.g. "true peak +0.8 dBTP → lower the
   limiter ceiling").

## Required measurements

| # | Measurement | Method / standard | Why | Typical trigger (convention unless cited) |
|---|---|---|---|---|
| 1 | **Integrated loudness (LUFS-I)** | BS.1770-5 K-weighting + gating (400 ms, 75 % overlap, −70 LKFS abs, −10 LU rel) [S1][S3] | Compare to platform normalisation and to references | Report vs Spotify −14 [S5]; show expected turn-down per platform |
| 2 | **Short-term max (LUFS-S max)** and **momentary max (LUFS-M max)** | 3 s / 0.4 s sliding windows [S3] | Loudness of the drop/peak sections; Shepherd's ≈ −10 LUFS-S guidance for loudest parts [S16] | Peak-section loudness vs reference |
| 3 | **Loudness range (LRA)** | EBU Tech 3342 (10th–95th percentile of gated short-term values) [S4] | Macro-dynamics between breakdowns and drops | Compare to reference; chart dance sample 3.5–13.8 LU [S11] |
| 4 | **True peak (dBTP max)** | BS.1770-5 Annex 2, ≥ 4× oversampling at 48 kHz (≥ 192 kHz effective) [S1] | Inter-sample overs, lossy-codec clipping | > −1 dBTP → warn for streaming [S5]; > −2 dBTP when LUFS-I > −14 [S5] |
| 5 | **Sample peak (dBFS max)** and **clip detection** | Count consecutive samples at/near full scale (≥ 2–3 consecutive at ±1.0 → likely clipped) **(threshold heuristic, to verify)** | Detect hard clipping in file or stems | Any run of full-scale samples → report time-codes |
| 6 | **PLR / crest factor** | PLR = dBTP − LUFS-I; crest = peak − RMS (per stem/bus) **(definitions to verify)** | Over-limiting vs dynamic headroom; transient health of drums | Compare to reference; dance chart PLR ≈ 6–11 dB [S11] |
| 7 | **Spectral balance vs reference** | Long-term average spectrum (e.g. 1/3-octave or smoothed FFT), loudness-matched to the reference before comparison **(method: COSOUND design, to verify)** | Tonal balance advice must be relative to a reference, not absolute | Deviations by band (sub < 60 Hz, low 60–250, low-mid 250–500 Hz, mids, presence, air) |
| 8 | **Stereo correlation (overall and per band)** | Correlation coefficient L/R, ideally per band (e.g. < 120 Hz, 120 Hz–2 kHz, > 2 kHz) [S15] | Mono compatibility, phase problems | Sustained negative → warn [S15] |
| 9 | **Stereo width per band (M/S ratio)** | Side/Mid energy ratio per band **(method to verify)** | Over-wide lows, narrow highs | Side energy in sub band → recommend mono below ≈ 100–150 Hz [S21][S14] |
| 10 | **Low-end mono check** | Sum to mono, compare low-band level of L+R vs stereo; correlation of < 100–150 Hz band [S21][S14] | Club systems sum lows to mono; vinyl cutting [S14][S21] | Level loss in mono or negative correlation in lows |
| 11 | **Kick/bass overlap** (if stems) | Per-stem spectrum + onset alignment; sidechain depth (bass level dip at kick onsets) | Headroom bottleneck, masking [S14] | Bass dip < 2 dB or > ~6 dB → comment (2–3 dB natural per [S14]) |
| 12 | **Tempo & beat grid / section map** | Onset/beat tracking; energy & novelty curve per bar | Arrangement advice by bar number (see `../music-production/arrangement.md`) | Section lengths not on 8/16-bar boundaries |

## Reporting format (minimum)

```
File: <name> | analysed range: 00:00–07:12 | sample rate / bit depth: 44.1 kHz / 24-bit
Integrated: −7.9 LUFS | S-max: −5.6 LUFS | M-max: −4.1 LUFS | LRA: 5.2 LU
True peak: +0.4 dBTP (4× OS) | Sample peak: −0.1 dBFS | Clipped runs: 0
PLR: 8.3 dB | Correlation (full / <120 Hz): +0.42 / +0.97
Reference: <ref> (loudness-matched) | biggest spectral deviations: +3 dB 200–400 Hz, −2 dB > 10 kHz
```
(Numbers above are a **format example only**, not data.)

## Notes for implementation (Prompt 03 hand-off)

- Use a BS.1770-5 / EBU Mode-compliant meter implementation and validate it against the EBU Tech 3341/3342 test
  signals (the documents define expected results, e.g. LRA test cases with ±1 LU tolerance) [S3][S4].
- Record algorithm, version and oversampling factor with every stored measurement so later advice can cite it.
