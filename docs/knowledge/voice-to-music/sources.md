# Voice-to-Music – source registry

| ID | Source | URL | Retrieved | Software version | Reliability | Limitations |
|---|---|---|---|---|---|---|
| S1 | OpenAI Whisper – GitHub README + LICENSE | https://github.com/openai/whisper | 2026-11-24 | v20250625 (latest release); code + weights MIT | B | Speech model; not designed for singing/humming transcription |
| S2 | Radford et al., "Robust Speech Recognition via Large-Scale Weak Supervision" (arXiv 2212.04356) | https://arxiv.org/abs/2212.04356 | 2026-11-24 | paper v1 (2022) | B | Evaluates speech only |
| S3 | whisper.cpp – GitHub README | https://github.com/ggml-org/whisper.cpp | 2026-11-24 | v1.9.6 (2026-11-23), MIT | B | Core ML path accelerates the encoder only; first run compiles the model (slow) |
| S4 | Apple – SFSpeechRecognizer | https://developer.apple.com/documentation/speech/sfspeechrecognizer | 2026-11-24 | macOS 10.15+ | A | Some languages need network |
| S5 | Apple – SFSpeechRecognizer.supportsOnDeviceRecognition | https://developer.apple.com/documentation/speech/sfspeechrecognizer/supportsondevicerecognition | 2026-11-24 | macOS 10.15+ | A | Per-locale |
| S6 | Apple – SpeechAnalyzer | https://developer.apple.com/documentation/speech/speechanalyzer | 2026-11-24 | macOS 26.0+ | A | Requires model assets via AssetInventory |
| S7 | Microsoft – Windows.Media.SpeechRecognition.SpeechRecognizer | https://learn.microsoft.com/en-us/uwp/api/windows.media.speechrecognition.speechrecognizer | 2026-11-24 | Windows 10 (10.0.10240)+ | A | Page does not state offline vs online per constraint type (to verify); desktop (non-packaged) app usage not stated (to verify) |
| S8 | de Cheveigné & Kawahara, "YIN, a fundamental frequency estimator for speech and music", JASA 2002, DOI 10.1121/1.1458024 | https://doi.org/10.1121/1.1458024 | 2026-11-24 | – | B | Metadata verified via Crossref; full text not read |
| S9 | Mauch & Dixon, "PYIN: A fundamental frequency estimator using probabilistic threshold distributions", ICASSP 2014, DOI 10.1109/ICASSP.2014.6853678 | https://doi.org/10.1109/ICASSP.2014.6853678 | 2026-11-24 | – | B | Metadata verified via Crossref; full text not read |
| S10 | librosa – source (`core/pitch.py`: `yin`, `pyin`; `beat.py`: `beat_track` citing Ellis 2007, `plp` citing Grosche & Müller 2011) | https://github.com/librosa/librosa | 2026-11-24 | 1.0.1 (2026-11-12), ISC | B | Online docs URLs returned 404 at retrieval; verified from source |
| S11 | Kim et al., "CREPE: A Convolutional Representation for Pitch Estimation" (arXiv 1802.06182, ICASSP 2018) | https://arxiv.org/abs/1802.06182 | 2026-11-24 | – | B | – |
| S12 | marl/crepe – GitHub | https://github.com/marl/crepe | 2026-11-24 | v0.0.7 (2018), MIT | B | Last release 2018; low maintenance |
| S13 | torchcrepe – GitHub / PyPI | https://github.com/maxrmorrison/torchcrepe | 2026-11-24 | 0.0.24 (PyPI), MIT | B | – |
| S14 | SwiftF0 – GitHub README | https://github.com/lars76/swift-f0 | 2026-11-24 | v0.3.0 (2026-09-25), MIT | B | Benchmark is the author's own; README states 14,386 params vs 95,842 in paper (model changed between versions) |
| S15 | Nieradzik, "SwiftF0: Fast and Accurate Monophonic Pitch Detection" (arXiv 2508.18440) | https://arxiv.org/abs/2508.18440 | 2026-11-24 | paper v1 (2025) | B | Preprint |
| S16 | Spotify basic-pitch – GitHub README | https://github.com/spotify/basic-pitch | 2026-11-24 | v0.4.0 (2024-08-16), Apache-2.0 | B | Last release 2024; Python 3.7–3.11 listed |
| S17 | Bittner et al., "A Lightweight Instrument-Agnostic Model for Polyphonic Note Transcription and Multipitch Estimation" (arXiv 2203.09893, ICASSP 2022) | https://arxiv.org/abs/2203.09893 | 2026-11-24 | – | B | – |
| S18 | Silero VAD – GitHub README | https://github.com/snakers4/silero-vad | 2026-11-24 | v6.3 (2026-11-23), MIT | B | Speech VAD, not a singing/speech classifier |
| S19 | Beat This! – GitHub README; Foscarin et al., ISMIR 2024 (arXiv 2407.21658) | https://github.com/CPJKU/beat_this | 2026-11-24 | v1.1.0 (2026-04-14), code + weights MIT | B | Some training data under restrictive licenses (README note) |
| S20 | madmom – LICENSE + `bin/` programs | https://github.com/CPJKU/madmom | 2026-11-24 | v0.16.1 (2018); code BSD-style, models CC BY-NC-SA 4.0 | B | Models non-commercial; PyPI release old (Python <3.10 per Beat This README) |
| S21 | Delgado et al., "A New Dataset for Amateur Vocal Percussion Analysis" (arXiv 2009.11737) | https://arxiv.org/abs/2009.11737 | 2026-11-24 | – | B | 4 classes only |
| S22 | Delgado et al., "Deep Embeddings for Robust User-Based Amateur Vocal Percussion Classification" (arXiv 2204.04646, SMC 2022) | https://arxiv.org/abs/2204.04646 | 2026-11-24 | – | B | Research, no packaged tool |
| S23 | "The Rhythm In Anything: Audio-Prompted Drums Generation with Masked Language Modeling" (arXiv 2509.15625, ISMIR 2025) | https://arxiv.org/abs/2509.15625 | 2026-11-24 | – | B | Generates audio, not MIDI; code/license not checked (to verify) |
| S24 | Apple – NSMicrophoneUsageDescription | https://developer.apple.com/documentation/bundleresources/information-property-list/nsmicrophoneusagedescription | 2026-11-24 | macOS 10.14+ | A | – |
| S25 | Apple – AVCaptureDevice.requestAccess(for:completionHandler:) | https://developer.apple.com/documentation/avfoundation/avcapturedevice/requestaccess(for:completionhandler:) | 2026-11-24 | macOS 10.14+ | A | – |
| S26 | Apple – com.apple.security.device.audio-input entitlement | https://developer.apple.com/documentation/bundleresources/entitlements/com.apple.security.device.audio-input | 2026-11-24 | macOS 10.7+ | A | – |
| S27 | Microsoft Support – Turn on app permissions for your microphone in Windows | https://support.microsoft.com/en-us/windows/turn-on-app-permissions-for-your-microphone-in-windows-94991183-f69d-b4cf-4679-c98ca45f577a | 2026-11-24 | Windows 10/11 | A | End-user doc, not developer API |
| S28 | Apple – Core ML | https://developer.apple.com/documentation/coreml | 2026-11-24 | macOS 10.13+ | A | – |
| S29 | ONNX Runtime – Execution Providers | https://onnxruntime.ai/docs/execution-providers/ | 2026-11-24 | onnxruntime 1.32.0 (PyPI) | B | CoreML EP listed as preview |
| S30 | Microsoft – What is Windows ML? | https://learn.microsoft.com/en-us/windows/ai/new-windows-ml/overview | 2026-11-24 | page dated 2026-11-13 | A | NPU EPs need Windows 11 24H2+ |
| S31 | Omnizart – GitHub README | https://github.com/Music-and-Culture-Technology-Lab/omnizart | 2026-11-24 | v0.6.3 (2026-05-31), MIT | B | README: incompatible with ARM macOS |
