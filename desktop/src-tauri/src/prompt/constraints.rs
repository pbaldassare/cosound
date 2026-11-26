//! Constraint Management: conditions the user sets ("non modificare la melodia", "mantieni il BPM originale",
//! "usa soltanto i miei sample", "senza basso", "non eliminare nessuna traccia", "non modificare il file originale").
//! Each constraint becomes data that travels with every instruction; the words it covers are masked so that, for
//! example, "melodia" in "non modificare la melodia" is not read as a request to write a melody.
use super::lexicon::{ELEMENTS, SECTIONS};
use super::text::Text;
use serde::Serialize;

#[derive(Clone, Debug, Serialize, PartialEq, serde::Deserialize)]
pub struct Constraint {
    pub kind: String,
    pub subject: String,
    pub text: String,
    pub scope: String,
    pub source: String,
}

/// Subjects that are not elements of the production.
const SUBJECTS: [(&str, &[&str]); 13] = [
    ("tempo", &["bpm originale", "bpm", "tempo originale", "tempo", "original bpm", "original tempo"]),
    ("key", &["tonalità", "tonalita", "chiave", "key"]),
    ("structure", &["struttura del brano", "struttura", "arrangiamento", "structure", "arrangement", "song structure"]),
    ("preset", &["preset", "patch"]),
    ("sound", &["questo suono", "il suono", "suono", "timbro", "this sound", "the sound", "sound", "timbre"]),
    ("original_file", &["file originale", "file originali", "original file", "original files", "originale", "original"]),
    ("tracks", &["nessuna traccia", "tracce", "traccia", "any track", "any tracks", "tracks", "track"]),
    ("own_samples", &["miei sample", "miei campioni", "miei suoni", "my samples", "my own samples", "my sounds"]),
    ("installed_plugins", &["plugin installati", "miei plugin", "my plugins", "installed plugins", "plugin di sistema", "stock plugins"]),
    ("notes", &["note", "notes", "midi"]),
    ("volume", &["volume", "livello", "level"]),
    ("harmony", &["armonia", "accordi", "progressione", "harmony", "progression"]),
    ("rhythm", &["ritmo", "groove", "rhythm"]),
];

const SKIP: [&str; 27] = ["mai", "never", "più", "il", "lo", "la", "l", "i", "gli", "le", "the", "my", "miei", "mie", "mio", "mia", "nessuna", "nessun", "any", "a", "un", "una", "del", "della", "questo", "questa", "this"];

/// (kind, trigger phrases). Longer triggers are tried first, so "non modificare il file originale" wins over
/// "non modificare".
const TRIGGERS: [(&str, &[&str]); 6] = [
    ("no_overwrite", &["non modificare il file originale", "non toccare il file originale", "non sovrascrivere", "don t overwrite", "do not overwrite", "don t modify the original", "lavora su una copia", "work on a copy"]),
    ("no_delete", &["non eliminare", "non cancellare", "non rimuovere", "don t delete", "do not delete", "don t remove", "do not remove"]),
    ("preserve", &["non modificare", "non cambiare", "non toccare", "non alterare", "lascia invariato", "lascia invariata", "lascia invariati", "senza cambiare", "senza modificare", "senza toccare", "mantieni", "mantenendo", "conserva", "preserva", "don t change", "don t touch", "don t modify", "do not change", "do not touch", "do not modify", "without changing", "without touching", "keep", "preserve", "leave"]),
    ("resource_restriction", &["usa soltanto", "usa solo", "usa esclusivamente", "utilizza soltanto", "utilizza solo", "use only", "only use"]),
    ("only", &["modifica esclusivamente", "modifica soltanto", "modifica solo", "cambia soltanto", "cambia solo", "lavora solo su", "esclusivamente", "soltanto", "unicamente", "solo", "only", "just"]),
    ("exclude", &["senza", "tranne", "eccetto", "without", "except", "but not"]),
];

fn subject_at(words: &[String], start: usize) -> Option<(String, usize)> {
    // own samples first, before articles are skipped ("i miei sample" must not become the element "sample")
    for j in start..(start + 3).min(words.len()) {
        if let Some(n) = SUBJECTS[7].1.iter().filter_map(|ph| {
            let pw: Vec<&str> = ph.split(' ').collect();
            (j + pw.len() <= words.len() && words[j..j + pw.len()].iter().zip(&pw).all(|(a, b)| a == b)).then_some(pw.len())
        }).max() {
            return Some(("own_samples".into(), j + n));
        }
    }
    let mut i = start;
    let mut skipped = 0;
    while i < words.len() && SKIP.contains(&words[i].as_str()) && skipped < 3 {
        i += 1;
        skipped += 1;
    }
    let try_list = |list: &[&str], i: usize| -> Option<usize> {
        list.iter().filter_map(|ph| {
            let pw: Vec<&str> = ph.split(' ').collect();
            (i + pw.len() <= words.len() && words[i..i + pw.len()].iter().zip(&pw).all(|(a, b)| a == b)).then_some(pw.len())
        }).max()
    };
    // elements, sections, then the other subjects
    for (id, _, ws) in ELEMENTS {
        if let Some(n) = try_list(ws, i) {
            return Some((id.to_string(), i + n));
        }
    }
    for (id, ws) in SECTIONS {
        if let Some(n) = try_list(ws, i) {
            return Some((id.to_string(), i + n));
        }
    }
    for (id, ws) in SUBJECTS {
        if let Some(n) = try_list(ws, i) {
            return Some((id.to_string(), i + n));
        }
    }
    None
}

/// Constraints and the word positions they cover.
pub fn parse(t: &Text) -> (Vec<Constraint>, Vec<bool>) {
    let w = &t.words;
    let mut mask = vec![false; w.len()];
    let mut out: Vec<Constraint> = vec![];
    let scope = if t.any(&["sempre", "always", "per sempre", "in ogni progetto", "in every project"]) {
        "permanent"
    } else if t.any(&["da ora in poi", "d ora in poi", "from now on", "per tutta la sessione", "per il resto della sessione", "for the rest of the session"]) {
        "session"
    } else {
        "request"
    };
    let mut i = 0;
    while i < w.len() {
        if mask[i] {
            i += 1;
            continue;
        }
        // longest trigger at i
        let mut best: Option<(&str, usize)> = None;
        for (kind, phrases) in TRIGGERS {
            for ph in phrases.iter() {
                let pw: Vec<&str> = ph.split(' ').collect();
                if i + pw.len() <= w.len() && w[i..i + pw.len()].iter().zip(&pw).all(|(a, b)| a == b) && best.is_none_or(|b| pw.len() > b.1) {
                    best = Some((kind, pw.len()));
                }
            }
        }
        let Some((kind, n)) = best else {
            i += 1;
            continue;
        };
        let after = i + n;
        let found = match kind {
            "no_overwrite" => Some(("original_file".to_string(), after)),
            _ => subject_at(w, after),
        };
        let Some((subject, end)) = found else {
            i += 1;
            continue;
        };
        let kind = match (kind, subject.as_str()) {
            ("only" | "resource_restriction", "own_samples" | "installed_plugins") => "resource_restriction",
            ("resource_restriction", _) => "only",
            ("preserve", "original_file") => "no_overwrite",
            ("no_delete", _) => "no_delete",
            (k, _) => k,
        };
        // "senza" + a verb other than change/touch is not an exclusion ("senza esagerare")
        let text = w[i..end].join(" ");
        for m in mask.iter_mut().take(end).skip(i) {
            *m = true;
        }
        if !out.iter().any(|c| c.kind == kind && c.subject == subject) {
            out.push(Constraint { kind: kind.into(), subject, text, scope: scope.into(), source: "user".into() });
        }
        i = end;
    }
    (out, mask)
}

/// The text with masked words replaced, for intent and target detection.
pub fn masked(t: &Text, mask: &[bool]) -> Text {
    let words: Vec<String> = t.words.iter().zip(mask).map(|(w, m)| if *m { "·".to_string() } else { w.clone() }).collect();
    Text { raw: t.raw.clone(), norm: format!(" {} ", words.join(" ")), words }
}

/// A constraint forbids touching `element` (preserve/exclude it, or only something else may change).
pub fn forbids(cs: &[Constraint], element: &str) -> bool {
    cs.iter().any(|c| (c.kind == "preserve" || c.kind == "exclude") && c.subject == element)
        || (cs.iter().any(|c| c.kind == "only") && !cs.iter().any(|c| c.kind == "only" && c.subject == element))
}

#[cfg(test)]
mod tests {
    use super::super::text::normalize;
    use super::*;

    fn kinds(s: &str) -> Vec<(String, String)> {
        parse(&normalize(s)).0.into_iter().map(|c| (c.kind, c.subject)).collect()
    }

    #[test]
    fn the_prompt_examples() {
        let k = |a: &str, b: &str| (a.to_string(), b.to_string());
        assert_eq!(kinds("Non modificare la melodia."), [k("preserve", "melody")]);
        assert_eq!(kinds("Mantieni il BPM originale."), [k("preserve", "tempo")]);
        assert_eq!(kinds("Utilizza soltanto i miei sample."), [k("resource_restriction", "own_samples")]);
        assert_eq!(kinds("Non cambiare il preset."), [k("preserve", "preset")]);
        assert_eq!(kinds("Modifica esclusivamente il drop."), [k("only", "drop")]);
        assert_eq!(kinds("Mantieni la struttura del brano."), [k("preserve", "structure")]);
        assert_eq!(kinds("Non eliminare nessuna traccia."), [k("no_delete", "tracks")]);
        assert_eq!(kinds("Non modificare il file originale."), [k("no_overwrite", "original_file")]);
        assert_eq!(kinds("Non modificare la melodia, cambia soltanto il suono."), [k("preserve", "melody"), k("only", "sound")]);
        assert_eq!(kinds("Un brano techno senza basso"), [k("exclude", "bass")]);
        assert_eq!(kinds("Keep the original tempo and don't touch the drums"), [k("preserve", "tempo"), k("preserve", "drums")]);
        assert!(kinds("Rendi il basso più potente").is_empty());
    }

    #[test]
    fn masking_and_scope() {
        let t = normalize("Non modificare la melodia, cambia soltanto il suono del basso");
        let (cs, mask) = parse(&t);
        let m = masked(&t, &mask);
        assert!(!m.has("melodia") && m.has("basso"), "{}", m.norm);
        assert!(forbids(&cs, "melody"));
        let (cs, _) = parse(&normalize("da ora in poi non toccare la batteria"));
        assert_eq!(cs[0].scope, "session");
        let (cs, _) = parse(&normalize("non cambiare mai la tonalità, sempre"));
        assert_eq!((cs[0].subject.as_str(), cs[0].scope.as_str()), ("key", "permanent"));
    }
}
