# COSOUND – PROMPT 08
## PLUGIN INTELLIGENCE SYSTEM – RICONOSCIMENTO E CONTROLLO INTELLIGENTE DEI PLUG-IN

> Testo del prompt ricevuto il 2026-11-26, conservato come riferimento. Rapporto: `docs/plugin-intelligence/PROMPT-08-REPORT.md`.

Sezioni del prompt:
1. Obiettivo: motore tecnico; nessun nuovo Central Agent; non duplicare il Plugin Intelligence Agent del Prompt 06.
   Esempio: basso profondo e caldo per Melodic Techno a 124 BPM con i synth installati.
2. Riconoscimento: VST2, VST3, AU, AAX, CLAP; macOS e Windows. Nome, produttore, versione, formato, categoria, percorso,
   disponibilità, compatibilità DAW. Mai "disponibile" solo perché compare in una lista.
3. Catalogazione multipla. Strumenti: synth, sampler, drum machine, rompler, piano, bassi, orchestrali, acustici,
   generatori. Effetti: EQ, compressori, limiter, multibanda, saturatori, distorsori, riverberi, delay, chorus, flanger,
   phaser, filtri, stereo, modulazioni, creativi, analizzatori. Ricerca per nome, produttore, categoria, funzione.
4. Database locale dei plug-in rilevati: ID, nome, produttore, versione, formato, categoria, percorso, OS, DAW compatibili
   (se verificato), parametri, preset, disponibilità, ultima verifica. Aggiornamento automatico o manuale. Niente doppioni
   tra formati.
5. Knowledge engine: documentazione ufficiale con fonte e versione; dichiarare i limiti, non inventare.
6. Preset: categorie, nome, strumento, parametri, timbro, percorso; ricerca semantica solo su preset reali; formati
   proprietari "individuato ma non analizzato".
7. Controllo dei parametri dei synth (oscillatori, unison, filtri, ADSR, LFO, distorsione, effetti, volume, pan…), rapporto
   parametro → suono, proposta, approvazione, applicazione. Sistema operativo dove le interfacce lo consentono.
8. Interazione, in ordine: API/SDK ufficiali; parametri standard ed esposti dalla DAW; automazione/scripting ufficiali;
   interfaccia grafica solo se verificata. Livelli: riconosciuto, analizzato, parametri disponibili, parametri
   controllabili, preset leggibili, preset modificabili, controllo operativo verificato.
9. Generazione e modifica di preset, ascolto, modifiche successive, mai sovrascrivere gli originali, reversibile.
10. Catene di effetti per obiettivo: ordine, compatibilità, rischio di sovraccarico; mai catene standard senza considerare
    il materiale.
11. Collaborazione con Music Intelligence: suono e note sono responsabilità distinte ma integrate.
12. Integrazione con Smart Library (Prompt 07) tramite le sue API, senza duplicarne il database.
13. DAW: prima Logic Pro, poi Ableton Live, FL Studio, Pro Tools, Cubase, REAPER, tramite il DAW Controller; non inventare
    integrazioni.
14. Compatibilità ed errori: isolamento, registro, report diagnostici.
15. Adaptive Workflow tramite l'interfaccia di memoria esistente, nessuna memoria parallela.
16. Dati per l'interfaccia: elenco, categorie, caratteristiche, preset, info tecniche, confronto, parametri, modifiche
    proposte, prima/dopo, salvataggio. Nessuna seconda applicazione.
17. Privacy e sicurezza: solo directory autorizzate o standard, nessuna modifica ai plug-in o cancellazione di preset,
    conferma per modifiche a progetti/preset; nessun controllo di licenze.
18. Moduli: Scanner, Registry, Knowledge Engine, Preset Analyzer, Parameter Discovery, Control Interface, Preset
    Management, Effect Chain Engine, DAW Integration Adapter, API, logging.
19. Interfacce per il Central Agent del socio; MOCK chiaramente identificato; nessuna API definitiva inventata.
20. Test (16 punti): riconoscimento, formati, catalogazione, versioni, preset, parametri, controllo, creazione/modifica
    preset, ricerca semantica, catene, Smart Library, Logic Pro, incompatibilità, sicurezza, agenti, Central Agent MOCK.
21. Consegna: codice, architettura, moduli, API, adapter DAW, documentazione, test e risultati, formati e plug-in
    verificati, limiti. Distinguere operativo, simulato, da integrare.
22. Obiettivo finale: conoscere e usare davvero gli strumenti installati, nei limiti delle interfacce tecniche.
