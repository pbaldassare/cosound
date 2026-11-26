//! Scale & Key Engine – keys: the 30 standard key signatures, circle of fifths, related keys, modulation plans
//! (pivot chords) and key estimation from notes with ranked candidates and an honest confidence.
//! Key finding: Krumhansl–Kessler profiles (theory::detect_key); guards against a single prevailing note.
use super::notes::italian;
use super::theory::{self as t, Key, Mode, Note};
use serde_json::{json, Value};

/// Major keys around the circle of fifths, from 7 flats to 7 sharps.
pub const CIRCLE_MAJOR: [&str; 15] = ["Cb", "Gb", "Db", "Ab", "Eb", "Bb", "F", "C", "G", "D", "A", "E", "B", "F#", "C#"];

pub fn key_name_it(k: Key) -> String {
    let m = match k.mode {
        Mode::Major => "maggiore",
        Mode::NaturalMinor => "minore",
        Mode::HarmonicMinor => "minore armonica",
        Mode::MelodicMinor => "minore melodica",
        Mode::Dorian => "dorico",
        Mode::Phrygian => "frigio",
        Mode::Lydian => "lidio",
        Mode::Mixolydian => "misolidio",
        Mode::Locrian => "locrio",
        Mode::MajorPentatonic => "pentatonica maggiore",
        Mode::MinorPentatonic => "pentatonica minore",
        Mode::Blues => "blues",
    };
    format!("{} {m}", italian(k.tonic))
}

/// "La minore", "Fa# maggiore", "A minor", "Am", "Sol dorico".
pub fn parse_key(s: &str) -> Result<Key, String> {
    let l = s.trim().to_lowercase().replace("tonalità di ", "").replace("tonalita di ", "");
    let words: Vec<&str> = l.split_whitespace().collect();
    if words.is_empty() {
        return Err("empty key".into());
    }
    // root may be two words ("fa diesis", "si bemolle")
    let (root, rest) = if words.len() >= 2 && ["diesis", "bemolle", "#", "b"].contains(&words[1]) { (format!("{} {}", words[0], words[1]), &words[2..]) } else { (words[0].to_string(), &words[1..]) };
    let mode_it = rest.join(" ");
    let mode = match mode_it.as_str() {
        "" | "maggiore" | "major" | "maj" => "major",
        "minore" | "minor" | "min" | "m" => "minor",
        "minore armonica" | "harmonic minor" => "harmonic_minor",
        "minore melodica" | "melodic minor" => "melodic_minor",
        "dorico" | "dorica" | "dorian" => "dorian",
        "frigio" | "frigia" | "phrygian" => "phrygian",
        "lidio" | "lidia" | "lydian" => "lydian",
        "misolidio" | "misolidia" | "mixolydian" => "mixolydian",
        "locrio" | "locria" | "locrian" => "locrian",
        other => other,
    };
    if rest.is_empty() && root.len() > 1 && root.ends_with('m') && !root.ends_with("im") {
        if let Ok(n) = super::notes::parse_note(&root[..root.len() - 1]) {
            return Ok(Key { tonic: n, mode: Mode::NaturalMinor });
        }
    }
    let tonic = super::notes::parse_note(&root)?;
    Ok(Key { tonic, mode: Mode::parse(mode)? })
}

pub fn key_view(k: Key) -> Value {
    let sig = k.signature();
    let accs: Vec<String> = {
        let order_sharp = ["F", "C", "G", "D", "A", "E", "B"];
        let order_flat = ["B", "E", "A", "D", "G", "C", "F"];
        if sig > 0 { order_sharp[..sig.min(7) as usize].iter().map(|n| format!("{n}#")).collect() } else { order_flat[..(-sig).min(7) as usize].iter().map(|n| format!("{n}b")).collect() }
    };
    json!({
        "name": k.name(), "nome": key_name_it(k), "tonic": k.tonic.name(), "mode": k.mode.id(),
        "scale": k.scale().iter().map(|n| n.name()).collect::<Vec<_>>(),
        "signature": { "count": sig.abs(), "type": if sig > 0 { "sharps" } else if sig < 0 { "flats" } else { "none" }, "accidentals": accs },
        "relative": k.relative().map(|r| r.name()), "parallel": k.parallel().map(|r| r.name()),
        "circle_position": circle_position(k),
        "closely_related": related(k).iter().map(|r| r.name()).collect::<Vec<_>>(),
    })
}

/// Position on the circle of fifths (0 = C / A minor, +1 per sharp, −1 per flat).
pub fn circle_position(k: Key) -> i32 {
    let base = match k.mode {
        Mode::HarmonicMinor | Mode::MelodicMinor | Mode::MinorPentatonic | Mode::Blues => Key { tonic: k.tonic, mode: Mode::NaturalMinor },
        Mode::MajorPentatonic => Key { tonic: k.tonic, mode: Mode::Major },
        _ => k,
    };
    base.signature()
}

/// Closely related keys: relative, dominant, subdominant and their relatives (one accidental apart).
pub fn related(k: Key) -> Vec<Key> {
    let (maj, min) = match k.mode {
        Mode::Major => (k, k.relative().unwrap()),
        _ => match (Key { tonic: k.tonic, mode: Mode::NaturalMinor }).relative() {
            Some(m) => (m, Key { tonic: k.tonic, mode: Mode::NaturalMinor }),
            None => return vec![],
        },
    };
    let fifth = |x: Key| Key { tonic: x.tonic.up(4, 7), mode: x.mode };
    let fourth = |x: Key| Key { tonic: x.tonic.up(3, 5), mode: x.mode };
    let mut v = vec![if k.mode == Mode::Major { min } else { maj }, fifth(maj), fourth(maj), fifth(min), fourth(min)];
    v.retain(|x| x.signature().abs() <= 7);
    v
}

/// Modulation from one key to another: distance on the circle, common tones, pivot chords (diatonic in both),
/// and a suggested route (pivot chord, else the new key's V7).
pub fn modulation(from: Key, to: Key) -> Value {
    let a = (0..7).map(|d| t::diatonic(from, d, false)).collect::<Vec<_>>();
    let b = (0..7).map(|d| t::diatonic(to, d, false)).collect::<Vec<_>>();
    let pivots: Vec<Value> = a
        .iter()
        .filter_map(|x| b.iter().find(|y| y.pcs() == x.pcs()).map(|y| json!({ "chord": x.symbol(), "in_from": x.roman, "in_to": y.roman })))
        .collect();
    let common: Vec<String> = from.pcs().iter().filter(|p| to.pcs().contains(p)).map(|p| Note::simple(*p, from.signature() < 0).name()).collect();
    let v7 = t::numeral(to, "V7", false).unwrap();
    let route = if let Some(p) = pivots.iter().find(|p| p["in_to"].as_str().is_some_and(|r| r.starts_with("ii") || r.starts_with("IV") || r.starts_with("iv"))).or(pivots.first()) {
        format!("pivot chord {} ({} in {} = {} in {}), then {} → {}", p["chord"].as_str().unwrap(), p["in_from"].as_str().unwrap(), from.name(), p["in_to"].as_str().unwrap(), to.name(), v7.symbol(), to.tonic.name())
    } else {
        format!("direct modulation through the new dominant {} → {}", v7.symbol(), to.tonic.name())
    };
    json!({
        "from": from.name(), "to": to.name(), "circle_distance": (circle_position(to) - circle_position(from)).abs(),
        "common_tones": common, "pivot_chords": pivots, "suggested_route": route,
        "closely_related": related(from).iter().any(|r| r.pcs() == to.pcs() || r.name() == to.name()),
        "source": "docs/knowledge/music-theory/harmony.md §4",
    })
}

/// Key estimation from notes (pitch, duration weight). Ranked candidates with confidence; flags material that
/// cannot support a key (one prevailing note, too few pitch classes) instead of naming a key with false certainty.
pub fn estimate(notes: &[(u8, f64)]) -> Value {
    let mut w = [0.0f64; 12];
    for (p, d) in notes {
        w[(*p % 12) as usize] += d.max(0.0);
    }
    let total: f64 = w.iter().sum();
    if total == 0.0 {
        return json!({ "key": null, "candidates": [], "reason": "no notes" });
    }
    let distinct = w.iter().filter(|x| **x > 0.0).count();
    let (top_pc, top_share) = w.iter().enumerate().map(|(i, x)| (i, x / total)).max_by(|a, b| a.1.total_cmp(&b.1)).unwrap();
    let ranked = t::detect_key(&w);
    let margin = ranked[0].1 - ranked[1].1;
    let mut flags = vec![];
    if distinct < 3 {
        flags.push(format!("only {distinct} pitch class(es): not enough material for a key"));
    }
    if top_share > 0.5 {
        flags.push(format!("one note ({}) carries {:.0}% of the duration: it is a tonal centre candidate, not proof of a key", Note::simple(top_pc as u8, false).name(), top_share * 100.0));
    }
    if margin < 0.05 {
        flags.push(format!("{} and {} fit almost equally", ranked[0].0.name(), ranked[1].0.name()));
    }
    let conf = if distinct < 3 { 0.0 } else { ((ranked[0].1 - 0.4) / 0.5).clamp(0.0, 1.0) * (margin / 0.15).clamp(0.3, 1.0) * if top_share > 0.5 { 0.5 } else { 1.0 } };
    let r3 = |x: f64| (x * 1000.0).round() / 1000.0;
    let pcs: Vec<u8> = (0..12).filter(|i| w[*i as usize] > 0.0).collect();
    json!({
        "key": if distinct < 3 { Value::Null } else { json!(ranked[0].0.name()) },
        "nome": if distinct < 3 { Value::Null } else { json!(key_name_it(ranked[0].0)) },
        "confidence": r3(conf),
        "candidates": ranked.iter().take(5).map(|(k, c)| json!({ "key": k.name(), "nome": key_name_it(*k), "correlation": r3(*c) })).collect::<Vec<_>>(),
        "tonal_centre_candidate": Note::simple(top_pc as u8, false).name(),
        "flags": flags,
        "fitting_scales": super::scales::fitting(&pcs, Some(ranked[0].0.tonic.pc())),
        "method": "Krumhansl–Kessler key profiles on duration-weighted pitch classes (estimate)",
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_italian_and_english_keys() {
        assert_eq!(parse_key("La minore").unwrap().name(), "A minor");
        assert_eq!(parse_key("tonalità di Fa diesis minore").unwrap().name(), "F# minor");
        assert_eq!(parse_key("Si bemolle maggiore").unwrap().name(), "Bb major");
        assert_eq!(parse_key("Am").unwrap().name(), "A minor");
        assert_eq!(parse_key("Re dorico").unwrap().name(), "D dorian");
        assert_eq!(key_name_it(parse_key("C# minor").unwrap()), "Do♯ minore");
    }

    #[test]
    fn signatures_circle_related_and_modulation() {
        let v = key_view(parse_key("E major").unwrap());
        assert_eq!(v["signature"]["accidentals"], json!(["F#", "C#", "G#", "D#"]));
        assert_eq!(key_view(parse_key("D minor").unwrap())["signature"]["accidentals"], json!(["Bb"]));
        assert_eq!(CIRCLE_MAJOR.iter().map(|k| parse_key(k).unwrap().signature()).collect::<Vec<_>>(), (-7..=7).collect::<Vec<_>>());
        let rel: Vec<String> = related(parse_key("C major").unwrap()).iter().map(|k| k.name()).collect();
        assert_eq!(rel, ["A minor", "G major", "F major", "E minor", "D minor"]);
        let m = modulation(parse_key("C major").unwrap(), parse_key("G major").unwrap());
        assert_eq!(m["circle_distance"], 1);
        assert!(m["pivot_chords"].as_array().unwrap().iter().any(|p| p["chord"] == "Am" && p["in_to"] == "ii"), "{m}");
        let far = modulation(parse_key("C major").unwrap(), parse_key("F# major").unwrap());
        assert!(far["pivot_chords"].as_array().unwrap().is_empty() && far["suggested_route"].as_str().unwrap().contains("C#7"));
    }

    #[test]
    fn estimation_with_candidates_and_guards() {
        // A natural minor melody
        let notes: Vec<(u8, f64)> = [57, 60, 64, 62, 60, 59, 57, 64, 65, 64, 62, 60, 57].iter().map(|p| (*p, 1.0)).collect();
        let e = estimate(&notes);
        assert!(e["key"] == "A minor" || e["candidates"][0]["key"] == "A minor", "{e}");
        assert!(e["candidates"].as_array().unwrap().len() == 5);
        // a single prevailing note is not called a key
        let drone: Vec<(u8, f64)> = vec![(45, 30.0), (57, 1.0), (52, 0.5)];
        let d = estimate(&drone);
        assert!(d["flags"].as_array().unwrap().iter().any(|f| f.as_str().unwrap().contains("tonal centre candidate")));
        assert!(d["confidence"].as_f64().unwrap() < 0.5);
        assert!(estimate(&[(60, 1.0), (67, 1.0)])["key"].is_null(), "two pitch classes: no key");
    }
}
