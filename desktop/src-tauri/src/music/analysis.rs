//! MIDI Analysis Engine – reads an existing Standard MIDI File and reports: notes and intervals, probable keys (with
//! candidates and guards) and scales, chords per bar and the progression in roman numerals, rhythm (grid, density,
//! syncopation), BPM when the file states it, phrases, repeated and varied bars, and the notes outside the key —
//! classified (chromatic passing/neighbour/approach, chord tone, borrowed, blue note), never called errors.
use super::chords::recognize;
use super::generate::{MidiNote, BAR, PPQ};
use super::harmony;
use super::keys;
use super::notes::Pitch;
use super::rhythm;
use super::validate::classify_out_of_key;
use crate::smartlib::midi as smf_reader;
use serde_json::{json, Value};

/// Notes of a file rescaled to COSOUND's 480 PPQ: (track, channel, note).
pub fn notes_of(bytes: &[u8]) -> Result<(smf_reader::Smf, Vec<(usize, u8, MidiNote)>), String> {
    let s = smf_reader::parse(bytes)?;
    let f = PPQ as f64 / s.ppq.max(1) as f64;
    let v = s
        .notes
        .iter()
        .map(|n| (n.track, n.channel, MidiNote { pitch: n.pitch, start: (n.start as f64 * f).round() as u32, dur: ((n.dur as f64 * f).round() as u32).max(1), vel: n.vel.max(1) }))
        .collect();
    Ok((s, v))
}

/// Chords per bar (and per half bar when the harmony moves faster): pitch classes weighted by sounding duration.
pub fn chords_per_bar(notes: &[MidiNote], bars: u32) -> Vec<Value> {
    let mut out = vec![];
    for b in 0..bars {
        for half in 0..2u32 {
            let (a, z) = (b * BAR + half * BAR / 2, b * BAR + (half + 1) * BAR / 2);
            let sounding: Vec<u8> = notes.iter().filter(|n| n.start < z && n.start + n.dur > a).map(|n| n.pitch).collect();
            if sounding.len() < 2 {
                continue;
            }
            let c = recognize(&sounding);
            if let Some(best) = c.first().filter(|x| x["confidence"].as_f64().unwrap_or(0.0) >= 0.5) {
                let label = best["symbol"].as_str().unwrap_or("").to_string();
                // merge with the previous half when it is the same chord
                if half == 1 && out.last().is_some_and(|l: &Value| l["bar"] == b + 1 && l["symbol"] == label) {
                    continue;
                }
                out.push(json!({ "bar": b + 1, "beat": if half == 0 { 1 } else { 3 }, "symbol": label, "confidence": best["confidence"], "alternatives": c.iter().skip(1).take(2).map(|x| x["symbol"].clone()).collect::<Vec<_>>() }));
            }
        }
    }
    out
}

/// Phrases of a monophonic line: split at rests of at least a beat (or long held notes).
fn phrases(line: &[MidiNote]) -> Vec<Value> {
    let mut out = vec![];
    let mut start = 0usize;
    for i in 1..=line.len() {
        let gap = if i < line.len() { line[i].start.saturating_sub(line[i - 1].start + line[i - 1].dur) } else { u32::MAX };
        if gap >= PPQ || i == line.len() {
            let seg = &line[start..i];
            if !seg.is_empty() {
                out.push(json!({ "from": rhythm::position(seg[0].start), "to": rhythm::position(seg[seg.len() - 1].start + seg[seg.len() - 1].dur), "notes": seg.len(),
                                  "ends_on": Pitch::from_midi(seg[seg.len() - 1].pitch as i32, false).name() }));
            }
            start = i;
        }
    }
    out
}

/// Bars that repeat (exactly, or transposed: same rhythm and intervals) and bars that vary an earlier one.
fn repetitions(line: &[MidiNote], bars: u32) -> Value {
    let sig = |b: u32| -> (Vec<u32>, Vec<i32>, Option<u8>) {
        let v: Vec<&MidiNote> = line.iter().filter(|n| n.start / BAR == b).collect();
        (v.iter().map(|n| n.start % BAR).collect(), v.windows(2).map(|w| w[1].pitch as i32 - w[0].pitch as i32).collect(), v.first().map(|n| n.pitch))
    };
    let mut rep = vec![];
    let mut var = vec![];
    for b in 1..bars {
        let s = sig(b);
        if s.0.is_empty() {
            continue;
        }
        for a in 0..b {
            let r = sig(a);
            if r.0 == s.0 && r.1 == s.1 {
                rep.push(json!({ "bar": b + 1, "repeats_bar": a + 1, "transposed_by": s.2.zip(r.2).map(|(x, y)| x as i32 - y as i32).unwrap_or(0) }));
                break;
            } else if r.0 == s.0 && !s.1.is_empty() && r.1.iter().zip(&s.1).filter(|(x, y)| x.signum() == y.signum()).count() * 2 >= s.1.len() {
                var.push(json!({ "bar": b + 1, "varies_bar": a + 1, "what": "same rhythm, related contour" }));
                break;
            }
        }
    }
    json!({ "repeated_bars": rep, "varied_bars": var })
}

pub fn analyze(bytes: &[u8]) -> Result<Value, String> {
    let (s, all) = notes_of(bytes)?;
    if all.is_empty() {
        return Err("the file has no notes".into());
    }
    let pitched: Vec<MidiNote> = all.iter().filter(|x| x.1 != 9).map(|x| x.2).collect();
    let end = all.iter().map(|x| x.2.start + x.2.dur).max().unwrap_or(BAR);
    let bars = end.div_ceil(BAR).max(1);
    let est = keys::estimate(&pitched.iter().map(|n| (n.pitch, n.dur as f64)).collect::<Vec<_>>());
    let key = est["key"].as_str().and_then(|k| keys::parse_key(k).ok());
    let chords = chords_per_bar(&pitched, bars);
    let symbols: Vec<String> = chords.iter().filter_map(|c| c["symbol"].as_str().map(String::from)).collect();
    let progression = if symbols.len() >= 2 { harmony::analyze(key, &symbols).ok() } else { None };
    // per track
    let mut tracks = vec![];
    let ids: std::collections::BTreeSet<(usize, u8)> = all.iter().map(|x| (x.0, x.1)).collect();
    for (tr, ch) in ids {
        let mut line: Vec<MidiNote> = all.iter().filter(|x| x.0 == tr && x.1 == ch).map(|x| x.2).collect();
        line.sort_by_key(|n| (n.start, n.pitch));
        let poly = line.windows(2).filter(|w| w[0].start + w[0].dur > w[1].start).count() as f64 / line.len().max(1) as f64;
        let mut t = json!({
            "track": tr, "name": s.track_names.get(tr).cloned().unwrap_or_default(), "channel": ch + 1, "notes": line.len(), "drums": ch == 9,
            "range": [Pitch::from_midi(line.iter().map(|n| n.pitch).min().unwrap_or(0) as i32, false).name(), Pitch::from_midi(line.iter().map(|n| n.pitch).max().unwrap_or(0) as i32, false).name()],
            "polyphony": (poly * 100.0).round() / 100.0, "syncopation": rhythm::syncopation(&line),
            "density_notes_per_bar": (line.len() as f64 / bars as f64 * 10.0).round() / 10.0,
        });
        if ch != 9 && poly < 0.3 {
            let iv: Vec<i32> = line.windows(2).map(|w| w[1].pitch as i32 - w[0].pitch as i32).collect();
            let mut hist = std::collections::BTreeMap::new();
            for i in &iv {
                *hist.entry(super::theory::interval_name(i.unsigned_abs() as u8 % 12)).or_insert(0) += 1;
            }
            t["intervals"] = json!(hist);
            t["steps_share"] = json!((iv.iter().filter(|i| i.abs() <= 2).count() as f64 / iv.len().max(1) as f64 * 100.0).round() / 100.0);
            t["phrases"] = json!(phrases(&line));
            t["patterns"] = repetitions(&line, bars);
            if let Some(k) = key {
                let outside: Vec<Value> = (0..line.len())
                    .filter(|i| !k.pcs().contains(&(line[*i].pitch % 12)))
                    .map(|i| json!({ "at": rhythm::position(line[i].start), "note": Pitch::from_midi(line[i].pitch as i32, k.signature() < 0).name(), "reason": classify_out_of_key(&line, i, k, None) }))
                    .collect();
                t["outside_key"] = json!(outside);
            }
        }
        tracks.push(t);
    }
    let grid = if pitched.iter().all(|n| n.start % (PPQ / 4) == 0) { "1/16" } else if pitched.iter().all(|n| n.start % (PPQ / 3) == 0 || n.start % (PPQ / 4) == 0) { "1/16 + triplets" } else { "off-grid (played or humanized)" };
    Ok(json!({
        "format": s.format, "ppq_in_file": s.ppq, "tracks_in_file": s.tracks, "bars": bars,
        "bpm": s.tempos.first().map(|t| ((60_000_000.0 / t.1 as f64) * 100.0).round() / 100.0),
        "bpm_source": if s.tempos.is_empty() { "not stated in the file" } else { "tempo event in the file" },
        "time_signature": s.time_signature.map(|t| format!("{}/{}", t.0, t.1)),
        "key_signature_in_file": s.key_signature.map(|k| json!({ "sharps_or_flats": k.0, "minor": k.1 })),
        "key": est, "chords": chords, "progression": progression,
        "rhythm": { "grid": grid, "syncopation": rhythm::syncopation(&pitched) },
        "tracks": tracks,
        "note": "keys, chords and phrases are estimates from the notes; notes outside the key are described, not judged",
    }))
}

/// Analyse notes given directly (not a file): same report on one track.
pub fn analyze_notes(notes: &[MidiNote], bpm: Option<f64>) -> Result<Value, String> {
    let t = super::smf::Track { name: "Input".into(), channel: 0, program: None, notes: notes.to_vec(), cc: vec![] };
    let key = super::theory::Key::parse("C major").unwrap();
    let bytes = super::smf::write_tracks(&[t], bpm.unwrap_or(120.0), key, "analysis", (4, 4));
    let mut v = analyze(&bytes)?;
    if bpm.is_none() {
        v["bpm"] = Value::Null;
        v["bpm_source"] = json!("not given");
    }
    v["key_signature_in_file"] = Value::Null;
    Ok(v)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::music::compose::{compose, midi};
    use crate::music::request::Request;
    use crate::music::styles::Styles;

    #[test]
    fn analyses_a_generated_file_and_finds_what_was_written() {
        let styles = Styles::load(None);
        let req = Request { parts: vec!["chords".into(), "melody".into(), "bass".into()], styles: vec![("melodic_techno".into(), 1.0)], key: Some("A minor".into()), bpm: Some(124.0), bars: Some(8), ..Default::default() };
        let c = compose(&req, &styles, None, 1).unwrap();
        let a = analyze(&midi(&c, "test")).unwrap();
        assert_eq!(a["bpm"], 124.0);
        assert_eq!(a["bpm_source"], "tempo event in the file");
        assert!(a["key"]["candidates"].as_array().unwrap().iter().take(2).any(|k| k["key"] == "A minor" || k["key"] == "C major"), "{}", a["key"]);
        // the chords found are the chords written
        let written: Vec<String> = c.timeline.iter().map(|s| s.chord.symbol()).collect();
        let found: Vec<String> = a["chords"].as_array().unwrap().iter().map(|x| x["symbol"].as_str().unwrap().split('/').next().unwrap().to_string()).collect();
        let hits = written.iter().filter(|w| found.contains(w)).count();
        assert!(hits * 10 >= written.len() * 7, "written {written:?} found {found:?}");
        let mel = a["tracks"].as_array().unwrap().iter().find(|t| t["name"] == "Melody").unwrap();
        assert!(mel["phrases"].as_array().unwrap().len() >= 1);
        assert!(mel["patterns"]["repeated_bars"].is_array());
        assert!(analyze(b"nope").is_err());
    }

    #[test]
    fn outside_notes_are_described_not_judged() {
        let n = |p: u8, s: u32| MidiNote { pitch: p, start: s, dur: 100, vel: 100 };
        let line = vec![n(60, 0), n(62, 120), n(63, 240), n(64, 360), n(65, 480), n(67, 600), n(69, 720), n(71, 840), n(72, 960), n(67, 1080), n(64, 1200), n(60, 1320)];
        let a = analyze_notes(&line, None).unwrap();
        let out = &a["tracks"][0]["outside_key"];
        assert_eq!(out[0]["reason"], "chromatic_passing_tone", "{a}");
        assert!(a["bpm"].is_null());
    }
}
