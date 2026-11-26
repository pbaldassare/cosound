# COSOUND – PROMPT 07
## SMART LIBRARY SYSTEM – GESTIONE INTELLIGENTE DELLE LIBRERIE MUSICALI

> Testo del prompt ricevuto il 2026-11-26, conservato come riferimento. Rapporto: `docs/smart-library/PROMPT-07-REPORT.md`.

Il testo completo è stato fornito in chat. Sezioni: 1 obiettivo (motore tecnico indipendente, nessun nuovo Central Agent);
2 supporto multidrive; 3 riconoscimento dei dispositivi (collegato, scollegato, cartella non disponibile, file spostato,
eliminato, modificato); 4 scansione e indicizzazione (audio, MIDI, campioni, preset; non dichiarare analizzabili formati
proprietari non verificati); 5 analisi audio (durata, sample rate, bit depth, canali, BPM e tonalità con affidabilità,
categoria, ritmo, timbro, spettro, RMS, picco, loudness, metadati; sconosciuto se non determinabile); 6 classificazione
multipla correggibile dall'utente; 7 ricerca in linguaggio naturale con filtri combinati; 8 ricerca semantica e similarità
(stima, non certezza); 9 loop: BPM, durata, battute, transienti, inizio/fine, silenzi, compatibilità ritmica e tonale, mai
modificare gli originali; 10 MIDI: tracce, note, intervalli, tonalità, BPM, durata, strumenti, tipo di pattern, ritmo,
struttura; 11 preset leggibili e collaborazione con Plugin Intelligence, mai dichiarare disponibile un preset non trovato;
12 duplicati per hash, audio identico con nomi diversi, possibili simili (la somiglianza non basta), nessuna cancellazione;
13 aggiornamento incrementale, manuale, programmato, file modificati/eliminati, dispositivi scollegati, efficienza;
14 integrazione con Central Agent e agenti del Prompt 06 (motore ≠ agente, nessuna duplicazione); 15 percorsi originali per
il DAW Controller (Logic Pro prima); 16 privacy local-first; 17 prestazioni (background, blocchi, cache, recupero, bassa
priorità durante la DAW, niente file interi in memoria); 18 test (18 punti); 19 consegna; 20 regola finale.
