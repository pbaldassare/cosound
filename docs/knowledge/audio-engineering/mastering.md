# Mastering, loudness and delivery

Verified 2026-11-24. Sources: [sources.md](sources.md).

## 1. Loudness measurement – ITU-R BS.1770-5 (in force, Nov 2023)

- BS.1770-5 (11/2023) is the revision in force; −4 (2015) and earlier are superseded [S1].
- **Algorithm** [S1]: K-weighting filter → mean-square per channel → channel-weighted sum → loudness
  `L = −0.691 + 10·log10(Σ Gi·zi)` in **LKFS** (Loudness, K-weighted, relative to Full Scale; 1 dB level change =
  1 LKFS change). LUFS (EBU term) is the same unit [S2].
- **Gating for integrated loudness** [S1][S3]: 400 ms blocks overlapping 75 %; absolute gate −70 LKFS; relative
  gate 10 LU below the absolute-gated level. (The relative gate moved from −8 to −10 LU in 2011 [S2][S3].)
- **Time scales ("EBU Mode")** [S3]:
  - **Momentary (M):** sliding rectangular window 0.4 s.
  - **Short-term (S):** sliding rectangular window 3 s.
  - **Integrated (I):** gated average over the programme (start/pause/reset controls).
  - Meters must be able to show max momentary and max short-term values.
- **Loudness Range (LRA)** – EBU Tech 3342 [S4]: from short-term (3 s) loudness values (computed at ≥ 10 Hz),
  absolute gate −70 LUFS, relative gate −20 LU, **LRA = 95th percentile − 10th percentile** (in LU). Low
  percentile stops fade-outs dominating; high percentile stops a single loud event dominating.

## 2. True peak (dBTP)

- Sample peaks under-read the reconstructed waveform; BS.1770-5 Annex 2 measures **true peak** by (optional
  12.04 dB attenuation for integer maths) → **4× oversampling** (48 kHz → 192 kHz) with an interpolating FIR →
  absolute value → `20·log10` (+12.04 dB if attenuated) → **dBTP** [S1]. Higher oversampling ratios are
  preferred; at 96 kHz input 2× is sufficient; meters should run at ≥ 192 kHz effective rate [S1].
- EBU Tech 3341 adds compliance tolerances for true-peak meters [S3]; R 128 measurement tolerance ±0.3 dB
  (signals band-limited to 20 kHz) [S2].
- Why it matters for delivery: lossy encoding (AAC/Ogg) and D/A oversampling can produce overs if the master
  sits at 0 dBFS; Apple recommends **at least 1 dB headroom** [S7]; Spotify recommends ≤ −1 dBTP, and ≤ −2 dBTP
  for masters louder than −14 LUFS [S5].

## 3. RMS, crest factor, PLR, dynamic range

- **RMS** = root-mean-square level (unweighted); used by VU-style metering [S12]. Not the same as LUFS (no
  K-weighting, no gating) – COSOUND must label which one it reports.
- **Crest factor** = peak level − RMS level (dB) **(definition: general engineering; no checked source – to
  verify)**.
- **PLR (peak-to-loudness ratio)** = true peak (dBTP) − integrated loudness (LUFS) **(definition as used by
  iZotope's analysis; formal standard not identified – to verify)**. Measured examples in dance/electronic chart
  tracks: ≈ 6.4–11.2 dB [S11]. **PSR** (peak-to-short-term ratio) is the short-term analogue **(to verify)**.
- **LRA** (above) describes macro-dynamics between sections; dance/electronic chart sample LRA 3.5–13.8 LU,
  average 6.6 [S11].

## 4. Platform normalisation (what happens after upload)

| Platform | Target | Mode / notes | Status |
|---|---|---|---|
| **Spotify** | **−14 LUFS** integrated (ITU 1770) | Premium user settings: **Loud −11**, **Normal −14**, **Quiet −19** LUFS. Louder masters are turned down. Quieter masters get positive gain only up to what true peak allows, keeping 1 dB headroom for lossy (e.g. −20 LUFS with −5 dBTP is raised only to −16). "Loud" uses a limiter (−1 dB sample threshold, 5 ms attack, 100 ms decay). **Album** mode keeps the album's relative levels; **track** mode for playlists/shuffle. Loudness measured at upload, applied at playback. | **Official** [S5] |
| **Apple Music** (Sound Check) | ≈ −16 LUFS | Sound Check analyses each track, stores the value in metadata and plays songs at roughly equal loudness; can work per album; user-toggleable. Apple's pages **do not state a number**. | Feature: official [S6][S7]; −16: secondary [S10] **(to verify)** |
| **YouTube** | ≈ −14 LUFS | Turn-down only (quieter content not raised); visible as "content loudness" in Stats for nerds. YouTube Music reported to behave differently. | Secondary [S10][S16]; detail unverified [S25] |
| **TIDAL** | ≈ −14 LUFS | Album-style normalisation (loudest track reference); reported not to raise quiet tracks | Secondary [S10]; detail unverified [S24] |
| **Amazon Music** | ≈ −14 LUFS | Listener toggle "Loudness Normalization" exists | Target secondary [S10]; toggle unverified [S26] |
| Deezer | ≈ −15 LUFS | — | Secondary [S10] |
| SoundCloud, Beatport, Bandcamp | no normalisation (as reported 2023–2025) | DJ-store files play as mastered | Secondary [S10][S16] |
| **AES TD1008** (recommendation, 2021) | Music **−16 LUFS**; album normalisation with loudest track ≈ −14 LUFS; speech/assorted **−18 LUFS** | Recommendation to services, not a mastering target | Secondary summaries [S9][S10]; original not fetched [S8] **(to verify)** |
| **EBU R 128** (broadcast) | **−23.0 LUFS** (±1.0 LU only where not practically achievable, e.g. live; ±0.2 LU measurement tolerance in QC); max **−1 dBTP** | TV/radio, not streaming | Standard [S2] |

- **Integrated LUFS is a distribution level, not a mastering target** – Ian Shepherd's position; he suggests the
  loudest sections of a master no higher than about −10 LUFS short-term and judging the rest by ear [S16].
  This is one engineer's recommendation (C).
- Practical streaming peak ceilings from iZotope: lossless ≈ below −0.3 dBTP; lossy ≥ 1 dB true-peak headroom;
  loud masters ≤ −1 dBTP [S10].

## 5. Club-track loudness in practice (convention, not a standard)

- No standard exists for club playback; DJ stores (Beatport etc.) do not normalise [S16].
- Measured sample: 10 top-charting dance/electronic songs (2025) – integrated **−11.3 to −6.2 LUFS**, average
  **−8.3**; short-term maxima up to ≈ −4.5 LUFS; true peaks often **above 0 dBTP** (up to +1.5) [S11]. iZotope
  states that 2024's chart average was −8.3 LUFS and that loud and dynamic masters both succeed [S11].
- **COSOUND rule:** when a user asks "how loud should my techno master be?", answer with: (1) the platform
  normalisation facts above, (2) the measured loudness of *their* reference tracks, (3) the trade-off (louder =
  less PLR, more turn-down on streaming). Never state a single "correct" club LUFS.

## 6. Limiting, mastering EQ and compression

- At an ∞:1 ratio a compressor behaves as a limiter [S19]. True-peak limiters control the oversampled
  (inter-sample) peak that BS.1770 measures [S1] **(limiter internals vary by product – to verify per plug-in)**. iZotope's author uses a sample-peak limiter at −1 dBFS followed by a true-peak limiter at
  −0.3 dBTP [S10].
- Louder can be achieved at lower measured levels by lowering the limiter ceiling while keeping the same drive
  (Shepherd) [S16].
- Mastering EQ/compression: broad, small moves, reference-matched; techniques in [mixing.md](mixing.md) apply
  at bus level **(specific mastering EQ/compression numbers: to verify – no checked source)**.
- Stereo image at mastering: keep lows mono-compatible; check correlation [S15][S21].

## 7. Export formats, bit depth, sample rate, dither

- **PCM masters (WAV/AIFF):** Apple asks for **24-bit** sources at the **highest native sample rate available**
  (no upsampling needed); 16-bit/44.1 kHz is the CD standard; 48/88.2/96/192 kHz are common production rates [S7].
  Apple Music streams AAC (256 kbps VBR) and lossless ALAC [S7].
- **Dither:** only when reducing bit depth, **once**, as the very last step; at 24-bit flat TPDF is fine and
  inaudible; noise shaping matters at 16-bit or lower; 32-bit float files need none [S22]. Don't apply dither
  twice (plug-in + DAW export) **(to verify against current DAW manuals)**.
- **Lossy (MP3/AAC):** encode from the high-resolution master; leave ≥ 1 dB true-peak headroom because encoding
  can create overs [S5][S7][S10]. Bit-rate recommendations for DJ promos (e.g. 320 kbps MP3) **(to verify)**.
- **Formats for DJ stores:** 16- or 24-bit WAV/AIFF at 44.1 kHz is common practice **(to verify against each
  store's / distributor's current delivery spec)**.
