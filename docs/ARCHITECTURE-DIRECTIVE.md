# COSOUND – Direttiva architetturale fondamentale (permanente)

Ricevuta il 2026-11-24. Vale per **tutti** i moduli di sviluppo successivi.

## Regola centrale
COSOUND ha **un solo Agent centrale**, già sviluppato dal socio del progetto. È il cervello di COSOUND.
- Non creare un secondo Agent centrale, non sostituirlo, non costruire sistemi paralleli con le stesse funzioni.
- Non modificare o riscrivere il suo codice senza autorizzazione.
- Non inventare sue caratteristiche tecniche non verificate sul codice.
- Ogni nuova funzionalità va assegnata al sotto-agente più adatto oppure implementata come componente tecnico (tool) a disposizione dell'Agent centrale.

## Compiti dell'Agent centrale
Ricevere e comprendere le richieste (chat e voce); interpretare prompt musicali e comandi vocali; coordinare i sotto-agenti e assegnare le attività; raccogliere e valutare i risultati; coordinare le operazioni sulla DAW e il flusso di produzione; verificare che le operazioni siano eseguite correttamente; mantenere il contesto del progetto musicale.

## Sotto-agenti (elenco modulare)
1. Music Producer – composizione, arrangiamento, produzione
2. Rhythm Engineer – BPM, groove, batteria, ritmica
3. Sound Designer – synth, preset, timbri, automazioni
4. Music Theory – note, scale, tonalità, accordi, armonia, melodie
5. Voice-to-Music – idee vocali → materiale musicale
6. Audio Analysis – analisi audio, caratteristiche sonore, stems
7. Reference Track – analisi brani di riferimento, ricerca info musicali
8. Smart Library – ricerca e gestione librerie audio/MIDI
9. Plugin Intelligence – riconoscimento e gestione plug-in
10. Mixing Engineer – analisi e lavorazione del mix
11. Mastering Engineer – finalizzazione e controllo tecnico
12. DAW Controller – comunicazione e operazioni sulle DAW

I sotto-agenti collaborano, ma **ogni decisione di coordinamento passa dall'Agent centrale**. Si possono proporre altri sotto-agenti solo se utili e senza duplicare competenze.

## Requisiti di integrazione
- Sotto-agenti indipendenti e specializzati, con strumenti e autorizzazioni adeguati.
- Comunicazioni strutturate e verificabili; i risultati tornano sempre all'Agent centrale, che controlla il flusso.
- Aggiungere sotto-agenti/competenze senza riscrivere il sistema centrale.

## Sicurezza
Nessuna operazione potenzialmente distruttiva senza consenso dell'utente. Previsti: controllo autorizzazioni, conferma delle operazioni importanti, protezione dei progetti, backup quando serve, log delle operazioni, annullamento quando tecnicamente possibile, gestione errori dei sotto-agenti.

## Consegna richiesta prima del Prompt 03
1. Analisi tecnica dell'Agent esistente (se il codice è disponibile)
2. Schema di integrazione con COSOUND
3. Architettura multi-agent
4. Ruoli e responsabilità dei sotto-agenti
5. Comunicazione Agent centrale ↔ sotto-agenti
6. Incompatibilità / problemi tecnici
7. Proposta di integrazione senza duplicazioni

## Stato
- 2026-11-24: l'Agent centrale **è in sviluppo** da parte del socio e non esiste ancora. Verrà consegnato appena pronto.
- Nel frattempo si costruisce tutto il resto (knowledge base, sotto-agenti, tool, connettori DAW) in modo **indipendente dall'implementazione dell'Agent centrale**: ogni sotto-agente espone un contratto documentato (input/output strutturati, permessi, errori) che l'Agent centrale potrà chiamare. Nessun orchestratore centrale viene costruito da noi; per i test si usa al massimo un harness minimo dichiarato come tale, da eliminare all'arrivo dell'Agent.
- All'arrivo del codice: analisi tecnica, verifica di compatibilità dei contratti, adattamento lato sotto-agenti (non lato Agent).
