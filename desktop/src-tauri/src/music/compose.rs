//! Composition Engine – from a request (words or structure) to a coherent, validated multi-part composition:
//! style (or a blend) → key → progression (with alternatives) → chords (voice-led), bass, melody, arpeggio, drums
//! → validation → variations → Standard MIDI File. Deterministic for a seed. The user's preferences (shared
//! memory, when allowed) only fill in what the request leaves open: they never override it.
use super::chords::lead;
use super::generate::{self as g, MidiNote, BAR, PPQ};
use super::harmony::{self, Timeline};
use super::request::Request;
use super::smf::Track;
use super::styles::{main_mode, Styles};
use super::theory::{self as t, Key};
use super::{arp, bass, melody, validate};
use serde_json::{json, Value};

pub struct Prefs {
    pub key: Option<String>,
    pub bpm: Option<f64>,
    pub style: Option<String>,
}

pub struct Composition {
    pub key: Key,
    pub bpm: f64,
    pub bars: u32,
    pub timeline: Timeline,
    pub tracks: Vec<Track>,
    pub report: Value,
}

fn chord_rhythm(style_id: &str) -> &'static str {
    match style_id.split('+').next().unwrap_or("") {
        "house" | "deep_house" | "tech_house" | "techno" | "minimal" => "stabs",
        "edm" | "trance" => "pulse",
        _ => "sustain",
    }
}

/// Chord part with voice leading (the next chord's inversion closest to the previous voicing).
pub fn chord_part(tl: &Timeline, rhythm: &str, seed: u64) -> Vec<MidiNote> {
    let mut rng = g::Rng::new(seed.wrapping_add(11));
    let mut out = vec![];
    let mut prev: Option<Vec<u8>> = None;
    for s in tl {
        let v = match &prev {
            Some(p) => lead(p, &s.chord),
            None => s.chord.voicing(0, 57),
        };
        let v: Vec<u8> = v.into_iter().filter(|p| (48..=84).contains(p)).collect();
        let hits: Vec<(u32, u32, u8)> = match rhythm {
            "stabs" => (0..s.len / PPQ).map(|b| (b * PPQ + PPQ / 2, PPQ / 4, 100)).collect(),
            "pulse" => (0..s.len / (PPQ / 2)).map(|e| (e * PPQ / 2, PPQ / 2 - 30, if e % 2 == 0 { 96 } else { 84 })).collect(),
            _ => vec![(0, s.len.saturating_sub(10).max(10), 80)],
        };
        for (off, d, vel) in hits {
            for &p in &v {
                out.push(MidiNote { pitch: p, start: s.start + off, dur: d.min(s.len - off), vel: rng.jitter(vel, 4) });
            }
        }
        prev = Some(v);
    }
    out
}

pub fn compose(req: &Request, styles: &Styles, prefs: Option<&Prefs>, seed: u64) -> Result<Composition, String> {
    let mut notes_used = vec![];
    // style: requested, else remembered, else melodic techno
    let parts_style: Vec<(String, f64)> = if !req.styles.is_empty() {
        req.styles.clone()
    } else if let Some(s) = prefs.and_then(|p| p.style.clone()).filter(|s| styles.get(s).is_some()) {
        notes_used.push(format!("style {s} from your preferences (memory)"));
        vec![(s, 1.0)]
    } else {
        vec![("melodic_techno".into(), 1.0)]
    };
    let style = styles.blend(&parts_style)?;
    let style_id = style["id"].as_str().unwrap_or("melodic_techno").to_string();
    let flavor_id = style["harmony"]["flavor"].as_str().unwrap_or("melodic_techno").to_string();
    let mut moods = req.moods.clone();
    for f in style["harmony"]["secondary_flavors"].as_array().into_iter().flatten().filter_map(|x| x.as_str()) {
        moods.push(f.to_string()); // secondary styles colour the harmony
    }
    // key: requested > remembered > style default
    let key = match &req.key {
        Some(k) => super::keys::parse_key(k)?,
        None => match prefs.and_then(|p| p.key.clone()).and_then(|k| super::keys::parse_key(&k).ok()) {
            Some(k) => {
                notes_used.push(format!("key {} from your preferences (memory)", k.name()));
                k
            }
            None => {
                let d = style["key_default"].as_str().unwrap_or("A minor");
                let k = super::keys::parse_key(d)?;
                let m = main_mode(&style);
                Key { tonic: k.tonic, mode: t::Mode::parse(&m).unwrap_or(k.mode) }
            }
        },
    };
    let bpm = req.bpm.or_else(|| prefs.and_then(|p| p.bpm)).or(style["bpm"]["default"].as_f64()).unwrap_or(124.0).clamp(40.0, 250.0);
    let bars = req.bars.unwrap_or(8).clamp(1, 64);
    // progression: generated alternatives (modal keys use their characteristic progression first)
    let alternatives = harmony::generate(key, &flavor_id, &moods, None, 4, seed);
    let chosen: Vec<String> = match harmony::modal(key) {
        Some(m) => t::progression(key, m, false)?.iter().map(|c| c.symbol()).collect(),
        None => alternatives.first().and_then(|a| a["chords"].as_array()).map(|v| v.iter().filter_map(|x| x.as_str().map(String::from)).collect()).ok_or("no progression could be built")?,
    };
    let bpc = style["harmony"]["bars_per_chord"].as_f64().unwrap_or(1.0).clamp(0.5, 4.0);
    let tl = harmony::timeline(key, &chosen, bpc, bars)?;
    let length = bars * BAR;
    let want = |p: &str| req.parts.iter().any(|x| x == p || x == "all");
    let mut tracks: Vec<Track> = vec![];
    let mut analyses = serde_json::Map::new();
    if want("chords") || want("progression") {
        let mut tr = Track { name: "Chords".into(), channel: 0, program: Some(89), notes: chord_part(&tl, chord_rhythm(&style_id), seed), cc: vec![] };
        if bars >= 8 && ["progressive_house", "melodic_techno", "trance", "cinematic"].contains(&style_id.split('+').next().unwrap_or("")) {
            // automation: CC74 (brightness, mapped to filter cutoff by many synths) opening over the section
            tr.cc = (0..=16).map(|i| (i * length / 16, 74u8, (40 + i * 60 / 16) as u8)).collect();
        }
        tracks.push(tr);
    }
    if want("bass") {
        let bstyle = req.bass_style.clone().unwrap_or_else(|| style["bass"]["style"].as_str().unwrap_or("offbeat").to_string());
        let (n, a) = bass::compose(key, &tl, bars, &bass::Opts { style: bstyle, kick: bass::four_on_floor(bars), avoid_kick: None, seed })?;
        analyses.insert("bass".into(), a);
        tracks.push(Track { name: "Bass".into(), channel: 2, program: Some(38), notes: n, cc: vec![] });
    }
    if want("melody") {
        let mstyle = req.melody_style.clone().unwrap_or_else(|| style["melody"]["style"].as_str().unwrap_or("simple").to_string());
        let reg = style["melody"]["register"].as_array().map(|r| (r[0].as_u64().unwrap_or(60) as u8, r[1].as_u64().unwrap_or(84) as u8)).unwrap_or((60, 84));
        let phrase = style["melody"]["phrase_bars"].as_u64().unwrap_or(4) as u32;
        let (n, a) = melody::compose(key, &tl, bars, &melody::Opts { style: mstyle, register: reg, phrase_bars: phrase.min(bars.max(1)), seed });
        analyses.insert("melody".into(), a);
        tracks.push(Track { name: "Melody".into(), channel: 1, program: Some(81), notes: n.into_iter().map(|l| l.note).collect(), cc: vec![] });
    }
    if want("arp") {
        let pattern = req.arp_pattern.clone().unwrap_or_else(|| style["arp"]["pattern"].as_str().unwrap_or("up").to_string());
        let pattern = if moods.iter().any(|m| m == "hypnotic") && req.arp_pattern.is_none() { "pedal".to_string() } else { pattern };
        let rate = style["arp"]["rate"].as_str().unwrap_or("1/16").to_string();
        let (n, a) = arp::compose(&tl, bars, &arp::Opts { pattern, rate, octaves: 2, gate: 0.55, low: 57, accent_every: 4, curve: "flat".into(), seed })?;
        analyses.insert("arp".into(), a);
        tracks.push(Track { name: "Arp".into(), channel: 3, program: Some(80), notes: n, cc: vec![] });
    }
    if req.parts.iter().any(|p| p == "drums" || p == "all") {
        if let Some(d) = style["drums"]["style"].as_str() {
            let tonic = [t::diatonic(key, 0, false)];
            let spec = g::Spec { key, chords: &tonic, bars, seed, swing: style["swing"].as_u64().unwrap_or(0) as u32 };
            let p = g::drums(&spec, d, bars >= 4)?;
            tracks.push(Track { name: "Drums".into(), channel: 9, program: None, notes: p.notes, cc: vec![] });
        } else {
            notes_used.push(format!("{} has no drum pattern by default", style["label"].as_str().unwrap_or("")));
        }
    }
    let dissonance_ok = moods.iter().any(|m| ["dark", "tense", "aggressive"].contains(&m.as_str()));
    let validation = validate::check(&tracks, &validate::Context { key, tl: Some(&tl), length, dissonance_ok });
    let progression = harmony::analyze(Some(key), &chosen)?;
    let report = json!({
        "key": key.name(), "nome": super::keys::key_name_it(key), "bpm": bpm, "bars": bars, "seed": seed,
        "style": { "id": style_id, "label": style["label"], "blended_from": style["blended_from"], "sources": style["sources"] },
        "moods": moods, "progression": progression,
        "alternative_progressions": alternatives.iter().skip(1).cloned().collect::<Vec<_>>(),
        "parts": tracks.iter().map(|t| json!({ "name": t.name, "channel": t.channel + 1, "program": t.program, "notes": t.notes.len(), "cc_events": t.cc.len() })).collect::<Vec<_>>(),
        "analysis": analyses, "validation": validation,
        "variations": { "melody": melody::VARIATIONS, "other": ["another seed", "an alternative progression", "another melody style"] },
        "choices": notes_used,
        "harmonic_rhythm_bars_per_chord": bpc,
    });
    Ok(Composition { key, bpm, bars, timeline: tl, tracks, report })
}

#[cfg(test)]
pub fn midi(c: &Composition, title: &str) -> Vec<u8> {
    super::smf::write_tracks(&c.tracks, c.bpm, c.key, title, (4, 4))
}

/// Notes of a track as JSON for the UI (start/dur in ticks, plus musical position).
pub fn tracks_view(tracks: &[Track]) -> Value {
    json!(tracks.iter().map(|t| json!({ "name": t.name, "channel": t.channel, "program": t.program, "cc": t.cc,
        "notes": t.notes.iter().map(|n| json!({ "pitch": n.pitch, "start": n.start, "dur": n.dur, "vel": n.vel })).collect::<Vec<_>>() })).collect::<Vec<_>>())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::music::request::parse;

    #[test]
    fn the_prompt_example_end_to_end() {
        let styles = Styles::load(None);
        let req = parse("Crea una melodia emozionale e ipnotica in tonalità di La minore, adatta a una produzione Melodic Techno a 124 BPM.");
        let c = compose(&req, &styles, None, 1).unwrap();
        assert_eq!((c.key.name(), c.bpm), ("A minor".to_string(), 124.0));
        assert_eq!(c.tracks.len(), 1, "a melody was asked for");
        assert_eq!(c.report["validation"]["passed"], true, "{}", c.report["validation"]);
        assert!(c.report["progression"]["chords"].as_array().unwrap().len() >= 2);
        assert!(c.report["alternative_progressions"].as_array().unwrap().len() >= 2, "alternatives proposed");
        let bytes = midi(&c, "COSOUND test");
        let (format, ppq, tracks, tempo) = crate::music::smf::read(&bytes).unwrap();
        assert_eq!((format, ppq, tracks.len()), (1, 480, 2), "format 1: conductor + melody");
        assert_eq!(tempo, Some(483_871), "124 BPM");
        assert!(tracks[1].iter().filter(|e| e.1 & 0xF0 == 0x90).count() > 8, "melody notes in the file");
    }

    #[test]
    fn every_style_full_composition_validates() {
        let styles = Styles::load(None);
        for id in styles.ids() {
            for seed in 0..4 {
                let req = Request { parts: vec!["all".into()], styles: vec![(id.clone(), 1.0)], bars: Some(8), ..Default::default() };
                let c = compose(&req, &styles, None, seed).unwrap();
                assert_eq!(c.report["validation"]["technical_errors"], 0, "{id} {seed}: {}", c.report["validation"]);
                assert!(c.tracks.iter().any(|t| t.name == "Melody") && c.tracks.iter().any(|t| t.name == "Bass") && c.tracks.iter().any(|t| t.name == "Chords"));
                let smf = midi(&c, "x");
                assert!(crate::smartlib::midi::parse(&smf).is_ok(), "{id}: the file reads back");
                let (_, _, evs, _) = crate::music::smf::read(&smf).unwrap();
                let ons: usize = evs.iter().map(|t| t.iter().filter(|e| e.1 & 0xF0 == 0x90).count()).sum();
                assert_eq!(ons, c.tracks.iter().map(|t| t.notes.len()).sum::<usize>(), "{id}: every note is in the file");
            }
        }
        // a blend and memory preferences that only fill gaps
        let req = parse("brano techno con elementi di ambient");
        let c = compose(&req, &styles, Some(&Prefs { key: Some("E minor".into()), bpm: Some(129.0), style: Some("house".into()) }), 2).unwrap();
        assert_eq!(c.report["style"]["id"], "techno+ambient", "the request wins over the remembered style");
        assert_eq!(c.key.name(), "E minor", "the remembered key fills the gap");
        assert!(c.report["choices"].as_array().unwrap().iter().any(|x| x.as_str().unwrap().contains("memory")));
    }
}
