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
| 06 | Multi-Agent System: agenti specializzati, registry, protocollo, connettore MOCK | ⏳ **Non iniziato** – testo salvato | `docs/prompts/PROMPT-06.md` |

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
| Smart Library v1 (volumi, cartelle autorizzate, indicizzazione, ricerca, dischi scollegati/ricollegati) | ✅ completato e testato |
| Progetti e importazione materiale (anche Suno/Udio, solo riferimenti + SHA-256) | ✅ completato e testato |
| DAW Manager (solo rilevamento) e Plugin Manager (solo inventario) | ✅ completato e testato · nessuna DAW collegata |
| Account in-app (accesso, piano, prova 3/7 giorni configurabile) | 🟡 implementato; Supabase raggiungibile da macOS; accesso con utente reale da provare a mano |
| Microfono e registrazioni vocali | 🟡 implementato; su macOS permessi e bundle corretti, registrazione reale da provare a mano (finestra di autorizzazione) |
| Agent Integration Layer (interfaccia, porta "non collegato", Capability Bus con 10 capability reali) | 🟡 provvisorio – adattatore in attesa dell'Agent |
| Connettore di test temporaneo | ⚠️ **provvisorio**, solo build di sviluppo, da eliminare all'arrivo dell'Agent |
| Compatibilità macOS/Windows, installer | 🟡 macOS verificato il 2026-11-25 (build, test, avvio, installer .app/.dmg aarch64 generati, non firmati); Windows solo via CI (pronta in `.github/workflows/desktop.yml`, non ancora pubblicata su GitHub: il login usato per il push non ha il permesso `workflow`); installer non firmati, nessun download pubblicato |
| Motore musicale M2: teoria musicale, generazione MIDI di accordi, basso, melodia e batteria, export Standard MIDI File, pagina "Bozza musicale" con piano-roll e ascolto | ✅ completato e testato (24 test Rust, 2.980 parti generate e verificate, file letti da `file` e `mido`) |
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
