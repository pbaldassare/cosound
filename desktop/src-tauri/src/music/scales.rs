//! Scale & Key Engine – the scale catalog. Every scale: interval formula, spelling rule, family, character and
//! source (docs/knowledge/music-theory/scales.md). Scales are built from any tonic with correct letter spelling for
//! the 7-note scales (each letter once) and plain sharps/flats for the others. Tables are static (no recomputation).
use super::notes::{italian, parse_note};
use super::theory::{interval_name, Note};
use serde_json::{json, Value};

pub struct ScaleDef {
    pub id: &'static str,
    pub en: &'static str,
    pub it: &'static str,
    pub intervals: &'static [u8],
    /// Letter steps above the tonic for each degree (None = plain sharps/flats by context).
    pub letters: Option<&'static [u8]>,
    pub family: &'static str,
    pub character: &'static str,
    pub carattere: &'static str,
    pub contexts: &'static str,
    pub source: &'static str,
}

const SEVEN: Option<&[u8]> = Some(&[0, 1, 2, 3, 4, 5, 6]);
const SRC: &str = "docs/knowledge/music-theory/scales.md";

macro_rules! s {
    ($id:expr, $en:expr, $it:expr, $iv:expr, $l:expr, $fam:expr, $ch:expr, $ca:expr, $ctx:expr) => {
        ScaleDef { id: $id, en: $en, it: $it, intervals: $iv, letters: $l, family: $fam, character: $ch, carattere: $ca, contexts: $ctx, source: SRC }
    };
}

pub const SCALES: [ScaleDef; 33] = [
    s!("major", "Major (Ionian)", "Maggiore (ionica)", &[0, 2, 4, 5, 7, 9, 11], SEVEN, "major_modes", "happy, open", "felice, aperta", "pop-EDM, progressive house, future bass"),
    s!("natural_minor", "Natural minor (Aeolian)", "Minore naturale (eolia)", &[0, 2, 3, 5, 7, 8, 10], SEVEN, "major_modes", "dark, serious, emotional", "scura, seria, emozionale", "techno, trance, deep/melodic house"),
    s!("harmonic_minor", "Harmonic minor", "Minore armonica", &[0, 2, 3, 5, 7, 8, 11], SEVEN, "harmonic_minor_modes", "dramatic, epic, strong V→i", "drammatica, epica, forte V→i", "trance leads, hardstyle, cinematic"),
    s!("melodic_minor", "Melodic minor (ascending)", "Minore melodica (ascendente)", &[0, 2, 3, 5, 7, 9, 11], SEVEN, "melodic_minor_modes", "sophisticated minor, jazzy", "minore raffinata, jazz", "neo-soul, jazzy house"),
    s!("dorian", "Dorian", "Dorica", &[0, 2, 3, 5, 7, 9, 10], SEVEN, "major_modes", "minor but warmer (♮6)", "minore ma più calda (♮6)", "deep house, nu-disco, lo-fi"),
    s!("phrygian", "Phrygian", "Frigia", &[0, 1, 3, 5, 7, 8, 10], SEVEN, "major_modes", "very dark, tense (♭2)", "molto scura, tesa (♭2)", "dark/industrial techno, psytrance basslines"),
    s!("lydian", "Lydian", "Lidia", &[0, 2, 4, 6, 7, 9, 11], SEVEN, "major_modes", "dreamy, floating (♯4)", "sognante, sospesa (♯4)", "ambient, cinematic, pads"),
    s!("mixolydian", "Mixolydian", "Misolidia", &[0, 2, 4, 5, 7, 9, 10], SEVEN, "major_modes", "bright but laid-back (♭7)", "luminosa ma rilassata (♭7)", "funky house, disco"),
    s!("locrian", "Locrian", "Locria", &[0, 1, 3, 5, 6, 8, 10], SEVEN, "major_modes", "unstable, rarely a home key", "instabile, raramente tonalità d'impianto", "tension passages"),
    s!("dorian_b2", "Dorian ♭2", "Dorica ♭2", &[0, 1, 3, 5, 7, 9, 10], SEVEN, "melodic_minor_modes", "dark with a raised 6th", "scura con sesta alzata", "jazz, modal"),
    s!("lydian_augmented", "Lydian augmented", "Lidia aumentata", &[0, 2, 4, 6, 8, 9, 11], SEVEN, "melodic_minor_modes", "bright, unstable (♯5)", "luminosa, instabile (♯5)", "jazz, cinematic"),
    s!("lydian_dominant", "Lydian dominant (acoustic)", "Lidia dominante (acustica)", &[0, 2, 4, 6, 7, 9, 10], SEVEN, "melodic_minor_modes", "bright with ♯4 and ♭7", "luminosa con ♯4 e ♭7", "jazz, fusion"),
    s!("mixolydian_b6", "Mixolydian ♭6", "Misolidia ♭6", &[0, 2, 4, 5, 7, 8, 10], SEVEN, "melodic_minor_modes", "major turning melancholic", "maggiore che vira al malinconico", "cinematic, film"),
    s!("locrian_nat2", "Locrian ♮2 (half-diminished)", "Locria ♮2 (semidiminuita)", &[0, 2, 3, 5, 6, 8, 10], SEVEN, "melodic_minor_modes", "tense, for m7♭5", "tesa, per m7♭5", "jazz"),
    s!("altered", "Altered (super-Locrian)", "Alterata (superlocria)", &[0, 1, 3, 4, 6, 8, 10], SEVEN, "melodic_minor_modes", "maximum tension on a dominant", "massima tensione su una dominante", "jazz dominants"),
    s!("locrian_nat6", "Locrian ♮6", "Locria ♮6", &[0, 1, 3, 5, 6, 9, 10], SEVEN, "harmonic_minor_modes", "dark, unstable", "scura, instabile", "modal"),
    s!("ionian_sharp5", "Ionian ♯5", "Ionica ♯5", &[0, 2, 4, 5, 8, 9, 11], SEVEN, "harmonic_minor_modes", "bright, augmented colour", "luminosa, colore aumentato", "modal"),
    s!("dorian_sharp4", "Dorian ♯4 (Ukrainian Dorian)", "Dorica ♯4 (dorica ucraina)", &[0, 2, 3, 6, 7, 9, 10], SEVEN, "harmonic_minor_modes", "folk, exotic minor", "minore esotica, popolare", "folk, world"),
    s!("phrygian_dominant", "Phrygian dominant", "Frigia dominante", &[0, 1, 4, 5, 7, 8, 10], SEVEN, "harmonic_minor_modes", "exotic, Middle-Eastern/flamenco", "esotica, mediorientale/flamenco", "psytrance, Middle-Eastern house"),
    s!("lydian_sharp2", "Lydian ♯2", "Lidia ♯2", &[0, 3, 4, 6, 7, 9, 11], SEVEN, "harmonic_minor_modes", "bright, exotic", "luminosa, esotica", "modal"),
    s!("ultralocrian", "Ultralocrian (altered ♭♭7)", "Ultralocria", &[0, 1, 3, 4, 6, 8, 9], SEVEN, "harmonic_minor_modes", "very tense", "molto tesa", "modal"),
    s!("major_pentatonic", "Major pentatonic", "Pentatonica maggiore", &[0, 2, 4, 7, 9], Some(&[0, 1, 2, 4, 5]), "pentatonic", "simple, safe, no semitones", "semplice, sicura, senza semitoni", "hooks, toplines, plucks"),
    s!("minor_pentatonic", "Minor pentatonic", "Pentatonica minore", &[0, 3, 5, 7, 10], Some(&[0, 2, 3, 4, 6]), "pentatonic", "simple, riff-friendly", "semplice, adatta ai riff", "riffs, hooks"),
    s!("blues", "Blues (minor)", "Blues (minore)", &[0, 3, 5, 6, 7, 10], Some(&[0, 2, 3, 4, 4, 6]), "blues", "gritty, vocal", "ruvida, vocale", "breaks, funky basslines"),
    s!("major_blues", "Major blues", "Blues maggiore", &[0, 2, 3, 4, 7, 9], Some(&[0, 1, 2, 2, 4, 5]), "blues", "bright with a blue note", "luminosa con blue note", "funk, gospel house"),
    s!("chromatic", "Chromatic", "Cromatica", &[0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11], None, "symmetric", "all twelve semitones", "tutti i dodici semitoni", "passing lines, FX"),
    s!("whole_tone", "Whole tone (hexatonic)", "Esatonale (per toni interi)", &[0, 2, 4, 6, 8, 10], None, "symmetric", "ambiguous, dreamlike", "ambigua, onirica", "transitions, risers, FX"),
    s!("diminished_hw", "Octatonic half–whole", "Ottatonica semitono–tono", &[0, 1, 3, 4, 6, 7, 9, 10], None, "symmetric", "tense, for dominant 7♭9", "tesa, per dominanti 7♭9", "jazz, film tension"),
    s!("diminished_wh", "Octatonic whole–half", "Ottatonica tono–semitono", &[0, 2, 3, 5, 6, 8, 9, 11], None, "symmetric", "tense, for diminished chords", "tesa, per accordi diminuiti", "film tension"),
    s!("augmented", "Augmented (hexatonic)", "Aumentata (esatonica)", &[0, 3, 4, 7, 8, 11], None, "symmetric", "symmetric, floating", "simmetrica, sospesa", "film, jazz"),
    s!("double_harmonic", "Double harmonic major (Byzantine)", "Doppia armonica maggiore (bizantina)", &[0, 1, 4, 5, 7, 8, 11], SEVEN, "other", "exotic, dramatic", "esotica, drammatica", "world, cinematic"),
    s!("hungarian_minor", "Hungarian minor", "Minore ungherese", &[0, 2, 3, 6, 7, 8, 11], SEVEN, "other", "dark, exotic", "scura, esotica", "world, cinematic"),
    s!("hirajoshi", "Hirajōshi", "Hirajōshi", &[0, 2, 3, 7, 8], Some(&[0, 1, 2, 4, 5]), "other", "Japanese pentatonic, melancholic", "pentatonica giapponese, malinconica", "ambient, world"),
];

const DEGREE_EN: [&str; 7] = ["tonic", "supertonic", "mediant", "subdominant", "dominant", "submediant", "leading tone"];
const DEGREE_IT: [&str; 7] = ["tonica", "sopratonica", "modale (mediante)", "sottodominante", "dominante", "sopradominante", "sensibile"];

pub fn find(id: &str) -> Option<&'static ScaleDef> {
    let k = id.trim().to_lowercase().replace([' ', '-'], "_");
    let alias = match k.as_str() {
        "ionian" | "maj" | "maggiore" | "ionica" => "major",
        "minor" | "min" | "aeolian" | "minore" | "minore_naturale" | "eolia" => "natural_minor",
        "minore_armonica" => "harmonic_minor",
        "minore_melodica" | "jazz_minor" => "melodic_minor",
        "dorica" => "dorian",
        "frigia" => "phrygian",
        "lidia" => "lydian",
        "misolidia" => "mixolydian",
        "locria" => "locrian",
        "pentatonic" | "pentatonica" | "pentatonica_maggiore" => "major_pentatonic",
        "pentatonica_minore" => "minor_pentatonic",
        "cromatica" => "chromatic",
        "esatonale" | "whole_tone_scale" | "toni_interi" => "whole_tone",
        "diminished" | "octatonic" | "diminuita" => "diminished_hw",
        "super_locrian" | "superlocria" | "alterata" => "altered",
        "spanish_phrygian" | "freygish" | "frigia_dominante" => "phrygian_dominant",
        "byzantine" | "arabic" => "double_harmonic",
        other => other,
    };
    SCALES.iter().find(|s| s.id == alias)
}

/// Notes of a scale on a tonic. Letter-based spelling where the scale defines it; otherwise flats when the tonic
/// is flat (or F), sharps otherwise.
pub fn build(tonic: Note, def: &ScaleDef) -> Vec<Note> {
    let flats = tonic.acc < 0 || (tonic.letter == 3 && tonic.acc == 0);
    def.intervals
        .iter()
        .enumerate()
        .map(|(i, &iv)| match def.letters {
            Some(l) => {
                let n = tonic.up(l[i], iv);
                if n.acc.abs() > 2 { Note::simple((tonic.pc() + iv) % 12, flats) } else { n }
            }
            None => Note::simple((tonic.pc() + iv) % 12, flats),
        })
        .collect()
}

/// Full description of a scale on a tonic: notes (IT/EN), MIDI (from the tonic's 4th octave), formula, steps,
/// degrees, character, diatonic chords (7-note scales).
pub fn scale_info(tonic: &str, scale: &str) -> Result<Value, String> {
    let t = parse_note(tonic)?;
    let def = find(scale).ok_or_else(|| format!("unknown scale {scale:?}"))?;
    let notes = build(t, def);
    let base = 60 + t.pc() as i32;
    let midi: Vec<i32> = def.intervals.iter().map(|&i| base + i as i32).collect();
    let steps: Vec<u8> = def.intervals.windows(2).map(|w| w[1] - w[0]).chain(std::iter::once(12 - def.intervals.last().unwrap())).collect();
    let step_names: Vec<&str> = steps.iter().map(|s| match s { 1 => "H", 2 => "W", 3 => "W+H", _ => "?" }).collect();
    let heptatonic = def.intervals.len() == 7;
    let degrees: Vec<Value> = notes
        .iter()
        .enumerate()
        .map(|(i, n)| {
            let mut d = json!({ "degree": i + 1, "note": n.name(), "italian": italian(*n), "interval": interval_name(def.intervals[i]), "midi": midi[i] });
            if heptatonic {
                let leading = def.intervals[6] == 11;
                d["name"] = json!(if i == 6 && !leading { "subtonic" } else { DEGREE_EN[i] });
                d["nome"] = json!(if i == 6 && !leading { "sottotonica" } else { DEGREE_IT[i] });
            }
            d
        })
        .collect();
    let mut v = json!({
        "tonic": t.name(), "tonic_italian": italian(t), "scale": def.id, "name": def.en, "nome": def.it,
        "notes": notes.iter().map(|n| n.name()).collect::<Vec<_>>(), "note_italiane": notes.iter().map(|n| italian(*n)).collect::<Vec<_>>(),
        "midi_from_octave_4": midi, "formula_semitones": def.intervals, "steps": step_names,
        "degrees": degrees, "family": def.family, "character": def.character, "carattere": def.carattere, "contexts": def.contexts,
        "source": def.source, "note_count": def.intervals.len(),
    });
    if heptatonic {
        let pcs: Vec<u8> = notes.iter().map(|n| n.pc()).collect();
        let chords: Vec<Value> = (0..7)
            .map(|d| {
                let tri: Vec<u8> = (0..3).map(|k| pcs[(d + 2 * k) % 7]).collect();
                let sev: Vec<u8> = (0..4).map(|k| pcs[(d + 2 * k) % 7]).collect();
                json!({ "degree": d + 1, "triad": super::chords::name_pcs(&tri, notes[d]), "seventh": super::chords::name_pcs(&sev, notes[d]) })
            })
            .collect();
        v["diatonic_chords"] = json!(chords);
    }
    if def.family == "major_modes" {
        let rot = [("major", 0), ("dorian", 1), ("phrygian", 2), ("lydian", 3), ("mixolydian", 4), ("natural_minor", 5), ("locrian", 6)];
        if let Some((_, k)) = rot.iter().find(|(id, _)| *id == def.id) {
            let parent = Note::simple((t.pc() + 12 - SCALES[0].intervals[*k]) % 12, t.acc < 0);
            v["parent_major"] = json!(parent.name());
            v["mode_of_major"] = json!(k + 1);
        }
    }
    Ok(v)
}

/// Scales (from the catalog) that contain all the given pitch classes, smallest first: used for "which scale is
/// this melody in" (several answers when the material allows several).
pub fn fitting(pcs: &[u8], tonic_hint: Option<u8>) -> Vec<Value> {
    let set: std::collections::BTreeSet<u8> = pcs.iter().map(|p| p % 12).collect();
    let mut out = vec![];
    for def in SCALES.iter().filter(|d| d.id != "chromatic") {
        for root in 0..12u8 {
            let sp: Vec<u8> = def.intervals.iter().map(|i| (root + i) % 12).collect();
            if set.iter().all(|p| sp.contains(p)) {
                let t = Note::simple(root, false);
                out.push((def.intervals.len(), tonic_hint != Some(root), json!({ "tonic": t.name(), "scale": def.id, "name": format!("{} {}", t.name(), def.en), "extra_notes": def.intervals.len() - set.len() })));
            }
        }
    }
    out.sort_by_key(|x| (x.0, x.1));
    out.into_iter().map(|x| x.2).take(12).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn notes(t: &str, s: &str) -> String {
        scale_info(t, s).unwrap()["notes"].as_array().unwrap().iter().map(|n| n.as_str().unwrap()).collect::<Vec<_>>().join(" ")
    }

    #[test]
    fn scales_from_any_tonic() {
        // the prompt's example: "Genera la scala di Fa diesis minore"
        let f = scale_info("Fa diesis", "minore").unwrap();
        assert_eq!(notes("F#", "natural_minor"), "F# G# A B C# D E");
        assert_eq!(f["note_italiane"][0], "Fa♯");
        assert_eq!(f["midi_from_octave_4"], json!([66, 68, 69, 71, 73, 74, 76]));
        assert_eq!(f["degrees"][4]["nome"], "dominante");
        assert_eq!(f["degrees"][6]["name"], "subtonic", "natural minor has a subtonic, not a leading tone");
        assert_eq!(notes("C", "harmonic_minor"), "C D Eb F G Ab B");
        assert_eq!(notes("Eb", "dorian"), "Eb F Gb Ab Bb C Db");
        assert_eq!(notes("E", "phrygian_dominant"), "E F G# A B C D");
        assert_eq!(notes("G", "altered"), "G Ab Bb Cb Db Eb F");
        assert_eq!(notes("C", "whole_tone"), "C D E F# G# A#");
        assert_eq!(notes("C", "diminished_hw"), "C C# D# E F# G A A#", "8-note scales: plain sharps on a natural tonic");
        assert_eq!(notes("Bb", "diminished_hw"), "Bb B Db D E F G Ab", "flats on a flat tonic");
        assert_eq!(notes("A", "minor_pentatonic"), "A C D E G");
        assert_eq!(notes("A", "blues"), "A C D Eb E G");
        assert_eq!(scale_info("C", "chromatic").unwrap()["note_count"], 12);
        assert_eq!(f["diatonic_chords"][0]["triad"], "F#m");
        assert_eq!(f["diatonic_chords"][4]["seventh"], "C#m7");
        assert_eq!(scale_info("D", "dorian").unwrap()["parent_major"], "C");
        // every 7-note scale on every tonic uses each letter once
        for def in SCALES.iter().filter(|d| d.letters == SEVEN) {
            for t in ["C", "C#", "Db", "D", "Eb", "E", "F", "F#", "Gb", "G", "Ab", "A", "Bb", "B"] {
                let mut l: Vec<u8> = build(Note::parse(t).unwrap(), def).iter().map(|n| n.letter).collect();
                l.sort();
                l.dedup();
                assert_eq!(l.len(), 7, "{t} {}", def.id);
            }
        }
        assert!(scale_info("C", "nope").is_err());
    }

    #[test]
    fn scales_that_fit_some_notes() {
        let f = fitting(&[9, 0, 4, 7, 2], Some(9));
        assert_eq!(f[0]["name"], "A Minor pentatonic", "the smallest scale containing A C D E G, with the hinted tonic first");
        assert!(f.iter().any(|x| x["name"] == "A Natural minor (Aeolian)"));
    }
}
