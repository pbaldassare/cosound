//! Integration API of the Music Intelligence Engine: one entry point for the app commands and the agents
//! (JSON in → JSON out). The central Agent decides which components to involve; this module only does music.
use super::compose::{self, Composition, Prefs};
use super::generate::MidiNote;
use super::harmony;
use super::smf::Track;
use super::styles::Styles;
use super::{analysis, chords, edit, keys, melody, notes, reference, request, rhythm, scales, voice};
use rusqlite::{params, Connection};
use serde_json::{json, Value};
use std::path::Path;

fn s<'a>(v: &'a Value, k: &str) -> Option<&'a str> {
    v[k].as_str().filter(|x| !x.is_empty())
}

/// Music preferences from the shared memory (categories music/genre/bpm/style), only when the user allows it.
/// They fill gaps in a request and never override it.
pub fn prefs(c: &Connection, allowed: bool) -> Option<Prefs> {
    if !allowed {
        return None;
    }
    let mut p = Prefs { key: None, bpm: None, style: None };
    for cat in ["music", "genre", "bpm", "style"] {
        for f in crate::memory::list(c, Some(cat)).unwrap_or_default() {
            let k = f.key.to_lowercase();
            let val = f.value.get("value").cloned().unwrap_or(f.value.clone());
            if k.contains("key") || k.contains("tonalit") {
                p.key = p.key.or(val.as_str().map(String::from));
            } else if k.contains("bpm") || cat == "bpm" {
                p.bpm = p.bpm.or(val.as_f64().or_else(|| val.as_str().and_then(|x| x.parse().ok())));
            } else if k.contains("genre") || k.contains("style") || cat == "genre" || cat == "style" {
                p.style = p.style.or(val.as_str().map(|x| x.to_lowercase().replace(' ', "_")));
            }
        }
    }
    (p.key.is_some() || p.bpm.is_some() || p.style.is_some()).then_some(p)
}

/// Theory lookups: note, interval, scale, key, chord, circle of fifths, modulation, catalogs.
pub fn theory(input: &Value) -> Result<Value, String> {
    match s(input, "what").unwrap_or("") {
        "note" => notes::note_info(s(input, "note").ok_or("note is required")?, input["flats"].as_bool()),
        "interval" => notes::interval_info(s(input, "from").ok_or("from is required")?, s(input, "to").ok_or("to is required (note or interval like m7)")?),
        "scale" => scales::scale_info(s(input, "tonic").unwrap_or("C"), s(input, "scale").unwrap_or("major")),
        "scales" => Ok(json!(scales::SCALES.iter().map(|d| json!({ "id": d.id, "name": d.en, "nome": d.it, "formula": d.intervals, "family": d.family, "character": d.character, "carattere": d.carattere })).collect::<Vec<_>>())),
        "key" => {
            let k = keys::parse_key(s(input, "key").ok_or("key is required")?)?;
            let mut v = keys::key_view(k);
            v["triads"] = json!((0..7).map(|d| super::theory::diatonic(k, d, false).view()).collect::<Vec<_>>());
            v["sevenths"] = json!((0..7).map(|d| super::theory::diatonic(k, d, true).view()).collect::<Vec<_>>());
            Ok(v)
        }
        "circle" => Ok(json!(keys::CIRCLE_MAJOR.iter().map(|m| { let k = keys::parse_key(m).unwrap(); json!({ "major": k.name(), "minor": k.relative().map(|r| r.name()), "signature": k.signature() }) }).collect::<Vec<_>>())),
        "modulation" => Ok(keys::modulation(keys::parse_key(s(input, "from").ok_or("from")?)?, keys::parse_key(s(input, "to").ok_or("to")?)?)),
        "chord" => Ok(chords::ChordSym::parse(s(input, "chord").ok_or("chord is required")?)?.view()),
        "chord_types" => Ok(json!(chords::TYPES.iter().map(|t| json!({ "id": t.id, "symbol": t.symbol, "intervals": t.intervals, "family": t.family })).collect::<Vec<_>>())),
        "recognize_chord" => {
            let p: Vec<u8> = input["pitches"].as_array().ok_or("pitches is required")?.iter().filter_map(|x| x.as_u64().map(|v| v as u8)).collect();
            Ok(json!({ "candidates": chords::recognize(&p) }))
        }
        "estimate_key" => {
            let n: Vec<(u8, f64)> = input["notes"].as_array().ok_or("notes is required")?.iter().filter_map(|x| Some((x["pitch"].as_u64()? as u8, x["dur"].as_f64().unwrap_or(1.0)))).collect();
            Ok(keys::estimate(&n))
        }
        "options" => Ok(json!({
            "melody_styles": melody::STYLES, "melody_variations": melody::VARIATIONS, "bass_styles": super::bass::STYLES,
            "arp_patterns": super::arp::PATTERNS, "arp_curves": super::arp::CURVES,
            "grooves": rhythm::GROOVES.iter().map(|g| json!({ "style": g.0, "kick": g.1, "hats": g.2, "extra": g.3 })).collect::<Vec<_>>(),
            "edit_examples": ["abbassa la melodia di un'ottava", "rendi il basso più sincopato", "allunga le note degli accordi", "rendi l'arpeggio più ipnotico", "trasforma in tonalità minore", "quantizza a 1/16", "umanizza", "alza di due semitoni"],
        })),
        other => Err(format!("unknown theory request {other:?} (note, interval, scale, scales, key, circle, modulation, chord, chord_types, recognize_chord, estimate_key, options)")),
    }
}

/// Harmony: generate progressions, analyse, substitutions.
pub fn harmony_api(input: &Value) -> Result<Value, String> {
    let key = s(input, "key").map(keys::parse_key).transpose()?;
    match s(input, "what").unwrap_or("generate") {
        "generate" => {
            let k = key.ok_or("key is required")?;
            let moods: Vec<String> = input["moods"].as_array().map(|a| a.iter().filter_map(|x| x.as_str().map(String::from)).collect()).unwrap_or_default();
            Ok(json!({ "key": k.name(), "progressions": harmony::generate(k, s(input, "style").unwrap_or(""), &moods, input["length"].as_u64().map(|x| x as usize), input["count"].as_u64().unwrap_or(5).min(12) as usize, input["seed"].as_u64().unwrap_or(1)) }))
        }
        "analyze" => {
            let ch: Vec<String> = input["chords"].as_array().ok_or("chords is required")?.iter().filter_map(|x| x.as_str().map(String::from)).collect();
            harmony::analyze(key, &ch)
        }
        "substitutions" => harmony::substitutions(key.ok_or("key is required")?, s(input, "chord").ok_or("chord is required")?),
        other => Err(format!("unknown harmony request {other:?} (generate, analyze, substitutions)")),
    }
}

pub fn rhythm_api(input: &Value) -> Result<Value, String> {
    rhythm::pattern(input)
}

/// Compose from words or from a structured request; stores the result as a clip (version 1).
pub fn compose_api(c: &Connection, styles: &Styles, input: &Value, allow_memory: bool) -> Result<Value, String> {
    let mut req = match s(input, "text") {
        Some(t) => request::parse(t),
        None => request::Request { parts: vec!["all".into()], ..Default::default() },
    };
    // structured fields override the words
    if let Some(k) = s(input, "key") {
        req.key = Some(keys::parse_key(k)?.name());
    }
    if let Some(b) = input["bpm"].as_f64() {
        req.bpm = Some(b);
    }
    if let Some(b) = input["bars"].as_u64() {
        req.bars = Some(b as u32);
    }
    if let Some(st) = s(input, "style") {
        req.styles = vec![(st.to_string(), 1.0)];
    }
    if let Some(p) = input["parts"].as_array() {
        req.parts = p.iter().filter_map(|x| x.as_str().map(String::from)).collect();
    }
    if let Some(m) = input["moods"].as_array() {
        req.moods = m.iter().filter_map(|x| x.as_str().map(String::from)).collect();
    }
    for (k, f) in [("melody_style", &mut req.melody_style), ("bass_style", &mut req.bass_style), ("arp_pattern", &mut req.arp_pattern)] {
        if let Some(v) = s(input, k) {
            *f = Some(v.to_string());
        }
    }
    let seed = input["seed"].as_u64().unwrap_or(1);
    let p = prefs(c, allow_memory);
    let comp = compose::compose(&req, styles, p.as_ref(), seed)?;
    let title = s(input, "title").map(String::from).unwrap_or_else(|| format!("{} – {} – {} BPM", comp.report["style"]["label"].as_str().unwrap_or("Sketch"), comp.key.name(), comp.bpm));
    let id = save(c, &comp, &title, input["text"].as_str())?;
    let mut out = comp.report.clone();
    out["clip_id"] = json!(id);
    out["title"] = json!(title);
    out["request_understood"] = json!(req);
    out["tracks"] = compose::tracks_view(&comp.tracks);
    out["timeline"] = timeline_view(&comp.timeline);
    out["length_ticks"] = json!(comp.bars * super::generate::BAR);
    out["ppq"] = json!(super::generate::PPQ);
    out["memory_used"] = json!(p.is_some());
    Ok(out)
}

fn timeline_view(tl: &harmony::Timeline) -> Value {
    json!(tl.iter().map(|s| json!({ "start": s.start, "len": s.len, "chord": s.chord.symbol(), "roman": s.roman })).collect::<Vec<_>>())
}

// ---------- clip store: versions, edits, compare, undo, export ----------

pub fn save(c: &Connection, comp: &Composition, title: &str, req: Option<&str>) -> Result<String, String> {
    let id = uuid::Uuid::new_v4().to_string();
    c.execute(
        "insert into music_clips(id, title, request, key_name, bpm, bars, style, timeline, report) values (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
        params![id, title, req, comp.key.name(), comp.bpm, comp.bars, comp.report["style"]["id"].as_str(), timeline_view(&comp.timeline).to_string(), comp.report.to_string()],
    )
    .map_err(|e| e.to_string())?;
    c.execute("insert into music_versions(clip_id, version, operation, description, key_name, tracks) values (?1, 1, 'compose', ?2, ?3, ?4)", params![id, req, comp.key.name(), json!(comp.tracks).to_string()])
        .map_err(|e| e.to_string())?;
    Ok(id)
}

fn latest(c: &Connection, id: &str) -> Result<(i64, String, Vec<Track>), String> {
    c.query_row("select version, key_name, tracks from music_versions where clip_id = ?1 order by version desc limit 1", [id], |r| {
        Ok((r.get::<_, i64>(0)?, r.get::<_, String>(1)?, r.get::<_, String>(2)?))
    })
    .map_err(|_| "unknown clip".to_string())
    .and_then(|(v, k, t)| Ok((v, k, serde_json::from_str(&t).map_err(|e| e.to_string())?)))
}

fn version(c: &Connection, id: &str, v: i64) -> Result<(String, Vec<Track>), String> {
    let (k, t): (String, String) = c.query_row("select key_name, tracks from music_versions where clip_id = ?1 and version = ?2", params![id, v], |r| Ok((r.get(0)?, r.get(1)?))).map_err(|_| format!("version {v} not found"))?;
    Ok((k, serde_json::from_str(&t).map_err(|e| e.to_string())?))
}

pub fn clip(c: &Connection, id: &str) -> Result<Value, String> {
    let (v, k, tracks) = latest(c, id)?;
    let (title, bpm, bars, tl, report): (String, f64, i64, String, String) = c
        .query_row("select title, bpm, bars, timeline, report from music_clips where id = ?1", [id], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?, r.get(4)?)))
        .map_err(|e| e.to_string())?;
    let history: Vec<Value> = c
        .prepare("select version, operation, description, created_at from music_versions where clip_id = ?1 order by version")
        .unwrap()
        .query_map([id], |r| Ok(json!({ "version": r.get::<_, i64>(0)?, "operation": r.get::<_, String>(1)?, "description": r.get::<_, Option<String>>(2)?, "at": r.get::<_, String>(3)? })))
        .map(|m| m.filter_map(Result::ok).collect())
        .unwrap_or_default();
    Ok(json!({ "clip_id": id, "title": title, "key": k, "bpm": bpm, "bars": bars, "version": v, "tracks": compose::tracks_view(&tracks),
               "timeline": serde_json::from_str::<Value>(&tl).unwrap_or(Value::Null), "report": serde_json::from_str::<Value>(&report).unwrap_or(Value::Null),
               "history": history, "length_ticks": bars as u32 * super::generate::BAR, "ppq": super::generate::PPQ }))
}

pub fn clips(c: &Connection) -> Vec<Value> {
    c.prepare("select id, title, key_name, bpm, bars, created_at, (select max(version) from music_versions v where v.clip_id = m.id) from music_clips m order by updated_at desc limit 100")
        .unwrap()
        .query_map([], |r| Ok(json!({ "clip_id": r.get::<_, String>(0)?, "title": r.get::<_, String>(1)?, "key": r.get::<_, String>(2)?, "bpm": r.get::<_, f64>(3)?, "bars": r.get::<_, i64>(4)?, "created_at": r.get::<_, String>(5)?, "versions": r.get::<_, i64>(6)? })))
        .map(|m| m.filter_map(Result::ok).collect())
        .unwrap_or_default()
}

/// Edit a clip in words (or with structured ops) on one track ("" = all pitched tracks): a new version with a diff.
pub fn edit_clip(c: &Connection, id: &str, track: &str, text: &str, seed: u64) -> Result<Value, String> {
    let (v, kname, tracks) = latest(c, id)?;
    let key = keys::parse_key(&kname)?;
    let (ops, unknown) = edit::parse(text);
    if ops.is_empty() {
        return Err(format!("no edit recognized in {text:?}{}", if unknown.is_empty() { String::new() } else { format!(" (not understood: {})", unknown.join(", ")) }));
    }
    let mut new_key = key;
    let mut diffs = serde_json::Map::new();
    let mut out = tracks.clone();
    let target = track.to_lowercase();
    // a mode change applies to every pitched track, so the parts stay together
    let mode_change = ops.iter().any(|o| matches!(o, edit::Op::ToMode { .. }));
    for t in out.iter_mut() {
        let hit = if mode_change || target.is_empty() { t.channel != 9 } else { t.name.to_lowercase().contains(&target) || t.name.to_lowercase().starts_with(&target[..target.len().min(4)]) };
        if !hit {
            continue;
        }
        let (n, k) = edit::apply(&t.notes, &ops, key, seed)?;
        diffs.insert(t.name.clone(), edit::diff(&t.notes, &n));
        t.notes = n;
        new_key = k;
    }
    if diffs.is_empty() {
        return Err(format!("no track called {track:?} in this clip"));
    }
    c.execute("insert into music_versions(clip_id, version, operation, description, key_name, tracks) values (?1, ?2, 'edit', ?3, ?4, ?5)", params![id, v + 1, text, new_key.name(), json!(out).to_string()]).map_err(|e| e.to_string())?;
    c.execute("update music_clips set updated_at = datetime('now'), key_name = ?2 where id = ?1", params![id, new_key.name()]).map_err(|e| e.to_string())?;
    Ok(json!({ "clip_id": id, "version": v + 1, "previous_version": v, "operations": ops, "not_understood": unknown, "key": new_key.name(), "diff": diffs, "tracks": compose::tracks_view(&out) }))
}

/// A melody variation of a clip that keeps the motif's identity (new version).
pub fn vary_clip(c: &Connection, id: &str, kind: &str, seed: u64) -> Result<Value, String> {
    let (v, kname, tracks) = latest(c, id)?;
    let key = keys::parse_key(&kname)?;
    let tl_json: String = c.query_row("select timeline from music_clips where id = ?1", [id], |r| r.get(0)).map_err(|e| e.to_string())?;
    let tl_v: Value = serde_json::from_str(&tl_json).unwrap_or(json!([]));
    let syms: Vec<String> = tl_v.as_array().into_iter().flatten().filter_map(|x| x["chord"].as_str().map(String::from)).collect();
    let bars = tracks.iter().flat_map(|t| t.notes.iter()).map(|n| n.start + n.dur).max().unwrap_or(super::generate::BAR).div_ceil(super::generate::BAR);
    let bpc = tl_v[0]["len"].as_f64().unwrap_or(super::generate::BAR as f64) / super::generate::BAR as f64;
    let tl = harmony::timeline(key, &syms, bpc, bars)?;
    let mut out = tracks.clone();
    let m = out.iter_mut().find(|t| t.name == "Melody").ok_or("this clip has no melody")?;
    let (n, info) = melody::vary(&m.notes, key, &tl, kind, seed)?;
    m.notes = n;
    c.execute("insert into music_versions(clip_id, version, operation, description, key_name, tracks) values (?1, ?2, 'variation', ?3, ?4, ?5)", params![id, v + 1, kind, kname, json!(out).to_string()]).map_err(|e| e.to_string())?;
    Ok(json!({ "clip_id": id, "version": v + 1, "variation": info, "tracks": compose::tracks_view(&out) }))
}

/// Undo: drop the latest version (version 1 is never removed).
pub fn undo(c: &Connection, id: &str) -> Result<Value, String> {
    let (v, _, _) = latest(c, id)?;
    if v <= 1 {
        return Err("nothing to undo: this is the original version".into());
    }
    c.execute("delete from music_versions where clip_id = ?1 and version = ?2", params![id, v]).map_err(|e| e.to_string())?;
    clip(c, id)
}

pub fn compare(c: &Connection, id: &str, a: i64, b: i64) -> Result<Value, String> {
    let (ka, ta) = version(c, id, a)?;
    let (kb, tb) = version(c, id, b)?;
    let per: Vec<Value> = ta.iter().map(|t| json!({ "track": t.name, "diff": tb.iter().find(|x| x.name == t.name).map(|x| edit::diff(&t.notes, &x.notes)) })).collect();
    Ok(json!({ "clip_id": id, "from": a, "to": b, "key_from": ka, "key_to": kb, "tracks": per, "a": compose::tracks_view(&ta), "b": compose::tracks_view(&tb) }))
}

/// Write the clip (a version, default latest) as a Standard MIDI File into COSOUND's folder.
pub fn export(c: &Connection, data_dir: &Path, id: &str, v: Option<i64>) -> Result<Value, String> {
    let (latest_v, _, _) = latest(c, id)?;
    let ver = v.unwrap_or(latest_v);
    let (k, tracks) = version(c, id, ver)?;
    let (title, bpm): (String, f64) = c.query_row("select title, bpm from music_clips where id = ?1", [id], |r| Ok((r.get(0)?, r.get(1)?))).map_err(|e| e.to_string())?;
    let bytes = super::smf::write_tracks(&tracks, bpm, keys::parse_key(&k)?, &title, (4, 4));
    let dir = data_dir.join("generated").join("music");
    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    let safe: String = title.chars().map(|c| if c.is_alphanumeric() || " -_".contains(c) { c } else { '_' }).collect();
    let mut path = dir.join(format!("{} v{ver}.mid", safe.trim()));
    let mut n = 2;
    while path.exists() {
        path = dir.join(format!("{} v{ver} ({n}).mid", safe.trim()));
        n += 1;
    }
    std::fs::write(&path, &bytes).map_err(|e| e.to_string())?;
    Ok(json!({ "path": path, "bytes": bytes.len(), "version": ver, "tracks": tracks.len(), "format": "Standard MIDI File, format 1, 480 PPQ", "midi_base64": super::caps::base64(&bytes) }))
}

pub fn analyze_file(path: &Path) -> Result<Value, String> {
    let bytes = std::fs::read(path).map_err(|e| e.to_string())?;
    analysis::analyze(&bytes)
}

pub fn analyze_notes(input: &Value) -> Result<Value, String> {
    let n: Vec<MidiNote> = serde_json::from_value(input["notes"].clone()).map_err(|e| format!("notes: {e}"))?;
    analysis::analyze_notes(&n, input["bpm"].as_f64())
}

pub fn voice_api(input: &Value) -> Result<Value, String> {
    let sung: Vec<voice::Sung> = serde_json::from_value(input["notes"].clone()).map_err(|e| format!("notes: {e}"))?;
    voice::to_music(&sung, input["bpm"].as_f64(), s(input, "grid").unwrap_or("1/16"), input["correct"].as_bool().unwrap_or(false))
}

pub fn reference_api(c: &Connection, styles: &Styles, input: &Value) -> Result<Value, String> {
    if input["compose"].as_bool() == Some(true) {
        let parts: Vec<String> = input["parts"].as_array().map(|a| a.iter().filter_map(|x| x.as_str().map(String::from)).collect()).unwrap_or_default();
        let (comp, checks) = reference::compose_inspired(input, styles, &parts, input["seed"].as_u64().unwrap_or(1))?;
        let title = format!("Inspired by {}", input["title"].as_str().unwrap_or("a reference"));
        let id = save(c, &comp, &title, Some("reference"))?;
        let mut out = comp.report.clone();
        out["clip_id"] = json!(id);
        out["originality"] = checks;
        out["tracks"] = compose::tracks_view(&comp.tracks);
        return Ok(out);
    }
    reference::read(input, styles)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compose_edit_compare_undo_export() {
        let c = crate::db::open_memory();
        let styles = Styles::load(None);
        let r = compose_api(&c, &styles, &json!({ "text": "melodia e basso in La minore melodic techno 124 bpm", "seed": 2 }), false).unwrap();
        let id = r["clip_id"].as_str().unwrap().to_string();
        assert_eq!(r["key"], "A minor");
        let e = edit_clip(&c, &id, "melodia", "Abbassa questa melodia di un'ottava", 1).unwrap();
        assert_eq!(e["version"], 2);
        assert_eq!(e["diff"]["Melody"]["average_pitch_change"], -12.0);
        assert!(e["diff"].get("Bass").is_none(), "only the melody changed");
        let b = edit_clip(&c, &id, "basso", "Rendi il basso più sincopato", 1).unwrap();
        assert_eq!(b["version"], 3);
        let cmp = compare(&c, &id, 1, 3).unwrap();
        assert_eq!(cmp["tracks"].as_array().unwrap().len(), 2);
        let m = edit_clip(&c, &id, "", "Trasforma in tonalità maggiore", 1).unwrap();
        assert_eq!(m["key"], "A major");
        let u = undo(&c, &id).unwrap();
        assert_eq!((u["version"].as_i64(), u["key"].as_str()), (Some(3), Some("A minor")), "undo restores the previous version and key");
        undo(&c, &id).unwrap();
        undo(&c, &id).unwrap();
        assert!(undo(&c, &id).is_err(), "the original stays");
        let dir = tempfile::tempdir().unwrap();
        let x = export(&c, dir.path(), &id, None).unwrap();
        assert!(Path::new(x["path"].as_str().unwrap()).exists());
        let a = analyze_file(Path::new(x["path"].as_str().unwrap())).unwrap();
        assert_eq!(a["bpm"], 124.0);
        assert!(edit_clip(&c, &id, "", "fallo come un violino", 1).is_err());
        let v = vary_clip(&c, &id, "sequence", 1).unwrap();
        assert!(v["variation"]["identity_kept"].as_f64().unwrap() > 0.3);
    }

    #[test]
    fn theory_and_harmony_requests() {
        assert_eq!(theory(&json!({ "what": "scale", "tonic": "Fa diesis", "scale": "minore" })).unwrap()["notes"][0], "F#");
        assert_eq!(theory(&json!({ "what": "interval", "from": "C4", "to": "G4" })).unwrap()["short"], "P5");
        assert_eq!(theory(&json!({ "what": "chord", "chord": "Am9" })).unwrap()["notes"], json!(["A", "C", "E", "G", "B"]));
        assert_eq!(theory(&json!({ "what": "recognize_chord", "pitches": [55, 59, 62, 65] })).unwrap()["candidates"][0]["symbol"], "G7");
        assert_eq!(theory(&json!({ "what": "circle" })).unwrap().as_array().unwrap().len(), 15);
        assert!(theory(&json!({ "what": "nope" })).is_err());
        assert_eq!(harmony_api(&json!({ "what": "generate", "key": "A minor", "style": "cinematic", "count": 3 })).unwrap()["progressions"].as_array().unwrap().len(), 3);
        assert_eq!(harmony_api(&json!({ "what": "analyze", "key": "C major", "chords": ["C", "G", "Am", "F"] })).unwrap()["chords"][1]["roman"], "V");
    }

    #[test]
    fn memory_preferences_only_when_allowed() {
        let c = crate::db::open_memory();
        crate::memory::add(&c, "genre", "favourite_genre", &json!("deep house"), "explicit", "custom", 1.0, None).unwrap();
        crate::memory::add(&c, "music", "preferred_key", &json!("D minor"), "explicit", "custom", 1.0, None).unwrap();
        assert!(prefs(&c, false).is_none(), "not used without permission");
        let p = prefs(&c, true).unwrap();
        assert_eq!((p.key.as_deref(), p.style.as_deref()), (Some("D minor"), Some("deep_house")));
        let styles = Styles::load(None);
        let r = compose_api(&c, &styles, &json!({ "text": "un brano" }), true).unwrap();
        assert_eq!((r["key"].as_str(), r["memory_used"].as_bool()), (Some("D minor"), Some(true)));
        let r2 = compose_api(&c, &styles, &json!({ "text": "un brano in Mi minore" }), true).unwrap();
        assert_eq!(r2["key"], "E minor", "the request wins over the memory");
    }
}
