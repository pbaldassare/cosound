# Reference Track Analysis – source registry

| ID | Source | URL | Retrieved | Software version | Reliability | Limitations |
|---|---|---|---|---|---|---|
| S1 | MusicBrainz API documentation | https://musicbrainz.org/doc/MusicBrainz_API | 2026-11-24 | ws/2 | A | – |
| S2 | MusicBrainz – Data License | https://musicbrainz.org/doc/About/Data_License | 2026-11-24 | – | A | Exact split core vs supplementary listed on another page (not read) |
| S3 | Discogs API root (`https://api.discogs.com/`, JSON response) | https://api.discogs.com/ | 2026-11-24 | API v2 | A | Only confirms API is live and catalog size |
| S4 | Discogs developer docs | https://www.discogs.com/developers | 2026-11-24 | – | U | HTTP 403 to automated fetch; rate limits (60/min auth, 25/min unauth) and User-Agent rule seen only in third-party copies (to verify) |
| S5 | Discogs API Terms of Use | https://support.discogs.com/hc/en-us/articles/360009334593 | 2026-11-24 | – | U | Seen only in search results (to verify) |
| S6 | AcousticBrainz home page | https://acousticbrainz.org/ | 2026-11-24 | – | A | Exact stop date not on page |
| S7 | Spotify for Developers blog – "Introducing some changes to our Web API" (2024-11-27) | https://developer.spotify.com/blog/2024-11-27-changes-to-the-web-api | 2026-11-24 | Web API | A | – |
| S8 | Spotify Developer Policy | https://developer.spotify.com/policy | 2026-11-24 | – | A | – |
| S9 | Spotify User Guidelines | https://www.spotify.com/us/legal/user-guidelines/ | 2026-11-24 | undated (© 2026) | A | – |
| S10 | Spotify Terms of Use (end-user agreement) | https://www.spotify.com/us/legal/end-user-agreement/ | 2026-11-24 | last updated 2026-09-04 | A | US version |
| S11 | YouTube Terms of Service | https://www.youtube.com/static?template=terms | 2026-11-24 | effective 2022-01-05 (localized copy served) | A | Served in German; paraphrased |
| S12 | Apple Music API docs + Songs attributes | https://developer.apple.com/documentation/applemusicapi | 2026-11-24 | – | A | Auth details (developer token/JWT) not re-verified here |
| S13 | all-in-one (allin1) – GitHub README / pyproject; Kim & Nam, WASPAA 2023 (arXiv 2307.16425) | https://github.com/mir-aidj/all-in-one | 2026-11-24 | PyPI 1.1.0, MIT code; last commit 2023-10 | B | Depends on Demucs, madmom (git), NATTEN; weights license not stated (to verify); low maintenance |
| S14 | MSAF – GitHub README | https://github.com/urinieto/msaf | 2026-11-24 | v0.1.84 (2026-11-22), MIT | B | Classic algorithms; unlabeled segments |
| S15 | librosa – source (`segment.py`: recurrence/cross-similarity, `subsegment`) | https://github.com/librosa/librosa | 2026-11-24 | 1.0.1, ISC | B | Building blocks, not a full structure analyzer |
| S16 | Beat This! – README (ISMIR 2024) | https://github.com/CPJKU/beat_this | 2026-11-24 | v1.1.0, MIT | B | – |
| S17 | madmom – LICENSE | https://github.com/CPJKU/madmom | 2026-11-24 | v0.16.1; models CC BY-NC-SA 4.0 | B | Non-commercial models |
| S18 | Essentia – licensing page | https://essentia.upf.edu/licensing_information.html | 2026-11-24 | AGPL-3.0 | B | – |
| S19 | Spotify basic-pitch – README | https://github.com/spotify/basic-pitch | 2026-11-24 | v0.4.0, Apache-2.0 | B | – |
| S20 | adefossez/demucs – README | https://github.com/adefossez/demucs | 2026-11-24 | 4.1.0, MIT | B | – |
| S21 | pyloudnorm – README; ITU-R BS.1770 page | https://github.com/csteinmetz1/pyloudnorm ; https://www.itu.int/rec/R-REC-BS.1770 | 2026-11-24 | pyloudnorm 0.2.0 (BS.1770-4); BS.1770-5 in force | A/B | – |
| S22 | U.S. Copyright Office – Copyright and Artificial Intelligence | https://www.copyright.gov/ai/ | 2026-11-24 | Part 1 2024-07-31; Part 2 2025-01-29; Part 3 pre-pub 2025-05-09 | A | US only; conclusions of Part 2 not read (to verify) |
| S23 | Tunebat (example of third-party BPM/key site) | https://tunebat.com | – | – | U | Not fetched; data provenance unknown |
