# COSOUND – Prompt 04: app desktop (rapporto di consegna)

Data: 2026-11-24 · Versione app: 0.1.0 (sviluppo) · Stato: 🟡 **funzionante nella misura implementata, non distribuibile**

## 1. Tecnologia (Prompt 04 §2)
È stata confermata la scelta del Prompt 03: **Tauri 2**. Il core è in Rust, l'interfaccia in React + TypeScript + Tailwind (ADR-001 in `docs/architecture/DECISIONS.md`). Non è stata introdotta nessuna tecnologia nuova.

| Criterio | Perché Tauri 2 |
|---|---|
| Prestazioni | Installer piccolo, poca RAM. La WebView è quella di sistema: WKWebView su macOS, WebView2 su Windows. |
| Accesso ai file | Il core Rust usa direttamente il filesystem: volumi, scansione, hash. La UI non tocca il disco: passa dai comandi del core. |
| Audio / MIDI | `cpal`/`midir` sono previsti nel core (ADR-008). Per ora il microfono usa l'API della WebView: vedi §5. |
| DAW | Le integrazioni DAW (OSC, MIDI virtuale, Accessibility/UI Automation) saranno moduli Rust del core. |
| Distribuzione | `tauri build` produce `.dmg/.app` (macOS) e `.msi/.exe` (Windows), installer indipendenti. |

## 2. Struttura dei moduli (Prompt 04 §14)
```
desktop/src-tauri/src/   core (Rust). Ogni modulo è indipendente e ha i suoi test
  db.rs        database locale SQLite, impostazioni, registro operazioni, annullamento
  library.rs   Smart Library Engine v1 (volumi, cartelle autorizzate, scansione, ricerca)
  projects.rs  progetti + materiale importato (riferimenti + SHA-256)
  daw.rs       rilevamento DAW (nessun collegamento)
  plugins.rs   inventario plug-in (solo file e metadati)
  ail/         Agent Integration Layer + temp_test_connector (solo build di sviluppo)
  lib.rs       solo i comandi esposti alla UI
desktop/src/             interfaccia (React). Parla col core solo tramite src/lib/api.ts
packages/agents/registry.json   catalogo delle capability dei 12 sotto-agenti
```
I motori futuri (MIDI, Audio Analysis, Voice-to-Music, Reference Track, connettori DAW) si aggiungono allo stesso modo, senza toccare la UI esistente:
1. un modulo Rust;
2. i suoi comandi;
3. le capability marcate `available` nel registro.

Un test controlla che il registro e il Capability Bus siano allineati.

## 3. Funzionalità completate (implementate e testate)
| Area | Cosa fa | Test |
|---|---|---|
| Avvio e navigazione | Finestra con barra laterale: Chat, Produzione, Smart Library, Materiale importato, Plug-in, DAW, Agenti, Impostazioni, Account. Identità grafica del sito. IT/EN. | E2E Linux |
| Configurazione guidata | 8 passi: account, nome dell'assistente, lingua, cartelle, librerie da indicizzare, DAW, microfono, preferenze (generi, DAW principale, BPM). Account e cartelle si possono saltare. Si può ripetere dalle Impostazioni. | E2E Linux |
| Nome dell'assistente | Scelto all'avvio, modificabile nelle Impostazioni. Usato nell'intestazione e nella chat. Salvato in locale e, se l'utente lo consente, in `profiles.assistant_name` (Supabase). COSOUND resta il nome del software. Lo stile di comunicazione è predisposto ma disattivato ("in arrivo"). | E2E Linux (locale), test di persistenza |
| Impostazioni | Salvate nel database locale, mantenute al riavvio. | E2E Linux (riavvio) |
| Smart Library | Elenco dei dispositivi: tipo interno/esterno/rete, SSD/HDD, filesystem, stato, spazio libero. Cartelle autorizzate con stato e numero di file. Indicizzazione in background con avanzamento. Ricerca per nome e tipo. Dischi scollegati: indice conservato, file segnati "disco scollegato", riconoscimento al ricollegamento anche con un altro punto di montaggio o un'altra lettera. Revoca della cartella. **Nessun file copiato, spostato o modificato.** | Test Rust (scollega/ricollega) + E2E Linux |
| Importazione | Scelta di WAV, AIFF, MP3, FLAC, OGG, M4A, MIDI, progetti DAW. Provenienza: mio / Suno / Udio / altro generatore AI / sconosciuta. Tipo: brano, stem, loop, campione, voce, MIDI, progetto, preset. Impronta SHA-256, file lasciato dov'è. | Test Rust + E2E Linux (hash verificati con `sha256sum`) |
| Progetti | Crea, apri, salva (DAW, BPM, tonalità, note), rinomina, duplica (senza copiare file), archivia/ripristina, progetti recenti, materiale importato, librerie usate, cronologia delle modifiche. Rinomina e archiviazione si possono annullare. | Test Rust + E2E Linux |
| Registro operazioni | Ogni operazione importante finisce nel registro: progetti, import, cartelle autorizzate/revocate, permessi, nome dell'assistente. È visibile nelle Impostazioni e nella cronologia del progetto. | Test Rust + E2E Linux |
| DAW Manager | Per Logic, Ableton, FL, Pro Tools, Cubase e REAPER mostra rilevamento, versione (da `Info.plist` su macOS; su Windows "non accessibile"), percorso, compatibilità, via di controllo prevista e limiti (dalla knowledge base). **Lo stato è sempre "Non collegata", perché non esiste ancora un connettore.** | Test Rust (bundle simulati) + E2E Linux (REAPER simulato) |
| Plugin Manager | VST3, AU, CLAP e VST2 nelle cartelle standard. Nome, formato, strumento/effetto, produttore e versione letti da `moduleinfo.json` e `Info.plist`. I plug-in non vengono caricati. Livello di controllo: "non misurato". | Test Rust + E2E Linux |
| Agent Panel | Mostra: l'Agent centrale (non collegato, in sviluppo); la porta AIL attiva; le decisioni D-1…D-6 in attesa; i 12 sotto-agenti con capability disponibili o previste; le operazioni eseguite con risultato o errore; le richieste di conferma (vuote, con spiegazione). | E2E Linux |
| Agent Integration Layer | Interfaccia `AgentPort` e porta `NoAgent`, che risponde onestamente "non collegato". Il Capability Bus ha 4 capability reali in sola lettura: `smart_library.search`, `smart_library.stats`, `plugin_intelligence.scan`, `daw_controller.detect`. Le capability previste restituiscono `NOT_IMPLEMENTED`: mai simulate. | Test Rust |
| Account | Accesso e creazione account con Supabase (lo stesso del sito); profilo; piano attivo e funzioni incluse; stato della prova, scadenza, prossimo rinnovo. La durata della prova si configura in `app_settings.trial_days` (3 o 7, ora 7). "Gestisci abbonamento" è disattivato: i pagamenti non sono attivi. | E2E Linux: connessione verificata con credenziali errate, risposta del server mostrata. Lettura di `app_settings` verificata. Accesso con un account reale **non testato** (vedi §6). |
| Permessi | Microfono, dispositivi esterni, rilevamento DAW e scansione plug-in si attivano e disattivano dalle Impostazioni. Le cartelle sono revocabili. Ci sono le istruzioni per revocare il permesso di sistema del microfono. Nessun permesso viene chiesto prima che serva. | E2E Linux |

## 4. Funzioni provvisorie (chiaramente marcate)
| Funzione | Perché è provvisoria | Come viene rimossa |
|---|---|---|
| **Connettore di test temporaneo** (`ail/temp_test_connector.rs`, feature `temp-connector`) | Serve a provare chat, Capability Bus ed errori senza l'Agent centrale. Accetta solo `/call <capability> {json}`, non capisce il linguaggio e non pianifica. Mostra un banner giallo fisso. È compilato **solo** in sviluppo (`npm run app:dev`); le build di release non lo contengono. | Quando l'Agent arriva: si scrive il suo adattatore `AgentPort`, si cancella la cartella del connettore e la feature. |
| Identità dei volumi | Impronta = filesystem + dimensione + etichetta. Due dischi identici e vuoti si confonderebbero. | Passaggio all'UUID di volume (DiskArbitration su macOS, numero di serie del volume su Windows) come da `specs/SMART-LIBRARY.md`. |
| Indicizzazione | Ogni scansione rifà l'indice della cartella da capo. | Scansione incrementale (mtime/size) e file watcher per librerie molto grandi. |
| Catalogo dei sotto-agenti | Un solo `registry.json` invece di un `agent.yaml` per sotto-agente. | Quando i pacchetti avranno istruzioni e schemi. |

## 5. Funzionalità ancora da sviluppare
- **Agent centrale**: collegamento tramite AIL quando il socio consegna il codice (D-1…D-6). La chat oggi non risponde e lo dice.
- **Voice-to-Music**: la registrazione dal microfono è pronta: pulsante, livello in tempo reale, sei modalità, file salvato e riascoltabile. Non c'è trascrizione né conversione in MIDI: **non simulata**. È il motore della milestone M5.
- **Audio Analysis / Stems** (M6): analisi e separazione del materiale importato.
- **MIDI Engine / Music Theory** (M2), **Reference Track** (M8), **Mix/Master** (M9).
- **Connettori DAW** (M4): REAPER e Logic per primi. Nessuna DAW risulta collegata.
- **Plugin Intelligence** (M7): mappe dei parametri e livello di controllo ("mano umana").
- **Safety Gate con conferme e backup**: il modello c'è (registro + annulla), ma le conferme medio/alto rischio servono solo quando un agente proporrà operazioni sulle DAW.
- **Pagamenti** (Stripe), rinnovo automatico con consenso registrato (`consents`), elenco dispositivi.
- **Firma e distribuzione**: Apple Developer ID e notarizzazione, firma Authenticode Windows, aggiornamenti automatici firmati.
- Stile di comunicazione dell'assistente (predisposto, disattivato).

## 6. Rapporto dei test
Ambiente: server Linux (Ubuntu 26.04). L'app gira dentro uno schermo virtuale Xvfb ed è stata comandata con clic reali (xdotool), con gli screenshot verificati a ogni passo. Come dati di prova sono stati usati una libreria campione, un plug-in VST3 e un'installazione REAPER simulata.

| Verifica (Prompt 04 §16) | Risultato | Come |
|---|---|---|
| Avvio dell'applicazione | ✅ Linux · ⏳ macOS/Windows | Build di release avviata; vedi riga "Compatibilità" |
| Navigazione | ✅ | Tutte le 9 sezioni aperte e controllate |
| Salvataggio impostazioni | ✅ | Nome "Orion", lingua IT e progetto aperto mantenuti dopo il riavvio; valori controllati nel DB |
| Gestione account | 🟡 | Connessione a Supabase e messaggio d'errore verificati. Accesso con un account reale non provato: servirebbe creare un utente nel database di produzione. |
| Accesso autorizzato ai file | ✅ | Cartella scelta dal dialogo di sistema; indicizzati 8 file su 9 (il `.txt` escluso, come previsto); file non modificati |
| Riconoscimento dispositivi esterni | 🟡 | Test Rust: scollegamento, file "offline", ricollegamento su altro mount → ritrovati. Su Linux c'è un solo disco: test con un vero disco USB da fare su Mac/PC. |
| Gestione microfono | 🟡 | Interfaccia e permessi verificati. Il server non ha microfono e WebKitGTK nega l'accesso: registrazione reale da provare su macOS/Windows. |
| Importazione file | ✅ | 2 file "Suno / Stem" importati; SHA-256 identico a `sha256sum`; nessuna copia creata |
| Visualizzazione DAW | ✅ | 6 DAW mostrate; REAPER simulato "Rilevata"; tutte "Non collegata" |
| Gestione progetti | ✅ | Creazione, libreria collegata, cronologia (E2E); rinomina, archivia, annulla, duplica (test Rust) |
| Compatibilità macOS e Windows | ⏳ **Non ancora verificata** | La CI (file pronto in `docs/desktop/ci-desktop.yml`, da spostare in `.github/workflows/` con un token con permesso `workflow`) compila, testa e crea gli installer (non firmati) su macOS e Windows. Parte al prossimo push su GitHub. |

Test automatici: `cargo test --features temp-connector` → **8 test superati, 0 falliti** (library, projects ×2, daw, plugins, ail ×2, temp connector). `tsc` senza errori.

Problemi trovati durante i test E2E e corretti:
1. Il messaggio "accesso al microfono negato" non compariva.
2. Percorsi lunghi allargavano il pannello progetto.
3. Le etichette dei menu nell'importazione erano disallineate.
4. L'errore di login di Supabase era solo in inglese.

## 7. Avvio e installazione
Vedi `desktop/README.md`.

Non esistono installer pubblicati né link di download. Il software **non è pronto per la distribuzione**: mancano la verifica su macOS e Windows e la firma del codice.

## 8. Database
- **Locale**: `desktop/src-tauri/migrations/001_init.sql`, sottoinsieme di `docs/architecture/DATABASE.md` §2.6.
- **Cloud**: migrazione `supabase/migrations/20261124120000_desktop_app.sql`, applicata il 2026-11-24. È solo additiva: `profiles.assistant_name`, tabella `app_settings` con `trial_days = 7` e vincolo 3 o 7.

## 9. Screenshot (test E2E del 2026-11-24, app reale)
`docs/desktop/screenshots/`: configurazione guidata, chat con il connettore di test, produzione, Smart Library, importazione Suno, agenti, DAW, plug-in. I dati mostrati sono dati di prova, su Linux.
