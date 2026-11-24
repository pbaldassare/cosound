# Reference Track Analysis (Section 9)

Using a reference track to **inspire an original composition** (tempo, feel, energy shape,
arrangement, sound palette) — never to copy it. Two modes. Verified 2026-11-24. Sources: `sources.md`.

## Mode 1 – Web Reference Research (metadata only, no audio)

| Source | What it legitimately gives | Terms / limits |
|---|---|---|
| **MusicBrainz** API (`/ws/2/`) | Artists, recordings, releases, release-groups, works, labels, genres, tags; lookups by ISRC / ISWC / disc ID [S1] | ≤1 request/s per client; meaningful User-Agent required [S1]. Core data CC0, supplementary data CC BY-NC-SA 3.0 [S2] |
| **Discogs** API v2 | Releases, artists, labels (≈19.6M releases reported by the API root) [S3]; genres/styles, credits, tracklists **(to verify)** | Docs blocked to automated fetch [S4]; reported 60 req/min authenticated, 25 unauthenticated, unique User-Agent required **(to verify)** [S4][S5] |
| **AcousticBrainz** | Historic audio descriptors (BPM, key, etc.) | **Stopped collecting data in 2022**; site/API still online [S6] — static, aging dataset |
| **Spotify Web API** | Catalog metadata | On 2024-11-27 **Audio Features, Audio Analysis**, Recommendations, Related Artists, 30-s preview URLs in multi-get, editorial playlists were restricted for new apps and dev-mode apps without pending extension [S7]. Developer Policy forbids analyzing Spotify Content "for any purpose" and training ML/AI with it [S8] → **no BPM/key from Spotify for COSOUND** |
| **Apple Music API** | Song attributes: name, artist, album, `genreNames`, `isrc`, `durationInMillis`, `releaseDate`, `composerName`, `previews`, `audioVariants` etc. [S12] | No BPM/key/energy attributes in the Song attributes list [S12] |
| Tunebat-like sites | BPM/key/"energy" per track | Secondary, provenance unknown (often derived from deprecated Spotify features) **(to verify)** [S23]; use only as a hint, never scrape |

Practical use: resolve the track (MusicBrainz/ISRC) → genre/year/credits context → the LLM's
genre knowledge (from `music-production/`) → *ask the user* for BPM/key or analyze their own file
(Mode 2).

## Mode 2 – Audio Reference Analysis (file supplied by the user)

The user supplies a file they lawfully have (purchased download, own bounce, licensed sample).
COSOUND analyzes locally and extracts **descriptors, not reusable audio**.

| Feature | Recommended tool (license) | Reliability / notes |
|---|---|---|
| BPM, beats, downbeats, meter | Beat This! (MIT) [S16] | High on steady-tempo pop/electronic; lower on rubato/live (general MIR knowledge, **to verify**) |
| Key | chroma templates on harmonic stems (librosa, ISC) [S15]; Essentia KeyExtractor is AGPL [S18] | Medium; relative major/minor ambiguity |
| Groove / drum pattern | Demucs drums stem [S20] → onsets per band / drum transcription (see `audio-stem-analysis/` §8) | Kick/snare/hat pattern usable; swing = offset of off-beat onsets from grid (COSOUND computation) |
| Structure segmentation | **all-in-one** (allin1, MIT code): beats, downbeats, tempo, segment boundaries + labels intro/verse/chorus/bridge/outro [S13]; **MSAF** (MIT, v0.1.84) boundary algorithms [S14]; librosa recurrence matrices [S15] | allin1 depends on Demucs, madmom (non-commercial models [S17]) and NATTEN; weights license and maintenance (last commit 2023-10) **to verify** [S13] |
| Bassline | Demucs bass stem [S20] → monophonic F0 or basic-pitch (Apache-2.0) [S19] → MIDI (contour + rhythm only) | Good on clean synth/electric bass |
| Harmony | chord draft on "other"+bass stems (see `audio-stem-analysis/` §5) | Draft quality; user confirms |
| Melody | vocals stem → F0 contour | Use contour/range/rhythmic density as descriptors, **not the exact notes** |
| Sound-design descriptors | spectral centroid/flatness/bandwidth, MFCC, stereo width, transient sharpness (librosa) [S15] | Descriptive ("bright, wide, short decay"), good for synth-patch suggestions |
| Energy curve | short-term loudness (BS.1770 meter, pyloudnorm) [S21] + RMS + onset density | Reliable as a relative curve |
| Breakdown / build-up / drop | heuristic over segments: energy dip + drum-stem drop (breakdown), rising onset density/brightness (build), energy + low-end jump at downbeat (drop) | COSOUND heuristic, **to verify on test set**; EDM-centric |

Output = a **"reference brief"**: tempo range, meter, swing amount, section map with bar counts,
energy curve, instrumentation roles, harmonic colour (major/minor, chord rhythm), sound-palette
adjectives. The composition agents use the brief, not the audio.

## Legal / ethical guardrails (not legal advice)

- **No ripping**: Spotify's User Guidelines forbid copying, ripping, recording or transferring
  content, circumventing protection, scraping, and using content to train ML/AI [S9]; its Terms
  limit use to personal, non-commercial and forbid redistribution [S10]. YouTube's Terms forbid
  accessing/copying/saving content except as the service allows or with permission, and forbid
  automated access [S11]. → COSOUND never downloads from streaming services; only user-supplied
  files.
- **Developer APIs**: Spotify forbids analyzing its content or training models with it [S8].
- **Original output**: use the reference for *abstract* features (tempo, structure, energy,
  palette, groove feel). Do not transfer exact melodies, lyrics, distinctive hooks, or
  chord-plus-rhythm combinations; do not sample its audio. Add a similarity check between the
  generated melody and the extracted reference melody (COSOUND design).
- **No training on user references** without explicit consent and rights.
- AI-output copyrightability is jurisdiction-dependent; the US Copyright Office has published a
  multi-part report (Part 2: copyrightability, Jan 2025) [S22]. Point users to legal counsel.

## Limitations

- Spotify/AcousticBrainz no longer provide usable audio features for new apps [S6][S7].
- Structure labels (verse vs chorus) are less reliable than boundaries **(to verify)**; allin1
  maintenance/weights license unclear [S13].
- Analysis quality is bounded by separation quality (bleed, artifacts) [S20].
- Genre bias: drop/build heuristics assume electronic/pop forms.
- Discogs terms and rate limits not verified from the official page [S4][S5].
