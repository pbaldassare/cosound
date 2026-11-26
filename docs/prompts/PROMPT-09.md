# COSOUND – PROMPT 09
## MUSIC INTELLIGENCE ENGINE – TEORIA MUSICALE, COMPOSIZIONE E INTELLIGENZA MIDI

> Testo del prompt ricevuto il 2026-11-26, conservato come riferimento. Rapporto: `docs/music-intelligence/PROMPT-09-REPORT.md`.

1. OBIETTIVO DEL MODULO
Sviluppa il Music Intelligence Engine di COSOUND.
Questo motore deve fornire al sistema una conoscenza approfondita della musica, permettendo di analizzare, comprendere, creare e modificare contenuti musicali coerenti.
Non deve essere un semplice generatore casuale di note MIDI.
Deve comprendere la struttura musicale, le relazioni tra le note, la tonalità, l'armonia, il ritmo, le progressioni e le caratteristiche compositive dei diversi generi.
L'obiettivo è permettere a COSOUND di comportarsi come un compositore, arrangiatore e musicista esperto, capace di collaborare con gli altri componenti del sistema.
Esempio:
L'utente chiede:
"Crea una melodia emozionale e ipnotica in tonalità di La minore, adatta a una produzione Melodic Techno a 124 BPM."
Il sistema deve poter:

* Comprendere la tonalità richiesta.
* Selezionare una scala coerente.
* Costruire una progressione armonica.
* Creare una melodia musicalmente sensata.
* Generare un pattern MIDI.
* Verificare la compatibilità tra melodia e armonia.
* Proporre variazioni.
* Rendere il risultato modificabile nella DAW.

2. ARCHITETTURA DEL MUSIC INTELLIGENCE ENGINE
Organizza il sistema in componenti modulari:

* Music Theory Engine.
* Scale & Key Engine.
* Chord Engine.
* Harmony Engine.
* Melody Engine.
* Bassline Engine.
* Arpeggio Engine.
* Rhythm & Groove Engine.
* Composition Engine.
* MIDI Generation Engine.
* MIDI Analysis Engine.
* Musical Structure Engine.
* Style Intelligence Engine.
* Musical Validation Engine.

Ogni componente deve avere responsabilità definite e interfacce documentate.
Non creare un nuovo Central Agent, un nuovo AI Core o un secondo sistema di orchestrazione.
Il Music Intelligence Agent del Prompt 06 utilizzerà questo motore per interpretare le richieste musicali e ottenere risultati.
3. TEORIA MUSICALE COMPLETA
Il sistema deve conoscere e utilizzare correttamente i principi della teoria musicale.
Prevedi:
Note musicali

* Do, Re, Mi, Fa, Sol, La, Si.
* Corrispondenza con C, D, E, F, G, A, B.
* Diesis e bemolli.
* Ottave.
* Enharmonia.
* Frequenze delle note.
* Sistema MIDI note number.

Intervalli

* Unisono.
* Seconda minore e maggiore.
* Terza minore e maggiore.
* Quarta giusta.
* Tritono.
* Quinta giusta.
* Sesta minore e maggiore.
* Settima minore e maggiore.
* Ottava.
* Intervalli composti.

Toni e semitoni

* Distanze tra le note.
* Costruzione degli intervalli.
* Relazioni tra note e scale.
* Riconoscimento delle distanze musicali.

Il motore deve conoscere anche le differenze tra notazione musicale, frequenza e rappresentazione MIDI.
4. SCALE MUSICALI
Implementa il riconoscimento e la costruzione delle principali scale musicali.
Prevedi:

* Maggiore.
* Minore naturale.
* Minore armonica.
* Minore melodica.
* Pentatonica maggiore.
* Pentatonica minore.
* Scala blues.
* Scala cromatica.
* Modi della scala maggiore.
* Modi della minore melodica.
* Scale simmetriche.
* Scale esatonali.
* Altre scale musicali documentate.

Per ogni scala il sistema deve conoscere:

* Formula intervallare.
* Note costitutive.
* Gradi.
* Tonalità.
* Relazioni armoniche.
* Caratteristiche musicali.

Deve poter generare una scala partendo da qualsiasi nota fondamentale.
Esempio:
"Genera la scala di Fa diesis minore."
Il sistema deve restituire le note corrette, con la corretta rappresentazione musicale e MIDI.
5. TONALITÀ E ARMATURA
Sviluppa un motore specifico per il riconoscimento e la gestione delle tonalità.
Deve conoscere:

* Tonalità maggiori.
* Tonalità minori.
* Armature di chiave.
* Circolo delle quinte.
* Tonalità relative.
* Tonalità vicine.
* Modulazioni.
* Relazioni tra tonalità.

Deve poter analizzare una sequenza di note e stimare la tonalità più probabile.
Quando il materiale è ambiguo, deve poter restituire più tonalità candidate con un livello di affidabilità.
Non deve confondere una singola nota prevalente con la tonalità dell'intera composizione.
6. ACCORDI E ARMONIA
Implementa un Chord Engine completo.
Deve conoscere:

* Accordi maggiori.
* Accordi minori.
* Diminuiti.
* Aumentati.
* Sus2 e Sus4.
* Accordi di settima.
* Maj7.
* Min7.
* Dominanti.
* Diminuiti di settima.
* Semidiminuiti.
* Accordi estesi.
* Nona.
* Undicesima.
* Tredicesima.
* Accordi con note aggiunte.
* Rivolti e voicing.

Deve poter costruire accordi partendo da:

* Una fondamentale.
* Una scala.
* Un grado armonico.
* Una sigla accordale.
* Una progressione richiesta.

Prevedi anche il riconoscimento degli accordi da una sequenza MIDI o da un contenuto audio, quando tecnicamente possibile.
7. PROGRESSIONI ARMONICHE
Sviluppa un sistema capace di costruire progressioni armoniche coerenti.
Deve conoscere:

* Gradi armonici.
* Funzioni tonali.
* Tonica.
* Sottodominante.
* Dominante.
* Cadenze.
* Risoluzioni.
* Progressioni maggiori e minori.
* Progressioni modali.
* Progressioni con accordi estesi.
* Sostituzioni armoniche.
* Modulazioni.

Deve poter proporre progressioni in base al genere e all'atmosfera richiesta.
Esempi:

* Melodic Techno.
* Progressive House.
* Techno.
* Deep House.
* House.
* EDM.
* Cinematic.
* Ambient.

Non utilizzare progressioni preconfezionate come unica soluzione: il sistema deve poter generare alternative coerenti.
8. MELODY ENGINE
Sviluppa un motore di composizione melodica.
Deve poter creare melodie basate su:

* Tonalità.
* Scala.
* Progressione armonica.
* Intervalli.
* Ritmo.
* Fraseggio.
* Ripetizione.
* Variazione.
* Tensione e risoluzione.
* Note di passaggio.
* Note di approccio.
* Anticipazioni.
* Sospensioni.

Deve conoscere:

* Motivi melodici.
* Frasi.
* Periodi musicali.
* Domanda e risposta.
* Sviluppo tematico.
* Variazione melodica.
* Ripetizione con evoluzione.

Le melodie devono avere una logica riconoscibile e un'identità musicale.
Prevedi la possibilità di creare:

* Melodie semplici.
* Melodie progressive.
* Melodie ipnotiche.
* Melodie emozionali.
* Melodie minimaliste.
* Melodie cinematografiche.
* Melodie ritmiche.

L'utente deve poter chiedere variazioni senza perdere necessariamente l'identità del motivo originale.
9. BASSLINE ENGINE
Sviluppa un motore specializzato nella composizione delle linee di basso.
Deve poter generare:

* Sub Bass.
* Bassline melodiche.
* Bassline ritmiche.
* Bassi sincopati.
* Bassi sequenziali.
* Bassi ipnotici.
* Bassi progressivi.
* Bassi in stile Techno e House.

Deve valutare:

* Tonalità.
* Fondamentali degli accordi.
* Note di passaggio.
* Intervalli.
* Ottave.
* Durata delle note.
* Pause.
* Accenti.
* Groove.
* Relazione con il kick.

Deve evitare conflitti armonici e ritmici con gli altri elementi del progetto.
10. ARPEGGIO ENGINE
Sviluppa un motore per la creazione di arpeggi.
Prevedi:

* Arpeggi ascendenti.
* Discendenti.
* Alternati.
* Spezzati.
* Pattern ritmici.
* Arpeggi polifonici.
* Sequenze con ottave.
* Variazioni di velocità.
* Variazioni ritmiche.
* Accenti e velocity.

Deve poter creare arpeggi in relazione agli accordi e alla tonalità del progetto.
11. RHYTHM & GROOVE ENGINE
Il sistema deve comprendere il rapporto tra ritmo e composizione musicale.
Prevedi:

* Suddivisioni binarie e ternarie.
* Quarti.
* Ottavi.
* Sedicesimi.
* Trentaduesimi.
* Terzine.
* Swing.
* Sincopi.
* Accenti.
* Pause.
* Poliritmie.
* Groove.
* Pattern ritmici.

Deve poter generare pattern MIDI coerenti con il BPM e con il genere richiesto.
Deve distinguere tra griglia metrica, posizione temporale e durata delle note.
12. STYLE INTELLIGENCE
Implementa un sistema di conoscenza degli stili musicali.
Deve poter utilizzare caratteristiche compositive tipiche di diversi generi, senza limitarsi a semplici preset.
Per ogni stile deve considerare:

* BPM indicativo.
* Groove.
* Struttura armonica.
* Densità melodica.
* Tipologia di bassline.
* Pattern ritmici.
* Struttura delle frasi.
* Caratteristiche dell'arrangiamento.
* Evoluzione energetica.

Prevedi profili musicali configurabili per:

* Techno.
* Melodic Techno.
* Progressive House.
* Deep House.
* House.
* Tech House.
* Minimal.
* EDM.
* Trance.
* Ambient.
* Cinematic.

I profili devono essere modificabili e aggiornabili.
Il genere musicale non deve diventare una gabbia: l'utente deve poter combinare caratteristiche di stili diversi.
13. MIDI GENERATION ENGINE
Questa è una funzione fondamentale.
Sviluppa un motore capace di generare file MIDI reali, modificabili e utilizzabili nelle DAW.
Prevedi:

* Creazione di note.
* Gestione di pitch.
* Durata.
* Velocity.
* Timing.
* Quantizzazione.
* Humanization.
* Canali MIDI.
* Tracce multiple.
* Program Change, quando appropriato.
* Controller MIDI.
* Automazioni e dati compatibili, dove supportati.

Il sistema deve poter generare:

* Melodie.
* Accordi.
* Bassline.
* Arpeggi.
* Pattern ritmici.
* Sequenze polifoniche.
* Composizioni multitraccia.

Prevedi esportazione in formato MIDI standard.
I file generati devono poter essere importati e modificati in Logic Pro e nelle altre DAW supportate.
14. MIDI EDITING E MODIFICA INTELLIGENTE
L'utente deve poter chiedere modifiche in linguaggio naturale.
Esempi:
"Abbassa questa melodia di un'ottava."
"Rendi il basso più sincopato."
"Allunga le note degli accordi."
"Rendi questo arpeggio più ipnotico."
"Trasforma questa progressione in tonalità minore."
Il sistema deve modificare i dati MIDI effettivi, mantenendo il più possibile le caratteristiche musicali richieste.
Prevedi:

* Trasposizione.
* Quantizzazione.
* Modifica della velocity.
* Modifica della durata.
* Spostamento ritmico.
* Humanization.
* Modifica delle ottave.
* Modifica delle note.
* Variazione delle frasi.
* Modifica delle progressioni.

Ogni modifica deve poter essere confrontata con la versione originale e annullata quando tecnicamente possibile.
15. ANALISI MUSICALE DEI FILE MIDI
Sviluppa un sistema capace di analizzare file MIDI esistenti.
Deve riconoscere:

* Note.
* Intervalli.
* Scale probabili.
* Tonalità.
* Accordi.
* Progressioni.
* Ritmo.
* BPM, quando disponibile.
* Frasi melodiche.
* Pattern.
* Ripetizioni.
* Variazioni.

Deve poter individuare eventuali note estranee alla tonalità senza classificarle automaticamente come errori: possono essere note cromatiche, tensioni o scelte compositive intenzionali.
16. VOICE-TO-MUSIC E MUSIC INTELLIGENCE
Il Music Intelligence Engine deve collaborare con il Voice-to-Music Agent e con il relativo motore tecnico.
Quando l'utente canta, fischia o vocalizza una melodia, il sistema deve poter ricevere le informazioni estratte e trasformarle in contenuti musicali.
Esempio:
L'utente canticchia una melodia.
COSOUND:

* Riceve le note stimate.
* Analizza gli intervalli.
* Ricostruisce il ritmo.
* Individua una tonalità possibile.
* Corregge eventuali imprecisioni solo se richiesto.
* Genera un MIDI.
* Propone accordi compatibili.

Non deve alterare arbitrariamente l'idea musicale originale dell'utente.
17. REFERENCE TRACK E ANALISI MUSICALE
Il motore deve poter collaborare con il Reference Track Agent.
Deve utilizzare le informazioni musicali disponibili per:

* Analizzare le progressioni armoniche.
* Comprendere la struttura melodica.
* Studiare le caratteristiche ritmiche.
* Individuare tonalità e scale.
* Comprendere le relazioni tra basso, accordi e melodia.

Quando viene fornito un file audio, deve poter lavorare con le informazioni effettivamente estratte dal sistema di analisi audio.
Quando si tratta di un brano non fornito, deve distinguere le informazioni reperite da fonti esterne dalle analisi audio reali.
Deve aiutare a creare composizioni originali ispirate a caratteristiche stilistiche generali, senza riprodurre melodie o sequenze protette.
18. MUSICAL VALIDATION ENGINE
Sviluppa un sistema di verifica della coerenza musicale.
Prima di restituire una composizione deve controllare:

* Compatibilità tra tonalità e note.
* Coerenza tra accordi e melodia.
* Relazione tra basso e armonia.
* Coerenza ritmica.
* Eventuali sovrapposizioni problematiche.
* Intervalli eccessivamente dissonanti rispetto alla richiesta.
* Continuità delle frasi.
* Coerenza della struttura.

Non deve eliminare automaticamente tensioni, dissonanze o note fuori scala se sono intenzionali.
Deve distinguere tra errore tecnico, scelta stilistica e libertà compositiva.
19. ADAPTIVE MUSICAL INTELLIGENCE
Il motore deve poter collaborare con la memoria musicale personale di COSOUND.
Quando autorizzato, deve poter utilizzare:

* Preferenze musicali dell'utente.
* Generi preferiti.
* BPM abituali.
* Tonalità utilizzate frequentemente.
* Tipologie di progressioni.
* Preferenze melodiche.
* Scelte compositive.
* Pattern ricorrenti.

Deve adattare le proposte senza trasformare le preferenze passate in vincoli obbligatori.
La memoria deve essere gestita attraverso le interfacce previste dall'architettura generale, senza creare un archivio personale duplicato.
20. INTEGRAZIONE CON GLI ALTRI MODULI
Il Music Intelligence Engine deve collaborare con:

* Central Agent del socio.
* Music Intelligence Agent del Prompt 06.
* Music Producer Agent.
* Rhythm Engineer Agent.
* Sound Designer Agent.
* Voice-to-Music Agent.
* Audio Analysis Agent.
* Reference Track Agent.
* Smart Library System del Prompt 07.
* Plugin Intelligence System del Prompt 08.
* DAW Controller Agent.

Deve esporre API documentate per ricevere richieste e restituire risultati strutturati.
Il Central Agent decide quali componenti coinvolgere.
Il Music Intelligence Engine esegue le elaborazioni musicali di propria competenza.
21. PRESTAZIONI E AFFIDABILITÀ
Il sistema deve essere progettato per lavorare in tempo reale o quasi reale quando tecnicamente possibile.
Prevedi:

* Elaborazione efficiente.
* Cache delle scale e delle strutture armoniche.
* Motore MIDI leggero.
* Elaborazione asincrona per operazioni complesse.
* Gestione degli errori.
* Validazione degli output.
* Riproducibilità dei risultati quando richiesta.

Se vengono utilizzati modelli AI, questi devono essere affiancati da verifiche musicali deterministiche, evitando che un modello linguistico produca dati musicali tecnicamente errati.
22. TEST E COLLAUDO
Prepara test specifici per:

1. Riconoscimento delle note.
2. Calcolo degli intervalli.
3. Costruzione delle scale.
4. Riconoscimento delle tonalità.
5. Costruzione degli accordi.
6. Generazione delle progressioni.
7. Composizione melodica.
8. Generazione delle bassline.
9. Generazione degli arpeggi.
10. Costruzione dei pattern ritmici.
11. Generazione MIDI.
12. Modifica MIDI.
13. Analisi MIDI.
14. Coerenza armonica.
15. Compatibilità tra melodia e basso.
16. Collaborazione con Voice-to-Music.
17. Collaborazione con Reference Track.
18. Integrazione con le DAW.
19. Gestione delle richieste in linguaggio naturale.

Verifica gli output con test musicali e controlli deterministici.
Non dichiarare completata una funzione senza averla testata.
23. CONSEGNA
Al termine consegna:

* Codice del Music Intelligence Engine.
* Architettura modulare.
* Music Theory Engine.
* Scale & Key Engine.
* Chord Engine.
* Harmony Engine.
* Melody Engine.
* Bassline Engine.
* Arpeggio Engine.
* Rhythm & Groove Engine.
* Composition Engine.
* MIDI Generation Engine.
* MIDI Analysis Engine.
* Musical Validation Engine.
* API di integrazione.
* Documentazione tecnica.
* Test e risultati.
* Limitazioni conosciute.

Indica chiaramente le funzioni operative, quelle simulate e quelle ancora da completare.
24. OBIETTIVO FINALE
COSOUND deve possedere una vera intelligenza musicale.
Deve essere capace di comprendere la musica, comporre melodie, costruire armonie, generare bassline, creare arpeggi, analizzare MIDI e trasformare idee musicali in composizioni concrete.
L'utente deve poter descrivere ciò che desidera ottenere e ricevere contenuti musicalmente coerenti, modificabili e utilizzabili direttamente nella propria DAW.
Il risultato deve essere un motore musicale professionale, modulare e integrabile con tutti gli altri componenti di COSOUND.
Non sviluppare un secondo Central Agent e non duplicare le responsabilità degli agenti specializzati.