# COSOUND – Rapporto Prompt 05 (2026-11-25)

Il Prompt 05 ha due parti:
- **Parte A**: verifica su macOS reale di quanto consegnato nel Prompt 04.
- **Parte B**: milestone **M2** della roadmap (`docs/architecture/RISKS-ROADMAP.md` §2), cioè motore di teoria musicale e generazione MIDI.

## 1. Parte A – Verifica su macOS
Risultati completi in `PROMPT-04-REPORT.md` §10. In sintesi, su MacBook Air Apple Silicon con macOS 15.8.1:

| Verifica | Esito |
|---|---|
| Build, test Rust, type check, avvio | ✅ |
| Installer `.app` + `.dmg` aarch64 | ✅ generati e avviati, **non firmati** |
| Disco esterno: scollega e ricollega | ✅ |
| Rilevamento DAW e plug-in nei percorsi reali di macOS | ✅ con installazioni simulate, perché su questo Mac non ci sono DAW né plug-in |
| Supabase raggiungibile | ✅ |
| Microfono | 🟡 da fare a mano: serve la finestra di autorizzazione di macOS |
| Accesso con account reale | 🟡 da fare a mano: servono le credenziali dell'utente |
| CI su GitHub (macOS, Windows, Linux) | ⏳ file pronto in `.github/workflows/desktop.yml`, non ancora pubblicato: il login usato per il push non ha il permesso `workflow` |

## 2. Parte B – Motore musicale M2

### 2.1 Decisione tecnica
ADR-002 prevedeva un processo Python separato per il motore musicale. Teoria e generazione MIDI **non usano nessuna libreria Python**: sono regole deterministiche più un piccolo scrittore di file MIDI. Per questo sono state scritte in Rust dentro il core dell'app (`desktop/src-tauri/src/music/`).

Conseguenze:
- l'app installata funziona senza Python sul computer dell'utente;
- il codice è coperto dagli stessi `cargo test` e dalla stessa CI del resto del core;
- il processo Python resta previsto per analisi audio, stem, voce e reference (M5, M6, M8, M9).

La decisione è registrata come emendamento di ADR-002 in `docs/architecture/DECISIONS.md`. Il contratto verso l'Agent centrale non cambia: i nomi delle capability restano gli stessi, qualunque processo le esegua.

### 2.2 Cosa è stato implementato
**Teoria musicale** (`theory.rs`)
- Note con diesis, bemolli, doppi diesis e doppi bemolli; classi di altezza.
- **Grafia corretta per lettera**: ogni grado della scala prende la lettera successiva. Fa# maggiore si scrive con Mi#, Mib minore con Dob.
- 12 scale e modi: maggiore, minore naturale, armonica e melodica, dorico, frigio, lidio, misolidio, locrio, pentatonica maggiore e minore, blues.
- Armatura di chiave, tonalità relativa e parallela, nomi degli intervalli.
- 13 qualità di accordo: triadi maggiori, minori, diminuite, aumentate, sus2, sus4 e sette tipi di settima.
- Accordi diatonici a tre e quattro suoni su ogni grado di ogni modo.
- Progressioni in numeri romani: maiuscolo e minuscolo, alterazioni (`bVII`), suffissi (`7`, `maj7`, `°`, `ø7`, `sus4`…). Il V in minore diventa maggiore come nella minore armonica.
- Funzione armonica di ogni accordo (tonica, predominante, dominante, in prestito), livello di tensione e riconoscimento delle cadenze: perfetta, plagale, d'inganno, semicadenza, con sensibile, di sottotonica.

**Generazione MIDI** (`generate.rs`), deterministica: stesse impostazioni e stessa variazione danno sempre le stesse note.
- **Accordi**: voicing stretto con **condotta delle parti**, cioè ogni accordo sceglie il rivolto più vicino al precedente. Ritmi: tenuti, stab in levare, pulsazione in ottavi.
- **Basso**: solo note dell'accordo (fondamentale, quinta, ottava). Stili: in levare (techno), sedicesimi rolling (melodic techno), pulsazione in ottavi, sincopato (house), tenuto.
- **Melodia**: note dell'accordo sui tempi forti, movimento per grado o piccoli salti sui tempi deboli. La scala si adatta agli accordi alterati, per esempio Sol# sopra Mi maggiore in La minore. Estensione Do4–Do6, chiusura sulla tonica quando appartiene all'accordo.
- **Batteria** General MIDI sul canale 10: stili techno, melodic techno, progressive house, house, EDM. Cassa in quattro, clap sul 2 e sul 4, hi-hat aperto in levare, swing regolabile (0–100%, al massimo il terzinato), fill nell'ultima battuta.
- 5 preset di genere: Techno, Melodic Techno, Progressive House, House, EDM, con tonalità, progressione, BPM e stili coerenti. Tutto è modificabile.
- **Controlli musicali** su ogni parte generata, restituiti insieme alle note: note nella tonalità o nell'accordo, note dell'accordo sui tempi forti, salti cantabili, accordo completo in ogni battuta, cassa su ogni tempo, niente note sovrapposte, durata rispettata. Una bozza che non supera i controlli **non può essere esportata**.

**File MIDI** (`smf.rs`): Standard MIDI File formato 1, 480 tick per quarto. Una traccia con titolo, tempo, 4/4 e armatura, poi una traccia per parte con nome e programma General MIDI.

**Capability Bus** (`caps.rs`, `ail/mod.rs`): 6 nuove capability reali, in sola lettura, segnate `available` in `packages/agents/registry.json`.

| Capability | Cosa restituisce |
|---|---|
| `music_theory.key_info` | scala, intervalli, armatura, relativa, parallela, triadi e settime diatoniche |
| `music_theory.progression` | accordi con numeri romani, note, funzione, tensione, cadenza finale e di ritorno |
| `music_theory.generate_chords` | parte di accordi + controlli + file MIDI in base64 |
| `music_theory.generate_bassline` | parte di basso + controlli + file MIDI in base64 |
| `music_theory.generate_melody` | melodia + controlli + file MIDI in base64 |
| `rhythm_engineer.generate_drums` | batteria + controlli + file MIDI in base64 |

Tutti i campi di input sono facoltativi: `genre`, `key` (es. `"A minor"`) oppure `tonic` + `mode`, `progression`, `bpm` (40–250), `bars` (1–64), `seed`, `swing` (0–100), `sevenths`, `chord_rhythm`, `bass_style`, `drum_style`, `fill`. Un input non valido risponde con l'errore `SCHEMA_INVALID` e un messaggio chiaro, mai con un risultato inventato. Esempio con il connettore di test:
```text
/call music_theory.generate_bassline {"genre":"melodic_techno","bars":8,"seed":42}
```

**App desktop**: nuova pagina **Bozza musicale**, con testi in italiano e inglese.
- Scelta di genere, tonalità, modo, BPM, battute, progressione, variazione, swing, settime, stili e fill.
- Riquadri con tonalità (scala, armatura, relativa, parallela), accordi (numero romano, funzione, note, cadenza di ritorno) e risultato dei controlli musicali.
- **Piano-roll** delle quattro parti, con accordo di ogni battuta, estensione e nomi dei suoni di batteria.
- **Ascolto** con suoni semplici integrati (Web Audio) e cursore di riproduzione. Le parti si possono escludere.
- **Esporta MIDI**: salva il file nella cartella dati di COSOUND (`generated/`) e lo registra in *Materiale importato* nel progetto aperto, con provenienza "propria". Nessun file dell'utente viene toccato.

### 2.3 Test e verifiche
| Verifica | Risultato |
|---|---|
| Test Rust totali (`cargo test --features temp-connector`) | **24 superati, 0 falliti**, di cui 16 del motore musicale |
| Teoria | Scale di 12 modi su 14 toniche, grafia per lettera, armature da 7 bemolli a 7 diesis, qualità degli accordi diatonici, numeri romani, cadenze |
| Generazione | **2.980 parti generate e verificate**: 5 generi × 5 tonalità × 25 variazioni × 4 parti, più 12 modi × 10 variazioni × 4 parti con stili, swing e fill diversi |
| Condotta delle parti | Nessuna voce si muove più di 5 semitoni tra un accordo e il successivo nella progressione di prova |
| File MIDI | Rilettura con un parser di test, poi due verifiche indipendenti: il comando di sistema `file` riconosce "Standard MIDI data (format 1) using 5 tracks at 1/480"; la libreria Python `mido` 1.3.3 legge tempo, tonalità, 4/4, canali, programmi e coppie nota on/off corrette. Durate corrette: 8 battute a 124 BPM durano 15,48 s |
| Capability Bus | Il test che confronta registro e bus passa; il connettore di test esegue `/call music_theory.generate_bassline` |
| Interfaccia | Type check senza errori. Pagina provata nel browser con i dati reali del motore: rendering, piano-roll, ascolto, esportazione, italiano e inglese |
| App reale | `npm run app:dev` su macOS: compila, si avvia, nessun errore |

I test hanno trovato due errori veri, corretti prima della consegna:
1. Con molto swing, l'ultima nota di basso poteva sforare la fine della bozza.
2. Sopra alcuni accordi alterati, la scala della melodia scartava per errore una nota dell'accordo stesso.

### 2.4 Limiti attuali
- Solo tempo in 4/4 e un accordo per battuta.
- La melodia segue regole di teoria, non sviluppa motivi né frasi lunghe come farebbe un compositore. Non è AI e non lo dichiara.
- Nessuna umanizzazione del tempo oltre allo swing; la dinamica varia di poco in modo controllato.
- L'ascolto in anteprima usa suoni sintetici semplici: serve solo a sentire l'idea.
- **Non è stato provato in una DAW vera**: su questo Mac non ce ne sono. I file passano due lettori indipendenti; resta da aprirli in Logic, Ableton o REAPER.
- Il clic su "Esporta MIDI" è stato provato nell'interfaccia con un backend simulato; la funzione di esportazione è verificata dai test del motore. Il clic nella finestra nativa resta da fare a mano.
- Nessuna lettura o analisi di file MIDI esistenti (arriverà con Smart Library e Audio Analysis).
- Nessun inserimento automatico nella DAW: è la milestone M4 (DAW Controller).

## 3. Collegamento con il Prompt 06
Il Prompt 06 (testo in `docs/prompts/PROMPT-06.md`) chiede agenti specializzati che usino motori reali del Livello 3. Con M2 sono disponibili i motori per **Music Theory Agent** e **Rhythm Engineer Agent**, e una base per **Music Producer Agent** (generi, struttura armonica, cadenze). Il Prompt 06 non è ancora iniziato.

## 4. File principali
- `desktop/src-tauri/src/music/` – `theory.rs`, `generate.rs`, `smf.rs`, `caps.rs`, `mod.rs`
- `desktop/src-tauri/src/ail/mod.rs` – collegamento delle 6 capability al bus
- `desktop/src-tauri/src/lib.rs` – comandi `music_options`, `music_sketch`, `music_export_midi`
- `desktop/src/pages/Sketch.tsx`, `desktop/src/lib/preview.ts` – pagina, piano-roll, ascolto
- `packages/agents/registry.json` – capability segnate `available`
- `docs/architecture/DECISIONS.md` – emendamento ADR-002
