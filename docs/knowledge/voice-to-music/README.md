# Voice-to-Music (Section 7)

How COSOUND turns a user's voice into musical material: sung/hummed/whistled melodies, a
vocalized bassline, a beatboxed rhythm, or a spoken description of an idea. Verified 2026-11-24.
Sources: `sources.md`. Anything marked **(to verify)** was not confirmed against a source.

## 1. Input types and what each needs

| User input | Core task | Primary technique |
|---|---|---|
| Spoken description ("dark 90 BPM trap beat in F minor") | Speech-to-text, then LLM parsing | ASR (Whisper / OS speech APIs) |
| Sung / hummed / whistled melody | Monophonic F0 tracking → note segmentation | pYIN / CREPE / SwiftF0, or basic-pitch |
| Vocalized bassline ("dum dum da-dum") | Same as melody, low register, then transpose | F0 tracker with low `fmin`; octave correction |
| Beatbox / mouth percussion | Onset detection + sound-class classification (kick/snare/hat) | Onset detector + small classifier (research-grade) |
| Clapping / tapping a tempo | Onset + tempo estimation | Onset detection, inter-onset intervals |

## 2. Speech recognition (verbal ideas)

- **Whisper** (OpenAI): code and model weights are MIT [S1]. The `turbo` model (809M params) is
  an optimized large-v3 with small accuracy loss; it is not trained for translation [S1].
  Accuracy varies strongly by language [S1][S2]. Italian and English both supported (multilingual
  models) [S1].
- **whisper.cpp** (C/C++ port, MIT, v1.9.6): runs on macOS (Metal, Accelerate, optional Core ML
  encoder on the Apple Neural Engine, reported >3x faster than CPU-only) and Windows (MSVC/MinGW,
  Vulkan, OpenVINO); includes built-in VAD [S3]. First Core ML run is slow because the ANE
  compiles the model [S3]. → **Recommended local ASR for both OSes.**
- **Apple Speech framework**: `SFSpeechRecognizer` (macOS 10.15+) — one language per recognizer;
  some languages need Internet [S4]. `supportsOnDeviceRecognition` tells whether the locale can
  run offline; only then is `requiresOnDeviceRecognition` honored [S5]. `SpeechAnalyzer`
  (macOS 26+) is the newer modular API (e.g. `SpeechTranscriber` module), results via
  `AsyncSequence`, model assets installed via `AssetInventory` [S6].
- **Windows**: `Windows.Media.SpeechRecognition.SpeechRecognizer` (Windows 10+) exists [S7];
  whether dictation runs fully offline and how it behaves in an unpackaged Win32 desktop app is
  **(to verify)**. Practical choice: whisper.cpp on Windows too, so behavior matches macOS.
- Design note: Whisper is a speech model [S2]; feeding it humming/music is out of scope and its
  behavior on non-speech is **(to verify)** — gate ASR with a VAD/classifier first (§7).

## 3. Pitch detection (F0) for sung / hummed / whistled input

| Method | Type | License / version | Notes |
|---|---|---|---|
| YIN | DSP (autocorrelation-difference) | paper [S8]; `librosa.yin` (ISC) [S10] | Fast, no model; octave errors on breathy input |
| pYIN | Probabilistic YIN + HMM | paper [S9]; `librosa.pyin` (librosa 1.0.1, ISC) [S10] | Gives voiced/unvoiced decision + probability; solid baseline |
| CREPE | CNN on raw waveform | paper [S11]; `marl/crepe` MIT v0.0.7 (2018) [S12]; `torchcrepe` MIT 0.0.24 [S13] | Accurate, heavier; original repo unmaintained since 2018 |
| SwiftF0 | Small CNN on spectrogram | MIT, v0.3.0 (2026-09) [S14]; paper [S15] | Range 46.875–2093.75 Hz (≈F♯1–C7), 16 kHz, 16 ms frames, per-frame confidence, streaming with 176 ms lookahead, deps only numpy + onnxruntime [S14]. Paper reports +12 pts over CREPE at 10 dB SNR and ~42x faster on CPU [S15]; README claims top F1 in author's 19-tracker benchmark [S14] (self-reported) |

→ **Recommended: SwiftF0** for live/real-time (ONNX, tiny, confidence output, MIT); **pYIN**
as DSP fallback/cross-check. Restrict `fmin`/`fmax` to the user's voice range to reduce
octave errors [S14].

## 4. Note transcription / audio-to-MIDI

- **Spotify basic-pitch** (Apache-2.0, v0.4.0, Aug 2024): instrument-agnostic, polyphonic,
  outputs MIDI with pitch bends; works best on one instrument at a time [S16]. Ships the model
  as TensorFlow plus Core ML, TFLite and ONNX conversions (Core ML default on macOS, ONNX default
  on Windows) [S16]. Model from the ICASSP 2022 paper [S17]. A TypeScript sibling exists [S16].
  Python support listed only up to 3.11 [S16] — embed via ONNX/Core ML rather than its Python
  package.
- **Monophonic path (preferred for voice)**: F0 track (SwiftF0/pYIN) → segment into notes
  (voicing + pitch stability + onsets) → median pitch per note → MIDI. Simpler, more controllable
  than a polyphonic model for a single voice. Segmentation rules are COSOUND design, not sourced.
- **Omnizart** (MIT, v0.6.3) has a vocal-melody transcription mode, but is documented as
  incompatible with ARM macOS [S31] → not suitable as a shipped dependency.

## 5. Onsets, beat, tempo

- librosa: `onset_detect`, `beat_track` (Ellis 2007 dynamic programming), `plp` (Grosche &
  Müller 2011 predominant local pulse) [S10].
- **Beat This!** (CPJKU, ISMIR 2024): beat + downbeat tracker without DBN post-processing; code
  and weights MIT, v1.1.0 [S19]. Designed for music recordings; on a solo hum it is **(to verify)**.
- madmom: rich onset/beat/tempo programs, but **models are CC BY-NC-SA 4.0 (non-commercial)**;
  commercial use needs a license from JKU [S20]. → avoid in a commercial product.

## 6. Beatbox / vocal percussion → drums

- Maturity: **research-grade**. The AVP dataset (28 amateur participants, 9,780 utterances,
  4 classes: kick, snare, closed hi-hat, open hi-hat) shows the task framing [S21]. Classifiers
  work best when trained on small **user-specific** data [S22] → plan a short per-user
  calibration ("make your kick sound 5 times").
- TRIA (ISMIR 2025) maps beatbox/tapping gestures to *drum audio* in a chosen timbre — useful
  inspiration, but outputs audio, not MIDI; code license **(to verify)** [S23].
- Recommended approach: onset detection → per-onset features/embedding → nearest-neighbour or
  small classifier against the user's calibration samples → quantize to grid → MIDI drum notes.
  No off-the-shelf permissively-licensed packaged tool was found.

## 7. Speech vs singing / humming discrimination

- **Silero VAD** (MIT, v6.3): detects speech, ONNX, <1 ms per 30+ ms chunk on one CPU thread,
  8/16 kHz [S18]. It is a speech-activity detector, not a speech-vs-singing classifier [S18].
- Practical heuristic (COSOUND design, unsourced): VAD marks voice activity; then use pitch
  stability from the F0 tracker (long stable voiced segments with discrete pitch steps → singing/
  humming; rapid pitch glides with frequent unvoiced consonants → speech). Also let the user pick a
  mode explicitly ("describe" vs "sing") — the most reliable option.

## 8. What can be recognized reliably vs unreliably

Assessment based on the cited tools' scope; quantitative claims beyond the sources are **(to verify)**.

| Feature | Reliability | Why |
|---|---|---|
| Pitch contour of a clean monophonic voice | High | Core task of pYIN/CREPE/SwiftF0 [S9][S11][S15] |
| Intervals (relative pitch) | High | Robust even if the user is out of tune in absolute terms |
| Absolute notes | Medium | Users drift off-pitch; snap to a key after estimating it |
| Note durations | Medium | Depend on segmentation of legato/glides |
| Key / scale | Medium on ≥8 bars, low on short phrases | Few notes → ambiguous (e.g. relative major/minor) |
| BPM of a short hum | **Low** | Few onsets, no steady pulse, expressive timing |
| Quantizing a hummed melody | **Low without a click** | Rubato breaks grid inference → record against a metronome/count-in |
| Phrasing / repetitions / variations | Medium | Can be found by comparing note sequences; COSOUND logic |
| Beatbox classes beyond kick/snare/hat | Low | Research covers 4 classes [S21] |
| Spoken intent (genre, mood, BPM words) | High in supported languages | Whisper [S1][S2] |

**Key design rule:** record with count-in + click when timing matters; ask the user to confirm
BPM/key rather than inferring them from a few seconds of humming.

## 9. Local processing on macOS / Windows

- **Core ML** runs models on-device across CPU, GPU and Neural Engine; models from other
  frameworks convert via coremltools [S28].
- **ONNX Runtime**: one model format for both OSes; execution providers include CoreML
  (marked preview), DirectML, OpenVINO, QNN [S29].
- **Windows ML**: Microsoft's maintained copy of ONNX Runtime, same ONNX APIs, x64/ARM64, CPU+GPU
  (DirectML) on all supported versions, NPU EPs require Windows 11 24H2+ [S30].
- **whisper.cpp** for ASR on both [S3]. SwiftF0, basic-pitch, Silero VAD all ship ONNX [S14][S16][S18].

## 10. Microphone permissions

- **macOS**: `NSMicrophoneUsageDescription` Info.plist key is required for any mic access
  (macOS 10.14+) [S24]. Starting capture or calling `requestAccess` without it raises an exception;
  until granted, the system delivers silent audio samples; the user can change it later in
  Settings [S25]. Hardened-runtime apps also need the `com.apple.security.device.audio-input`
  entitlement [S26].
- **Windows 10/11**: Settings › Privacy & security › Microphone: global "Microphone access",
  "Let apps access your microphone", and a **separate "Let desktop apps access your microphone"**
  toggle that governs Win32 desktop apps [S27]. COSOUND should detect a silent/blocked input and
  deep-link the user to these settings (how to detect programmatically: **to verify**).

## 11. Recommended pipeline

1. **Capture** (CoreAudio / WASAPI), 16 kHz mono copy for analysis, full-rate copy kept; optional
   count-in + click.
2. **VAD + mode** — Silero VAD [S18]; user-selected mode or pitch-stability heuristic (§7).
3. **Branch**:
   - Speech → whisper.cpp [S3] → text → LLM extracts intent (genre, BPM, key, instruments).
   - Melody/bass → SwiftF0 [S14] (pYIN fallback [S10]) → note segmentation → key estimate →
     optional snap-to-scale → quantize to click grid → MIDI. basic-pitch [S16] as alternate path.
   - Beatbox → onset detection [S10] → per-user classifier [S22] → drum MIDI.
4. **Confirm** — show detected notes/BPM/key and let the user correct before writing to the DAW.
5. All local; no audio leaves the machine by default.

## Limitations / open points
- No mature, permissively licensed beatbox-to-MIDI tool found.
- Windows built-in speech API offline/desktop behavior **(to verify)**.
- SwiftF0 accuracy numbers are author-reported [S14][S15].
- basic-pitch last released Aug 2024; Python support stops at 3.11 [S16].
