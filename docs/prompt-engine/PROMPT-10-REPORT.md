# COSOUND – Rapporto Prompt 10: Prompt Engine (2026-11-27)

Il Prompt 10 chiedeva il motore che capisce le richieste in linguaggio naturale e le trasforma in istruzioni
strutturate per il Central Agent del socio, senza un secondo Central Agent e senza un secondo sistema di
orchestrazione. Testo: `docs/prompts/PROMPT-10.md`. Architettura: [ARCHITECTURE.md](ARCHITECTURE.md).
Schema: [SCHEMA.md](SCHEMA.md). Limiti: [LIMITATIONS.md](LIMITATIONS.md).

## 1. Consegne (Prompt 10 §21)
| Consegna | Dove |
|---|---|
| Codice completo del Prompt Engine | `desktop/src-tauri/src/prompt/` (12 moduli più i test) |
| Architettura tecnica | `ARCHITECTURE.md` |
| Natural Language Understanding | `prompt/text.rs`, `prompt/lexicon.rs` |
| Intent Recognition | `prompt/intent.rs` (19 intenti, atti di conversazione) |
| Context Interpretation | `prompt/context.rs` |
| Musical Parameter Extraction | `prompt/extract.rs` (riusa i motori musicale e dei plug-in) |
| Prompt Enhancement | `prompt/enhance.rs` |
| Constraint Management | `prompt/constraints.rs` |
| Structured Instruction Generator | `prompt/instructions.rs` |
| Model Adapter | `prompt/model.rs` |
| API di integrazione | `prompt/api.rs`; Agent API `prompt.interpret`, `prompt.history`, `prompt.schema`; `AgentPort::send_interpreted`; 9 comandi dell'app |
| Schemi JSON documentati | `packages/contracts/prompt-interpretation.schema.json` (versione 1.0), `SCHEMA.md`, esempio reale `example-interpretation.json` |
| Template e versionamento | `packages/prompt-templates/interpret.v1.json` (versione 1.0.0, changelog) |
| Test e risultati | §4 e §5 |
| Documentazione tecnica | questa cartella; `docs/agents/AGENT-PROTOCOL.md` e `INTEGRATION-GUIDE.md` aggiornati |
| Limitazioni conosciute | `LIMITATIONS.md` |

## 2. Operativo (funziona davvero, provato)
- **Comprensione in italiano e inglese**, anche mescolati come fanno i produttori ("il kick più punchy"):
  - richieste semplici, complesse, in più passi, descrizioni sonore, indicazioni tecniche;
  - riconosce l'intenzione dal senso della frase, non da singole parole:
    - "Vorrei un pad più largo" modifica un pad che c'è, non ne crea uno;
    - "Cerca un kick secco" è una ricerca, non una modifica del suono;
    - "ipnotica" in "crea una melodia ipnotica" descrive la melodia nuova, non è una modifica MIDI;
    - "melodia" in "non modificare la melodia" è un vincolo, non una richiesta di comporre.
- **19 intenti**, anche più di uno per richiesta (creazione, composizione, arrangiamento, modifica MIDI, sound design,
  sample, librerie, plug-in, tracce, mix, mastering, analisi audio, reference, importazione, DAW, progetti, abitudini,
  domande, preferenze).
- **Parametri** generali, ritmici, armonici, sonori, strutturali e tecnici:
  - ogni parametro detto riporta le parole dell'utente da cui viene;
  - ciò che è dedotto è separato e ha la sua fonte (progetto, conversazione, preferenza salvata);
  - nessun valore inventato: "abbassa leggermente il volume" dà direzione e quantità qualitativa, non un numero
    di dB;
  - "100 Hz" non è un tempo; valori impossibili e contraddizioni sono segnalati.
- **Contesto e riferimenti:**
  - "il basso" diventa la traccia "Sub Bass";
  - "questo elemento" usa la traccia selezionata, poi la conversazione;
  - se il riferimento non è certo, il motore fa una domanda con le tracce possibili come scelte e non produce
    istruzioni.
- **Continuità della conversazione:** l'esempio del prompt ("Crea un basso profondo" → "Rendilo più aggressivo" →
  "Adesso abbassa leggermente il volume" → "Mantieni questo suono, ma rendilo più dinamico") resta sullo stesso
  elemento. Distingue nuova richiesta, modifica, correzione ("No, intendevo il pad"), annullamento, ripristino,
  conferma e rinuncia. I vincoli "da ora in poi" valgono per tutta la conversazione.
- **Vincoli:** tutti gli esempi del prompt (non modificare la melodia, mantieni il BPM, solo i miei sample, non
  cambiare il preset, solo il drop, mantieni la struttura, non eliminare tracce, non modificare il file originale),
  più "senza basso". Viaggiano con ogni istruzione.
- **Ambiguità nelle tre situazioni del prompt:**
  - A: valore di partenza dichiarato (tempo e tonalità tipici dello stile);
  - B: recuperato dal progetto o dalle preferenze;
  - C: una domanda breve.
  Nessuna domanda superflua.
- **Operazioni distruttive** (eliminare tracce o regioni, sovrascrivere, sostituire preset, rifare la struttura,
  esportare sopra un file, cancellare la memoria):
  - vengono segnalate con oggetto e reversibilità e richiedono la conferma dell'utente;
  - non sono mai pre-autorizzate né eseguite;
  - "elimina il riverbero dal pad" non è l'eliminazione di una traccia;
  - un vincolo in conflitto ("non eliminare nessuna traccia ma elimina…") porta a una domanda.
- **Prompt Enhancement:** separa parole dell'utente, interpretazione, aspetti tecnici suggeriti e operazioni
  autorizzate. "Fammi un synth più bello e spaziale" diventa "Individuare il Synth e proporre una configurazione con
  più ricchezza timbrica, più spazialità…", con suggerimenti (ampiezza stereo, riverbero e delay, profondità,
  movimento) che non sono autorizzazioni.
- **Istruzioni strutturate** per i componenti competenti (Music Intelligence, Rhythm, Smart Library, Plugin
  Intelligence, Sound Designer, Mixing, Mastering, DAW Controller, Adaptive Workflow…):
  - ognuna dice quale operazione esiste nel registro e se è già disponibile;
  - nessun ordine, nessuna esecuzione: decide il Central Agent.
- **Schema JSON versionato** e validato a ogni uscita.
- **Personalizzazione** nella scheda Assistente delle Impostazioni (prima era "in arrivo"):
  - il nome non sostituisce mai il marchio COSOUND;
  - lingua delle risposte;
  - stile: cordiale, professionale, sintetico, tecnico;
  - dettaglio;
  - modalità: procede da solo, chiede sempre conferma, un passo alla volta.
  Lo stile cambia la risposta, mai le istruzioni tecniche.
- **Sicurezza:**
  - i contenuti esterni (testi, file) sono dati: le frasi che sembrano ordini vengono segnalate e ignorate;
  - nessuna esecuzione di codice;
  - caratteri invisibili rimossi e lunghezza limitata;
  - ogni interpretazione viene registrata localmente.
- **Architettura indipendente dal modello:** adattatore, template versionati, output strutturato validato, controlli
  deterministici, ritorno automatico alle regole, limiti giornalieri di richieste, token e costi, configurazione senza
  segreti.
- **Integrazione:**
  - ogni messaggio della chat passa dal Prompt Engine e arriva alla porta del Central Agent già interpretato;
  - il Central Agent può chiamare `prompt.interpret` dall'Agent API, anche per le trascrizioni vocali;
  - nella chat, "Cosa ho capito" mostra intenti, elementi, parametri detti e dedotti, vincoli, conferme
    necessarie, istruzioni e la domanda di chiarimento con le scelte rapide;
  - c'è il pulsante "Nuova conversazione".

## 3. Simulato, provvisorio o da sviluppare
| Voce | Stato |
|---|---|
| Central Agent | **MOCK** (solo build di sviluppo): riceve l'interpretazione ed esegue solo 4 operazioni sicure (comporre, cercare, loop, strumento adatto); il resto resta "in attesa del Central Agent". *In attesa dell'Agent* |
| Modello linguistico (locale o cloud) | **Da sviluppare**: interfaccia, template, validazione, ritorno alle regole e limiti sono pronti e provati con un modello di prova; nessun modello reale collegato. Oggi capiscono le regole di COSOUND |
| Voce | **In parte**: il motore accetta trascrizioni con affidabilità; la trascrizione dal microfono è del motore Voice-to-Music (M5), non ancora costruito |
| Tracce e plug-in del progetto nella DAW | **Da integrare**: arrivano dal Central Agent o dai connettori DAW (M4); oggi il contesto locale è il progetto aperto in COSOUND, la Smart Library e il catalogo plug-in |
| Lingue oltre italiano e inglese | **Da sviluppare**: spagnolo, francese, tedesco e portoghese vengono riconosciuti e la risposta lo dice |
| Operazioni previste negli agenti (mix applicato, mastering, esecuzione nella DAW) | Indicate come "previste" nelle istruzioni e all'utente |

## 4. I 20 test richiesti (Prompt 10 §20)
Legenda: ✅ verificato con un test eseguito · 🟡 in parte. Test in `desktop/src-tauri/src/prompt/tests.rs` (end to end)
e nei test dei singoli moduli.

| # | Test | Esito | Come |
|---|---|---|---|
| 1 | Richieste semplici | ✅ | `t01`: "Rendi il basso più potente" → sound design sul basso, `increase_aggressiveness`, `sound_designer.modify_sound`; "Abbassa il kick di un paio di decibel" → mix, −2 dB approssimato |
| 2 | Richieste complesse | ✅ | `t02`: l'esempio del prompt (Melodic Techno 124 BPM, basso profondo, atmosfera cinematografica, melodia emozionale) → creazione + composizione + sound design; richiesta in tre passi → tre componenti |
| 3 | Descrizioni musicali | ✅ | `t03`: "pad più largo e atmosferico" (modifica, non creazione, con ampiezza stereo e riverbero suggeriti); "il drop deve essere più energico" |
| 4 | Estrazione dei BPM | ✅ | `t04`: "a 124 BPM", "128bpm", "tempo 90", "BPM 140"; "100 Hz" non è un tempo; 500 BPM impossibile; due tempi = contraddizione |
| 5 | Estrazione delle tonalità | ✅ | `t05`: La minore, Fa diesis minore, Am, Re dorico, C major, Sib maggiore |
| 6 | Riconoscimento degli intenti | ✅ | `t06` (12 richieste, 12 intenti diversi) e `intent::intents_of_real_requests` |
| 7 | Comprensione del contesto | ✅ | `t07`: traccia giusta dal contesto; BPM e tonalità del progetto usati quando mancano; DAW del progetto → preparazione dell'inserimento con approvazione |
| 8 | Riferimenti a tracce e strumenti | ✅ | `t07`, `context::*`: "questo elemento" → domanda con 5 tracce, poi traccia selezionata; "il synth" con due synth → domanda; elemento assente → domanda se si modifica, nuovo se si crea |
| 9 | Richieste successive | ✅ | `t09`: la sequenza del prompt sullo stesso elemento, correzione, annullamento, ripristino, conferma, rinuncia; un'altra conversazione non vede i turni precedenti |
| 10 | Rispetto dei vincoli | ✅ | `t10`, `constraints::the_prompt_examples`: tutti gli esempi del prompt; i vincoli sono in ogni istruzione; "senza basso" esclude la parte; vincoli di sessione |
| 11 | Gestione delle ambiguità | ✅ | `t11`: le tre situazioni A/B/C; nessuna istruzione finché c'è una domanda aperta |
| 12 | Operazioni distruttive | ✅ | `t12`, `safety::*`: sei tipi segnalati, mai pre-autorizzati; conflitto con un vincolo → domanda |
| 13 | Istruzioni strutturate | ✅ | `t13`: "Crea una bassline melodica in La minore a 124 BPM" → `music_theory.compose` con tonalità e BPM; operazioni previste indicate come non disponibili |
| 14 | Validazione JSON | ✅ | `t13`: ogni uscita è validata; versione, intento, affidabilità o vincoli sbagliati vengono rifiutati |
| 15 | Central Agent MOCK | ✅ | `t15`: `prompt.interpret`, `prompt.history`, `prompt.schema` dall'Agent API; dalla chat il MOCK esegue la composizione e non esegue l'eliminazione di una traccia |
| 16 | Gestione multilingue | 🟡 | `t16`: italiano e inglese con lo stesso significato interno, gergo misto, trascrizione vocale poco sicura → conferma; spagnolo riconosciuto come non supportato. Altre lingue da sviluppare; voce solo come trascrizione |
| 17 | Personalizzazione dell'assistente | ✅ | `t17`, `profile::*`: nome "Nova" nelle risposte, marchio COSOUND invariato, stile diverso → risposta diversa ma istruzioni identiche, modalità "chiedi sempre", lingua fissa |
| 18 | Protezione da input malevoli | ✅ | `t18`: ordini dentro un testo esterno ignorati e segnalati; "esegui questo script: rm -rf" rifiutato; caratteri invisibili rimossi; testo troppo lungo accorciato e segnalato |
| 19 | Gestione degli errori | ✅ | `t19`: richiesta incomprensibile, pubblicazione su Spotify, 500 BPM, 90.000 Hz → spiegazione e soluzione concreta, nessuna istruzione; BPM insolito per lo stile → avviso, non blocco |
| 20 | Coerenza richiesta/interpretazione | ✅ | `t20`: testo originale invariato, nessun elemento aggiunto o perso, ogni parametro detto ha le parole dell'utente come prova; l'esempio "synth più bello e spaziale"; un modello non può togliere vincoli né inserire valori impossibili; risposta non valida → regole |

Controlli in più:
- l'Agent Integration Layer interpreta ogni turno anche senza Agent collegato e non invia nulla (`ail::tests`);
- i comandi espliciti del MOCK funzionano come prima;
- il template rifiuta i valori che provano a chiudere i propri delimitatori;
- la configurazione rifiuta credenziali nell'indirizzo.

Interfaccia provata nel browser con interpretazioni reali del motore:
- l'esempio del prompt;
- "Rendi questo elemento più aggressivo" con la domanda e le tracce come scelte;
- l'eliminazione di una traccia con la conferma richiesta;
- la scheda Assistente delle Impostazioni.

Nessun errore in console a pagina caricata da zero.

## 5. Risultati (2026-11-27, macOS Apple silicon)
- `cargo test --features temp-connector`: **154 superati, 0 falliti**.
  - 34 nuovi nel Prompt Engine: 17 end to end e 17 dei moduli.
  - 2 test esistenti aggiornati: la chat ora riceve prima l'interpretazione.
  - 3 test manuali ignorati, del Prompt 07.
- Interfaccia: `tsc` senza errori, build Vite riuscita.
- Tempi: un'interpretazione richiede pochi millisecondi in build di debug, senza rete. I 17 test end to end, con
  circa 100 interpretazioni, finiscono in meno di mezzo secondo.
- Problemi trovati dai test e dalle prove e corretti:
  1. "Adesso abbassa leggermente il volume" non ereditava l'elemento della richiesta precedente.
  2. "Crea una melodia … ipnotica" veniva letta come modifica MIDI.
  3. "Cerca nella mia libreria un kick secco" veniva letta come sound design.
  4. Lo schema richiedeva campi che possono essere nulli; il validatore li leggeva come mancanti.
  5. Le risposte in italiano avevano preposizioni sbagliate ("di il basso", "nel il drop") e "124.0 BPM".

## 6. Dipendenze
- Nessuna libreria nuova.
- **Database locale**: migrazione `006_prompt.sql` (`prompt_turns`).
- **Impostazioni nuove**: `assistant_style`, `assistant_detail`, `assistant_mode`, `assistant_language`,
  `assistant_preferences`, `prompt_model_config`, `prompt_model_usage`.
- **Contratti**: `prompt-interpretation.schema.json` (nuovo); `api-message.schema.json` con `prompt.interpret`,
  `prompt.history`, `prompt.schema`.

## 7. Da fare a mano / con il socio
- Provare nella chat le tue frasi abituali: quelle non capite ci dicono quali parole aggiungere.
- Con il socio: decidere se il suo Agent usa l'interpretazione così com'è (`send_interpreted`) o chiama
  `prompt.interpret` da solo, e quale modello linguistico collegare all'adattatore.
