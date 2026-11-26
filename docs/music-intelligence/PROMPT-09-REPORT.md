# COSOUND – Rapporto Prompt 09: Music Intelligence Engine (2026-11-27)

Il Prompt 09 chiedeva il motore che capisce, compone, modifica e analizza la musica, senza un secondo Central Agent
e senza duplicare gli agenti specializzati. Testo: `docs/prompts/PROMPT-09.md`.
Architettura: [ARCHITECTURE.md](ARCHITECTURE.md). Limiti: [LIMITATIONS.md](LIMITATIONS.md).

## 1. Consegne (Prompt 09 §23)
| Consegna | Dove |
|---|---|
| Codice del motore | `desktop/src-tauri/src/music/` (17 moduli nuovi, più i 5 del motore M2 che restano in uso), pagina `desktop/src/pages/Composer.tsx` |
| Architettura modulare | `ARCHITECTURE.md` §1 |
| Music Theory Engine | `music/notes.rs` (note IT/EN, frequenze, intervalli) |
| Scale & Key Engine | `music/scales.rs` (33 scale), `music/keys.rs` (tonalità, armature, circolo delle quinte, modulazioni, stima) |
| Chord Engine | `music/chords.rs` (36 tipi, rivolti, voicing, condotta delle voci, riconoscimento) |
| Harmony Engine | `music/harmony.rs` (analisi con numeri romani e funzioni, cadenze, sostituzioni, generazione di progressioni) |
| Melody Engine | `music/melody.rs` (7 stili, frasi, ruoli delle note, 7 variazioni) |
| Bassline Engine | `music/bass.rs` (11 stili, rispetto della cassa) |
| Arpeggio Engine | `music/arp.rs` (9 pattern, 4 curve di dinamica) |
| Rhythm & Groove Engine | `music/rhythm.rs` (griglie, euclidei, poliritmi, swing, quantizzazione, umanizzazione) |
| Style Intelligence | `music/styles.rs` + `packages/music-styles/styles.json` (11 stili con fonti, sostituibili e miscelabili) |
| Composition Engine | `music/compose.rs` + `music/request.rs` (richieste a parole IT/EN) |
| MIDI Generation Engine | `music/smf.rs` (Standard MIDI File formato 1, con CC) |
| Modifica MIDI a parole | `music/edit.rs` + versioni in `music/api.rs` (confronto e annullamento) |
| MIDI Analysis Engine | `music/analysis.rs` |
| Musical Validation Engine | `music/validate.rs` |
| Voice-to-Music e Reference Track | `music/voice.rs`, `music/reference.rs` |
| API di integrazione | `music/api.rs`, 15 comandi dell'app, 17 operazioni nuove degli agenti (`ARCHITECTURE.md` §5) |
| Documentazione tecnica | questa cartella; `docs/agents/AGENTS.md` e `REGISTRIES.md` rigenerati |
| Test e risultati | §4 e §5 |
| Limitazioni conosciute | `LIMITATIONS.md` |

## 2. Operativo (funziona davvero, provato)
- **Teoria:**
  - note in italiano e inglese ("Fa diesis", "Sib", "F#4"), MIDI e frequenza, scarto in cent;
  - intervalli con nome corretto per lettera (terza maggiore ≠ quarta diminuita), composti, consonanza;
  - 33 scale costruite da qualunque tonica con l'ortografia giusta, gradi, carattere, contesti d'uso e fonte;
  - tonalità: armatura, relativa e parallela, circolo delle quinte, tonalità vicine, modulazioni con accordi pivot;
  - stima della tonalità con più candidati e affidabilità; avvisa quando i dati non bastano.
- **Accordi e armonia:**
  - 36 tipi di accordo, simboli con radici italiane e basso ("Lam7/Sol"), rivolti, voicing, condotta delle voci;
  - riconoscimento degli accordi dalle note, anche con la quinta omessa;
  - analisi con numeri romani e funzioni, dominanti secondarie, accordi presi in prestito, cadenze;
  - sostituzioni spiegate;
  - progressioni generate per stile e atmosfera, con alternative.
- **Composizione:** l'esempio del prompt ("melodia emozionale e ipnotica in La minore, Melodic Techno a 124 BPM")
  produce tonalità, BPM e stile giusti, una progressione con alternative, la melodia di 8 battute, la verifica e il
  file MIDI. Su richiesta anche accordi, basso, arpeggio e batteria, coerenti tra loro.
- **Melodie, bassi, arpeggi, ritmo:** 7 stili di melodia con motivo, risposta e cadenza; 11 stili di basso che
  seguono l'armonia ed evitano la cassa; 9 pattern di arpeggio; ritmi euclidei, poliritmi, swing, quantizzazione e
  umanizzazione.
- **Stili:** 11 profili con fonti; miscele ("techno con elementi di ambient" = 70/30); profili modificabili
  dall'utente con un file JSON.
- **Modifica MIDI a parole**, con versioni, confronto e annullamento:
  - "Abbassa questa melodia di un'ottava", "Rendi il basso più sincopato", "Allunga le note degli accordi",
    "Rendi questo arpeggio più ipnotico", "Trasforma questa progressione in tonalità minore", "alza di due
    semitoni", "quantizza a 1/16 e umanizza" e altre;
  - cambia solo la parte indicata; le parole non capite vengono mostrate;
  - la versione 1 resta sempre.
- **Analisi MIDI:** tempo e fonte, tonalità con candidati, accordi per battuta, progressione, frasi, ripetizioni,
  griglia. Le note fuori tonalità sono **descritte** (cromatismo di passaggio, blue note, prestito…), mai segnate
  come errori.
- **Verifica musicale:** distingue errore tecnico, scelta stilistica e libertà compositiva; solo gli errori tecnici
  fanno fallire la verifica.
- **Voice-to-Music:** da note cantate già stimate a tempo, griglia, tonalità, accordi suggeriti e MIDI. L'idea
  dell'utente non viene cambiata: le note vengono portate nella tonalità solo se lo si chiede.
- **Reference Track:** ogni dato tiene la sua fonte (misurato da COSOUND, fonte esterna, utente); il materiale nuovo
  riprende solo tratti generali e viene verificato come diverso dal riferimento (progressione diversa, somiglianza
  melodica sotto 0,5).
- **Memoria:** con il permesso "Preferenze musicali" (spento all'inizio), tonalità, BPM e stile preferiti
  completano solo ciò che la richiesta non dice; il rapporto lo dichiara. Senza permesso la memoria non viene letta.
- **Integrazioni:** agenti Music Theory, Rhythm Engineer, Voice-to-Music, Reference Track; il DAW Controller
  riceve il file esportato e prepara l'importazione; Agent API e Central Agent MOCK.
- **Interfaccia:** nuova pagina **Compositore** (IT/EN):
  - richiesta a parole, cosa è stato capito, progressione con funzioni e alternative;
  - piano-roll di ogni parte con ascolto ed esportazione MIDI;
  - verifica musicale per tipo;
  - modifica a parole con versioni, confronto e annullamento, variazioni della melodia;
  - teoria (scale, accordi, intervalli, tonalità), analisi di un file MIDI, composizioni salvate.

## 3. Simulato, provvisorio o da completare
| Voce | Stato |
|---|---|
| Riconoscimento dell'intonazione dalla voce (audio → note) | **Da completare** (motore M5). Il ponte Voice-to-Music lavora su note già stimate; le operazioni di trascrizione restano "previste" |
| Accordi riconosciuti dall'audio | **Da completare** (motore M6). Si analizzano file MIDI; per l'audio si usano tonalità e BPM della Smart Library |
| Note scritte direttamente nella DAW | **Da integrare** con i connettori (M4). Oggi la DAW riceve un file MIDI con il piano di importazione |
| Richieste a parole complesse (negazioni, riferimenti) | **In parte**: il parser riconosce parole chiave IT/EN e mostra quelle non capite. Il Central Agent del socio potrà passare dati strutturati |
| Messaggi dettagliati del motore | In inglese; etichette dell'app in italiano e inglese |
| Ascolto nell'app | Suoni semplici Web Audio, non gli strumenti dell'utente |
| Central Agent | **MOCK** del Prompt 06, invariato: l'esempio del prompt passa dall'Agent API come farà l'adattatore del socio (*in attesa dell'Agent*) |

## 4. I 19 test richiesti (Prompt 09 §22)
Legenda: ✅ verificato con un test eseguito · 🟡 in parte.

| # | Test | Esito | Come (test in `desktop/src-tauri/src/`) |
|---|---|---|---|
| 1 | Riconoscimento delle note | ✅ | `notes::italian_and_english_names_octaves_midi_frequency`: Fa diesis, Sib, La3 = 57, 440 Hz = A4, 446 Hz = +23,4 cent, enarmonie |
| 2 | Calcolo degli intervalli | ✅ | `notes::intervals_by_letter_and_semitones`: Do–Mi = 3M (anche al contrario), Do–Mi♭ = 3m, Do–Fa♯ = 4A ≠ Do–Sol♭ = 5d, composti |
| 3 | Costruzione delle scale | ✅ | `scales::scales_from_any_tonic` (33 scale, ortografia per lettera, gradi IT/EN), `scales_that_fit_some_notes` |
| 4 | Riconoscimento delle tonalità | ✅ | `keys::estimation_with_candidates_and_guards`, `parse_italian_and_english_keys`, `signatures_circle_related_and_modulation`, `harmony` (tonalità dagli accordi) |
| 5 | Costruzione degli accordi | ✅ | `chords::symbols_spelling_and_types`, `inversions_voicings_and_leading`, `chords_recognized_from_notes` |
| 6 | Generazione delle progressioni | ✅ | `harmony::generated_progressions_are_coherent_varied_and_styled`, `analysis_roman_functions_secondary_and_borrowed`, `substitutions_explained` |
| 7 | Composizione melodica | ✅ | `melody::every_style_is_musical`, `phrases_answer_questions_and_motifs_repeat`, `variations_keep_identity` |
| 8 | Generazione delle bassline | ✅ | `bass::every_style_follows_the_harmony_and_respects_the_kick` |
| 9 | Generazione degli arpeggi | ✅ | `arp::arpeggios_follow_the_chords` |
| 10 | Pattern ritmici | ✅ | `rhythm::euclid_polyrhythm_swing_quantize_humanize`, `grid_position_duration_are_distinct` (E(3,8) = tresillo) |
| 11 | Generazione MIDI | ✅ | `compose::every_style_full_composition_validates`: 11 stili × 4 varianti, ogni file riletto, ogni nota presente nel file; `smf::writes_a_valid_format_1_file` |
| 12 | Modifica MIDI | ✅ | `edit::the_prompt_examples_in_words`, `edits_change_the_real_notes`, `api::compose_edit_compare_undo_export` (versioni, confronto, annullamento) |
| 13 | Analisi MIDI | ✅ | `analysis::analyses_a_generated_file_and_finds_what_was_written`, `outside_notes_are_described_not_judged` |
| 14 | Coerenza armonica | ✅ | `validate::errors_choices_and_freedom_are_told_apart`; zero errori tecnici in tutte le composizioni dei test 7, 8, 11 |
| 15 | Compatibilità tra melodia e basso | ✅ | `validate` (dissonanze melodia/basso, incroci di registro), `bass` (note dell'accordo ai cambi) |
| 16 | Collaborazione con Voice-to-Music | 🟡 | `voice::hummed_idea_becomes_midi_and_is_not_altered`: un'idea canticchiata stonata e fuori tempo diventa La3 Do4 Mi4 Re4 Do4 Si3 La3 a 100 BPM in La minore, senza correzioni non richieste. Manca l'intonazione dall'audio (M5) |
| 17 | Collaborazione con Reference Track | ✅ | `reference::sources_are_separated_and_the_result_is_original`: fonti separate, progressione diversa dal riferimento, somiglianza < 0,5 |
| 18 | Integrazione con le DAW | 🟡 | `agents::central_agent_mock_composes_edits_and_exports_music`: il file esportato va al DAW Controller (tempo, tonalità, traccia strumento, `import_midi`). Nessuna DAW pilotata davvero (connettori M4) |
| 19 | Richieste in linguaggio naturale | ✅ | `request::the_prompt_example_and_others`, `compose::the_prompt_example_end_to_end`, esempio completo tramite Agent API nel test MOCK |

Altri controlli eseguiti:
- `api::memory_preferences_only_when_allowed`: senza permesso la memoria non viene letta; con il permesso completa solo
  ciò che manca, e la richiesta vince sempre.
- `agents::every_available_operation_runs`: tutte le 82 operazioni disponibili degli agenti eseguite (17 nuove).
- Pagina Compositore provata nel browser con dati reali del motore: composizione dell'esempio del prompt, modifica a
  parole (versione 2, un'ottava sotto, più sincopata), teoria (Fa♯ minore). Nessun errore in console.

## 5. Risultati (2026-11-27, macOS Apple silicon)
- `cargo test --features temp-connector`: **120 superati, 0 falliti** (35 nuovi: 34 nel motore musicale, 1 negli
  agenti; 3 test manuali ignorati, del Prompt 07). Gli stessi test girano nella CI su macOS, Windows e Linux.
- Interfaccia: `tsc` senza errori, build Vite riuscita.
- Tempi: una composizione completa di 8 battute con verifica e MIDI richiede pochi millisecondi (44 composizioni in
  0,28 s in build di debug).
- Problemi trovati dai test e corretti:
  1. Sib in Do maggiore veniva letto come ♯VI invece di ♭VII.
  2. "Fadd9" veniva letto come Fa + "dd9": ora la radice italiana vale solo se il resto è un tipo di accordo valido.
  3. La tonalità di una progressione veniva stimata dalle sole note; ora contano anche accordo iniziale e finale e
     la cadenza V→I.
  4. La formula dei ritmi euclidei non era quella canonica (E(3,8) deve dare il tresillo).
  5. Il Reference Track sceglieva lo stile più vicino solo dall'intervallo di BPM; ora pesa anche il BPM tipico.
  6. "rendila più sincopata" segnalava "rendila" come parola non capita.

## 6. Dipendenze
- Nessuna libreria nuova. Il lettore MIDI è quello della Smart Library (Prompt 07).
- **Database locale**: migrazione `005_music.sql` (`music_clips`, `music_versions`).
- **Permesso nuovo**: `memory.music` ("Preferenze musicali", spento all'inizio).

## 7. Da fare a mano
- Ascoltare alcune composizioni nella DAW (importare il `.mid` esportato) e dire se lo stile è riconoscibile.
- Provare la pagina Compositore nell'app installata su Windows.
