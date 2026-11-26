//! Capability handlers (JSON in → JSON out) for the Capability Bus and the app UI.
//! Read-only: nothing is written to disk here. Every input field is optional; a `genre`
//! sets coherent defaults (key, progression, tempo, styles) that explicit fields override.
use super::generate::{self as g, Part, Spec, BAR, PPQ};
use super::{smf, theory as t};
use serde_json::{json, Map, Value};

pub struct Input {
    pub key: t::Key,
    pub chords: Vec<t::Chord>,
    pub bpm: f64,
    pub bars: u32,
    pub seed: u64,
    pub swing: u32,
    pub bass_style: String,
    pub drum_style: String,
    pub chord_rhythm: String,
    pub fill: bool,
    pub genre: Option<&'static str>,
}

impl Input {
    pub fn spec(&self) -> Spec<'_> {
        Spec { key: self.key, chords: &self.chords, bars: self.bars, seed: self.seed, swing: self.swing }
    }
    pub fn title(&self) -> String {
        format!("COSOUND sketch - {} - {} BPM", self.key.name(), self.bpm)
    }
}

fn number(input: &Value, k: &str, default: f64, lo: f64, hi: f64) -> Result<f64, String> {
    match &input[k] {
        Value::Null => Ok(default),
        v => v.as_f64().filter(|x| (lo..=hi).contains(x)).ok_or_else(|| format!("{k} must be a number between {lo} and {hi}")),
    }
}

fn text<'a>(input: &'a Value, k: &str, default: &'a str) -> Result<&'a str, String> {
    match &input[k] {
        Value::Null => Ok(default),
        v => v.as_str().ok_or_else(|| format!("{k} must be a string")),
    }
}

pub fn parse(input: &Value) -> Result<Input, String> {
    let genre = match &input["genre"] {
        Value::Null => None,
        v => {
            let id = v.as_str().unwrap_or("");
            Some(g::genre(id).ok_or_else(|| format!("unknown genre {id:?} (use {})", g::GENRES.map(|g| g.id).join(", ")))?)
        }
    };
    let key = match (&input["key"], &input["tonic"]) {
        (Value::String(k), _) => t::Key::parse(k)?,
        (_, Value::String(tonic)) => t::Key::new(tonic, text(input, "mode", "major")?)?,
        _ => t::Key::parse(genre.map_or("A minor", |g| g.key))?,
    };
    let prog = match &input["progression"] {
        Value::String(s) => s.clone(),
        Value::Array(a) => a.iter().map(|v| v.as_str().unwrap_or("?")).collect::<Vec<_>>().join("-"),
        Value::Null => genre.map_or_else(|| t::default_progression(key.mode), |g| g.progression).to_string(),
        _ => return Err("progression must be a string like \"i-VI-III-VII\" or a list".into()),
    };
    let sevenths = input["sevenths"].as_bool().unwrap_or(false);
    let bars = number(input, "bars", 8.0, 1.0, 64.0)? as u32;
    Ok(Input {
        key,
        chords: t::progression(key, &prog, sevenths)?,
        bpm: number(input, "bpm", genre.map_or(124.0, |g| g.bpm), 40.0, 250.0)?,
        bars,
        seed: input["seed"].as_u64().unwrap_or(1),
        swing: number(input, "swing", genre.map_or(0.0, |g| g.swing as f64), 0.0, 100.0)? as u32,
        bass_style: text(input, "bass_style", genre.map_or("offbeat", |g| g.bass))?.into(),
        drum_style: text(input, "drum_style", genre.map_or("techno", |g| g.drums))?.into(),
        chord_rhythm: text(input, "chord_rhythm", genre.map_or("sustain", |g| g.chord_rhythm))?.into(),
        fill: input["fill"].as_bool().unwrap_or(bars >= 4),
        genre: genre.map(|g| g.id),
    })
}

fn key_view(k: t::Key) -> Value {
    let sig = k.signature();
    json!({
        "name": k.name(),
        "tonic": k.tonic.name(),
        "mode": k.mode.id(),
        "scale": k.scale().iter().map(|n| n.name()).collect::<Vec<_>>(),
        "intervals": k.mode.intervals().iter().map(|&i| t::interval_name(i)).collect::<Vec<_>>(),
        "pitch_classes": k.pcs(),
        "signature": { "count": sig.abs(), "accidental": if sig > 0 { "sharps" } else if sig < 0 { "flats" } else { "none" } },
        "relative": k.relative().map(|r| r.name()),
        "parallel": k.parallel().map(|r| r.name()),
    })
}

fn progression_view(i: &Input) -> Value {
    let ch = &i.chords;
    json!({
        "chords": ch.iter().map(|c| c.view()).collect::<Vec<_>>(),
        "final_cadence": if ch.len() > 1 { t::cadence(&ch[ch.len() - 2], &ch[ch.len() - 1]) } else { "single_chord" },
        "loop_cadence": t::cadence(&ch[ch.len() - 1], &ch[0]),
        "tension_curve": ch.iter().map(|c| c.tension()).collect::<Vec<_>>(),
    })
}

/// music_theory.key_info
pub fn key_info(input: &Value) -> Result<Value, String> {
    let k = parse(input)?.key;
    let chords = |sev| (0..7).map(|d| t::diatonic(k, d, sev).view()).collect::<Vec<_>>();
    let mut v = key_view(k);
    v["triads"] = json!(chords(false));
    v["sevenths"] = json!(chords(true));
    if k.mode.harmonic_base() != k.mode {
        v["chords_from"] = json!(t::Key { tonic: k.tonic, mode: k.mode.harmonic_base() }.name());
    }
    Ok(v)
}

/// music_theory.progression
pub fn progression(input: &Value) -> Result<Value, String> {
    let i = parse(input)?;
    Ok(json!({ "key": key_view(i.key), "progression": progression_view(&i) }))
}

fn make(i: &Input, part: &str) -> Result<Part, String> {
    let spec = i.spec();
    match part {
        "chords" => g::chords(&spec, &i.chord_rhythm),
        "bass" => g::bass(&spec, &i.bass_style),
        "melody" => Ok(g::melody(&spec)),
        "drums" => g::drums(&spec, &i.drum_style, i.fill),
        _ => Err(format!("unknown part {part}")),
    }
}

pub fn base64(b: &[u8]) -> String {
    const A: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut s = String::with_capacity(b.len().div_ceil(3) * 4);
    for c in b.chunks(3) {
        let n = (c[0] as u32) << 16 | (*c.get(1).unwrap_or(&0) as u32) << 8 | *c.get(2).unwrap_or(&0) as u32;
        for k in 0..4 {
            s.push(if k <= c.len() { A[(n >> (18 - 6 * k) & 63) as usize] as char } else { '=' });
        }
    }
    s
}

fn checks_view(i: &Input, parts: &[Part]) -> (Value, bool) {
    let spec = i.spec();
    let mut all = true;
    let mut m = Map::new();
    for p in parts {
        let c = g::check(p, &spec);
        all &= g::all_pass(&c);
        m.insert(p.id.into(), Value::Object(c));
    }
    (Value::Object(m), all)
}

fn header(i: &Input) -> Value {
    json!({
        "engine": "cosound-music-engine (deterministic, no AI)",
        "genre": i.genre,
        "key": key_view(i.key),
        "bpm": i.bpm,
        "bars": i.bars,
        "time_signature": "4/4",
        "ppq": PPQ,
        "length_ticks": i.bars * BAR,
        "seed": i.seed,
        "swing": i.swing,
        "styles": { "chords": i.chord_rhythm, "bass": i.bass_style, "drums": i.drum_style, "fill": i.fill },
        "progression": progression_view(i),
    })
}

/// music_theory.generate_chords / generate_bassline / generate_melody, rhythm_engineer.generate_drums.
/// Returns the notes, the checks actually run on them and the part as a Standard MIDI File (base64).
pub fn generate(input: &Value, part: &str) -> Result<Value, String> {
    let i = parse(input)?;
    let p = make(&i, part)?;
    let (checks, ok) = checks_view(&i, std::slice::from_ref(&p));
    let midi = smf::write(std::slice::from_ref(&p), i.bpm, i.key, &i.title());
    let mut v = header(&i);
    v["part"] = json!(p);
    v["checks"] = checks;
    v["all_checks_passed"] = json!(ok);
    v["midi"] = json!({ "format": 1, "bytes": midi.len(), "base64": base64(&midi) });
    Ok(v)
}

/// Choices the UI offers: genres with their defaults, modes, styles. Single source: the engine.
pub fn options() -> Value {
    json!({
        "genres": g::GENRES.iter().map(|g| json!({
            "id": g.id, "label": g.label, "bpm": g.bpm, "key": g.key, "progression": g.progression,
            "bass_style": g.bass, "drum_style": g.drums, "chord_rhythm": g.chord_rhythm, "swing": g.swing,
        })).collect::<Vec<_>>(),
        "modes": t::Mode::ALL.iter().map(|m| json!({ "id": m.id(), "default_progression": t::default_progression(*m) })).collect::<Vec<_>>(),
        "bass_styles": g::BASS_STYLES,
        "drum_styles": g::DRUM_STYLES,
        "chord_rhythms": g::CHORD_RHYTHMS,
    })
}

/// rhythm_engineer.generate_transition: snare build, tom fill or drop impact.
pub fn transition(input: &Value) -> Result<Value, String> {
    let kind = input["kind"].as_str().ok_or("kind is required")?;
    let bars = input["bars"].as_u64().unwrap_or(if kind == "snare_build" { 8 } else { 1 }).clamp(1, 16) as u32;
    let bpm = number(input, "bpm", 124.0, 40.0, 250.0)?;
    let seed = input["seed"].as_u64().unwrap_or(1);
    let part = g::transition(kind, bars, seed)?;
    let key = t::Key::parse("C major")?;
    let chords = t::progression(key, "I", false)?;
    let len_bars = if kind == "snare_build" { bars } else { 1 };
    let spec = Spec { key, chords: &chords, bars: len_bars, seed, swing: 0 };
    let checks = g::check(&part, &spec);
    let ok = g::all_pass(&checks);
    let midi = smf::write(std::slice::from_ref(&part), bpm, key, &format!("COSOUND {kind} - {bpm} BPM"));
    Ok(json!({
        "kind": kind, "bars": len_bars, "bpm": bpm, "ppq": PPQ, "length_ticks": len_bars * BAR, "part": part,
        "checks": { "transition": checks }, "all_checks_passed": ok,
        "midi": { "format": 1, "bytes": midi.len(), "base64": base64(&midi) },
    }))
}

/// rhythm_engineer.groove_timing
pub fn groove(input: &Value) -> Result<Value, String> {
    let bpm = number(input, "bpm", 124.0, 40.0, 250.0)?;
    let swing = number(input, "swing", 0.0, 0.0, 100.0)? as u32;
    Ok(g::groove_timing(bpm, swing))
}

/// music_theory.check_coherence: run the musical checks on parts written elsewhere
/// (by the user, another tool or a future engine). PPQ must be 480.
pub fn check_parts(input: &Value) -> Result<Value, String> {
    if input["ppq"].as_u64().is_some_and(|p| p != PPQ as u64) {
        return Err(format!("ppq must be {PPQ}"));
    }
    let i = parse(input)?;
    let spec = i.spec();
    let mut out = Map::new();
    let mut all = true;
    for p in input["parts"].as_array().ok_or("parts is required")? {
        let id: &'static str = match p["id"].as_str() {
            Some("chords") => "chords",
            Some("bass") => "bass",
            Some("melody") => "melody",
            other => return Err(format!("part id {other:?} must be chords, bass or melody")),
        };
        let notes: Vec<g::MidiNote> = serde_json::from_value(p["notes"].clone()).map_err(|e| format!("notes of {id}: {e}"))?;
        let part = Part { id, name: id, channel: 0, program: 0, notes };
        let c = g::check(&part, &spec);
        all &= g::all_pass(&c);
        out.insert(id.into(), Value::Object(c));
    }
    let mut v = header(&i);
    v["checks"] = Value::Object(out);
    v["all_checks_passed"] = json!(all);
    Ok(v)
}

const ALL_PARTS: [&str; 4] = ["chords", "bass", "melody", "drums"];

fn selected(input: &Value) -> Vec<&str> {
    match input["parts"].as_array() {
        Some(a) => ALL_PARTS.into_iter().filter(|p| a.iter().any(|v| v.as_str() == Some(p))).collect(),
        None => ALL_PARTS.to_vec(),
    }
}

/// Full sketch for the app UI: all four parts on the same progression and seed.
pub fn sketch(input: &Value) -> Result<Value, String> {
    let i = parse(input)?;
    let parts: Vec<Part> = selected(input).iter().map(|p| make(&i, p)).collect::<Result<_, _>>()?;
    let (checks, ok) = checks_view(&i, &parts);
    let mut v = header(&i);
    v["parts"] = json!(parts);
    v["checks"] = checks;
    v["all_checks_passed"] = json!(ok);
    Ok(v)
}

/// Standard MIDI File of the sketch (same input → same file). Returns bytes and a file title.
pub fn export(input: &Value) -> Result<(Vec<u8>, String), String> {
    let i = parse(input)?;
    let parts: Vec<Part> = selected(input).iter().map(|p| make(&i, p)).collect::<Result<_, _>>()?;
    if parts.is_empty() {
        return Err("no parts selected".into());
    }
    let (_, ok) = checks_view(&i, &parts);
    if !ok {
        return Err("generated parts failed the musical checks; not exported".into());
    }
    Ok((smf::write(&parts, i.bpm, i.key, &i.title()), format!("{} {} {}bpm", i.genre.unwrap_or("sketch"), i.key.name(), i.bpm)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_genres_and_errors() {
        let v = generate(&json!({}), "bass").unwrap();
        assert_eq!(v["key"]["name"], "A minor");
        assert_eq!(v["all_checks_passed"], true);
        let v = generate(&json!({"genre": "melodic_techno", "bpm": 124, "bars": 16, "seed": 42}), "drums").unwrap();
        assert_eq!((v["bpm"].as_f64(), v["bars"].as_u64()), (Some(124.0), Some(16)));
        assert!(v["midi"]["base64"].as_str().unwrap().starts_with("TVRoZA")); // "MThd"
        let v = progression(&json!({"key": "C major", "progression": ["ii7", "V7", "Imaj7"]})).unwrap();
        assert_eq!(v["progression"]["final_cadence"], "authentic");
        let k = key_info(&json!({"tonic": "Eb", "mode": "minor"})).unwrap();
        assert_eq!((k["signature"]["count"].as_i64(), k["signature"]["accidental"].as_str(), k["relative"].as_str()), (Some(6), Some("flats"), Some("Gb major")));
        assert_eq!(k["triads"][4]["symbol"], "Bbm");
        assert!(parse(&json!({"genre": "polka"})).is_err());
        assert!(parse(&json!({"bpm": 900})).is_err());
        assert!(parse(&json!({"bars": "eight"})).is_err());
        assert!(parse(&json!({"key": "H major"})).is_err());
        assert!(parse(&json!({"progression": "i-XI"})).is_err());
        assert!(generate(&json!({"bass_style": "dubstep"}), "bass").is_err());
    }

    #[test]
    fn sketch_and_export() {
        let input = json!({"genre": "edm", "seed": 9, "parts": ["melody", "drums"]});
        let s = sketch(&input).unwrap();
        assert_eq!(s["parts"].as_array().unwrap().len(), 2);
        assert_eq!(s["all_checks_passed"], true);
        let (a, title) = export(&input).unwrap();
        let (b, _) = export(&input).unwrap();
        assert_eq!(a, b, "same input, same file");
        assert!(title.contains("D major"));
        assert!(export(&json!({"parts": []})).is_err());
        // every genre preset is usable as is
        for g in options()["genres"].as_array().unwrap() {
            assert_eq!(sketch(&json!({"genre": g["id"]})).unwrap()["all_checks_passed"], true, "{}", g["id"]);
        }
    }

    #[test]
    fn base64_matches_rfc4648() {
        for (i, o) in [("", ""), ("f", "Zg=="), ("fo", "Zm8="), ("foo", "Zm9v"), ("foobar", "Zm9vYmFy")] {
            assert_eq!(base64(i.as_bytes()), o);
        }
    }
}
