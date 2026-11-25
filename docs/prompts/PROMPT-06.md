# COSOUND – PROMPT 06
## MULTI-AGENT SYSTEM – ARCHITETTURA E SVILUPPO DEGLI AGENTI SPECIALIZZATI

> Testo del prompt ricevuto il 2026-11-25, conservato qui come riferimento. Da eseguire dopo il Prompt 05.

### 1. OBIETTIVO DEL LAVORO
Sviluppa il sistema Multi-Agent di COSOUND, composto da agenti AI specializzati nella produzione musicale, nell'analisi audio, nella gestione delle librerie, nella teoria musicale e nell'interazione con le DAW.

ATTENZIONE: IL CENTRAL AGENT NON DEVE ESSERE SVILUPPATO IN QUESTO PROMPT.
Il Central Agent è un componente separato, già in fase di sviluppo da parte del socio del progetto.
Il tuo compito è realizzare gli agenti specializzati e predisporre un sistema di integrazione che permetta al Central Agent di comunicare con loro, assegnare attività, ricevere risultati e verificarne lo stato.
Non creare un secondo orchestratore centrale, non duplicare le responsabilità del Central Agent e non sostituire la sua architettura.

### 2. ARCHITETTURA DEL SISTEMA
Il sistema dovrà essere organizzato su quattro livelli distinti:

**Livello 1 – Central Agent**
- Componente sviluppato separatamente dal socio.
- Riceve le richieste dell'utente.
- Pianifica e coordina il lavoro.
- Decide quali agenti specializzati coinvolgere.
- Raccoglie e valuta i risultati.

In questa fase devi predisporre esclusivamente le interfacce necessarie alla comunicazione con questo componente.

**Livello 2 – Specialized Agents**
Agenti AI specializzati, ciascuno con competenze, strumenti, responsabilità e limiti definiti.

**Livello 3 – Engines & Tools**
Motori tecnici e strumenti utilizzati dagli agenti per svolgere operazioni concrete: analisi audio, elaborazione MIDI, ricerca, gestione file e altre operazioni.

**Livello 4 – Execution Environment**
Ambiente in cui vengono eseguite le operazioni, compresi computer dell'utente, applicazione COSOUND, DAW, librerie musicali e servizi cloud autorizzati.

### 3. AGENTI SPECIALIZZATI DA SVILUPPARE
Progetta e implementa i seguenti agenti, mantenendoli modulari e indipendenti.

**A. MUSIC PRODUCER AGENT** – specialista della produzione musicale.
Competenze: comprensione delle richieste musicali; creazione di idee e strutture musicali; costruzione di arrangiamenti; organizzazione delle sezioni del brano; selezione degli elementi musicali più appropriati; collaborazione con gli altri agenti per una produzione coerente; proposta di modifiche e varianti creative.
Deve poter lavorare con generi come Techno, Melodic Techno, Progressive House, House, EDM e altri stili musicali.

**B. RHYTHM ENGINEER AGENT** – costruzione ritmica.
Competenze: kick, clap, snare, hi-hat e percussioni; pattern ritmici; groove e swing; BPM e suddivisione ritmica; pattern MIDI; variazioni ritmiche; costruzione di drop e transizioni; sincronizzazione degli elementi.

**C. SOUND DESIGNER AGENT** – progettazione e selezione dei suoni.
Competenze: analisi dei timbri; selezione di synth e preset; sound design; bassi, lead, pad, pluck, arpeggi e texture; suggerimenti per la modifica dei parametri dei sintetizzatori; creazione di varianti sonore; collaborazione con Plugin Intelligence e Smart Library.
Deve poter utilizzare i plug-in e gli strumenti effettivamente disponibili e autorizzati sul computer dell'utente.

**D. MUSIC THEORY AGENT** – teoria musicale e composizione.
Competenze: note, diesis, bemolli, toni e semitoni; intervalli e scale; scale maggiori, minori, modali, pentatoniche e blues; tonalità e relative tonalità; accordi, rivolti e progressioni armoniche; melodie e bassline; arpeggi; tensione e risoluzione armonica; coerenza musicale tra le diverse parti.
Deve generare contenuti musicalmente coerenti, non sequenze casuali di note.

**E. VOICE-TO-MUSIC AGENT** – conversione della voce in contenuti musicali.
Competenze: analisi di melodie cantate o fischiettate; riconoscimento delle note quando tecnicamente possibile; analisi di pitch, intervalli, durata e ritmo; individuazione del BPM quando ricavabile; conversione delle melodie in MIDI; conversione di vocalizzazioni ritmiche in pattern; distinzione tra parlato, canto, fischio e percussioni vocali.
L'accesso al microfono deve essere subordinato all'autorizzazione esplicita dell'utente.

**F. AUDIO ANALYSIS AGENT** – analisi dei file audio.
Competenze: analisi di file audio; rilevamento BPM e tonalità; analisi della struttura musicale; individuazione di strumenti ed elementi sonori; analisi di frequenze, dinamica e spettro; rilevamento di sezioni, transizioni, breakdown e drop; analisi di file audio importati dall'utente; analisi di tracce e Stems.
Deve distinguere i dati effettivamente rilevati dalle stime e indicare il livello di affidabilità quando possibile.

**G. REFERENCE TRACK AGENT** – analisi dei brani di riferimento.
Competenze: analisi di brani indicati dall'utente; ricerca di informazioni musicali disponibili online; analisi audio quando il file è fornito o accessibile legittimamente; analisi di BPM, tonalità, struttura, groove e arrangiamento; individuazione delle caratteristiche sonore e produttive; preparazione di indicazioni per creare un brano originale con caratteristiche stilistiche analoghe.
Non deve copiare melodie, registrazioni, arrangiamenti protetti o altri elementi espressivi di opere esistenti.
Deve distinguere chiaramente tra informazioni reperite online e analisi realmente effettuate sul file audio.

**H. SMART LIBRARY AGENT** – gestione delle librerie musicali.
Competenze: ricerca e indicizzazione di sample, loop, MIDI, preset e altri file musicali; ricerca su hard disk interni ed esterni, SSD, USB e percorsi autorizzati; riconoscimento dei dispositivi precedentemente indicizzati; classificazione dei contenuti; ricerca per BPM, tonalità, durata, categoria e caratteristiche sonore, quando disponibili; individuazione di duplicati; restituzione dei percorsi originali dei file; collaborazione con gli altri agenti per individuare le risorse più adatte.
Non deve spostare, modificare o cancellare autonomamente i file originali.

**I. PLUGIN INTELLIGENCE AGENT** – intelligenza relativa ai plug-in e agli strumenti musicali.
Competenze: riconoscimento dei plug-in disponibili; organizzazione per categoria e produttore; analisi delle caratteristiche e delle funzionalità; individuazione dei sintetizzatori e degli effetti adatti a una richiesta; consultazione della documentazione tecnica disponibile; supporto all'utilizzo di strumenti come Massive, Predator e altri sintetizzatori; comunicazione con il DAW Controller per le operazioni autorizzate.
Non deve dichiarare disponibile un plug-in che non è stato effettivamente rilevato.

**J. MIXING ENGINEER AGENT** – mixaggio.
Competenze: bilanciamento dei livelli; equalizzazione; compressione; saturazione; riverbero e delay; stereo imaging; gestione delle frequenze; controllo delle sovrapposizioni sonore; analisi della dinamica; suggerimenti e interventi di mixaggio.
Deve poter analizzare le singole tracce e il mix complessivo, distinguendo le operazioni suggerite da quelle realmente eseguite.

**K. MASTERING ENGINEER AGENT** – mastering.
Competenze: analisi del mix finale; controllo del livello di loudness; true peak e picchi; gamma dinamica; bilanciamento tonale; stereo image; controllo della distorsione; preparazione di versioni adatte alla distribuzione; suggerimenti per differenti destinazioni di pubblicazione.
Non deve dichiarare un mastering completato se non è stato realmente elaborato ed esportato il file.

**L. DAW CONTROLLER AGENT** – comunicazione operativa con le DAW.
Competenze: interazione con le DAW supportate; creazione e gestione delle tracce; inserimento di file audio e MIDI; organizzazione degli elementi sulla timeline; gestione delle regioni e delle clip; interazione con strumenti e plug-in; modifica di parametri e automazioni; gestione delle operazioni autorizzate dall'utente.
Prima fase: Logic Pro. Successivamente: Ableton Live, FL Studio e Pro Tools. In seguito: Cubase, REAPER e altre DAW compatibili.
Deve utilizzare esclusivamente interfacce, API, SDK, scripting e metodi tecnicamente verificati per ciascuna DAW.

**M. ADAPTIVE WORKFLOW AGENT** – apprendimento delle abitudini operative dell'utente.
Competenze: apprendimento delle scorciatoie da tastiera personali; riconoscimento dei comandi rapidi; memorizzazione delle sequenze operative ripetute; apprendimento dei workflow utilizzati nelle DAW; riconoscimento delle preferenze operative; recupero delle procedure già apprese quando pertinenti; aggiornamento delle procedure in seguito a modifiche dell'utente.
Deve distinguere tra comandi standard e personalizzazioni dell'utente.
Non deve registrare indiscriminatamente digitazioni, password o informazioni private.
La memoria deve essere consultabile, modificabile, cancellabile e disattivabile dall'utente.

### 4. REGOLE DI COMUNICAZIONE TRA GLI AGENTI
Ogni agente deve essere indipendente, ma predisposto a collaborare con gli altri. La comunicazione deve avvenire attraverso interfacce standardizzate.
Prevedi: Agent Registry; Agent API; Agent Protocol; Tool Registry; sistema di messaggistica ed eventi; identificativi univoci per task e progetto; gestione degli input e degli output; gestione degli errori; tracciamento delle attività; versionamento delle interfacce.

Ogni agente deve dichiarare: nome e identificativo; versione; competenze; funzioni disponibili; strumenti utilizzabili; requisiti e autorizzazioni; limiti tecnici; formati di input accettati; formati di output prodotti.

### 5. PROTOCOLLO DI RISPOSTA DEGLI AGENTI
Ogni agente deve restituire risultati strutturati, comprendenti almeno: ID della richiesta; ID del task; agente responsabile; operazione richiesta; stato dell'operazione; risultato; eventuali file prodotti; strumenti utilizzati; errori o limitazioni; eventuali richieste di approvazione.

Utilizza gli stati standard già previsti nell'architettura COSOUND: PENDING, RUNNING, WAITING_APPROVAL, COMPLETED, FAILED, CANCELLED.
Non dichiarare mai completata un'operazione che non è stata effettivamente eseguita.

### 6. INTEGRAZIONE CON IL CENTRAL AGENT DEL SOCIO
Questa è una parte fondamentale del lavoro. Il sistema Multi-Agent deve essere predisposto per collegarsi al Central Agent sviluppato dal socio.
Il socio deve poter: registrare gli agenti disponibili; consultare le loro competenze; assegnare task; inviare richieste e dati; ricevere risultati; verificare lo stato delle operazioni; gestire errori e richieste di approvazione.

Non inventare API, protocolli o framework che il socio non ha ancora comunicato.
Se le specifiche tecniche del Central Agent non sono ancora disponibili, realizza un Agent Integration Layer modulare e documentato.
Predisponi un connettore temporaneo MOCK, chiaramente identificato come simulazione.
Il MOCK deve poter essere sostituito con il Central Agent reale senza riscrivere gli agenti specializzati.
Quando saranno disponibili codice e documentazione del socio, analizza le interfacce effettive e adatta l'integrazione senza modificare arbitrariamente il suo codice.

### 7. INDIPENDENZA E MODULARITÀ
Ogni agente deve poter essere: sviluppato indipendentemente; testato singolarmente; aggiornato senza compromettere gli altri; sostituito con una versione più evoluta; attivato o disattivato; collegato a nuovi motori e strumenti.
Evita dipendenze circolari e duplicazioni di responsabilità.
Gli agenti non devono diventare un insieme di chatbot indipendenti: devono essere componenti specializzati di un unico sistema coordinato dal Central Agent.

### 8. SICUREZZA E AUTORIZZAZIONI
Prevedi: permessi granulari; accesso controllato ai file; accesso autorizzato alle DAW; protezione dei dati personali; gestione sicura delle credenziali; controllo delle operazioni distruttive; conferma dell'utente per le operazioni sensibili; log delle attività; possibilità di interrompere le operazioni.
Nessun agente deve avere accesso illimitato al computer.

### 9. TEST E VERIFICHE
Realizza test specifici per ogni agente. Verifica: corretta identificazione delle competenze; gestione degli input; qualità e struttura degli output; comunicazione con gli altri agenti; gestione degli errori; gestione delle autorizzazioni; rispetto dei limiti dichiarati; comunicazione con il connettore MOCK; possibilità di sostituire il MOCK con il connettore reale.

Prevedi anche test di collaborazione tra più agenti. Esempio: l'utente chiede di creare un brano Melodic Techno a 124 BPM. Il Central Agent assegna i task:
1. Music Producer Agent: pianifica la struttura.
2. Rhythm Engineer Agent: prepara il comparto ritmico.
3. Music Theory Agent: definisce tonalità e progressione armonica.
4. Smart Library Agent: individua sample e loop.
5. Sound Designer Agent: individua suoni e preset.
6. DAW Controller Agent: prepara le operazioni nella DAW.

Il Central Agent raccoglie i risultati, li valuta e coordina le fasi successive.
Questo esempio serve a verificare la collaborazione: non devi realizzare un secondo Central Agent che esegua autonomamente il coordinamento.

### 10. DOCUMENTAZIONE E CONSEGNA
Al termine del lavoro consegna:
1. Architettura completa del Multi-Agent System.
2. Codice degli agenti effettivamente sviluppati.
3. Documentazione delle responsabilità di ogni agente.
4. Agent API e Agent Protocol.
5. Agent Registry e Tool Registry.
6. Documentazione dell'Agent Integration Layer.
7. Connettore MOCK per i test.
8. Documentazione delle dipendenze.
9. Test individuali e di collaborazione.
10. Istruzioni per l'integrazione con il Central Agent del socio.

Indica chiaramente: cosa è stato realmente implementato; cosa è ancora simulato; cosa richiede componenti esterni; cosa necessita delle specifiche del Central Agent; quali sono i limiti tecnici ancora presenti.

### 11. REGOLE FINALI
Non sviluppare: un nuovo Central Agent; un secondo sistema di orchestrazione centrale; un duplicato dell'AI Core; una seconda memoria centrale indipendente; funzionalità già assegnate ad altri moduli senza una precisa necessità.
Non modificare arbitrariamente i componenti già sviluppati e approvati.
Non dichiarare operative funzionalità che sono soltanto simulate.
Mantieni la compatibilità con l'architettura generale di COSOUND e con i Prompt 01, 02, 03, 04 e 05.

**OBIETTIVO FINALE:** realizzare un ecosistema di agenti specializzati, autonomi nelle rispettive competenze ma integrabili e coordinabili dal Central Agent del socio. COSOUND dovrà presentarsi all'utente come un unico assistente intelligente, capace di utilizzare tutte queste competenze senza obbligare l'utente a interagire direttamente con i singoli agenti.
