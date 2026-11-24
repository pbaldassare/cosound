# Audio and Stem Analysis – source registry

| ID | Source | URL | Retrieved | Software version | Reliability | Limitations |
|---|---|---|---|---|---|---|
| S1 | libsndfile – GitHub README / LICENSE | https://github.com/libsndfile/libsndfile | 2026-11-24 | 1.2.2 (2023-08-13), LGPL-2.1 | B | MP3 only when built with `ENABLE_MPEG` (mpg123/LAME) |
| S2 | FFmpeg – License and Legal Considerations | https://ffmpeg.org/legal.html | 2026-11-24 | – | A | Not legal advice (page says so); patent questions jurisdiction-dependent |
| S3 | facebookresearch/demucs – README | https://github.com/facebookresearch/demucs | 2026-11-24 | archived repo, MIT | B | Repo archived; points to fork |
| S4 | adefossez/demucs – README + LICENSE; PyPI `demucs` | https://github.com/adefossez/demucs | 2026-11-24 | 4.1.0 (README news 2026-07-11), MIT | B | Author states not actively developed; slow replies |
| S5 | Rouard, Massa, Défossez, "Hybrid Transformers for Music Source Separation" (arXiv 2211.08553) | https://arxiv.org/abs/2211.08553 | 2026-11-24 | – | B | – |
| S6 | Open-Unmix (PyTorch) – GitHub | https://github.com/sigsep/open-unmix-pytorch | 2026-11-24 | v1.3.0 (2024-04), MIT | B | Low activity since 2024 |
| S7 | Lu et al., "Music Source Separation with Band-Split RoPE Transformer" (arXiv 2309.02612) | https://arxiv.org/abs/2309.02612 | 2026-11-24 | – | B | SDX23 challenge system paper |
| S8 | Wang et al., "Mel-Band RoFormer for Music Source Separation" (arXiv 2310.01809) | https://arxiv.org/abs/2310.01809 | 2026-11-24 | – | B | ISMIR 2023 late-breaking demo |
| S9 | python-audio-separator – README | https://github.com/nomadkaraoke/python-audio-separator | 2026-11-24 | v0.48.0 (2026-10-29), MIT | B | Individual model checkpoints (UVR community) have their own, often undocumented, licenses (to verify per model) |
| S10 | ZFTurbo Music-Source-Separation-Training – GitHub | https://github.com/ZFTurbo/Music-Source-Separation-Training | 2026-11-24 | v1.0.23 (2026-11-08), MIT code | B | Checkpoint licenses not verified |
| S11 | lucidrains/BS-RoFormer – GitHub | https://github.com/lucidrains/BS-RoFormer | 2026-11-24 | 1.1.0 (2026-02), MIT | B | Architecture implementation; no official weights |
| S12 | madmom – LICENSE and `bin/` programs | https://github.com/CPJKU/madmom | 2026-11-24 | v0.16.1 (2018-11); code BSD-style; models CC BY-NC-SA 4.0 | B | Last PyPI release 2018; last main commit 2024-08 |
| S13 | librosa – source | https://github.com/librosa/librosa | 2026-11-24 | 1.0.1 (2026-11-12), ISC | B | – |
| S14 | Beat This! – README; ISMIR 2024 paper arXiv 2407.21658 | https://github.com/CPJKU/beat_this | 2026-11-24 | v1.1.0 (2026-04), MIT code + weights | B | Some training data restrictively licensed (README) |
| S15 | Essentia – README/COPYING; Licensing page | https://github.com/MTG/essentia ; https://essentia.upf.edu/licensing_information.html | 2026-11-24 | PyPI 2.1b6.dev1438; AGPL-3.0; models CC BY-NC-SA 4.0 | B | Commercial use requires a license from UPF-MTG |
| S16 | Essentia – KeyExtractor reference | https://essentia.upf.edu/reference/std_KeyExtractor.html | 2026-11-24 | 2.1-beta6-dev | B | – |
| S17 | autochord – GitHub README / PyPI | https://github.com/cjbayron/autochord | 2026-11-24 | 0.1.4, Apache-2.0; last push 2023-04 | B | Unmaintained; TensorFlow; accuracy not stated in README |
| S18 | NNLS Chroma / Chordino Vamp plugin – c4dm GitHub; isophonics page | https://github.com/c4dm/nnls-chroma | 2026-11-24 | GPL-2.0; last push 2020-04 | B | GPL; Vamp plugin host needed |
| S19 | Omnizart – README | https://github.com/Music-and-Culture-Technology-Lab/omnizart | 2026-11-24 | v0.6.3 (2026-05-31), MIT | B | Incompatible with ARM macOS (README); drum model training bug noted |
| S20 | ADTOF – README/LICENSE; Zehren et al. ISMIR 2021; Signals 2023 DOI 10.3390/signals4040042 | https://github.com/MZehren/ADTOF | 2026-11-24 | CC BY-NC-SA 4.0; last push 2025-09 | B | Non-commercial |
| S21 | pyloudnorm – README | https://github.com/csteinmetz1/pyloudnorm | 2026-11-24 | v0.2.0 (2026-01-04), MIT | B | Implements BS.1770-4 (not -5) per README |
| S22 | libebur128 – GitHub | https://github.com/jiixyj/libebur128 | 2026-11-24 | v1.2.6 (2021), MIT | B | – |
| S23 | ITU-R BS.1770 recommendation page | https://www.itu.int/rec/R-REC-BS.1770 | 2026-11-24 | BS.1770-5 (11/2023) in force | A | – |
| S24 | Spotify basic-pitch – README | https://github.com/spotify/basic-pitch | 2026-11-24 | v0.4.0, Apache-2.0 | B | Best on one instrument at a time |
| S25 | SwiftF0 – README | https://github.com/lars76/swift-f0 | 2026-11-24 | v0.3.0, MIT | B | Monophonic only |
| S26 | Suno Help – "What types of files can I download?" | https://help.suno.com/en/articles/13926081 | 2026-11-24 | article undated ("Edited") | A | Options may vary by song/plan/device |
| S27 | Suno Help – "How do I get stems?" | https://help.suno.com/en/articles/13925185 | 2026-11-24 | undated | A | Output format of stems not stated |
| S28 | Suno Help – "Upcoming Changes FAQ: Downloads, Models, and Terms of Service" | https://help.suno.com/en/articles/13614785 | 2026-11-24 | refers to changes from 2026-09-03 | A | FAQ wording on Studio downloads slightly inconsistent |
| S29 | Suno Terms of Service | https://suno.com/terms | 2026-11-24 | effective 2026-09-03 (revised 2026-08-10) | A | Read via fetch summary; legal text, no legal advice |
| S30 | Udio Help – "Changes associated with the Universal Music Group (UMG) partnership" | https://help.udio.com/en/articles/12683565-changes-associated-with-the-universal-music-group-umg-partnership | 2026-11-24 | article dated 2026-02-17 | A | May be outdated by Nov 2026 |
| S31 | Udio Terms of Service | https://www.udio.com/terms-of-service | 2026-11-24 | – | U | Page body not retrievable (JS); ownership/commercial terms (to verify) |
| S32 | Suno Help – "Exporting from Studio" (MIDI from stem, 10 credits) | https://help.suno.com/en/articles/13925249 | 2026-11-24 | – | U | Seen only in search snippet; not opened |
| S33 | aubio – GitHub | https://github.com/aubio/aubio | 2026-11-24 | 0.4.9 (2019), GPL-3.0 | B | GPL; old release |
