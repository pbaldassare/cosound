# Plugin Intelligence – formats and plug-ins actually verified (Prompt 08)

Checked on 2026-11-26 on a MacBook Air, Apple silicon, macOS 15 (SDK 15.5). No third-party plug-in and no DAW are
installed on this Mac, so the real-plug-in tests use the Audio Units built into macOS.

## Formats

| Format | Recognition | Version / architecture | Load, parameters, render, presets | How it was verified |
|---|---|---|---|---|
| **AU (AUv2)** | ✅ folders + macOS registry | ✅ registry version, Mach-O architectures | ✅ isolated host | Real: Apple's built-in units (table below); synthetic `.component` bundle for folder reading, registry merge and unregistered bundles |
| **AUv3** (app extensions) | ✅ registry (flag v3) | ✅ registry version | ✅ same host, not tested | No AUv3 installed on this Mac |
| **VST3** | ✅ folders, `moduleinfo.json` | ✅ version, vendor, sub-categories, SDK; arch from the bundle layout | ❌ not hosted | Synthetic bundle only |
| **VST2** | ✅ folders (`.vst` bundles, `.dll` on Windows) | ✅ plist version, Mach-O / PE arch | ❌ not hosted | Synthetic bundle only (x86_64-only → flagged incompatible on Apple silicon) |
| **CLAP** | ✅ folders | plist version when present | ❌ not hosted | Synthetic bundle only |
| **AAX** | ✅ Avid folder | plist version when present | ❌ cannot be hosted (Avid offers no third-party hosting) | Synthetic bundle only |

On Windows only the folder paths and PE-header reading are implemented and unit-tested; nothing was run on Windows.

## Real plug-ins tested (Apple, built into macOS, AU 1.6.0)

| Plug-in | What was verified |
|---|---|
| AULowpass | Recognized; categories filter + EQ; 2 parameters read (Cutoff Frequency, hertz, 10…21829.5; Resonance, dB); parameter IDs match Apple's header; cutoff changed and swept (automation) in renders; state saved as `.aupreset`, reloaded with the same cutoff; preset-folder scan attributes it back |
| AUHipass | Chosen for the "remove rumble" stage of the techno-bass chain; rendered |
| AUDistortion | 16 parameters, 22 factory presets; drive changes measured (centroid 354 → 976 Hz); factory-preset timbre measured by rendering |
| AUReverb2 | 13 factory presets; semantic search "large hall" → Large Hall |
| AUMatrixReverb | Reverb of the pad design; its Modulation Rate/Depth used for "movimento lento" |
| AUDynamicsProcessor, AUPeakLimiter | Compressor and limiter stages of the chains, rendered on real audio |
| AUDelay | Missing/back detection (install removed from a scan, then restored) |
| DLSMusicDevice | Instrument: General MIDI program change, notes from the music engine, before/after audition |
| AUSampler, AUMIDISynth | Recognized as instruments; parameters readable |
| All 30 production units | Registry: 23 effects, 3 instruments, 4 generators, with categories |

**Not verified:** third-party plug-ins of any format (Massive, Serum, Vital …), AUv3 extensions, Windows, any DAW
(Logic Pro is not installed on this Mac).

## Preset formats
| Format | Read | Written |
|---|---|---|
| Factory presets (AU) | ✅ names and numbers from the plug-in; timbre by rendering | — |
| `.aupreset` | ✅ AU codes and name (plist); loaded into the plug-in by the host | ✅ new files from the plug-in's own state (`fullState`) |
| `.vstpreset`, `.vital`, `.fxp`/`.fxb` | ✅ container metadata (Smart Library reader) | ❌ |
| `.nksf`, `.nmsv`, `.serumpreset`, `.h2p`, `.adv` | found, contents not read | ❌ |
