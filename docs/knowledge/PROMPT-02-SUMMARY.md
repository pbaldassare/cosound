# COSOUND – Prompt 02: consegna finale

Data: 2026-11-24. Ricerca tecnica e musicale, knowledge base. Nessun codice applicativo scritto, sito non modificato.
Dettagli e fonti in ogni cartella (`README.md` + `sources.md`); qui la sintesi.

## 1–9. Dove si trova ogni consegna

| # | Consegna | Dove |
|---|---|---|
| 1 | Knowledge base tecnica | `integration-sdks/`, `audio-engineering/`, `smart-library/`, `security-permissions/`, `music-memory/` |
| 2 | Knowledge base musicale | `music-theory/` (note, scale, armonia, melodia), `music-production/` (ritmo, 11 generi, arrangiamento), `midi-intelligence/` |
| 3 | Documentazione DAW | `logic-pro/`, `ableton-live/`, `fl-studio/`, `pro-tools/`, `cubase/`, `reaper/` |
| 4 | Tabella comparativa integrazione | `daw-integration/README.md` |
| 5 | Voice-to-Music | `voice-to-music/` |
| 6 | Audio-to-MIDI e Stem Analysis | `audio-stem-analysis/` |
| 7 | Reference Track Analyzer | `reference-track/` |
| 8 | Smart Library Multidrive | `smart-library/` |
| 9 | Plugin Intelligence e sintesi | `integration-sdks/`, `synthesizers/`, `plugins-effects/` |

Architettura multi-agent (direttiva del 2026-11-24): `../ARCHITECTURE-DIRECTIVE.md`, `../architecture/MULTI-AGENT-ARCHITECTURE.md`.

## 10. Tecnologie consigliate (licenze permissive, uso commerciale possibile)

| Compito | Scelta | Licenza |
|---|---|---|
| Speech-to-text locale | whisper.cpp 1.9.6 (macOS 26+: anche Apple SpeechAnalyzer) | MIT |
| Pitch tracking voce | SwiftF0 0.3.0; fallback pYIN (librosa 1.0.1) | MIT / ISC |
| Audio→MIDI | basic-pitch 0.4.0 (Core ML + ONNX) | Apache-2.0 |
| Voice activity | Silero VAD 6.3 | MIT |
| Beat / tempo | Beat This! 1.1.0 | MIT |
| Separazione stem | Demucs 4.1.0; RoFormer via python-audio-separator (licenza dei singoli modelli da verificare) | MIT |
| Loudness | pyloudnorm / libebur128 (BS.1770) | MIT |
| Decodifica audio | libsndfile + FFmpeg solo LGPL, linkato dinamicamente | LGPL |
| Tonalità | template matching su chroma (librosa), sviluppo interno | ISC |
| Struttura brano | all-in-one / MSAF (pesi all-in-one da verificare) | MIT |
| File MIDI / teoria | mido 1.3.3, pretty_midi 0.2.11, music21 10.5.2 | MIT / MIT / BSD-3 |
| Watch filesystem | crate `notify` (FSEvents, ReadDirectoryChangesW) | CC0 |
| Metadati audio | Symphonia + lofty | MPL-2.0 / Apache-2.0 |
| Duplicati | XXH3 / BLAKE3 | BSD-2 / CC0-Apache |
| Indice libreria | SQLite + FTS5 + sqlite-vec | public domain / MIT-Apache |
| Classificazione suoni | CLAP (LAION) | CC0 |
| Plugin formats | VST3 SDK 3.8.1, CLAP 1.2.10 | MIT |
| Segreti | Keychain / Credential Manager-DPAPI (crate `keyring`) | – |
| Prima DAW per test end-to-end | REAPER (ReaScript completo) | – |

**Da escludere dal prodotto (licenza):** Essentia (AGPL, modelli non commerciali), modelli madmom e ADTOF (CC BY-NC-SA), Chordino (GPL-2), aubio (GPL-3), pedalboard (GPL-3), JUCE senza licenza commerciale (AGPL), modelli openWakeWord inclusi (non commerciali).

## 11. Limitazioni tecniche principali
1. **Logic Pro 13 non ha API pubblica.** Restano control surface/OSC (mixer, trasporto, parametri), import di file MIDI e automazione UI via Accessibility (fragile tra versioni). L'assistente AI integrato in Logic 13 non è richiamabile dall'esterno.
2. **Pro Tools**: API ufficiale PTSL (gRPC) ma senza comandi per note MIDI, tempo, inserimento plug-in; SDK dietro account Avid, licenza non ancora letta.
3. **Ableton Live**: controllo completo solo via Max for Live (Suite/add-on); scrittura automazioni da verificare; export solo via UI.
4. **FL Studio**: scripting reattivo al MIDI, non crea arrangiamenti; render offline da riga di comando.
5. **Cubase**: controllo tramite script MIDI Remote + porte MIDI virtuali; unica DAW con DAWproject nativo.
6. **Plug-in**: i *file* preset sono in gran parte formati chiusi (Serum, Massive, Sylenth1); solo i preset Vital (JSON) sono modificabili come file. **Questo non limita l'uso dei synth**: COSOUND li opera come la mano dell'utente, dentro la DAW (parametri dell'host + mano su schermo con verifica visiva + controllo all'ascolto). Vedi `../architecture/MULTI-AGENT-ARCHITECTURE.md` §7b. Massive XL espone all'host solo Macro e Morpher: il resto si regola con la mano su schermo.
7. **Voce**: BPM da un canticchiato breve e quantizzazione sono inaffidabili → registrazione con count-in/click e conferma dell'utente. Beatbox→batteria è a livello di ricerca.
8. **Analisi**: nessun riconoscitore di accordi o trascrittore di batteria maturo con licenza permissiva → da sviluppare internamente. Uno stem diventa MIDI solo se monofonico e pulito (o batteria); pad densi, voci mixate ed effetti restano audio.
9. **Reference track**: Spotify ha bloccato audio-features/analysis per le nuove app (2024-11-27) e vieta l'analisi dei suoi contenuti; AcousticBrainz chiuso; Apple Music API senza BPM/tonalità. L'analisi audio si fa solo su file forniti legittimamente dall'utente, per creare musica originale.
10. **Generatori AI**: Suno – stem fino a 12, WAV su Pro/Premier, MIDI solo Premier (Suno Studio), uso commerciale solo piani a pagamento, vietato usare l'output per addestrare altri modelli. Udio – download disattivati da febbraio 2026 (stato attuale da verificare).
11. **Mastering**: Spotify −14 LUFS e ≤ −1 dBTP (fonte ufficiale); valori di Apple/YouTube/TIDAL/Amazon solo da fonti secondarie. Nessuna misura va dichiarata senza averla calcolata.
12. **Ottave MIDI**: il nome della nota 60 cambia per DAW (C3 Logic/Ableton/Cubase, C5 FL Studio, C4 REAPER; alcune voci da verificare) → COSOUND deve convertire.
13. **Windows**: lettere di unità variabili (indicizzare per ID volume), permesso microfono globale per le app desktop; eventi file inaffidabili su NAS/exFAT → polling.

## 12. Fonti
455 fonti registrate in 20 cartelle, ciascuna con URL, data (2026-11-24), versione, affidabilità (A ufficiale / B tecnica autorevole / C secondaria / U non verificata) e limitazioni. Verifiche eseguite:
- ogni citazione `[S#]` nel testo corrisponde a una voce del registro (nessuna citazione orfana; 4 fonti registrate ma non citate);
- 376 URL delle fonti non-U controllati automaticamente il 2026-11-24: 369 rispondono, 5 bloccano i controlli automatici (403: Avid, EBU, Zenodo, un DOI) ma erano stati letti dai ricercatori, 2 falsi negativi del controllo (URL Apple con parentesi).
- Le voci "(to verify)" sono elencate in ogni cartella e non vanno usate come fatti.

## 13. Indicazioni per il Prompt 03
1. **Rispettare la direttiva**: nessun orchestratore nostro; sotto-agenti e servizi dietro il contratto di `MULTI-AGENT-ARCHITECTURE.md` §5, testati con un harness usa-e-getta. Analizzare l'Agent centrale appena consegnato.
2. **Scegliere lo stack desktop** (Tauri vs Electron vs nativo) considerando: accesso Accessibility/AppleScript su macOS, porte MIDI virtuali, esecuzione di modelli ML locali (ONNX Runtime / Core ML), dimensione dell'installer.
3. **Primo DAW Controller**: REAPER per provare il flusso completo; in parallelo prototipo Logic (MIDI file import + control surface/OSC + Accessibility) perché è la DAW prioritaria.
4. **Via universale** valida per tutte le DAW: generare file MIDI/audio da trascinare nel progetto.
5. **Verifiche aperte da chiudere su macchine reali** (Mac/PC con le DAW installate): messaggi OSC di Logic, key command Cubase, PTSL (scaricare SDK e leggere licenza), automazioni Ableton/FL, parametri esposti dai synth.
6. **Knowledge base → indice strutturato**: convertire i README in record (fatto, versione, fonte, affidabilità) per il recupero da parte di ciascun sotto-agente.
7. **Verifica licenze** dei pesi dei modelli prima di includerli (RoFormer, all-in-one, YAMNet).
