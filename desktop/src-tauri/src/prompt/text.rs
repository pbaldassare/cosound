//! Text normalization for the Prompt Engine: lower case, apostrophes and punctuation split, accents kept (Italian
//! "più" and "piu" are both accepted by the lexicon), language detection, number words, and phrase matching on word
//! boundaries (so "bass" never matches inside "bassline" unless listed).

#[derive(Clone, Debug)]
pub struct Text {
    pub raw: String,
    /// " word word word " – padded so phrases can be matched with surrounding spaces
    pub norm: String,
    pub words: Vec<String>,
}

pub fn normalize(raw: &str) -> Text {
    let mut s = raw.to_lowercase().replace(['’', '\'', '`'], " ");
    for c in [',', '.', ';', ':', '!', '?', '(', ')', '"', '«', '»', '[', ']', '{', '}', '\n', '\t', '–', '—'] {
        s = s.replace(c, " ");
    }
    s = s.replace('%', " % ");
    // "124bpm" → "124 bpm", "-3db" → "-3 db", "+2db"
    let mut out = String::new();
    let chars: Vec<char> = s.chars().collect();
    for (i, c) in chars.iter().enumerate() {
        out.push(*c);
        if c.is_ascii_digit() && chars.get(i + 1).is_some_and(|n| n.is_alphabetic()) {
            out.push(' ');
        }
    }
    let words: Vec<String> = out.split_whitespace().map(String::from).collect();
    Text { raw: raw.to_string(), norm: format!(" {} ", words.join(" ")), words }
}

impl Text {
    /// The phrase appears as whole words.
    pub fn has(&self, phrase: &str) -> bool {
        self.norm.contains(&format!(" {phrase} "))
    }
    pub fn any(&self, phrases: &[&str]) -> bool {
        phrases.iter().any(|p| self.has(p))
    }
    /// First phrase found, if any.
    pub fn first<'a>(&self, phrases: &[&'a str]) -> Option<&'a str> {
        phrases.iter().copied().filter(|p| self.has(p)).min_by_key(|p| self.norm.find(&format!(" {p} ")).unwrap_or(usize::MAX))
    }
    /// Byte position of a phrase (for ordering), if present.
    pub fn pos(&self, phrase: &str) -> Option<usize> {
        self.norm.find(&format!(" {phrase} "))
    }
    /// Words after a phrase (up to n).
    pub fn after(&self, phrase: &str, n: usize) -> Vec<String> {
        match self.pos(phrase) {
            Some(p) => self.norm[p + phrase.len() + 1..].split_whitespace().take(n).map(String::from).collect(),
            None => vec![],
        }
    }
}

const IT: [&str; 34] = [
    "il", "lo", "la", "gli", "le", "un", "una", "di", "del", "della", "che", "non", "più", "piu", "rendi", "fammi", "voglio", "vorrei", "questo",
    "questa", "con", "per", "basso", "crea", "abbassa", "alza", "e", "mantieni", "solo", "soltanto", "sul", "nel", "è", "deve",
];
const EN: [&str; 32] = [
    "the", "a", "an", "of", "with", "make", "more", "less", "this", "that", "please", "and", "to", "it", "is", "keep", "only", "don", "dont",
    "create", "i", "want", "should", "lower", "raise", "add", "remove", "into", "my", "on", "in", "be",
];
/// Words that mark a language COSOUND does not support yet (first phase: Italian and English).
const OTHER: [(&str, &[&str]); 4] = [
    ("es", &["quiero", "una", "pista", "con", "hazme", "más", "graves", "canción", "por", "favor", "el", "los"]),
    ("fr", &["je", "veux", "une", "avec", "plus", "le", "les", "fais", "moi", "basse", "morceau", "s'il"]),
    ("de", &["ich", "will", "einen", "mit", "mehr", "der", "die", "das", "und", "mach", "bitte", "lauter"]),
    ("pt", &["eu", "quero", "uma", "com", "mais", "faça", "música", "baixo", "por", "favor", "não", "você"]),
];

/// "it", "en", or another language code when the text is clearly in a language not supported yet.
/// Music words shared by every language (techno, bpm, synth, drop) are neutral. Ties go to `fallback`.
pub fn language(t: &Text, fallback: &str) -> String {
    let count = |list: &[&str]| t.words.iter().filter(|w| list.contains(&w.as_str())).count();
    let (it, en) = (count(&IT), count(&EN));
    let other = OTHER.iter().map(|(code, ws)| (*code, count(ws))).max_by_key(|x| x.1).unwrap();
    if other.1 >= 3 && other.1 > it + 1 && other.1 > en + 1 {
        return other.0.to_string();
    }
    if it > en {
        "it".into()
    } else if en > it {
        "en".into()
    } else {
        fallback.to_string()
    }
}

/// Number words (IT/EN) and digits; "un paio" = 2, "qualche" ≈ 2–3 (returned as 2, marked approximate by callers).
pub fn number(w: &str) -> Option<f64> {
    let w = w.trim_start_matches('+');
    if let Ok(n) = w.replace(',', ".").parse::<f64>() {
        return Some(n);
    }
    Some(match w {
        "un" | "uno" | "una" | "one" | "a" | "an" => 1.0,
        "due" | "two" | "paio" | "couple" => 2.0,
        "tre" | "three" => 3.0,
        "quattro" | "four" => 4.0,
        "cinque" | "five" => 5.0,
        "sei" | "six" => 6.0,
        "sette" | "seven" => 7.0,
        "otto" | "eight" => 8.0,
        "dieci" | "ten" => 10.0,
        "dodici" | "twelve" => 12.0,
        "sedici" | "sixteen" => 16.0,
        "trentadue" | "thirty-two" => 32.0,
        "mezzo" | "half" => 0.5,
        _ => return None,
    })
}

/// Control characters other than newline/tab are removed; length is capped. Returns (clean, was_truncated).
pub fn sanitize(raw: &str, max_chars: usize) -> (String, bool) {
    // control characters and invisible bidi overrides (used to hide text) are removed
    let clean: String = raw.chars().filter(|c| (!c.is_control() || *c == '\n' || *c == '\t') && !('\u{202a}'..='\u{202e}').contains(c) && !('\u{2066}'..='\u{2069}').contains(c)).collect();
    let n = clean.chars().count();
    if n > max_chars {
        (clean.chars().take(max_chars).collect(), true)
    } else {
        (clean, false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalization_language_numbers() {
        let t = normalize("Rendi il basso più potente, a 124BPM!");
        assert!(t.has("basso") && t.has("124 bpm") && t.has("più"));
        assert!(!normalize("una bassline").has("bass"));
        assert_eq!(language(&t, "en"), "it");
        assert_eq!(language(&normalize("Make the bass more powerful"), "it"), "en");
        assert_eq!(language(&normalize("techno 124 bpm"), "it"), "it", "neutral words keep the fallback");
        assert_eq!(language(&normalize("Quiero una pista techno con más graves por favor"), "it"), "es");
        assert_eq!(language(&normalize("Je veux une basse plus profonde avec le kick"), "it"), "fr");
        assert_eq!(language(&normalize("Ich will einen Bass mit mehr Druck bitte"), "it"), "de");
        assert_eq!(language(&normalize("Eu quero uma música com mais baixo por favor"), "it"), "pt");
        assert_eq!(number("paio"), Some(2.0));
        assert_eq!(number("-3"), Some(-3.0));
        assert_eq!(number("2,5"), Some(2.5));
        let (s, cut) = sanitize("ciao\u{0007}\u{202e}mondo", 6);
        assert!(cut && !s.contains('\u{0007}') && !s.contains('\u{202e}'));
    }
}
