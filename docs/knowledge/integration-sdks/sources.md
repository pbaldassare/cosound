# Sources – integration-sdks

| ID | Source | URL | Retrieved | Software version | Reliability | Limitations |
|---|---|---|---|---|---|---|
| S1 | Steinberg VST 3 Developer Portal – VST 3 License | https://steinbergmedia.github.io/vst3_dev_portal/pages/VST+3+Licensing/VST3+License.html | 2026-11-24 | VST SDK 3.8.x | A | |
| S2 | Steinberg VST 3 Developer Portal – FAQ: Licensing (incl. "Specific VST 2") | https://steinbergmedia.github.io/vst3_dev_portal/pages/FAQ/Licensing.html | 2026-11-24 | VST SDK 3.8.x | A | |
| S3 | Steinberg VST 3 Developer Portal – Change History | https://steinbergmedia.github.io/vst3_dev_portal/pages/Versions/Index.html | 2026-11-24 | 3.8.1 (2026-08-11) latest | A | |
| S4 | GitHub steinbergmedia/vst3sdk README | https://github.com/steinbergmedia/vst3sdk | 2026-11-24 | 3.8.x | A | |
| S5 | VST 3 Portal – Plug-in Locations | https://steinbergmedia.github.io/vst3_dev_portal/pages/Technical+Documentation/Locations+Format/Plugin+Locations.html | 2026-11-24 | 3.8.x | A | |
| S6 | VST 3 Portal – Preset Locations | https://steinbergmedia.github.io/vst3_dev_portal/pages/Technical+Documentation/Locations+Format/Preset+Locations.html | 2026-11-24 | 3.8.x | A | |
| S7 | VST 3 Portal – Preset Format (.vstpreset) | https://steinbergmedia.github.io/vst3_dev_portal/pages/Technical+Documentation/Locations+Format/Preset+Format.html | 2026-11-24 | 3.8.x | A | Chunk contents are plug-in-defined |
| S8 | VST 3 Portal – Parameters and Automation | https://steinbergmedia.github.io/vst3_dev_portal/pages/Technical+Documentation/Parameters+Automation/Index.html | 2026-11-24 | 3.8.x | A | |
| S9 | VST 3 Portal – VST 3 API Documentation (Basic concept, Persistence) | https://steinbergmedia.github.io/vst3_dev_portal/pages/Technical+Documentation/API+Documentation/Index.html | 2026-11-24 | 3.8.x | A | |
| S10 | VST 3 Portal – Moduleinfo (moduleinfo.json) | https://steinbergmedia.github.io/vst3_dev_portal/pages/Technical+Documentation/VST+Module+Architecture/ModuleInfo-JSON.html | 2026-11-24 | since 3.7.5 | A | Optional file |
| S11 | VST 3 Portal – Main benefits of VST 3 | https://steinbergmedia.github.io/vst3_dev_portal/pages/Main+benefits+of+VST+3/Index.html | 2026-11-24 | 3.8.x | A | |
| S12 | Apple Developer – Audio Unit v3 Plug-Ins (AudioToolbox) | https://developer.apple.com/documentation/audiotoolbox/audio-unit-v3-plug-ins | 2026-11-24 | current SDK | A | Read via the docs JSON endpoint; abstract only |
| S13 | Apple Developer – `AUAudioUnit.fullState` | https://developer.apple.com/documentation/audiotoolbox/auaudiounit/fullstate | 2026-11-24 | macOS 10.11+, iOS 9+, visionOS 1+ | A | |
| S14 | Apple Developer – `AUAudioUnit.parameterTree` | https://developer.apple.com/documentation/audiotoolbox/auaudiounit/parametertree | 2026-11-24 | macOS 10.11+ | A | |
| S15 | Apple Developer – `AVAudioUnitComponentManager` | https://developer.apple.com/documentation/avfaudio/avaudiounitcomponentmanager | 2026-11-24 | macOS 10.10+ | A | |
| S16 | Apple Developer – `AUAudioUnit.factoryPresets` / `userPresets` | https://developer.apple.com/documentation/audiotoolbox/auaudiounit/userpresets | 2026-11-24 | userPresets macOS 10.15+ | A | |
| S17 | Apple – *Audio Unit Programming Guide* (archive), "The Audio Unit" | https://developer.apple.com/library/archive/documentation/MusicAudio/Conceptual/AudioUnitProgrammingGuide/TheAudioUnit/TheAudioUnit.html | 2026-11-24 | Archived (AUv2 era) | A | Legacy doc, no longer updated |
| S18 | Apple – *Audio Unit Programming Guide* (archive), "Audio Unit Development Fundamentals" | https://developer.apple.com/library/archive/documentation/MusicAudio/Conceptual/AudioUnitProgrammingGuide/AudioUnitDevelopmentFundamentals/AudioUnitDevelopmentFundamentals.html | 2026-11-24 | Archived | A | Legacy doc |
| S19 | Avid Developer – AAX SDK page and FAQ | https://developer.avid.com/aax/ | 2026-11-24 | current | A | Says nothing about third-party hosting |
| S20 | GitHub free-audio/clap README | https://github.com/free-audio/clap | 2026-11-24 | CLAP 1.2.10 | A | Standard's own repo |
| S21 | CLAP `include/clap/entry.h` (search paths) and `version.h` | https://github.com/free-audio/clap/blob/main/include/clap/entry.h | 2026-11-24 | 1.2.10 | A | |
| S22 | CLAP `include/clap/ext/params.h` | https://github.com/free-audio/clap/blob/main/include/clap/ext/params.h | 2026-11-24 | 1.2.10 | A | |
| S23 | cleveraudio.org – Hosts and Plug-Ins | https://cleveraudio.org/hosts-and-plug-ins/ | 2026-11-24 | List "as of November 2024" | A | Possibly stale; clapdb.tech is the live list (not checked) |
| S24 | JUCE docs – `AudioPluginFormatManager` | https://docs.juce.com/master/classAudioPluginFormatManager.html | 2026-11-24 | JUCE master (9.x) | B | |
| S25 | JUCE docs – `KnownPluginList` | https://docs.juce.com/master/classKnownPluginList.html | 2026-11-24 | JUCE 9.x | B | |
| S26 | JUCE docs – `AudioProcessor` (get/setStateInformation) | https://docs.juce.com/master/classAudioProcessor.html | 2026-11-24 | JUCE 9.x | B | |
| S27 | GitHub juce-framework/JUCE – releases (9.0.6, 2026-11-19), `modules/juce_audio_processors/format_types`, CHANGE_LIST.md | https://github.com/juce-framework/JUCE | 2026-11-24 | 9.0.6 | B | Format list inferred from source file names |
| S28 | JUCE – Get JUCE / licensing FAQ | https://juce.com/get-juce/ | 2026-11-24 | JUCE 9 | A | Vendor licence terms |
| S29 | Spotify pedalboard docs (home) | https://spotify.github.io/pedalboard/ | 2026-11-24 | 0.9.26 (release 2026-10-07) | B | |
| S30 | Spotify pedalboard API reference | https://spotify.github.io/pedalboard/reference/pedalboard.html | 2026-11-24 | 0.9.26 | B | Docs built on a platform without the AudioUnitPlugin class section |
| S31 | REAPER ReaScript API documentation | https://www.reaper.fm/sdk/reascript/reascripthelp.html | 2026-11-24 | current REAPER | A | |
| S32 | Cycling '74 – Live Object Model, DeviceParameter | https://docs.cycling74.com/apiref/lom/deviceparameter/ | 2026-11-24 | Live 12 / Max 9 | A | |
| S33 | Image-Line – FL Studio MIDI scripting (plugins module) | https://www.image-line.com/fl-studio-learning/fl-studio-online-manual/html/midi_scripting.htm | 2026-11-24 | current FL Studio | A | |
| S34 | Steinberg – MIDI Remote API documentation | https://steinbergmedia.github.io/midiremote_api_doc/ | 2026-11-24 | MIDI Remote API v1.3 | A | |
| S35 | Apple – Logic Pro User Guide, Scripter JavaScript objects (TargetEvent) | https://support.apple.com/en-us/guide/logicpro/lgce0d0efc5a/mac | 2026-11-24 | Logic Pro 13 guide | A | |
| S36 | Ableton – Live 12 Manual, "Using Plug-Ins" | https://www.ableton.com/en/live-manual/12/using-plug-ins/ | 2026-11-24 | Live 12 | A | |
| S37 | Ableton Help – "Supported Plug-in Formats" | https://help.ableton.com/hc/en-us/articles/5937501570460 | 2026-11-24 | Live 12 | U | 403 to automated fetch; content seen only in search snippets |
| S38 | Steinberg Help Center – VST 2 plug-ins in Cubase/Nuendo 14 and 15 | https://helpcenter.steinberg.de/hc/en-us/articles/22554401894162 | 2026-11-24 | Cubase 14/15 | U | 403 to automated fetch; content seen only in search snippets |
| S39 | Xfer Records forum – "File Types???" (developer statement on preset format) | https://www.xferrecords.com/forums/posts/78894 | 2026-11-24 | Serum 1/2 | B | Forum post |
| S40 | Vital source (GitHub mtytel/vital), `synth_constants.h`, `synth_base.cpp` | https://github.com/mtytel/vital | 2026-11-24 | source 1.0.6 | B | |
| S41 | LennarDigital – Sylenth1 User Manual v2.5 (PDF) | http://files.lennardigital.com/manual/Sylenth1Manual.pdf | 2026-11-24 | manual v2.5 | A | |
| S42 | Native Instruments – Massive XL User Guide (PDF) | https://docs.native-instruments.com/pdf-guides/Massive_XL_user_guide_English.pdf | 2026-11-24 | 2.0 (10/2026) | A | |
| S43 | Apple – Logic Pro User Guide for Mac (table of contents incl. "Work with Audio Units in Logic Pro for Mac", "Use the Plug-in Manager") | https://support.apple.com/guide/logicpro/plug-in-manager-overview-lgcp4ba08c00/mac | 2026-11-24 | Logic Pro 13 | A | Only section titles checked for this claim |
