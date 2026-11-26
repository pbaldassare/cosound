//! Italian and English vocabulary of the Prompt Engine: elements of a production, sections, intent cues, conversation
//! acts, destructive operations. Internal ids are English; users may speak either language (and mix them, as producers
//! do: "il drop", "il kick", "più punchy"). Sources: the knowledge base (docs/knowledge: arrangement, mixing, synths)
//! and the words already used by the music, plug-in and library engines.

/// Element role id → (kind, words). Longer phrases first where it matters.
pub const ELEMENTS: [(&str, &str, &[&str]); 19] = [
    ("kick", "part", &["kick", "cassa", "bass drum", "kik", "grancassa"]),
    ("snare", "part", &["rullante", "snare", "clap", "rimshot"]),
    ("hihat", "part", &["hi-hat", "hihat", "hi hat", "charleston", "hats", "hat"]),
    ("percussion", "part", &["percussioni", "percussione", "percussion", "perc", "shaker"]),
    ("drums", "part", &["batteria", "drums", "drum", "beat", "parte ritmica"]),
    ("bass", "part", &["sub bass", "linea di basso", "bassline", "basso", "bassi", "bass", "sub"]),
    ("lead", "part", &["lead", "solista", "synth lead"]),
    ("pad", "part", &["pad", "pads", "tappeto"]),
    ("melody", "part", &["melodia", "melodie", "melody", "topline", "tema", "hook"]),
    ("chords", "part", &["accordi", "chords", "stab", "stabs"]),
    ("arp", "part", &["arpeggio", "arpeggi", "arp", "arpeggiatore", "arpeggiator"]),
    ("synth", "instrument", &["sintetizzatore", "synth", "synthesizer", "sintetizzatori", "synths"]),
    ("piano", "instrument", &["pianoforte", "piano", "keys", "tastiere"]),
    ("strings", "instrument", &["archi", "strings"]),
    ("vocal", "part", &["voce", "voci", "vocal", "vocals", "vox", "cantato"]),
    ("fx", "part", &["effetti sonori", "riser", "sweep", "fx", "impact"]),
    ("master", "mix", &["master", "bus master", "master bus"]),
    ("mix", "mix", &["mix", "missaggio", "mixdown"]),
    ("track", "project", &["brano", "pezzo", "traccia completa", "canzone", "song", "track"]),
];

/// Words that point at something already present without naming it ("this element", "rendilo").
pub const DEICTIC: [&str; 22] = [
    "questo elemento", "questa traccia", "questo suono", "questa parte", "questo", "questa", "quello", "quella", "this element", "this track",
    "this sound", "this part", "this one", "this", "that", "it", "lo stesso", "la stessa", "same", "elemento", "suono", "sound",
];
/// Italian clitic pronouns on verbs: rendilo, alzala, abbassalo, mantienilo, fallo …
pub const CLITIC_ENDINGS: [&str; 4] = ["lo", "la", "li", "le"];
pub const CLITIC_VERBS: [&str; 14] = ["rendi", "alza", "abbassa", "mantieni", "fa", "metti", "sposta", "allunga", "accorcia", "cambia", "trasforma", "lascia", "porta", "duplica"];

pub const SECTIONS: [(&str, &[&str]); 8] = [
    ("intro", &["intro", "introduzione", "inizio"]),
    ("verse", &["strofa", "strofe", "verse"]),
    ("chorus", &["ritornello", "chorus", "refrain"]),
    ("breakdown", &["breakdown", "break"]),
    ("buildup", &["build-up", "build up", "buildup", "salita", "crescendo"]),
    ("drop", &["drop", "drops"]),
    ("bridge", &["bridge", "ponte"]),
    ("outro", &["outro", "chiusura", "finale del brano"]),
];

/// Intent id → cue phrases. A request can match several.
pub const INTENTS: [(&str, &[&str]); 19] = [
    ("music_creation", &["crea", "creami", "fammi", "genera", "produci", "voglio un", "voglio una", "vorrei un", "vorrei una", "nuovo brano", "create", "make me", "generate", "produce", "i want a", "i would like a", "build me"]),
    ("composition", &["componi", "scrivi", "melodia", "progressione", "giro di accordi", "accordi", "compose", "write", "melody", "progression", "chords", "armonia", "harmony", "bassline", "arpeggio", "topline"]),
    ("arrangement", &["arrangiamento", "arrangia", "struttura", "sezione", "sezioni", "intro", "drop", "breakdown", "build-up", "build up", "buildup", "outro", "strofa", "ritornello", "bridge", "arrangement", "arrange", "structure", "section", "energia", "energico", "energica", "energetic", "energy"]),
    ("midi_editing", &["midi", "nota", "note", "quantizza", "quantize", "trasponi", "transpose", "ottava", "octave", "semitoni", "semitono", "semitones", "velocity", "sincopato", "sincopata", "syncopated", "umanizza", "humanize", "legato", "staccato"]),
    ("sound_design", &["suono", "suoni", "timbro", "sound", "timbre", "preset", "patch", "sound design", "più largo", "più larga", "atmosferico", "atmosferica", "aggressivo", "aggressiva", "potente", "caldo", "calda", "brillante", "scuro", "profondo", "profonda", "spaziale", "bello", "warm", "bright", "aggressive", "powerful", "wide", "wider", "deep", "spacey"]),
    ("sample_selection", &["sample", "samples", "campione", "campioni", "loop", "loops", "one shot", "one-shot", "i miei sample", "my samples"]),
    ("library_search", &["cerca", "trova", "trovami", "cercami", "libreria", "librerie", "find", "search", "look for", "library"]),
    ("plugin_configuration", &["plugin", "plug-in", "plugins", "vst", "vst3", "audio unit", "compressore", "compressor", "equalizzatore", "eq", "riverbero", "reverb", "delay", "limiter", "saturatore", "saturation", "catena di effetti", "effect chain", "inserisci un", "insert a"]),
    ("track_editing", &["traccia", "tracce", "tracks", "regione", "regioni", "region", "clip", "elimina", "cancella", "rimuovi", "delete", "remove", "rinomina", "rename", "duplica", "duplicate", "muta", "mute"]),
    ("mixing", &["volume", "livello", "level", "db", "decibel", "decibels", "pan", "panning", "mix", "missaggio", "sidechain", "side-chain", "bilanciamento", "balance", "più forte", "più piano", "louder", "quieter", "compressione", "compression", "equalizza"]),
    ("mastering", &["master", "mastering", "loudness", "lufs", "true peak", "limitatore", "masterizza"]),
    ("audio_analysis", &["analizza", "analisi", "analyze", "analyse", "analysis", "misura", "measure", "che tonalità", "quanti bpm", "what key", "what tempo"]),
    ("reference_analysis", &["reference", "riferimento", "traccia di riferimento", "come in", "nello stile di", "simile a", "ispirato a", "like in", "in the style of", "similar to", "inspired by"]),
    ("file_import", &["importa", "import", "carica il file", "carica questo file", "aggiungi il file", "load the file", "add this file", "trascina"]),
    ("daw_management", &["daw", "logic", "logic pro", "ableton", "ableton live", "reaper", "fl studio", "cubase", "pro tools", "play", "stop", "riproduci", "registra", "record", "esporta", "export", "bounce", "inserisci nella daw", "nella daw"]),
    ("project_management", &["progetto", "progetti", "project", "projects", "salva il progetto", "nuovo progetto", "apri il progetto", "save the project", "new project", "open the project"]),
    ("workflow_learning", &["ricordati", "ricorda", "ogni volta", "di solito", "da ora in poi", "d ora in poi", "impara", "sempre così", "remember", "every time", "from now on", "usually", "learn", "always do"]),
    ("information", &["cos è", "cosa è", "che cos è", "come si", "come faccio", "spiegami", "perché", "cosa significa", "quali sono", "what is", "what are", "how do", "how does", "explain", "why", "what does"]),
    ("preferences", &["preferisco", "preferenza", "preferenze", "la mia daw", "chiamati", "il tuo nome", "rispondi in", "parla in", "rispondimi", "lingua", "prefer", "preference", "preferences", "call yourself", "your name", "answer in", "speak in", "language"]),
];

/// Conversation acts that do not create something new.
pub const UNDO: [&str; 9] = ["annulla l ultima", "annulla", "torna indietro", "undo", "go back", "annulla la modifica", "togli l ultima modifica", "revert", "indietro"];
pub const RESTORE: [&str; 10] = ["ripristina", "rimetti com era", "torna alla versione", "torna all originale", "versione originale", "com era prima", "restore", "back to the original", "back to version", "rimettilo com era"];
pub const CONFIRM: [&str; 14] = ["va bene così", "va bene", "perfetto", "confermo", "conferma", "tieni questa", "tieni questo", "mi piace", "ok tienilo", "keep this version", "keep it", "confirm", "sounds good", "perfect"];
pub const CANCEL: [&str; 8] = ["lascia stare", "lascia perdere", "non farlo", "non fare niente", "never mind", "forget it", "don t do it", "cancel"];
pub const CORRECTION: [&str; 9] = ["no intendevo", "intendevo", "non quello", "non quella", "ho sbagliato", "volevo dire", "i meant", "not that", "i mean"];
pub const FOLLOW_UP: [&str; 14] = ["adesso", "ora", "invece", "anche", "ancora", "un po di più", "ancora di più", "now", "instead", "also", "too", "even more", "a bit more", "poi"];
pub const SMALLTALK: [&str; 10] = ["ciao", "grazie", "buongiorno", "buonasera", "hello", "hi", "thanks", "thank you", "come stai", "how are you"];

/// Destructive operation kind → phrases. Reversible only through COSOUND's own undo; never executed here.
pub const DESTRUCTIVE: [(&str, &[&str], bool); 7] = [
    ("delete_tracks", &["elimina la traccia", "elimina le tracce", "cancella la traccia", "cancella le tracce", "rimuovi la traccia", "rimuovi le tracce", "elimina tutte le tracce", "delete the track", "delete tracks", "remove the track", "remove tracks", "elimina il", "elimina la", "cancella il", "cancella la", "delete the"], false),
    ("delete_regions", &["cancella la regione", "cancella le regioni", "elimina la regione", "elimina le regioni", "elimina le note", "cancella le note", "delete the region", "delete regions", "delete the notes", "svuota"], false),
    ("overwrite_file", &["sovrascrivi", "sovrascrivere", "overwrite", "sopra il file", "sostituisci il file", "replace the file", "salva sopra"], false),
    ("replace_preset", &["sostituisci il preset", "cambia il preset", "rimpiazza il preset", "replace the preset", "swap the preset", "sovrascrivi il preset"], true),
    ("restructure", &["rifai la struttura", "cambia la struttura", "ristruttura", "rifai tutto", "riscrivi tutto il brano", "restructure", "redo the whole", "change the structure", "riorganizza tutto"], true),
    ("export_over", &["esporta sopra", "esporta sostituendo", "export over", "esporta e sovrascrivi"], false),
    ("clear_memory", &["cancella la memoria", "dimentica tutto", "forget everything", "clear the memory"], false),
];

/// Requests nobody in COSOUND can carry out: (phrases, what, suggestion).
pub const UNSUPPORTED: [(&[&str], &str, &str); 4] = [
    (&["pubblica su", "carica su spotify", "carica su youtube", "upload to spotify", "publish on", "distribuisci"], "publishing to streaming platforms", "export the file and publish it from the platform"),
    (&["compra", "acquista", "buy", "purchase"], "purchases", "buy plug-ins or samples from the vendor's site, then rescan"),
    (&["scarica da internet", "download from", "scarica il plugin", "download the plugin"], "downloading from the internet", "download it yourself, then add the folder to the Smart Library"),
    (&["manda una mail", "invia un messaggio", "send an email", "send a message"], "sending messages", "share the exported file yourself"),
];

/// Phrases that ask COSOUND to run code or shell commands: refused, a prompt never becomes executable code.
pub const CODE_EXECUTION: [&str; 10] = ["esegui questo script", "esegui il comando", "esegui questo codice", "run this script", "run the command", "execute this", "rm -rf", "sudo ", "powershell", "terminal command"];

/// Text inside external content (files, pages, lyrics, metadata) that tries to act as an instruction.
pub const INJECTION: [&str; 16] = [
    "ignore previous", "ignore all previous", "ignora le istruzioni", "ignora tutte le istruzioni", "system prompt", "you are now", "sei ora",
    "disregard", "new instructions", "nuove istruzioni", "delete all", "elimina tutto", "cancella tutto", "send the file", "invia il file", "execute",
];
