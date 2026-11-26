# Smart Library – known technical limitations (Prompt 07)

2026-11-26. Everything below was observed or is a direct consequence of the design. Nothing here is hidden behind a
"simulated" result: when the engine cannot tell, the value stays **unknown**.

## Analysis
1. **Estimates, not measurements of musical truth.**
   - Tempo uses autocorrelation of onset strength over 70–180 BPM, with a preference for tempos that give a whole number of
     bars.
   - Key uses a chroma profile with Krumhansl–Schmuckler weights.
   - Category and sound character come from spectral rules.
   - Every value carries `source` and `confidence`, and the user can correct BPM, key and labels.
2. **Only the first 90 s of a file are analyzed.** Full songs, long stems and DJ sets get tempo and key from that window
   only. Loudness and peak also cover the first 90 s; duration is read from the container.
3. **Key needs tonal content.**
   - Drum loops, noise and single notes return key *unknown*, with a root note when one pitch dominates.
   - Relative major/minor pairs can be confused. When the two best candidates are close, the confidence is lowered.
   - Mixed material (for example a loop with a loud tuned kick plus a quiet arpeggio) may return unknown.
4. **Tempo out of 70–180 BPM** is reported at half or double time (e.g. 60 BPM → 120). Search treats half and double time
   as a partial match.
5. **AAC (M4A/MP4) encoder delay.** Symphonia does not trim the AAC priming and padding samples. An 8.00 s loop encoded
   to AAC decodes as 8.06 s, so the loop-length tempo reads 119.15 instead of 120. Lossless formats are exact.
6. **No machine-learning models.** Instrument recognition (vocal vs synth, male vs female voice, genre from audio) is not
   done from audio. These labels come from names, folders, tags, presets and MIDI programs. Genre is never guessed from
   audio.
7. **Similarity is a timbre estimate.** It compares a 14-value vector (spectrum, envelope, rhythm density, tonality). It
   does not compare melody or harmony. Two different kicks can score high; a kick and a pad score low. "Similar" never
   means "duplicate".

## Formats
8. Not decoded: Opus, WMA, APE, DSD. AAC in ADTS (`.aac`) and OGG Vorbis are supported by the decoder but were not tested
   with real files (FORMATS.md).
9. Proprietary preset formats (NKS, Massive, Serum 2, Ableton racks, and others) are only recognized: no name, category or
   parameters are read. Instrument libraries and DAW projects are never opened.
10. MIDI files with SMPTE time division (rare, used for film/video sync) are not read: they stay indexed (name, folder, drive) with analysis state "error" and the reason shown in the file details.

## Drives and updates
11. **No live file watcher.**
    - Changes are picked up at app start, when a drive comes back, by scheduled rescans (off, or every 15 min to
      24 h, set in the Library page) or by a manual scan.
    - A file added while the app is open appears at the next one of those events.
    - Rescans are cheap: 0.9 s for 100,000 unchanged files.
12. **Move detection** works within the same authorized folder and across folders when size and content fingerprint
    match, or name and size when there is no fingerprint yet. A file that is moved *and* changed shows up as missing +
    new.
13. **Drive identity** comes from the volume fingerprint (Prompt 04). A drive reformatted under the same name is a new
    drive.
14. Network drives are scanned like local ones. Slow shares make scans slower. There is no special handling for
    connection drops in the middle of a scan: the scan ends as interrupted and is resumed later.

## Search
15. **Rule-based natural-language parser** (Italian and English dictionaries), not an AI model. Unknown words become free
    text matched against names and paths. A lone number between 60 and 200 is read as BPM. Complex sentences
    ("not too dark but not bright either") are not understood. A language model could later turn such requests into the
    same structured query, through the central Agent or the AI Core (D-2, D-3).
16. Similar-sound search compares against at most 50,000 analyzed files per request.

## Platform
17. Verified on macOS (Apple Silicon): internal APFS disk, exFAT external volume (disk image), unplug and replug.
    - Not verified on Windows: the code is portable, and CI is ready but not yet published.
    - Not verified with a physical USB stick or a NAS in this prompt; Prompt 04 had verified a real external disk with
      the earlier library code.
18. Pausing the analysis while a DAW runs relies on process names (Logic Pro, Ableton Live, FL Studio, Pro Tools,
    Cubase, REAPER). Other DAWs are not detected for this purpose.
