# COSOUND – Rapporto Prompt 07: Smart Library System (2026-11-26)

Il Prompt 07 chiedeva il catalogo intelligente delle risorse musicali: un **motore tecnico indipendente**, senza un secondo
Central Agent. Testo: `docs/prompts/PROMPT-07.md`. Architettura: [ARCHITECTURE.md](ARCHITECTURE.md). Formati:
[FORMATS.md](FORMATS.md). Limiti: [LIMITATIONS.md](LIMITATIONS.md).

## 1. Consegne (Prompt 07 §19)
| Consegna | Dove |
|---|---|
| Codice completo | `desktop/src-tauri/src/smartlib/` (13 moduli più i test), `library.rs`, pagina `desktop/src/pages/Library.tsx` + `components/SmartLibrary.tsx` |
| Architettura tecnica | `ARCHITECTURE.md` §1–2 |
| Database e struttura dell'indice | `ARCHITECTURE.md` §3, migrazione `migrations/003_smartlib.sql`, `docs/architecture/DATABASE.md` |
| Motore di scansione | `smartlib/scan.rs` + `service.rs` |
| Riconoscimento multidrive | `library.rs` (identità del disco, tipo, collegato/scollegato) + `scan.rs` (stati dei file) |
| Motore di analisi audio | `smartlib/decode.rs` (Symphonia) + `features.rs` |
| Motore di classificazione | `smartlib/classify.rs` + `analyze.rs` (scelta della fonte migliore) |
| Ricerca testuale e semantica | `smartlib/query.rs` (italiano/inglese) + `search.rs` |
| Gestione MIDI | `smartlib/midi.rs` |
| Gestione dei preset supportati | `smartlib/presets.rs` |
| API di integrazione | `smartlib/api.rs`, comandi Tauri, 13 operazioni dell'agente Smart Library (`ARCHITECTURE.md` §4) |
| Documentazione tecnica | questa cartella; `docs/agents/AGENTS.md` e `REGISTRIES.md` rigenerati dai manifest |
| Test e risultati | §4 e §5 di questo rapporto |
| Formati effettivamente supportati | `FORMATS.md` |
| Limitazioni conosciute | `LIMITATIONS.md` |

## 2. Cosa funziona davvero (operativo)
- **Dischi e cartelle**:
  - dischi interni, esterni, SSD, USB e di rete;
  - un disco già indicizzato viene riconosciuto quando torna, anche con un altro punto di montaggio;
  - stati gestiti: collegato, scollegato, cartella non disponibile, file spostato, mancante, modificato, tornato;
  - esclusione di sottocartelle e di interi dischi.
- **Scansione incrementale**:
  - i file non cambiati non vengono riletti;
  - gli ID dei file restano stabili, quindi etichette e correzioni sopravvivono alle nuove scansioni;
  - una scansione interrotta viene ripresa al riavvio.
- **Aggiornamenti**:
  - automatici all'avvio e quando un disco torna;
  - programmati (ogni 15 min, 1 h, 6 h o 24 h, scelta nella pagina);
  - manuali, con pulsante di stop.
- **Analisi audio** in background, a bassa priorità, in pausa automatica mentre è aperta una DAW, sospendibile.
  - Misure: durata, sample rate, bit depth, canali, picco, RMS, LUFS, true peak.
  - Metadati incorporati: tag, chunk ACID e `smpl` dei WAV.
  - Stime: BPM, tonalità, nota fondamentale, loop o one-shot, battute, carattere del suono, vettore di timbro.
  - Ogni stima ha **fonte e affidabilità**. Se non è determinabile resta **sconosciuta**: per esempio la tonalità di un loop
    di batteria.
- **Classificazione multipla** (categoria, tipo, genere, carattere, plug-in), con fonte e affidabilità.
  - L'utente corregge BPM, tonalità ed etichette; le correzioni sono registrate e riapplicate dopo ogni nuova analisi.
- **Ricerca in linguaggio naturale** in italiano e inglese con filtri combinati.
  - Mostra cosa ha capito, il punteggio e il perché.
  - Esempi del prompt verificati nei test: "kick profondo e potente per melodic techno a 124 BPM", "Trova un pad
    atmosferico in La minore", "preset di pad per Massive", loop per BPM e tonalità, filtro per disco.
- **Suoni simili**: stima di timbro, dichiarata come tale; sotto il 10 % un suono non viene mostrato.
- **Loop adatti al progetto**: percentuale di stretch e compatibilità di tonalità; gli originali non vengono mai modificati.
- **MIDI**: tracce, note, intervalli, tonalità, BPM, durata, strumenti GM, tipo di pattern, ritmo, battute.
- **Preset leggibili**: Vital, VST3, AU, FXP/FXB. Gli altri sono solo riconosciuti, con la dicitura "contenuto non letto".
  Un preset compare solo se il file esiste.
- **Duplicati** su tre livelli:
  - file identici;
  - stesso audio con nome o formato diverso;
  - suoni simili, segnalati come **non duplicati**.
  Nessuna cancellazione.
- **Integrazione con gli agenti** (Prompt 06): l'agente Smart Library usa il motore e non ha un indice suo.
  - Sound Designer e Plugin Intelligence ricevono i preset presenti.
  - Il DAW Controller riceve il **percorso originale** tramite l'ID del file; con il disco scollegato rifiuta con
    `FILE_NOT_AVAILABLE`.
  - Audio Analysis, Mixing e Mastering ora leggono anche AIFF, CAF, FLAC, MP3, M4A e OGG.
- **Privacy**:
  - tutto in locale, nessun codice di rete nel motore;
  - lette solo le cartelle autorizzate;
  - file originali aperti in sola lettura e mai spostati, rinominati, modificati o cancellati.

## 3. Simulato, provvisorio o da sviluppare
| Voce | Stato |
|---|---|
| Comprensione del linguaggio | **Reale ma a regole** (dizionari IT/EN), non un modello di AI. Frasi complesse non comprese → testo libero. In futuro l'Agent centrale o l'AI Core (D-2, D-3) potranno tradurle nella stessa query strutturata |
| Riconoscimento di strumenti e voce dall'audio, genere dall'audio | **Non sviluppato** (servono modelli, milestone M6). Queste etichette arrivano da nomi, cartelle, tag, preset e MIDI |
| Osservazione continua delle cartelle (file watcher) | **Non sviluppato**: aggiornamento all'avvio, al ritorno del disco, programmato o manuale |
| Disco "TEST-SSD" nei test automatici | **Simulazione nei test**: è un disco descritto nel test, ma i file sono veri e vengono letti davvero dal disco interno. La prova con un volume esterno reale è al §4 punto 2–4 |
| Connettore MOCK dell'Agent centrale | Invariato dal Prompt 06, solo build di sviluppo |
| Windows | Codice portabile, **non verificato** (CI pronta, non ancora pubblicata) |

## 4. I 18 test richiesti (Prompt 07 §18)
Legenda: ✅ verificato con un test eseguito · 🟡 verificato in parte (vedi nota) · ❌ non verificato.

| # | Test | Esito | Come |
|---|---|---|---|
| 1 | Scansione di dischi interni | ✅ | `real_drives`: cartella su disco APFS interno del Mac, riconosciuto come "internal", 8 file indicizzati e analizzati |
| 2 | Scansione di dischi esterni | ✅ | `real_drives`: immagine disco exFAT montata in `/Volumes/COSTEST`, riconosciuta come "external", 8 file |
| 3 | Scansione di SSD e USB | 🟡 | Il volume montato del test 2 segue lo stesso percorso di un disco USB (`/Volumes`, rimovibile, exFAT); il tipo SSD/HDD viene da sysinfo. **Nessuna chiavetta USB fisica provata in questo prompt** (un disco esterno reale era stato provato nel Prompt 04) |
| 4 | Dischi scollegati e ricollegati | ✅ | `real_drives`: `hdiutil detach` → cartella "offline", indice conservato, percorso nullo, scansione rifiutata; ricollegato → stesso disco, stessi ID, 0 nuovi, 0 modificati. Anche `incremental_updates_and_device_states` (disco simulato) |
| 5 | Indicizzazione di file audio | ✅ | WAV, AIFF, AIFC, CAF, FLAC, ALAC, AAC (M4A) da afconvert e un MP3 reale: tutti decodificati e analizzati (`real_files`); test automatico `scans_indexes_and_analyzes_every_format` |
| 6 | Indicizzazione di file MIDI | ✅ | 3 file MIDI di test: tracce, note, BPM incorporato, battute, tonalità, tipo di pattern |
| 7 | Catalogazione dei preset supportati | ✅ | Vital, VST3, FXP (Massive) e AU: nome, plug-in e categoria letti; formati proprietari solo riconosciuti. Solo file creati dai test secondo i formati pubblicati, nessun preset di terzi disponibile |
| 8 | Analisi BPM e tonalità | ✅ | Loop sintetici a 95, 124, 128 e 140 BPM entro 0,5 BPM; su file reali 120,0 BPM esatti nei formati lossless; tonalità La minore riconosciuta; nessuna tonalità inventata per la batteria. Limite AAC: 119,15 invece di 120 (LIMITATIONS §5) |
| 9 | Classificazione automatica | ✅ | Kick, hi-hat, pad, basso, arpeggio, batteria, preset e MIDI con categoria, fonte e affidabilità; correzioni utente mantenute dopo una nuova scansione |
| 10 | Ricerca in linguaggio naturale | ✅ | Tutti gli esempi del prompt (IT/EN) danno il file atteso al primo posto; frasi con numeri di tempo, range, battute, durata, formato, disco |
| 11 | Ricerca semantica | ✅ | Descrizioni del suono ("caldo", "profondo", "aggressivo") e suoni simili: le copie risultano al 100 %, kick e hi-hat sotto il 20 % |
| 12 | Duplicati | ✅ | File identici, stesso audio in WAV e AIFF, simili segnalati a parte; tra disco interno ed esterno reali trovati 8 gruppi identici |
| 13 | Aggiornamento incrementale | ✅ | File nuovo, modificato, spostato, cancellato, ritornato; ID stabili. 100.000 file non cambiati: 0,89 s |
| 14 | Gestione degli errori | ✅ | File WAV rotto → stato "error" con motivo, gli altri file analizzati lo stesso (`originals_never_change_and_bad_files_do_not_stop_the_rest`); cartella non disponibile; scansione annullata; scansione interrotta ripresa al riavvio (`interrupted_scan_is_resumed`); percorso non risolvibile segnalato |
| 15 | Rispetto dei permessi | ✅ | Sottocartella esclusa mai letta; file fuori dalle cartelle autorizzate mai indicizzato; agenti che chiedono file fuori dalle cartelle autorizzate rifiutati (`PERMISSION_DENIED`); nessuna scrittura sui file originali: SHA-256 di tutti i file identico prima e dopo scansione, analisi, duplicati, correzioni e ricerche |
| 16 | Integrazione con gli agenti | ✅ | 13 operazioni dell'agente Smart Library + test `daw_controller_uses_library_paths` (dall'ID del file alla parte di import DAW) |
| 17 | Recupero dei percorsi originali | ✅ | `resolves_original_paths_for_the_daw_controller` + `real_drives` (percorso reale su `/Volumes/COSTEST`, nullo da scollegato) |
| 18 | Prestazioni con librerie grandi | ✅ | `large_library`, build release: 20.000 file 0,16 s, 100.000 file 1,65 s (prima scansione) e 0,89 s (nuova scansione); ricerca ≈ 115 ms su 100.000 file |

## 5. Risultati dei test (2026-11-26, macOS Apple Silicon)
- `cargo test --features temp-connector`: **67 superati, 0 falliti**. I 3 test ignorati sono manuali e sono stati eseguiti a
  parte: `large_library`, `real_files`, `real_drives`.
- Test nuovi del Prompt 07: 19 nel motore e 1 negli agenti. Rivisti i 13 casi dell'agente Smart Library.
- Interfaccia:
  - `tsc` senza errori, build Vite riuscita;
  - pagina controllata nel browser con i dati reali del motore sulla libreria di test: ricerca, "Ho capito", motivi in
    italiano, dettaglio, correzione BPM, suoni simili;
  - app desktop avviata (`npm run app:dev`) senza errori; la migrazione 003 è stata applicata al database reale.
- Problemi trovati durante la verifica e corretti:
  - la prima scansione cresceva in modo quadratico: 12,6 s → 0,16 s per 20.000 file;
  - la somiglianza era troppo generosa (un pad risultava simile al 75 % a un kick);
  - una tonalità veniva inventata per i loop di batteria;
  - un brano intero veniva etichettato come "stem";
  - un numero da solo ("loop di batteria 124") non veniva letto come BPM;
  - le cartelle non venivano riscansionate all'avvio.

## 6. Dipendenze nuove
- **Symphonia 0.6.1** (decodifica audio, Rust puro): licenza **MPL-2.0**. La politica ADR-011 nomina solo licenze
  permissive o LGPL. L'MPL-2.0 è un copyleft "per file": consente di distribuire un'app chiusa se i file di Symphonia
  restano disponibili come sorgente, e non li modifichiamo. Proposta di emendamento in `docs/architecture/DECISIONS.md`
  (ADR-011a, **da confermare da parte tua**).
- Nessun'altra libreria nuova: `plist` (preset AU) e `sysinfo` (dischi, ora anche processi per riconoscere una DAW aperta) erano già presenti.

## 7. Cosa resta da fare a mano
- Provare una chiavetta USB fisica e un disco di rete reale (test 3).
- Provare la pagina Smart Library nell'app con le tue cartelle vere: autorizza una cartella di campioni, attendi l'analisi
  e prova le ricerche.
- Confermare l'emendamento di licenza ADR-011a.
- Windows: tutto, tramite la CI quando sarà pubblicata.
