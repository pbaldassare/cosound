//! Intent Recognition and conversation acts. Intents come from the meaning of the whole request – cue words, the
//! parameters found, the elements named, comparatives ("più largo" changes something that exists, it does not create
//! it) – not from single keywords. The text given here has the user's constraints masked out.
use super::extract::Extracted;
use super::lexicon::{CANCEL, CONFIRM, CORRECTION, FOLLOW_UP, INTENTS, RESTORE, SMALLTALK, UNDO};
use super::text::Text;
use serde::Serialize;

#[derive(Clone, Debug, Serialize, PartialEq)]
pub struct Intent {
    pub id: &'static str,
    pub confidence: f64,
    pub evidence: Vec<String>,
}

/// Priority used to pick the primary intent when confidences tie (what the user most likely wants done).
const PRIORITY: [&str; 19] = [
    "midi_editing", "sound_design", "mixing", "mastering", "arrangement", "composition", "music_creation", "track_editing", "plugin_configuration",
    "sample_selection", "library_search", "reference_analysis", "audio_analysis", "file_import", "daw_management", "project_management",
    "workflow_learning", "preferences", "information",
];

const CREATE_VERBS: [&str; 17] = ["crea", "creami", "fammi", "genera", "produci", "componi", "scrivi", "aggiungi", "voglio un", "voglio una", "vorrei un", "vorrei una", "create", "make me", "generate", "write", "add a"];
const COMPARATIVE: [&str; 12] = ["più", "piu", "meno", "more", "less", "wider", "brighter", "darker", "warmer", "louder", "quieter", "deeper"];
const MUSICAL_PARTS: [&str; 10] = ["bass", "melody", "chords", "arp", "drums", "lead", "pad", "kick", "percussion", "hihat"];

pub fn comparative(t: &Text) -> bool {
    t.any(&COMPARATIVE) || t.words.iter().any(|w| w.ends_with("er") && ["wid", "bright", "dark", "warm", "loud", "quiet", "deep", "punch", "fat"].iter().any(|s| w.starts_with(s)))
}

pub fn creating(t: &Text) -> bool {
    t.any(&CREATE_VERBS)
}

pub fn classify(t: &Text, x: &Extracted, elements: &[(String, &'static str)], midi_ops: usize) -> Vec<Intent> {
    let mut found: Vec<Intent> = vec![];
    let add = |id: &'static str, conf: f64, ev: &str, found: &mut Vec<Intent>| {
        if let Some(i) = found.iter_mut().find(|i| i.id == id) {
            i.confidence = (i.confidence + 0.1).min(0.97);
            if !i.evidence.iter().any(|e| e == ev) {
                i.evidence.push(ev.to_string());
            }
        } else {
            found.push(Intent { id, confidence: conf, evidence: vec![ev.to_string()] });
        }
    };
    for (id, cues) in INTENTS {
        for c in cues.iter().filter(|c| t.has(c)) {
            add(id, 0.6, c, &mut found);
        }
    }
    let parts: Vec<&str> = elements.iter().map(|e| e.0.as_str()).filter(|e| MUSICAL_PARTS.contains(e)).collect();
    let cmp = comparative(t);
    let create = creating(t);
    let has = |n: &str| x.params.iter().any(|p| p.name == n);
    // creating musical parts is composition; with sound words it is also sound design
    if create && !parts.is_empty() && !cmp {
        add("music_creation", 0.75, "create + part", &mut found);
        add("composition", 0.75, parts[0], &mut found);
    }
    // MIDI edits change notes that exist: in a creation request "ipnotica" describes the new part, it is not an edit
    if midi_ops > 0 && !(create && !parts.is_empty() && !cmp) {
        add("midi_editing", 0.85, "MIDI edit", &mut found);
    }
    let sound_q = x.qualities.iter().any(|q| !["louder", "quieter", "energetic"].contains(&q.0.as_str()));
    if sound_q && (!elements.is_empty() || t.any(&["suono", "sound", "timbro", "synth"])) {
        add("sound_design", 0.8, "sound qualities", &mut found);
    }
    if has("gain_change") || x.qualities.iter().any(|q| q.0 == "louder" || q.0 == "quieter") {
        add("mixing", 0.9, "level change", &mut found);
    }
    if !x.sections.is_empty() {
        add("arrangement", 0.7, "section", &mut found);
    }
    if has("genre") && (create || has("bpm")) {
        add("music_creation", 0.7, "genre", &mut found);
    }
    // "più largo", "più potente" change something that exists: not a creation, even after "vorrei un"
    if cmp && !elements.is_empty() && !t.any(&["nuovo", "nuova", "new", "aggiungi", "add"]) {
        found.retain(|i| i.id != "music_creation");
    }
    // "rendilo più aggressivo": sound words on an implicit element
    if found.is_empty() && cmp && sound_q {
        add("sound_design", 0.7, "sound qualities", &mut found);
    }
    // "cerca un kick secco": the qualities describe what to find, nothing is being modified
    if t.any(&["cerca", "cercami", "trova", "trovami", "find", "search", "look for"]) && !cmp {
        found.retain(|i| i.id != "sound_design");
        if let Some(i) = found.iter_mut().find(|i| i.id == "library_search") {
            i.confidence = (i.confidence + 0.2).min(0.97);
        } else {
            add("library_search", 0.8, "search", &mut found);
        }
    }
    // a question with no action is information
    let question = t.raw.trim_end().ends_with('?') || t.any(&["cos è", "cosa è", "che cos è", "what is", "how do", "spiegami", "explain"]);
    if question && !create && midi_ops == 0 && !has("gain_change") {
        found.retain(|i| ["information", "preferences", "workflow_learning"].contains(&i.id));
        add("information", 0.8, "question", &mut found);
    }
    found.sort_by(|a, b| b.confidence.total_cmp(&a.confidence).then(PRIORITY.iter().position(|p| *p == a.id).cmp(&PRIORITY.iter().position(|p| *p == b.id))));
    found
}

pub fn act(t: &Text, intents: &[Intent], has_previous: bool, refers_implicitly: bool) -> &'static str {
    let only = |list: &[&str]| t.any(list) && intents.iter().all(|i| ["information", "preferences"].contains(&i.id)) || t.any(list) && t.words.len() <= 4;
    if t.any(&RESTORE) {
        return "restore";
    }
    if t.any(&UNDO) && (intents.is_empty() || t.words.len() <= 6) {
        return "undo";
    }
    if only(&CANCEL) {
        return "cancel";
    }
    if t.any(&CORRECTION) && has_previous {
        return "correction";
    }
    if intents.is_empty() && t.any(&CONFIRM) && has_previous {
        return "confirm";
    }
    if intents.is_empty() && t.any(&SMALLTALK) {
        return "smalltalk";
    }
    if !intents.is_empty() && intents.iter().all(|i| i.id == "information") {
        return "question";
    }
    if has_previous && (refers_implicitly || t.any(&FOLLOW_UP)) && !creating(t) {
        return "modification";
    }
    "new_request"
}

#[cfg(test)]
mod tests {
    use super::super::context::elements;
    use super::super::extract::extract;
    use super::super::text::normalize;
    use super::*;

    fn ids(s: &str) -> Vec<&'static str> {
        let t = normalize(s);
        let ops = crate::music::edit::parse(s).0.len();
        classify(&t, &extract(&t), &elements(&t), ops).iter().map(|i| i.id).collect()
    }

    #[test]
    fn intents_of_real_requests() {
        assert_eq!(ids("Rendi il basso più potente.")[0], "sound_design");
        assert!(ids("Il drop deve essere più energico.").contains(&"arrangement"));
        assert_eq!(ids("Abbassa il kick di un paio di decibel.")[0], "mixing");
        assert_eq!(ids("Vorrei un pad più largo e atmosferico.")[0], "sound_design");
        assert!(!ids("Vorrei un pad più largo e atmosferico.").contains(&"music_creation"), "a comparative changes, it does not create");
        let v = ids("Voglio un brano Melodic Techno a 124 BPM, con un basso profondo, un'atmosfera cinematografica e una melodia emozionale.");
        assert!(v.contains(&"music_creation") && v.contains(&"composition") && v.contains(&"sound_design"), "{v:?}");
        assert_eq!(ids("Abbassa questa melodia di un'ottava.")[0], "midi_editing");
        assert_eq!(ids("Cerca nella mia libreria un kick secco")[0], "library_search", "the quality describes what to find");
        let v = ids("Cerca nella mia libreria un loop di percussioni a 124 BPM");
        assert!(v.contains(&"sample_selection") && v.contains(&"library_search"), "{v:?}");
        assert_eq!(ids("Che cos'è una scala dorica?"), ["information"]);
        assert!(ids("Porta il master a -14 LUFS").contains(&"mastering"));
        assert!(ids("Importa il file e analizza la tonalità").contains(&"file_import"));
        assert!(ids("Ricordati che uso sempre Logic Pro").contains(&"workflow_learning"));
        assert!(ids("Rispondimi in inglese").contains(&"preferences"));
        assert!(ids("Crea una bassline melodica in La minore a 124 BPM.").contains(&"composition"));
        let v = ids("Crea una melodia emozionale e ipnotica in La minore, Melodic Techno a 124 BPM");
        assert!(!v.contains(&"midi_editing") && v[0] != "midi_editing", "{v:?}");
    }

    #[test]
    fn conversation_acts() {
        let a = |s: &str, prev: bool, implicit: bool| {
            let t = normalize(s);
            let i = classify(&t, &extract(&t), &elements(&t), crate::music::edit::parse(s).0.len());
            act(&t, &i, prev, implicit)
        };
        assert_eq!(a("Crea un basso profondo.", false, false), "new_request");
        assert_eq!(a("Rendilo più aggressivo.", true, true), "modification");
        assert_eq!(a("Adesso abbassa leggermente il volume.", true, true), "modification");
        assert_eq!(a("Annulla", true, false), "undo");
        assert_eq!(a("Torna alla versione originale", true, false), "restore");
        assert_eq!(a("Perfetto, tieni questa", true, false), "confirm");
        assert_eq!(a("No, intendevo il pad", true, false), "correction");
        assert_eq!(a("Lascia stare", true, false), "cancel");
        assert_eq!(a("Ciao!", false, false), "smalltalk");
    }
}
