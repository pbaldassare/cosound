# COSOUND – PROMPT 10
## PROMPT ENGINE – COMPRENSIONE DEL LINGUAGGIO NATURALE E TRASFORMAZIONE DELLE RICHIESTE IN OPERAZIONI MUSICALI

> Testo del prompt ricevuto il 2026-11-27, conservato come riferimento. Rapporto: `docs/prompt-engine/PROMPT-10-REPORT.md`.

1. OBIETTIVO DEL MODULO
Sviluppa il Prompt Engine di COSOUND.
Il suo compito è interpretare le richieste dell'utente, comprenderne il significato musicale e operativo e trasformarle in istruzioni strutturate che possano essere utilizzate dal Central Agent e dai componenti specializzati.
COSOUND deve permettere all'utente di comunicare in linguaggio naturale, come se parlasse con un produttore musicale, un arrangiatore o un tecnico del suono esperto.
L'utente non deve conoscere:

* Comandi MIDI.
* Parametri tecnici dei sintetizzatori.
* Procedure specifiche delle DAW.
* Nomi delle funzioni interne.
* Struttura tecnica degli agenti.
* API e protocolli software.

Deve semplicemente descrivere ciò che vuole ottenere.
Esempio:
"Voglio un brano Melodic Techno a 124 BPM, con un basso profondo, un'atmosfera cinematografica e una melodia emozionale."
Il Prompt Engine deve comprendere la richiesta e trasformarla in una specifica musicale e operativa utilizzabile dagli altri componenti.
ATTENZIONE: non sviluppare un nuovo Central Agent o un secondo sistema di orchestrazione. Il Prompt Engine interpreta e struttura le richieste; il Central Agent, sviluppato dal socio, decide come coordinare ed eseguire i task.
2. COMPRENSIONE DEL LINGUAGGIO NATURALE
Implementa un sistema capace di comprendere richieste scritte e, attraverso le interfacce vocali previste dall'applicazione, richieste pronunciate.
Deve comprendere:

* Richieste semplici.
* Richieste complesse.
* Istruzioni in più fasi.
* Descrizioni musicali.
* Indicazioni tecniche.
* Modifiche a progetti esistenti.
* Richieste incomplete.
* Riferimenti a elementi già presenti nel progetto.
* Richieste successive che modificano solo una parte delle istruzioni precedenti.

Deve riconoscere l'intenzione dell'utente, non limitarsi a individuare parole chiave.
Esempi:
"Rendi il basso più potente."
"Il drop deve essere più energico."
"Non modificare la melodia, cambia soltanto il suono."
"Vorrei un pad più largo e atmosferico."
"Abbassa il kick di un paio di decibel."
Il sistema deve comprendere la differenza tra modifiche musicali, sonore, ritmiche e tecniche.
3. INTERPRETAZIONE DEL CONTESTO
Il Prompt Engine deve poter utilizzare il contesto disponibile per comprendere correttamente le richieste.
Prevedi la possibilità di ricevere informazioni su:

* Progetto musicale aperto.
* DAW utilizzata.
* BPM.
* Tonalità.
* Genere musicale.
* Tracce presenti.
* Strumenti caricati.
* Plug-in disponibili.
* Librerie indicizzate.
* File audio e MIDI.
* Sezione musicale interessata.
* Operazioni già effettuate.
* Preferenze personali autorizzate.

Esempio:
L'utente dice:
"Rendi questo elemento più aggressivo."
Il sistema deve capire a quale elemento si riferisce, utilizzando il contesto del progetto e le informazioni ricevute dal Central Agent.
Se il riferimento non è identificabile con sufficiente certezza, deve chiedere un chiarimento invece di modificare una traccia sbagliata.
4. ANALISI DELL'INTENZIONE
Ogni richiesta deve essere classificata in base alla sua finalità.
Prevedi categorie come:

* Creazione musicale.
* Composizione.
* Arrangiamento.
* Modifica MIDI.
* Sound design.
* Selezione dei sample.
* Ricerca nelle librerie.
* Configurazione dei plug-in.
* Modifica delle tracce.
* Mixing.
* Mastering.
* Analisi audio.
* Analisi di reference track.
* Importazione di file.
* Gestione delle DAW.
* Gestione dei progetti.
* Apprendimento di workflow.
* Richieste informative.
* Gestione delle preferenze.

Una richiesta può appartenere a più categorie contemporaneamente.
5. ESTRAZIONE DEI PARAMETRI MUSICALI
Il Prompt Engine deve individuare automaticamente i parametri musicali contenuti nelle richieste.
Esempi:
Parametri generali

* Genere.
* BPM.
* Tonalità.
* Durata desiderata.
* Atmosfera.
* Energia.
* Stile.
* Riferimenti musicali.

Parametri ritmici

* Groove.
* Swing.
* Sincopi.
* Pattern.
* Intensità.
* Densità ritmica.

Parametri armonici

* Scala.
* Accordi.
* Progressione.
* Tensione.
* Risoluzione.

Parametri sonori

* Tipo di suono.
* Timbro.
* Profondità.
* Brillantezza.
* Calore.
* Aggressività.
* Stereo image.

Parametri strutturali

* Intro.
* Strofa, quando pertinente al genere.
* Breakdown.
* Build-up.
* Drop.
* Bridge.
* Outro.

Il sistema deve distinguere tra parametri esplicitamente richiesti e parametri dedotti.
Non deve inventare valori mancanti.
6. GESTIONE DELLE RICHIESTE AMBIGUE
Quando una richiesta non contiene informazioni sufficienti, il sistema deve comportarsi in modo intelligente.
Deve distinguere tre situazioni:
A. Informazioni mancanti ma non essenziali
Può utilizzare impostazioni predefinite ragionevoli, purché siano compatibili con il contesto e dichiarate quando necessario.
B. Informazioni importanti ma recuperabili
Deve consultare le informazioni disponibili sul progetto, sulle tracce o sulle preferenze dell'utente.
C. Informazioni indispensabili e non disponibili
Deve chiedere una domanda di chiarimento precisa e breve.
Non deve interrompere inutilmente il lavoro con domande superflue.
Non deve interpretare arbitrariamente richieste che potrebbero modificare in modo sostanziale il progetto.
7. COMPRENSIONE DELLE MODIFICHE SUCCESSIVE
COSOUND deve mantenere la continuità delle conversazioni.
Esempio:

1. "Crea un basso profondo."
2. "Rendilo più aggressivo."
3. "Adesso abbassa leggermente il volume."
4. "Mantieni questo suono, ma rendilo più dinamico."

Il sistema deve comprendere che le richieste successive si riferiscono allo stesso elemento, quando il contesto lo conferma.
Deve modificare soltanto gli aspetti richiesti, mantenendo invariati gli altri parametri.
Deve distinguere tra:

* Nuova richiesta.
* Modifica di una richiesta precedente.
* Correzione.
* Annullamento.
* Ripristino.
* Conferma di una versione.

8. PROMPT INTERPRETATION ENGINE
Sviluppa un componente specifico per trasformare il linguaggio naturale in una rappresentazione strutturata.
Ogni richiesta deve poter essere convertita in un oggetto contenente:

* Identificativo della richiesta.
* Utente.
* Progetto.
* Contesto.
* Intento.
* Obiettivo.
* Elementi interessati.
* Parametri espliciti.
* Parametri dedotti.
* Vincoli.
* Risorse necessarie.
* Risultato atteso.
* Requisiti di approvazione.
* Eventuali ambiguità.
* Livello di confidenza.

La struttura deve essere indipendente dal modello AI utilizzato.
Prevedi uno schema JSON versionato e documentato.
Esempio illustrativo:

```
{
  "intent": "modify_sound",
  "target": "bass",
  "goal": "increase_aggressiveness",
  "constraints": {
    "preserve_original_melody": true,
    "preserve_project_tempo": true
  },
  "expected_output": "modified_bass_sound"
}
```

Questo esempio è indicativo: definisci uno schema completo e coerente con l'architettura COSOUND.
9. PROMPT ENHANCEMENT
Il sistema deve poter migliorare internamente la precisione delle richieste senza alterare l'intenzione originale dell'utente.
Esempio:
Richiesta originale:
"Fammi un synth più bello e spaziale."
Interpretazione strutturata:
"Individuare il sintetizzatore interessato e proporre una configurazione con maggiore ampiezza stereo, movimento timbrico, profondità e ambiente spaziale, mantenendo la compatibilità con il contesto musicale."
Il Prompt Engine deve distinguere:

* Parole utilizzate dall'utente.
* Interpretazione semantica.
* Parametri tecnici suggeriti.
* Operazioni effettivamente autorizzate.

Non deve trasformare una richiesta generica in modifiche operative arbitrarie.
10. CONVERSIONE IN ISTRUZIONI OPERATIVE
Il Prompt Engine deve poter convertire una richiesta in una o più istruzioni strutturate destinate ai componenti competenti.
Esempio:
Richiesta:
"Crea una bassline melodica in La minore a 124 BPM."
Possibili istruzioni:

* Music Intelligence Engine: definire tonalità e struttura melodica.
* Rhythm Engine: definire il pattern ritmico.
* Smart Library: cercare sample e risorse compatibili, se richiesto.
* Plugin Intelligence: individuare un sintetizzatore adatto, se necessario.
* DAW Controller: predisporre l'inserimento del MIDI nella DAW, quando autorizzato.

Il Prompt Engine non deve decidere autonomamente l'intera strategia di esecuzione.
Deve fornire le informazioni strutturate al Central Agent, che assegnerà i task agli agenti e ai motori appropriati.
11. GESTIONE DEI VINCOLI
Il sistema deve riconoscere e rispettare le condizioni espresse dall'utente.
Esempi:

* "Non modificare la melodia."
* "Mantieni il BPM originale."
* "Utilizza soltanto i miei sample."
* "Non cambiare il preset."
* "Modifica esclusivamente il drop."
* "Mantieni la struttura del brano."
* "Non eliminare nessuna traccia."
* "Non modificare il file originale."

I vincoli devono essere trasferiti nelle istruzioni operative e mantenuti durante tutte le fasi del task.
Un vincolo esplicito non deve essere ignorato per migliorare un risultato secondo una valutazione autonoma del sistema.
12. GESTIONE DELLE OPERAZIONI DISTRUTTIVE
Il Prompt Engine deve riconoscere le richieste che possono comportare modifiche importanti o irreversibili.
Esempi:

* Eliminare tracce.
* Sovrascrivere file.
* Sostituire preset.
* Cancellare regioni.
* Modificare strutture complete.
* Esportare sopra un file esistente.

Deve contrassegnare queste operazioni e trasmettere al Central Agent le informazioni necessarie per gestire autorizzazioni e conferme.
Non deve eseguire direttamente operazioni distruttive.
13. GESTIONE DELLA LINGUA E DELLA VOCE
Prevedi un sistema multilingue.
Prima fase:

* Italiano.
* Inglese.

Successivamente:

* Altre lingue.

L'utente deve poter parlare nella propria lingua e ricevere risposte coerenti.
Il sistema deve comprendere:

* Termini musicali internazionali.
* Terminologia tecnica delle DAW.
* Abbreviazioni.
* Espressioni colloquiali.
* Descrizioni sonore.
* Comandi vocali.

La lingua utilizzata per comunicare con l'assistente deve poter essere diversa da quella utilizzata internamente nelle API e nei parametri tecnici.
Il Prompt Engine deve poter ricevere trascrizioni vocali dal sistema di input di COSOUND, senza gestire direttamente l'acquisizione del microfono.
14. PERSONALIZZAZIONE DELL'ASSISTENTE
Il sistema deve supportare la personalizzazione dell'assistente da parte dell'utente.
Prevedi:

* Nome personalizzato.
* Lingua.
* Stile comunicativo.
* Livello di dettaglio delle risposte.
* Modalità di interazione.
* Preferenze operative.

L'utente deve poter assegnare un nome al proprio assistente e cambiarlo in qualsiasi momento.
Il nome personalizzato non deve modificare il marchio COSOUND.
L'assistente deve poter mantenere uno stile comunicativo coerente senza alterare la precisione tecnica delle istruzioni.
15. ADAPTIVE CONTEXT E MEMORIA
Il Prompt Engine deve poter utilizzare le informazioni disponibili attraverso le interfacce di memoria previste dall'architettura COSOUND.
Deve distinguere:

* Contesto della conversazione.
* Contesto del progetto musicale.
* Preferenze personali.
* Informazioni operative.
* Workflow appresi.
* Vincoli temporanei.
* Istruzioni permanenti autorizzate.

Esempio:
Se l'utente utilizza abitualmente una determinata DAW, COSOUND può tenerne conto quando interpreta una richiesta, senza dover chiedere ogni volta quale software utilizza.
Le informazioni devono essere aggiornate e consultate attraverso le interfacce di memoria esistenti.
Non creare un secondo sistema di memoria indipendente.
16. GESTIONE DEGLI ERRORI DI INTERPRETAZIONE
Prevedi un sistema per rilevare:

* Richieste non comprese.
* Parametri incompatibili.
* Informazioni contraddittorie.
* Riferimenti non identificabili.
* Risorse mancanti.
* Operazioni non supportate.
* Richieste tecnicamente impossibili.

Il sistema deve spiegare il problema in modo comprensibile e proporre una soluzione concreta.
Non deve dichiarare di poter eseguire un'operazione se le API, la DAW o gli strumenti disponibili non la supportano.
17. MODEL AGNOSTIC ARCHITECTURE
Il Prompt Engine deve essere indipendente dal modello AI utilizzato.
Prevedi un'architettura che permetta di utilizzare e sostituire diversi modelli linguistici, locali o cloud, senza riscrivere l'intero sistema.
Prevedi:

* Model Adapter.
* Prompt Templates.
* Structured Output.
* Validazione degli output.
* Gestione degli errori.
* Versionamento dei prompt.
* Configurazione dei modelli.
* Gestione dei limiti e dei costi.

Le funzioni musicali deterministiche devono rimanere indipendenti dai modelli linguistici.
Il sistema deve poter combinare intelligenza generativa e verifiche tecniche.
18. SICUREZZA E PRIVACY
Prevedi:

* Protezione dei dati dell'utente.
* Gestione dei permessi.
* Controllo delle richieste.
* Validazione degli input.
* Protezione contro istruzioni malevole contenute in file o contenuti esterni.
* Separazione tra contenuti analizzati e istruzioni di sistema.
* Nessuna esecuzione arbitraria di codice generato da un prompt.
* Tracciamento delle operazioni.

Il Prompt Engine deve trattare i contenuti esterni come dati da analizzare, non come autorizzazioni a eseguire operazioni.
19. INTEGRAZIONE CON IL CENTRAL AGENT
Il Central Agent è sviluppato separatamente dal socio.
Il Prompt Engine deve fornire al Central Agent:

* Richiesta originale.
* Interpretazione.
* Intento.
* Contesto.
* Parametri.
* Vincoli.
* Risultato atteso.
* Eventuali ambiguità.
* Requisiti di approvazione.

Il Central Agent deve poter utilizzare queste informazioni per pianificare e coordinare le operazioni.
Non creare un secondo orchestratore.
Se le API definitive del socio non sono ancora disponibili, predisponi un adapter temporaneo MOCK, chiaramente identificato e sostituibile.
Non inventare protocolli definitivi.
20. TEST E COLLAUDO
Prepara test specifici per:

1. Comprensione di richieste semplici.
2. Comprensione di richieste complesse.
3. Interpretazione di descrizioni musicali.
4. Estrazione dei BPM.
5. Estrazione delle tonalità.
6. Riconoscimento degli intenti.
7. Comprensione del contesto.
8. Riferimenti a tracce e strumenti.
9. Gestione delle richieste successive.
10. Rispetto dei vincoli.
11. Gestione delle ambiguità.
12. Identificazione delle operazioni distruttive.
13. Generazione di istruzioni strutturate.
14. Validazione JSON.
15. Comunicazione con il Central Agent MOCK.
16. Gestione multilingue.
17. Personalizzazione dell'assistente.
18. Protezione da input malevoli.
19. Gestione degli errori.
20. Coerenza tra richiesta originale e risultato interpretato.

Prevedi test con richieste reali e casi limite.
21. CONSEGNA
Al termine consegna:

* Codice completo del Prompt Engine.
* Architettura tecnica.
* Natural Language Understanding.
* Intent Recognition.
* Context Interpretation.
* Musical Parameter Extraction.
* Prompt Enhancement.
* Constraint Management.
* Structured Instruction Generator.
* Model Adapter.
* API di integrazione.
* Schemi JSON documentati.
* Template e versionamento.
* Test e risultati.
* Documentazione tecnica.
* Limitazioni conosciute.

Distingui chiaramente tra funzioni operative, simulate e ancora da sviluppare.
22. OBIETTIVO FINALE
COSOUND deve comprendere ciò che l'utente vuole ottenere, anche quando viene espresso in modo semplice, colloquiale o poco tecnico.
Deve trasformare una richiesta naturale in una descrizione precisa dell'obiettivo musicale e operativo, mantenendo intatti i vincoli e le intenzioni originali.
L'utente deve poter comunicare con il proprio assistente come con un vero produttore musicale, senza dover imparare il funzionamento interno del software.
Il Prompt Engine deve essere modulare, indipendente dai modelli AI e perfettamente integrabile con il Central Agent e con tutti gli altri componenti COSOUND.
Non sviluppare un secondo Central Agent e non duplicare le responsabilità già assegnate agli altri moduli.
