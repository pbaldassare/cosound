# COSOUND – Rapporto Prompt 08: Plugin Intelligence System (2026-11-26)

Il Prompt 08 chiedeva il motore tecnico che conosce e usa gli strumenti e gli effetti installati, senza un secondo
Central Agent e senza duplicare il Plugin Intelligence Agent. Testo: `docs/prompts/PROMPT-08.md`.
Architettura: [ARCHITECTURE.md](ARCHITECTURE.md). Formati e plug-in verificati: [FORMATS.md](FORMATS.md).
Limiti: [LIMITATIONS.md](LIMITATIONS.md).

## 1. Consegne (Prompt 08 §21)
| Consegna | Dove |
|---|---|
| Codice completo | `desktop/src-tauri/src/plugins/` (11 moduli più i test), `desktop/src-tauri/auhost/main.swift`, pagina `desktop/src/pages/Plugins.tsx` + `components/PluginIntelligence.tsx` |
| Architettura tecnica | `ARCHITECTURE.md` |
| Plugin Scanner | `plugins/scanner.rs` |
| Plugin Registry | `plugins/registry.rs`, `plugins/categories.rs`, migrazione `004_plugins.sql` |
| Knowledge Engine | `plugins/knowledge.rs`, `packages/plugin-knowledge/apple-au.json` (+ `extract_apple_au.py`) |
| Preset Analyzer | `plugins/presets.rs` |
| Parameter Discovery Engine | `plugins/params.rs` |
| Plugin Control Interface + Preset Management | `plugins/design.rs`, host isolato `plugins/host.rs` + `auhost/main.swift` |
| Effect Chain Engine | `plugins/chains.rs` |
| API | `plugins/api.rs`, 20 comandi dell'app, 19 operazioni nuove degli agenti (`ARCHITECTURE.md` §5) |
| Adapter per le DAW | `plugins/daw.rs` + operazione `plugin_setup_plan` del DAW Controller |
| Documentazione | questa cartella; `docs/agents/AGENTS.md` e `REGISTRIES.md` rigenerati |
| Test e risultati | §4 e §5 |
| Formati e plug-in verificati | `FORMATS.md` |
| Limitazioni | `LIMITATIONS.md` |

## 2. Operativo (funziona davvero, provato su questo Mac)
- **Riconoscimento:**
  - formati AU, AUv3, VST3, VST2, CLAP e AAX nelle cartelle standard di macOS e Windows, più il registro Audio Unit di
    macOS;
  - per ogni plug-in: nome, produttore, versione, architettura (arm64 / x86_64), percorso, stato e motivo, DAW
    compatibili (documentate);
  - un plug-in è "disponibile" solo se è stato trovato davvero;
  - lo stesso plug-in in più formati è un solo prodotto;
  - un plug-in disinstallato diventa "mancante" con la data, e torna se viene reinstallato.
- **Catalogo:**
  - categorie multiple, con fonte e affidabilità;
  - ricerca per nome, produttore, categoria e funzione, anche in italiano ("riverbero", "compressore").
- **Documentazione ufficiale con fonte e versione:**
  - 33 unità Apple e 211 parametri, presi dall'header ufficiale `AudioUnitParameters.h` (SDK 15.5);
  - aggiornabile con file JSON;
  - quando manca la documentazione, il sistema lo dice.
- **Host isolato** (`cosound-auhost`):
  - carica le Audio Unit in un processo separato con limite di tempo;
  - legge i parametri reali e i preset di fabbrica;
  - imposta i valori e li fa variare nel tempo;
  - suona le note e fa il render audio;
  - salva lo stato come nuovo `.aupreset`, rifiutando di sovrascrivere.

  Un plug-in che si blocca viene fermato, uno che si chiude da solo viene segnalato; COSOUND non ne risente.
  L'helper è incluso nel pacchetto `COSOUND.app` (verificato).
- **Parametri:** ogni parametro ha un ruolo sonoro (cutoff, risonanza, drive, attacco, LFO…), ricavato dal nome
  reale. I parametri senza un ruolo riconosciuto non vengono mai mossi.
- **Progettazione del suono** (l'esempio del prompt):
  - sceglie uno strumento installato, aggiunge gli effetti che mancano e imposta i valori di partenza;
  - le note vengono dal motore musicale (tonalità, genere, BPM);
  - le richieste a parole diventano modifiche sui parametri veri, per esempio "Rendi questo basso più aggressivo, ma
    mantienilo caldo e profondo";
  - ascolto prima/dopo con misure e verifica di ogni qualità richiesta;
  - salvataggio come **nuovi** preset;
  - copia nella cartella preset del plug-in solo con permesso e approvazione, con annullamento.
- **Preset:**
  - preset di fabbrica, cartelle dei preset standard e Smart Library (tramite la sua API, senza copiarli);
  - ricerca semantica solo su preset esistenti;
  - timbro misurato facendo suonare il preset;
  - "individuato ma non letto" per i formati proprietari.
- **Catene di effetti:**
  - quattro obiettivi, ordine giustificato, valori con fonte;
  - adattamento all'audio misurato;
  - render dell'audio attraverso i plug-in installati e controllo di sovraccarico e perdita di livello.
- **Integrazioni:**
  - motore musicale per le note: suono e note restano responsabilità distinte;
  - Smart Library;
  - DAW Controller (piano per DAW);
  - memoria condivisa per lo strumento abituale per ruolo, tramite Adaptive Workflow: nessuna memoria parallela;
  - Agent API e Central Agent MOCK.
- **Interfaccia:** pagina Plug-in con catalogo e filtri, scheda tecnica (installazioni, livelli di interazione,
  parametri, preset, documentazione, errori), confronto, progettazione con ascolto e confronto prima/dopo, catene,
  ricerca preset, capacità delle DAW, diagnostica. In italiano e inglese.

## 3. Simulato, provvisorio o da integrare
| Voce | Stato |
|---|---|
| Plug-in di terze parti (Massive, Serum, Vital…) | **Non verificato**: nessuno installato sul Mac di prova. Il codice è lo stesso delle Audio Unit Apple, ma va provato |
| VST3, VST2, CLAP, AAX | **Riconosciuti, non caricati**. Caricare VST3 richiede un host C++ con l'SDK Steinberg; AAX non è caricabile da terzi |
| Controllo dei parametri dentro la DAW | **Da integrare** con i connettori (M4). Oggi il suono arriva alla DAW come preset da scegliere |
| Logic Pro | **Non installato**: preset scritti nella cartella standard delle Audio Unit; la loro comparsa nel menu di Logic non è verificata |
| Windows | Codice per le cartelle e gli eseguibili Windows con test unitari; **niente provato su Windows**, nessun caricamento di plug-in |
| Ruoli dei parametri, qualità misurate | **Reali ma stimati**: dai nomi dei parametri e da misure indirette (centroide, descrittori) |
| Central Agent | **MOCK** del Prompt 06, invariato; lo scenario del Prompt 08 passa dall'Agent API come farà l'adattatore del socio |

## 4. I 16 test richiesti (Prompt 08 §20)
Legenda: ✅ verificato con un test eseguito · 🟡 in parte · ❌ non verificato.

| # | Test | Esito | Come |
|---|---|---|---|
| 1 | Riconoscimento dei plug-in | ✅ | 30 unità reali di macOS riconosciute (23 effetti, 3 strumenti, 4 generatori); seconda scansione senza duplicati |
| 2 | Identificazione dei formati | 🟡 | AU reali; VST3, VST2, CLAP e AAX solo con pacchetti di prova sintetici (nessun plug-in di terze parti installato) |
| 3 | Catalogazione | ✅ | AULowpass → filtro + EQ, AUMultibandCompressor → multibanda + compressore; ricerca "riverbero" e "compressore" |
| 4 | Rilevamento delle versioni | ✅ | Versioni dal registro (1.6.0) e dai file `Info.plist` / `moduleinfo.json`; architetture lette dagli eseguibili |
| 5 | Individuazione dei preset | ✅ | 22 preset di fabbrica di AUDistortion, 13 di AUReverb2; un preset scritto nella cartella preset viene attribuito al plug-in giusto |
| 6 | Analisi dei parametri | ✅ | Parametri reali con unità, intervalli e ruoli; gli ID coincidono con la documentazione Apple |
| 7 | Controllo dei parametri | ✅ nell'host / ❌ nella DAW | Modifiche applicate, render, misure (centroide 354 → 976 Hz), stato ricaricato con lo stesso valore di cutoff |
| 8 | Creazione e modifica dei preset | ✅ | Nuovi `.aupreset`, secondo salvataggio con nome diverso, mai sovrascritto; installazione e annullamento |
| 9 | Ricerca semantica | ✅ | "large hall" → Large Hall; per richieste senza corrispondenza non viene proposto nulla di inventato |
| 10 | Generazione di catene di effetti | ✅ | Catene adattate all'audio, delay a tempo, render e controllo del livello; trovato e corretto un errore (passa-banda al posto del passa-alto) |
| 11 | Integrazione con Smart Library | ✅ | Preset della libreria tramite la sua API; Massive segnalato come "non installato"; nulla copiato |
| 12 | Integrazione con Logic Pro | 🟡 | Logic non installato: matrice delle capacità, piano per il DAW Controller, preset nella cartella standard. Nessuna prova dentro Logic |
| 13 | Gestione delle incompatibilità | ✅ | Plug-in solo x86_64 segnalato su Apple silicon; Audio Unit non registrata segnalata; plug-in non utilizzabile rifiutato prima del caricamento |
| 14 | Sicurezza e autorizzazioni | ✅ | Permessi "Caricare plug-in" e "Scrivere preset"; scrittura fuori da COSOUND solo con approvazione dell'utente; il Central Agent non può approvare; mai sovrascrivere |
| 15 | Comunicazione con gli agenti | ✅ | Tutte le 64 operazioni disponibili eseguite nei test (19 nuove) |
| 16 | Central Agent MOCK | ✅ | L'esempio del prompt interamente tramite Agent API: scansione, scelta del synth, progetto, modifica, ascolto misurato, salvataggio, approvazione dell'utente, piano per Logic |

Altri controlli eseguiti:
- un plug-in che non risponde viene fermato (test del limite di tempo);
- un componente inesistente dà un errore registrato, e COSOUND continua.

## 5. Risultati (2026-11-26, macOS Apple silicon)
- `cargo test --features temp-connector`: **85 superati, 0 falliti** (18 nuovi nel motore, 1 nuovo negli agenti; 3
  test manuali ignorati, del Prompt 07).
- Interfaccia:
  - `tsc` senza errori, build Vite riuscita;
  - pagina controllata nel browser con i dati reali del motore: catalogo, scheda, progettazione, ascolto, catene,
    capacità delle DAW;
  - app desktop avviata senza errori;
  - pacchetto `COSOUND.app` costruito con l'helper incluso.
- Tempi: elenco del registro Audio Unit ≈ 3 s (solo alla scansione); lettura dei parametri di un plug-in 0,01 s;
  render di 4 s in meno di 0,1 s.
- Problemi trovati dalle verifiche misurate e corretti:
  1. "mantienilo caldo e profondo" riapplicava quelle qualità invece di proteggerle. Il filtro si chiudeva e ogni
     modifica avrebbe abbassato il basso di un'ottava.
  2. I parametri in dB con range molto ampio venivano scalati male: la distorsione restava di fatto spenta.
  3. Le correzioni del filtro per "caldo" e "profondo" erano troppo forti (taglio a 68 Hz).
  4. La verifica di "aggressivo" usava una misura non adatta ai bassi; ora ogni qualità ha la misura giusta.
  5. La catena del basso usava un passa-banda e toglieva 16 dB di segnale; ora usa un passa-alto (-4 dB), e una forte
     perdita di livello viene segnalata.
  6. Lo stereo pan veniva letto come ampiezza stereo.

## 6. Dipendenze
- Nessuna libreria nuova. L'helper usa solo framework Apple di sistema (AVFoundation, AudioToolbox) ed è compilato con
  gli strumenti di Xcode già presenti.
- `plist` e `sysinfo` erano già nel progetto.

## 7. Da fare a mano
- Installare un synth di terze parti in formato Audio Unit (per esempio Vital, gratuito) e rifare la scansione. Poi
  provare progettazione e ascolto, e "Leggi i parametri".
- Con Logic Pro installato: salvare un suono, usare "Metti nella cartella preset del plug-in" (dopo aver acceso
  *Scrivere preset* nelle Impostazioni) e verificare che il preset compaia nel menu del plug-in in Logic.
- Windows: tutto, tramite la CI quando sarà pubblicata.
