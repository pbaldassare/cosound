# Audio and Stem Analysis (Section 8)

Analyzing audio the user brings: WAV/AIFF/MP3/FLAC files, stems, loops, recordings,
AI-generated songs (Suno/Udio), exports from other DAWs. Verified 2026-11-24. Sources: `sources.md`.

> **License is the first filter.** Several popular MIR tools are AGPL or ship
> non-commercial (CC BY-NC-SA) models. COSOUND is a commercial desktop app, so these are marked.

## 1. Decoding

| Library | License | Notes |
|---|---|---|
| libsndfile 1.2.2 | LGPL-2.1 [S1] | WAV/AIFF/FLAC/Ogg/Opus; MP3 only if built with `ENABLE_MPEG` (mpg123 + LAME) [S1] |
| FFmpeg | LGPL-2.1+ by default; becomes GPL with `--enable-gpl` components (e.g. libx264); `--enable-nonfree` is not LGPL-compatible [S2] | LGPL compliance: dynamic linking, ship matching source + configure options, credit FFmpeg, EULA must not forbid reverse engineering; no commercial license exists; patent risk is mainly for commercial products [S2] |
| OS decoders (Core Audio / Media Foundation) | platform | Option to avoid bundling codecs **(to verify formats per OS)** |

→ Recommended: **libsndfile** (WAV/AIFF/FLAC) + LGPL-only, dynamically linked **FFmpeg** for MP3/other
formats, following the FFmpeg checklist [S2].

## 2. Stem / source separation

| Tool | License | Status | Notes |
|---|---|---|---|
| Demucs v4 (HTDemucs) | MIT [S4] | Meta repo **archived**; maintained fork `adefossez/demucs` released 4.1.0 (2026) but author states it is not actively developed [S3][S4] | 4 stems (drums/bass/vocals/other); 9.00 dB SDR on MUSDB-HQ (9.20 with sparse + fine-tune) [S4][S5]; experimental 6-stem model adds guitar/piano, piano has heavy bleeding [S4] |
| Open-Unmix | MIT [S6] | v1.3.0 (2024) | Older baseline |
| BS-RoFormer / Mel-Band RoFormer | papers [S7][S8]; MIT implementations [S10][S11] | Active community training (ZFTurbo, v1.0.23 Nov 2026) [S10] | Current high-quality architecture family; **checkpoint licenses vary and are mostly undocumented (to verify per model)** |
| python-audio-separator | MIT [S9] | v0.48.0 (Oct 2026), active | Wrapper running MDX-Net, VR, Demucs, MDXC/RoFormer models; macOS uses MPS (PyTorch) and CoreML EP (ONNX); Windows DirectML experimental [S9]. README asks to credit UVR [S9] |

→ Recommended: **Demucs htdemucs/htdemucs_ft** as the license-clean default (MIT code + weights
from the same author); **RoFormer via python-audio-separator** as a quality upgrade only after
clearing each checkpoint's license.

## 3. Beat, downbeat, tempo

| Tool | License | Notes |
|---|---|---|
| **Beat This!** | MIT code + weights [S14] | ISMIR 2024; beats + downbeats without DBN; v1.1.0 (2026) [S14] |
| librosa 1.0.1 | ISC [S13] | `beat_track` (Ellis DP), `plp`, `onset_detect`, tempo [S13] |
| madmom 0.16.1 | code BSD; **models CC BY-NC-SA 4.0** [S12] | DBN beat/downbeat, TCN tempo, etc. [S12]; non-commercial models; last PyPI release 2018 |
| Essentia | **AGPL-3.0**; commercial license from UPF-MTG [S15] | RhythmExtractor etc. |
| aubio | **GPL-3.0**, 0.4.9 (2019) [S33] | Lightweight onset/tempo |

→ Recommended: **Beat This!** (+ librosa for onsets/tempo curves).

## 4. Key detection

- **Essentia KeyExtractor**: HPCP + key profiles; profiles include krumhansl, temperley, edma,
  edmm, bgate (default) and more; outputs key, scale, strength [S16]. AGPL / commercial license [S15].
- **madmom KeyRecognition** (CNN) [S12] — non-commercial model.
- **librosa** provides chroma features (`chroma_cqt`, etc.) but no key estimator; key = correlate
  averaged chroma with major/minor templates (Krumhansl-style) — COSOUND code, ISC-clean [S13].
→ Recommended: own template-matching on librosa chroma, run on the **harmonic stems** (no drums).
Expect relative-major/minor and modal ambiguity; show confidence and let the user override.

## 5. Chord recognition

| Tool | License | Notes |
|---|---|---|
| madmom CNNChordRecognition / DCChordRecognition [S12] | non-commercial models | Major/minor vocab **(to verify)** |
| Chordino (NNLS Chroma Vamp plugin) [S18] | **GPL-2.0** | Classic DSP approach; last push 2020 |
| autochord 0.1.4 [S17] | Apache-2.0 | TensorFlow; unmaintained since 2023 |
| Omnizart chord mode [S19] | MIT | Not usable on ARM macOS [S19] |

Accuracy limits: no tool's current benchmark accuracy was verified here **(to verify)**. Treat chord
output as a draft: extended chords, inversions, slash chords and dense mixes are where automatic
recognition is weakest **(to verify with MIREX/ISMIR results)**. Running on separated harmonic
stems and constraining to the detected key is advisable. No permissively licensed, maintained
chord recognizer stands out → candidate for an in-house model.

## 6. Spectral, transient, loudness, pitch

- **Spectral descriptors** (centroid, rolloff, flatness, bandwidth, MFCC, chroma), STFT/CQT,
  onset strength, HPSS: librosa (ISC) [S13].
- **Transients/onsets**: librosa `onset_detect` [S13]; madmom SuperFlux/CNN onsets (models NC) [S12].
- **Loudness**: ITU-R BS.1770-5 (Nov 2023) is the in-force standard [S23]. **pyloudnorm** 0.2.0
  (MIT) implements BS.1770-4 integrated loudness and an LRA estimate per EBU Tech 3342 [S21];
  **libebur128** (MIT, v1.2.6) is the C option [S22]. Whether either implements -5 changes is
  **(to verify)**.
- **Pitch on monophonic stems**: SwiftF0 (MIT) [S25]; **polyphonic notes**: basic-pitch
  (Apache-2.0), best on one instrument at a time [S24].

## 7. Suno and Udio exports and terms

**Suno** (as of retrieval; Terms effective 2026-09-03):
- Downloads: MP3 on all plans; **WAV on Pro/Premier, suno.com web only**; **MIDI only on
  Premier, for songs created in Suno Studio**; video downloads exist; options vary by song/plan/device [S26].
- Stems: *Auto Split* up to 12 stems (Pro/Premier, 50 credits), *Split from Mix* single stem +
  complement (Pro/Premier, 10 credits/stem), *Advanced Split* from ~100 instruments (Premier,
  10 credits/stem) [S27]. Stem output format not stated [S27]. MIDI extraction from a stem in
  Studio (10 credits) seen only in a search snippet **(to verify)** [S32].
- Download quotas from 2026-09-03: Free 0, Pro 20/month, Premier 60/month; stems and re-downloads
  don't count extra [S28].
- Ownership/commercial use: Pro/Premier — Suno assigns its rights in Output, commercial use
  allowed for permitted downloads; Suno does not guarantee copyright vests in Output. Free —
  personal, non-commercial only. Remixes are non-commercial on every tier. Terms forbid using
  Output to train other AI models and forbid stream-ripping; users grant Suno a broad license to
  their Content [S29]. → **COSOUND must not use user-supplied Suno files to train models.**

**Udio**:
- Since the UMG partnership (2025-10-29) **downloading of audio, video and stems is disabled**
  (help article dated 2026-02-17); no re-enable date given [S30]. Current status in Nov 2026 and
  ownership/commercial terms **(to verify)** [S31]. Users may still have older Udio files
  downloaded earlier; treat their rights as user responsibility.

COSOUND UX: ask the source and plan of AI-generated audio; show a reminder that commercial rights
depend on the user's plan/terms. Not legal advice.

## 8. Audio material vs rebuildable musical material

Decide per stem whether to **convert to MIDI** (editable, re-voiceable) or **keep as audio**
(sample, chop, warp). COSOUND design guidance based on the tools' documented scope:

| Stem content | Rebuild as MIDI? | Tool |
|---|---|---|
| Clean monophonic line (bass, lead synth, solo vocal melody, whistle) | **Yes** — high confidence | SwiftF0 → notes [S25], or basic-pitch [S24] |
| Single polyphonic instrument (piano, guitar chords) clean | Partially — draft, needs cleanup | basic-pitch [S24] |
| Drums (separated) | **Yes for kick/snare/hi-hat** pattern; ghost notes, toms, cymbal detail weaker | ADTOF models (5 classes, **CC BY-NC-SA**) [S20]; Omnizart drum (MIT, not ARM mac) [S19]; or onset + classifier in-house |
| Dense pads, stacked synths, reverb tails | **No** — keep audio; extract chords/key as metadata only | §4–5 |
| Mixed/lead vocals with lyrics, doubles, harmonies | **No** for the audio itself; melody line can be extracted as a guide MIDI | SwiftF0 on isolated vocal |
| FX, risers, impacts, foley, textures | **No** — audio only | – |
| Heavily processed / distorted / pitch-bent leads | Usually no — pitch tracking unstable | – |
| Separation artifacts present (bleed) | Lower confidence on everything | Check stem quality first |

Rule: MIDI only when the result is **monophonic or sparsely polyphonic, pitched, and clean**;
everything else stays audio plus descriptive metadata (key, chords, tempo, loudness, spectrum).

## 9. Recommended analysis pipeline

1. Decode (libsndfile/FFmpeg-LGPL) → float, keep original rate [S1][S2].
2. Loudness / true-peak (pyloudnorm or libebur128) [S21][S22].
3. If full mix: separate with Demucs [S4] (RoFormer optional, license-checked [S9]).
4. Beats/downbeats/tempo: Beat This! [S14].
5. Key: chroma template on harmonic stems [S13]; chords: draft, user-confirmed.
6. Per-stem classification → MIDI-rebuildable vs audio (§8) → basic-pitch/SwiftF0/drum transcription.
7. Spectral descriptors for sound-design notes [S13].

## Limitations / uncertainties
- No permissively licensed, maintained chord or drum-transcription model verified; ADTOF and madmom
  models are non-commercial [S12][S20]; Essentia AGPL [S15].
- RoFormer checkpoint licenses unverified [S9][S10].
- Udio current terms unverified [S31]; Suno features change often [S26]–[S29].
