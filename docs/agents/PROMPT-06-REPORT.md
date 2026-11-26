# COSOUND – Rapporto Prompt 06: Multi-Agent System (2026-11-26)

Il Prompt 06 chiedeva gli agenti specializzati e l'integrazione con l'Agent centrale del socio, **senza** costruire l'Agent centrale.
Testo del prompt: `docs/prompts/PROMPT-06.md`. Architettura: `MULTI-AGENT-SYSTEM.md`.

## 1. Consegne (Prompt 06 §10)
| # | Consegna | Dove |
|---|---|---|
| 1 | Architettura completa del Multi-Agent System | `docs/agents/MULTI-AGENT-SYSTEM.md` |
| 2 | Codice degli agenti | `desktop/src-tauri/src/agents/` (13 agenti, runtime, protocollo) e strumenti in `music/`, `audio/`, `sound.rs`, `memory.rs`, `library.rs` |
| 3 | Responsabilità di ogni agente | `docs/agents/AGENTS.md` (generato dai manifest) |
| 4 | Agent API e Agent Protocol | `docs/agents/AGENT-PROTOCOL.md`, schemi JSON in `packages/contracts/` |
| 5 | Agent Registry e Tool Registry | `packages/agents/*.json`, `docs/agents/REGISTRIES.md` |
| 6 | Agent Integration Layer | `docs/architecture/AGENT-INTEGRATION-LAYER.md` §6c e §7, codice in `desktop/src-tauri/src/ail/` |
| 7 | Connettore MOCK | `ail/temp_test_connector.rs` + `agents/scenario.rs` (solo build di sviluppo) |
| 8 | Dipendenze | `MULTI-AGENT-SYSTEM.md` §6: nessuna nuova libreria esterna |
| 9 | Test individuali e di collaborazione | `desktop/src-tauri/src/agents/tests.rs` e test dei singoli moduli |
| 10 | Istruzioni per l'integrazione con il socio | `docs/agents/INTEGRATION-GUIDE.md` |

## 2. Cosa è stato realmente implementato
- **Runtime degli agenti**. Esegue solo i task che riceve: non pianifica, non sceglie agenti e non combina risultati. Gestisce:
  - validazione degli input;
  - attivazione e disattivazione degli agenti;
  - permessi;
  - approvazioni, che **solo l'utente** può concedere;
  - esecuzione sincrona o in background con annullamento;
  - stati, eventi e registro persistente (`ail_tasks`), anche dopo un riavvio.
- **Protocollo** con i sei stati richiesti (PENDING, RUNNING, WAITING_APPROVAL, COMPLETED, FAILED, CANCELLED). Ogni risultato contiene:
  - ID di richiesta, task e trace;
  - agente e versione, operazione e stato;
  - risultato e base dei dati (misurato, da regole, convenzione, stima, web, nome del file);
  - file prodotti con SHA-256, strumenti usati, errori, limiti;
  - lavori suggeriti (`needs`), operazioni DAW proposte e richiesta di approvazione.
- **Agent API**: un messaggio JSON in entrata e uno in uscita, indipendente dal trasporto. È ciò che userà l'adattatore verso l'Agent del socio.
- **Registri**:
  - un manifest JSON per agente, con identità, versione, competenze, operazioni e schema di input, strumenti, permessi, limiti e formati;
  - registro degli strumenti (Livello 3);
  - catalogo dei permessi.
  Un test fallisce se codice e manifest non coincidono.
- **13 agenti, 36 operazioni eseguibili** su motori reali:

| Agente | Operazioni reali | Su cosa si basa |
|---|---|---|
| A Music Producer | struttura, varianti armoniche, piano di produzione con `needs` | template della knowledge base (convenzioni), teoria |
| B Rhythm Engineer | batteria, transizioni (build di rullante, fill, impatto), tempi in ms e swing | motore MIDI |
| C Sound Designer | ricette per ruolo con varianti, synth **installati**, preset in libreria | knowledge base, scansione plug-in, Smart Library |
| D Music Theory | tonalità, progressioni, accordi, basso, melodia, riconoscimento della tonalità, verifica di coerenza | motore di teoria |
| E Voice-to-Music | nessuna: tutte previste (motore M5) | – |
| F Audio Analysis | misure di file WAV e stem | nuovo misuratore BS.1770 |
| G Reference Track | profilo con la fonte di ogni dato e brief per un brano **originale** | dati web forniti dall'Agent, misure audio |
| H Smart Library | ricerca con BPM e tonalità dal nome del file, statistiche, duplicati probabili, dischi | indice locale |
| I Plugin Intelligence | inventario, strumenti per ruolo, scheda dei synth noti | scansione, catalogo |
| J Mixing Engineer | misure per traccia e proposte (mai eseguite) | misuratore, convenzioni di mix |
| K Mastering Engineer | misure del master e confronto con le piattaforme, con l'affidabilità della fonte | misuratore, knowledge base |
| L DAW Controller | rilevamento DAW, piano delle operazioni con metodo verificato per ogni DAW e file MIDI pronti | rilevamento, scrittura nella cartella di COSOUND |
| M Adaptive Workflow | memorizza, ritrova, modifica, cancella scorciatoie e procedure; cancellazione totale con conferma | Music Memory (`memory_facts`) |

- **Nuovi strumenti di Livello 3**:
  - misuratore WAV con loudness ITU-R BS.1770, true peak con sovracampionamento 4x, RMS, correlazione stereo e del basso, bande e silenzi;
  - template di arrangiamento per 9 generi;
  - riconoscimento della tonalità (Krumhansl);
  - transizioni di batteria e tempi;
  - ricette di suono e catalogo dei synth;
  - memoria `memory_facts` con filtro dei dati privati;
  - duplicati e metadati dal nome del file nella Smart Library.
- **App desktop**:
  - pagina Agenti rifatta con, per ogni agente, versione, stato, interruttore, operazioni con rischio, competenze, permessi e limiti;
  - richieste di conferma con Approva e Rifiuta;
  - task con stato e pulsante Ferma;
  - strumenti di Livello 3;
  - memoria dei workflow consultabile, cancellabile e disattivabile;
  - interruttore "Memoria dei workflow" nelle Impostazioni (spento per impostazione predefinita);
  - chat aggiornata ai nuovi risultati e ai comandi del MOCK. Tutti i testi sono in IT/EN.

## 3. Cosa è ancora simulato
Solo il **connettore MOCK** e il suo **script dello scenario** (Melodic Techno a 124 BPM in sei passi). Sono:
- etichettati "MOCK (simulazione)" nel codice, nell'interfaccia e nelle risposte;
- compilati solo nella build di sviluppo;
- privi di decisioni proprie: passi e input sono scritti a mano, e il MOCK passa soltanto i risultati da un passo all'altro.

Nessun agente simula un risultato. Le operazioni non ancora costruite rispondono `NOT_IMPLEMENTED` e indicano cosa manca.

## 4. Cosa richiede componenti esterni (17 operazioni previste)
- Voice-to-Music, tutte le operazioni: motore M5.
- Tempo, tonalità, struttura e stem da audio: motore M6.
- Programmazione dei synth e mappe dei parametri: M7 con un connettore DAW.
- Esecuzione nella DAW e applicazione del mix: connettori M4.
- Mastering elaborato ed esportato: catena di elaborazione M9.
- Ricerca online e brief da linguaggio naturale: Agent centrale o AI Core.
- Apprendimento dalle abitudini nella DAW: flusso di eventi DAW, M4. I tasti premuti non vengono mai registrati.

## 5. Cosa necessita delle specifiche dell'Agent centrale
Le decisioni D-1…D-6 (`AGENT-INTEGRATION-LAYER.md` §8): trasporto, dove ragionano gli agenti, modelli linguistici, proprietà della memoria,
autenticazione, streaming e annullamento. Finché non arrivano, l'adattatore vero non si può scrivere. Tutto il resto è pronto.
La procedura è in `INTEGRATION-GUIDE.md`.

## 6. Test
`cargo test --features temp-connector`: **47 superati, 0 falliti**, di cui 24 nuovi.

| Verifica richiesta (Prompt 06 §9) | Test |
|---|---|
| Identificazione delle competenze, registro coerente | `registry_is_consistent`: 13 manifest validati con lo schema, operazioni dichiarate = implementate, strumenti e permessi esistenti |
| Gestione degli input | `errors_are_typed` e il validatore di schema: campi mancanti, tipi, valori ammessi, tonalità non valide |
| Qualità e struttura degli output | `every_available_operation_runs` esegue tutte le 36 operazioni; `outputs_are_honest_and_specific` controlla base dei dati, limiti, file con hash, proposte non eseguite, mastering mai dichiarato completato |
| Comunicazione tra agenti | `needs` del Music Producer; output della batteria passato al DAW Controller; nessun agente ne chiama un altro |
| Gestione degli errori | 12 codici d'errore verificati, compresi protocollo, task duplicato, file non autorizzato, formato non supportato |
| Autorizzazioni | `permissions_and_enable_disable`, `approvals_only_from_the_user`: l'Agent centrale non può approvare |
| Rispetto dei limiti | operazioni previste mai eseguite; file fuori dalle cartelle autorizzate rifiutati; dati privati rifiutati |
| Comunicazione con il MOCK | `collaboration_melodic_techno_124_via_mock` e il test del connettore MOCK in chat (`/call`, `/scenario`, `/status`, `/cancel`, `/agents`) |
| Sostituibilità del MOCK | `mock_is_replaceable_by_a_json_connector`: lo stesso scenario solo tramite messaggi JSON dà risultati identici |
| Collaborazione tra più agenti | lo scenario a sei passi: 6/6 COMPLETED, stesso BPM e stessa tonalità ovunque, unico trace, file MIDI della batteria reale nel piano DAW, marker dalle sezioni del Producer |
| Annullamento | `background_tasks_can_be_cancelled`: analisi lunga in background fermata |
| Registro delle attività | `activity_log_survives_restart`: task conservati; quelli rimasti a metà diventano CANCELLED con INTERRUPTED |

Il misuratore audio è verificato sui toni di riferimento dello standard: un seno a 997 Hz a 0 dBFS su un canale misura −3,01 LKFS; a −20 dBFS
su due canali misura −20 LKFS. Il true peak rileva il picco tra i campioni di un seno a fs/4 sfasato di 45°. È verificato anche il gating del silenzio.

Durante lo sviluppo i test hanno trovato quattro errori, tutti corretti:
- due attese sbagliate nei miei test del misuratore;
- un numero di carta scritto a gruppi che il filtro della privacy non riconosceva;
- lo schema di `detect_key`, che chiedeva un campo inutile.

Altre verifiche:
- interfaccia: type check senza errori; pagine Agenti e Chat provate nel browser con i dati veri del runtime (scenario MOCK, richiesta di conferma, memoria);
- app reale: si avvia senza errori;
- build senza il MOCK: compila senza avvisi.

## 7. Limiti tecnici ancora presenti
Elenco completo in `MULTI-AGENT-SYSTEM.md` §8. I principali:
- le misure funzionano solo su file WAV;
- nessun BPM né tonalità ricavati dall'audio;
- le bande di frequenza sono approssimative;
- la generazione MIDI è solo in 4/4;
- i piani DAW non vengono eseguiti;
- l'annullamento è cooperativo;
- campi tecnici come "requires" nei manifest sono solo in inglese.
