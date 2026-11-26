# COSOUND – Project status

| Prompt | Contenuto | Stato | Dove |
|---|---|---|---|
| 01 | Sito pubblico, account Supabase, area personale, admin, IT/EN | ✅ Completato | `web/`, `supabase/`, `docs/SETUP.md` |
| 02 | Ricerca tecnica e musicale, knowledge base | ✅ Completato (2026-11-24) | `docs/knowledge/`, sintesi in `docs/knowledge/PROMPT-02-SUMMARY.md` |
| – | Direttiva architetturale: Agent centrale unico + 12 sotto-agenti | 📌 Requisito permanente | `docs/ARCHITECTURE-DIRECTIVE.md` |
| – | Architettura multi-agent | 📝 Proposta v0.1, da rivedere sull'Agent centrale | `docs/architecture/MULTI-AGENT-ARCHITECTURE.md` |
| – | Direttiva di sviluppo parallelo (Agent Integration Layer, connettore temporaneo) | 📌 Requisito permanente | `docs/ARCHITECTURE-DIRECTIVE.md` |
| 03 | Architettura software (15 consegne) | 🟡 **Provvisoria** – completa tranne le parti che dipendono dall'Agent centrale (decisioni D-1…D-6) | `docs/architecture/README.md` |
| 04 | App desktop macOS/Windows (Tauri 2) | 🟡 **Funzionante nella misura implementata, testata su Linux e macOS (2026-11-25); Windows da verificare (CI pronta, non ancora pubblicata); non distribuibile** | `desktop/`, rapporto `docs/desktop/PROMPT-04-REPORT.md` |
| 05 | A: verifica del Prompt 04 su macOS · B: motore musicale M2 (teoria, generazione MIDI, export, piano-roll) | ✅ **Completato (2026-11-25)**; restano a mano microfono e accesso con account reale | `desktop/src-tauri/src/music/`, rapporto `docs/desktop/PROMPT-05-REPORT.md` |
| 06 | Multi-Agent System: 13 agenti specializzati, registry, protocollo, Agent API, connettore MOCK | ✅ **Completato (2026-11-26)** per la parte che non dipende dall'Agent centrale: 36 operazioni reali su 53, le altre previste con il componente mancante | `docs/agents/`, rapporto `docs/agents/PROMPT-06-REPORT.md` |
| 07 | Smart Library System: scansione multidrive incrementale, analisi audio, MIDI e preset, classificazione, ricerca IT/EN, somiglianza, loop, duplicati | ✅ **Completato (2026-11-26)** su macOS; 17 dei 18 test richiesti verificati, USB fisica e NAS da provare a mano; licenza Symphonia (MPL-2.0) da confermare (ADR-011a) | `desktop/src-tauri/src/smartlib/`, `docs/smart-library/`, rapporto `docs/smart-library/PROMPT-07-REPORT.md` |
| 08 | Plugin Intelligence System: riconoscimento AU/AUv3/VST3/VST2/CLAP/AAX, catalogo, documentazione, parametri reali, preset, progettazione del suono con ascolto prima/dopo, catene di effetti, adattatore DAW | ✅ **Completato (2026-11-26)** su macOS con le Audio Unit di sistema; 13 dei 16 test verificati per intero, 3 in parte (formati di terze parti, Logic Pro, controllo dentro la DAW) | `desktop/src-tauri/src/plugins/`, `docs/plugin-intelligence/`, rapporto `docs/plugin-intelligence/PROMPT-08-REPORT.md` |
| 09 | Music Intelligence Engine: teoria (note, intervalli, 33 scale, tonalità), accordi e armonia, progressioni, melodie, bassi, arpeggi, ritmo, stili, composizione da richieste a parole IT/EN, modifica MIDI a parole con versioni, analisi MIDI, verifica musicale, ponti Voice-to-Music e Reference Track | ✅ **Completato (2026-11-27)**; 17 dei 19 test verificati per intero, 2 in parte (intonazione dalla voce M5, DAW pilotata M4) | `desktop/src-tauri/src/music/`, `docs/music-intelligence/`, rapporto `docs/music-intelligence/PROMPT-09-REPORT.md` |
| 10 | Prompt Engine: comprensione delle richieste in italiano e inglese, intenti, parametri, contesto e riferimenti, continuità della conversazione, vincoli, ambiguità, operazioni distruttive, istruzioni strutturate per il Central Agent (schema JSON 1.0), personalizzazione dell'assistente, architettura indipendente dal modello | ✅ **Completato (2026-11-27)**; 19 dei 20 test verificati per intero, 1 in parte (lingue oltre IT/EN e voce solo come trascrizione); nessun modello AI collegato (regole deterministiche) | `desktop/src-tauri/src/prompt/`, `docs/prompt-engine/`, rapporto `docs/prompt-engine/PROMPT-10-REPORT.md` |

## Verifica del Prompt 02 (eseguita il 2026-11-24)
- 20 sezioni della knowledge base, ciascuna con `README.md` e registro fonti; tutte le 14 sezioni richieste sono coperte (mappa in `docs/knowledge/README.md`).
- 455 fonti registrate; ogni citazione nel testo ha la sua fonte; 376 URL controllati, tutti raggiungibili o bloccati solo per i controlli automatici.
- Fatti non confermati marcati "(to verify)", fonti non raggiungibili marcate U.
- Sito (`web/`) e database (`supabase/`) non modificati.

## In attesa
- **Agent centrale** (in sviluppo dal socio): al suo arrivo, analisi tecnica e adattamento dei contratti dei sotto-agenti.
- **Pro Tools Scripting SDK**: va scaricato con un account Avid per leggerne documentazione e licenza.
- Verifiche pratiche su Mac/PC con le DAW installate (elenco in `PROMPT-02-SUMMARY.md` §13.5).

## Prompt 03 – stato delle consegne (2026-11-24)
| # | Consegna | Stato | File |
|---|---|---|---|
| 1 | Documento architettura | provvisorio | `docs/architecture/ARCHITECTURE.md` |
| 2 | Diagramma generale | fatto | `ARCHITECTURE.md` §2 |
| 3 | Diagramma Agent centrale + sotto-agenti | provvisorio (Agent non analizzato) | `ARCHITECTURE.md` §4, `MULTI-AGENT-ARCHITECTURE.md` |
| 4 | Schema comunicazione | provvisorio (trasporto verso l'Agent = D-1) | `ARCHITECTURE.md` §5, `AGENT-INTEGRATION-LAYER.md` |
| 5 | Struttura database | fatto (migrazioni non ancora scritte) | `DATABASE.md` |
| 6 | Struttura cartelle | fatto | `ARCHITECTURE.md` §12 |
| 7 | Tecnologie e motivazioni | fatto (ADR-005/006 in parte in attesa dell'Agent) | `DECISIONS.md` |
| 8–13 | Specifiche Smart Library, Voice, Audio/Stems, Reference, Plugin, DAW | fatto | `docs/architecture/specs/` |
| 14 | Rischi e limitazioni | fatto | `RISKS-ROADMAP.md` §1 |
| 15 | Roadmap tecnica | fatto | `RISKS-ROADMAP.md` §2 |

## Componenti software
| Componente | Stato |
|---|---|
| Sito, auth, admin (Prompt 01) | ✅ completato |
| Knowledge base (Prompt 02) | ✅ completato |
| App desktop: shell, navigazione, IT/EN, configurazione guidata, nome assistente, impostazioni e permessi, registro operazioni | ✅ completato e testato (Linux E2E) |
| Smart Library (Prompt 07): volumi, cartelle autorizzate, scansione incrementale, analisi audio (BPM, tonalità, carattere) con fonte e affidabilità, MIDI, preset leggibili, classificazione correggibile, ricerca in linguaggio naturale IT/EN, suoni simili, loop per il progetto, duplicati, aggiornamenti automatici | ✅ completato e testato (dischi interni ed esterni reali su macOS, 100.000 file) |
| Progetti e importazione materiale (anche Suno/Udio, solo riferimenti + SHA-256) | ✅ completato e testato |
| DAW Manager (solo rilevamento) | ✅ completato e testato · nessuna DAW collegata |
| Plugin Intelligence (Prompt 08): riconoscimento, catalogo, parametri reali, preset, progettazione del suono, catene di effetti | ✅ completato e testato con le Audio Unit di macOS · plug-in di terze parti e DAW non provati |
| Account in-app (accesso, piano, prova 3/7 giorni configurabile) | 🟡 implementato; Supabase raggiungibile da macOS; accesso con utente reale da provare a mano |
| Microfono e registrazioni vocali | 🟡 implementato; su macOS permessi e bundle corretti, registrazione reale da provare a mano (finestra di autorizzazione) |
| Agent Integration Layer (AgentPort, porta "non collegato", Agent API JSON verso gli agenti) | 🟡 provvisorio – adattatore in attesa dell'Agent (D-1…D-6) |
| Multi-Agent System (Prompt 06): runtime, protocollo con 6 stati, registry di agenti/strumenti/permessi, approvazioni solo dall'utente, annullamento, eventi, registro `ail_tasks` | ✅ completato e testato |
| 13 agenti specializzati | ✅ tutti con operazioni reali (82 su 97 dopo il Prompt 09) · Voice-to-Music: `notes_to_music` operativa, trascrizione dall'audio prevista (motore M5) · 15 operazioni previste |
| Misuratore audio WAV (BS.1770, true peak, correlazione), template di arrangiamento, catalogo suoni e synth, Music Memory | ✅ completati e testati |
| Connettore MOCK dell'Agent centrale (ex connettore di test) + script dello scenario | ⚠️ **simulazione**, solo build di sviluppo, da eliminare all'arrivo dell'Agent |
| Compatibilità macOS/Windows, installer | 🟡 macOS verificato il 2026-11-25 (build, test, avvio, installer .app/.dmg aarch64 generati, non firmati); Windows e Linux verificati dalla CI (`.github/workflows/desktop.yml`, attiva dal 2026-11-26: test Rust, `tsc`, build Tauri su tre sistemi); app non ancora provata a mano su Windows; installer non firmati, nessun download pubblicato |
| Motore musicale M2: teoria musicale, generazione MIDI di accordi, basso, melodia e batteria, export Standard MIDI File, pagina "Bozza musicale" con piano-roll e ascolto | ✅ completato e testato (24 test Rust, 2.980 parti generate e verificate, file letti da `file` e `mido`) |
| Music Intelligence Engine (Prompt 09): teoria, scale e tonalità, accordi, armonia, melodie, bassi, arpeggi, ritmo, 11 stili, composizione a parole, modifica MIDI con versioni, analisi MIDI, verifica musicale, pagina Compositore | ✅ completato e testato (34 test nuovi, 44 composizioni complete verificate in ogni esecuzione) · intonazione dalla voce (M5) e accordi dall'audio (M6) da completare |
| Prompt Engine (Prompt 10): ogni messaggio della chat viene interpretato (intenti, elementi, parametri detti e dedotti, vincoli, conferme, domande di chiarimento) e consegnato al Central Agent come JSON validato; personalizzazione dell'assistente nelle Impostazioni | ✅ completato e testato (34 test nuovi) · modello linguistico, voce e lingue oltre IT/EN da sviluppare |
| Audio Analysis, Voice-to-Music, Reference, Mix/Master, connettori DAW, Plugin Intelligence | 📝 progettati, non implementati (roadmap M4–M9) |
| Integrazione con l'Agent centrale | ⏳ in attesa del codice dell'Agent |
| Pagamenti | ⏸️ non attivi (gestione abbonamento disattivata nell'app) |

## Prompt 04 – riepilogo (2026-11-24)
- **Completato**: vedi `docs/desktop/PROMPT-04-REPORT.md` §3. Test: 8 test Rust superati, type-check UI ok, test E2E dell'app reale su Linux (Xvfb).
- **Provvisorio**: connettore di test (feature `temp-connector`), impronta dei volumi, indicizzazione completa a ogni scansione, catalogo agenti in un solo JSON.
- **In attesa di integrazione**: Agent centrale (D-1…D-6), conferme del Safety Gate (servono quando gli agenti proporranno operazioni DAW).
- **Database cloud**: migrazione `20261124120000_desktop_app.sql` applicata (solo aggiunte: `profiles.assistant_name`, `app_settings.trial_days = 7`).
- **Verificato su macOS reale (2026-11-25)**: avvio, test, disco esterno (scollega/ricollega), rilevamento DAW e plug-in nei percorsi reali, Supabase. Vedi `PROMPT-04-REPORT.md` §10.
- **Ancora da fare a mano**: microfono e accesso con account reale su macOS (richiedono l'utente); tutto su Windows (CI, pronta in `.github/workflows/desktop.yml`, non ancora pubblicata su GitHub: il login usato per il push non ha il permesso `workflow`).

## Prompt 06 – riepilogo (2026-11-26)
- **Completato**: 13 agenti specializzati in `desktop/src-tauri/src/agents/`. Ognuno ha un manifest in `packages/agents/`; i contratti sono in `packages/contracts/`. Il runtime esegue i task senza pianificare; le approvazioni le concede solo l'utente. Agent API JSON per l'adattatore del socio; pagina Agenti nell'app. Test: 47 superati, di cui 24 nuovi, compresa la collaborazione in sei passi e la sostituibilità del MOCK.
- **Simulato**: solo il connettore MOCK e il suo script dello scenario, dichiarati come simulazione.
- **Richiede componenti esterni**: 17 operazioni previste (voce M5, audio M6, plug-in M7, DAW M4, mastering M9, ricerca web e linguaggio naturale tramite Agent centrale o AI Core).
- **In attesa delle specifiche dell'Agent centrale**: decisioni D-1…D-6; procedura in `docs/agents/INTEGRATION-GUIDE.md`.
- **Database locale**: migrazione `002_agents.sql` (`ail_tasks`, `memory_facts`).

## Prompt 07 – riepilogo (2026-11-26)
- **Completato**: motore Smart Library in `desktop/src-tauri/src/smartlib/` (13 moduli), separato dall'agente Smart Library, che
  ora ha 13 operazioni reali. Pagina Smart Library rifatta: ricerca in linguaggio naturale con "Ho capito" e motivi, dettaglio
  con correzioni, suoni simili, loop per il progetto, duplicati, stato dell'indice, esclusioni, aggiornamento programmato.
  Test: 67 superati, più 3 test manuali eseguiti (file reali, dischi reali, 100.000 file).
- **Simulato**: nulla nel motore. Nei test automatici il disco "TEST-SSD" è descritto dal test, ma i file sono letti davvero.
- **Da sviluppare**: riconoscimento di strumenti, voce e genere dall'audio (servono modelli, M6); osservazione continua delle
  cartelle; comprensione di frasi complesse (Agent centrale o AI Core).
- **Database locale**: migrazione `003_smartlib.sql`.
- **Da fare a mano**: chiavetta USB fisica e NAS; prova con le tue cartelle di campioni; conferma della licenza MPL-2.0 di
  Symphonia (ADR-011a).

## Prompt 08 – riepilogo (2026-11-26)
- **Completato**: motore Plugin Intelligence in `desktop/src-tauri/src/plugins/` (11 moduli) e host isolato
  `cosound-auhost` per le Audio Unit (processo separato, incluso nel pacchetto dell'app). Pagina Plug-in rifatta.
  19 operazioni nuove negli agenti. Test: 85 superati.
- **Simulato**: nulla nel motore; il Central Agent resta il MOCK del Prompt 06.
- **Da integrare**: controllo dei parametri dentro le DAW (connettori M4), caricamento di VST3/CLAP, Windows.
- **Database locale**: migrazione `004_plugins.sql`.
- **Da fare a mano**: provare un synth Audio Unit di terze parti; con Logic Pro installato, verificare che i preset
  creati da COSOUND compaiano nel menu del plug-in.

## Prompt 09 – riepilogo (2026-11-27)
- **Completato**: Music Intelligence Engine in `desktop/src-tauri/src/music/` (17 moduli nuovi sopra il motore M2):
  teoria, scale e tonalità, accordi, armonia, melodie, bassi, arpeggi, ritmo, stili, composizione da richieste a parole,
  modifica MIDI a parole con versioni e annullamento, analisi MIDI, verifica musicale, ponti Voice-to-Music e Reference
  Track. Nuova pagina Compositore. 17 operazioni nuove negli agenti. Test: 120 superati.
- **Simulato**: nulla nel motore; il Central Agent resta il MOCK del Prompt 06 (*in attesa dell'Agent*).
- **Da completare**: intonazione dalla voce (M5), accordi dall'audio (M6), note scritte direttamente nella DAW (M4).
- **Database locale**: migrazione `005_music.sql`. Permesso nuovo: "Preferenze musicali" (spento all'inizio).
- **Da fare a mano**: ascoltare alcune composizioni nella DAW; provare la pagina Compositore su Windows.

## Prompt 10 – riepilogo (2026-11-27)
- **Completato**: Prompt Engine in `desktop/src-tauri/src/prompt/` (12 moduli). Ogni richiesta, in italiano o in
  inglese, diventa un'interpretazione JSON versionata (schema 1.0) con intenti, elementi, parametri, vincoli,
  ambiguità, conferme necessarie e istruzioni per i componenti. Non pianifica e non esegue: lo fa il Central Agent.
  Chat con "Cosa ho capito"; scheda Assistente nelle Impostazioni (stile, dettaglio, modalità, lingua). Test: 154
  superati.
- **Simulato**: il Central Agent resta il MOCK, che ora riceve l'interpretazione ed esegue solo operazioni sicure
  (*in attesa dell'Agent*).
- **Da sviluppare**: collegamento a un modello linguistico (l'interfaccia è pronta), trascrizione vocale (M5), lingue
  oltre italiano e inglese, tracce lette direttamente dalla DAW (M4).
- **Database locale**: migrazione `006_prompt.sql`. Contratto nuovo: `prompt-interpretation.schema.json`.
- **Da fare a mano**: provare in chat le tue frasi abituali; con il socio, scegliere come il suo Agent usa
  l'interpretazione e quale modello collegare.
