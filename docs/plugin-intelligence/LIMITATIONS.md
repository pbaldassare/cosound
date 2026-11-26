# Plugin Intelligence – known limitations (Prompt 08)

2026-11-26. What the system does not do, or does only partly. It declares these limits in its answers instead of
inventing results.

## Hosting and control
1. **Only Audio Units are loaded, only on macOS.**
   - VST3, VST2, CLAP and AAX are recognized but not loaded. Hosting VST3 would need the Steinberg SDK (a C++ host),
     and CLAP a custom host. AAX cannot be hosted by third parties.
   - On Windows nothing is loaded.
2. **Control happens in COSOUND's host, not inside the DAW.**
   - Designed sounds reach the DAW as new preset files that the user (or later the DAW connector) loads.
   - Setting parameters of the instance in the project needs the DAW connectors (milestone M4); see `daw.rs` for the
     route per DAW.
3. **Logic Pro is not installed on the test Mac.**
   - COSOUND writes presets into `~/Library/Audio/Presets/<Manufacturer>/<Plug-in>/`, the Audio Unit user-preset
     folder.
   - That Logic lists them in its plug-in preset menu is documented practice but **was not verified** here.
4. **Parameter roles come from names and units.**
   - A plug-in with generic names ("Param 12") gets no roles, so its parameters are shown but never moved
     automatically.
   - Plug-ins with hundreds of parameters may get partial mappings.
5. **Audition renders are offline and simplified.**
   - Notes are applied at 256-sample block boundaries (≈5 ms at 48 kHz).
   - Parameter ramps are linear.
   - The test uses the plug-in's default output.
6. **Measured checks are estimates.**
   - "Aggressive", "bright" and "dark" are checked with the spectral centroid.
   - "Warm", "deep", "soft", "wide" and "atmospheric" use the Smart Library's sound-character values.
   - These are proxies, not perception: the user's ear decides.
7. **Plug-ins with copy protection or license dialogs** may refuse to load in a separate process, or show dialogs.
   They are logged as errors; COSOUND does not handle licenses.

## Knowledge
8. **Official documentation is in the knowledge base only for Apple's Audio Units** (from the SDK header).
   - For third-party plug-ins, COSOUND uses what the plug-in exposes and the synth profiles of the knowledge base.
   - Vendor manuals can be added as JSON files in `<app data>/plugin-knowledge/`; there is no automatic download.
9. **DAW compatibility is "documented"** (format support per DAW from the knowledge base), never tested inside a DAW.

## Presets
10. **Proprietary preset formats** (NKS, Massive, Serum 2, Ableton racks) are found but not read.
11. `.vstpreset`/`.fxp` written by COSOUND: not implemented (the plug-in state chunk is private to each plug-in).
12. **Preset timbre** is measured only for Audio Unit presets, by rendering them. Other presets are matched by name,
    category and folder.
13. **Semantic search** uses COSOUND's rule-based Italian/English parser, the same as the Smart Library. Presets
    without useful names and not analyzed are hard to find by description.

## Effect chains
14. **Four goals** (techno bass, melodic lead, atmospheric pad, mix bus).
    - Settings are starting points from the knowledge base.
    - The overload check renders only the stages that COSOUND can load. Stages without a loadable plug-in are listed
      as not rendered.
15. **Starting settings are applied by sound role.** A plug-in that does not expose that role keeps its default for
    that setting.

## Platform
16. **The helper needs the Swift compiler at build time** (Xcode Command Line Tools). Without it the app builds, and
    hosting reports "not available".
17. **Listing the Audio Unit registry takes ≈3 s.** It runs only at scan time.
