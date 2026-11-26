//! Tool `music.arrangement`: per-genre arrangement templates.
//! Source: docs/knowledge/music-production/arrangement.md §6. Those templates are COSOUND starting
//! points built from practitioner sources: conventions, not measurements of real tracks. Every
//! output says so (basis "convention").
use serde_json::{json, Value};

pub const SOURCE: &str = "docs/knowledge/music-production/arrangement.md §6";

pub struct Template {
    pub genre: &'static str,
    pub label: &'static str,
    pub bpm: f64,
    /// (first bar, last bar, kind, name, content)
    pub sections: &'static [(u32, u32, &'static str, &'static str, &'static str)],
    pub note: &'static str,
}

pub const TEMPLATES: [Template; 9] = [
    Template { genre: "techno", label: "Techno", bpm: 132.0, note: "Add or remove something about every 16 bars.", sections: &[
        (1, 16, "intro", "Intro", "Kick + percussion + atmosphere, no bass"),
        (17, 32, "intro", "Intro 2", "+ hats, rumble"),
        (33, 64, "groove", "Construction", "+ rolling bass, open hats, main sequence"),
        (65, 80, "break", "Mini-break", "Kick out or filter-down 8–16 bars"),
        (81, 112, "peak", "Peak 1", "Full groove, acid/lead automation"),
        (113, 128, "breakdown", "Breakdown", "Kick/bass out, atmosphere and sequence"),
        (129, 144, "build", "Build/drop", "Filter/riser into full return"),
        (145, 192, "peak", "Peak 2", "Full groove + variation every 8/16 bars"),
        (193, 224, "outro", "Outro", "Deconstruct to percussion"),
    ]},
    Template { genre: "melodic_techno", label: "Melodic Techno", bpm: 122.0, note: "Melodic house uses the same skeleton.", sections: &[
        (1, 16, "intro", "Intro", "Kick, hats, perc, pads"),
        (17, 32, "groove", "Groove", "Full drums, bass, bass accent"),
        (33, 48, "theme", "Theme", "Lead melody enters"),
        (49, 64, "variation", "Variation", "Remove shaker/rim (energy dip)"),
        (65, 96, "peak", "Peak 1", "Full arrangement, chord layers"),
        (97, 128, "breakdown", "Breakdown", "Drums out, cinematic chords + lead, long reverbs"),
        (129, 144, "build", "Build", "Riser, filter, snare/perc roll"),
        (145, 192, "drop", "Drop / Peak 2", "Full + new layer"),
        (193, 224, "outro", "Outro", "Strip back to drums"),
    ]},
    Template { genre: "progressive_house", label: "Progressive House", bpm: 126.0, note: "Long builds are typical.", sections: &[
        (1, 32, "intro", "Percussive intro", "Percussion-led DJ intro"),
        (33, 64, "groove", "Construction", "Bass, chords filtered in"),
        (65, 80, "break", "Short break", "Energy dip"),
        (81, 112, "peak", "Full beat", "Full groove"),
        (113, 144, "breakdown", "Main breakdown", "Main breakdown into a long main build-up"),
        (145, 176, "drop", "Main drop", "Full arrangement"),
        (177, 192, "build", "Build-up", "Second build"),
        (193, 224, "drop", "Drop 2", "Full arrangement"),
        (225, 256, "outro", "Deconstruction / outro", "Strip back for the DJ"),
    ]},
    Template { genre: "deep_house", label: "Deep House", bpm: 124.0, note: "Recombine the same elements later in the track.", sections: &[
        (1, 8, "intro", "Intro", "Beats + chords"),
        (9, 16, "intro", "Intro 2", "+ organ"),
        (17, 32, "theme", "Theme", "+ lead 1"),
        (33, 64, "groove", "Groove / drop", "Bass in, FX hit, lead 2 enters after 8 bars"),
        (65, 96, "peak", "Peak 1", "Recombined elements"),
        (97, 112, "breakdown", "Breakdown", "Pads, vocal, no kick"),
        (113, 160, "peak", "Peak 2", "Full"),
        (161, 192, "outro", "Outro", "Drums + chords out"),
    ]},
    Template { genre: "tech_house", label: "Tech House", bpm: 126.0, note: "Elements change every 8 bars.", sections: &[
        (1, 32, "intro", "DJ intro", "Drums, perc loops"),
        (33, 64, "groove", "Groove", "Bass in, elements switch every 8 bars"),
        (65, 80, "breakdown", "Beatless breakdown", "Vocal hook / stab"),
        (81, 96, "build", "Build", "Riser, snare roll"),
        (97, 144, "drop", "Drop", "Full groove, hook"),
        (145, 160, "break", "Short break", "Energy dip"),
        (161, 192, "outro", "DJ outro", "Drums only"),
    ]},
    Template { genre: "afro_house", label: "Afro House", bpm: 120.0, note: "Percussion layering carries the energy.", sections: &[
        (1, 32, "intro", "Intro", "Kick + shuffling tops + percussion layering"),
        (33, 64, "groove", "Groove", "Sub, keys groove"),
        (65, 96, "theme", "Theme", "Lead line / vocal"),
        (97, 128, "breakdown", "Breakdown", "Pads, percussion only"),
        (129, 176, "peak", "Peak", "All elements"),
        (177, 224, "outro", "Outro", "Percussion"),
    ]},
    Template { genre: "trance", label: "Trance", bpm: 138.0, note: "Usually no EDM-style drop; at least one percussion-free breakdown.", sections: &[
        (1, 32, "intro", "Intro", "Elements introduced gradually"),
        (33, 64, "build", "Build 1", "Long build"),
        (65, 96, "groove", "Main groove", "Bass + lead teaser"),
        (97, 128, "breakdown", "Breakdown", "Percussion out, pads, vocal"),
        (129, 160, "build", "Build 2", "Long build with uplifters"),
        (161, 208, "peak", "Climax", "Full lead + bass"),
        (209, 256, "outro", "Outro", "Gradual removal"),
    ]},
    Template { genre: "minimal", label: "Minimal", bpm: 127.0, note: "Few hard sections: micro-variations every 8–16 bars.", sections: &[
        (1, 32, "intro", "Intro", "Kick, clicks"),
        (33, 96, "groove", "Gradual layering", "Layers added one by one"),
        (97, 128, "break", "Subtractive break", "No kick or filtered"),
        (129, 192, "peak", "Re-entry", "Re-entry with variations"),
        (193, 224, "outro", "Outro", "Strip back"),
    ]},
    Template { genre: "edm", label: "EDM (big room / electro / future house)", bpm: 128.0, note: "Club version; radio edits are common. One build is often much longer.", sections: &[
        (1, 16, "intro", "DJ intro", "DJ-friendly intro"),
        (17, 32, "verse", "Verse / breakdown 1", "Lower energy, theme presented"),
        (33, 48, "build", "Build 1", "Snare roll, riser, pitch up"),
        (49, 80, "drop", "Drop 1", "Full drop"),
        (81, 96, "breakdown", "Breakdown 2", "Energy dip"),
        (97, 112, "build", "Build 2", "Longer build"),
        (113, 144, "drop", "Drop 2", "Full drop"),
        (145, 160, "outro", "DJ outro", "DJ-friendly outro"),
    ]},
];

/// Genres without a template of their own that map to a documented one.
pub fn template(genre: &str) -> Option<(&'static Template, Option<&'static str>)> {
    let (id, alias) = match genre {
        "house" => ("deep_house", Some("house uses the deep house template (closest documented one)")),
        "melodic_house" => ("melodic_techno", Some("melodic house uses the melodic techno template")),
        "progressive_techno" => ("progressive_house", Some("progressive techno uses the progressive house skeleton with a techno palette")),
        g => (g, None),
    };
    TEMPLATES.iter().find(|t| t.genre == id).map(|t| (t, alias))
}

fn energy_and_elements(kind: &str, nth: usize) -> (u8, &'static [&'static str]) {
    match kind {
        "intro" => (3, &["kick", "percussion", "atmosphere"]),
        "groove" => (6, &["kick", "hats", "percussion", "bass"]),
        "theme" => (7, &["kick", "hats", "percussion", "bass", "lead"]),
        "variation" => (6, &["kick", "hats", "bass", "lead"]),
        "verse" => (5, &["kick", "hats", "chords", "lead"]),
        "break" => (4, &["hats", "percussion", "pads"]),
        "breakdown" => (3, &["pads", "chords", "lead", "fx"]),
        "build" => (7, &["percussion", "snare_roll", "riser", "chords"]),
        "drop" => (10, &["kick", "hats", "percussion", "bass", "chords", "lead", "fx"]),
        "peak" => (if nth == 0 { 8 } else { 9 }, &["kick", "hats", "percussion", "bass", "chords", "lead"]),
        _ => (3, &["kick", "percussion"]),
    }
}

/// Section map with energy curve and timing at `bpm` (template BPM when None).
pub fn plan(genre: &str, bpm: Option<f64>) -> Option<Value> {
    let (t, alias) = template(genre)?;
    let bpm = bpm.unwrap_or(t.bpm);
    let bar_s = 240.0 / bpm;
    let mut seen: Vec<&str> = Vec::new();
    let sections: Vec<Value> = t
        .sections
        .iter()
        .map(|&(a, b, kind, name, content)| {
            let nth = seen.iter().filter(|k| **k == kind).count();
            seen.push(kind);
            let (energy, elements) = energy_and_elements(kind, nth);
            json!({
                "name": name, "kind": kind, "start_bar": a, "end_bar": b, "bars": b - a + 1,
                "start_s": ((a - 1) as f64 * bar_s * 10.0).round() / 10.0,
                "energy": energy, "elements": elements, "content": content,
            })
        })
        .collect();
    let total = t.sections.last().map_or(0, |s| s.1);
    let secs = total as f64 * bar_s;
    Some(json!({
        "genre": t.genre, "label": t.label, "bpm": bpm, "template_bpm": t.bpm,
        "total_bars": total, "duration_s": secs.round(), "duration": format!("{}:{:02}", (secs / 60.0) as u32, (secs % 60.0).round() as u32),
        "phrase_rule": "sections start on bar 16n + 1", "note": t.note, "alias": alias,
        "sections": sections, "source": SOURCE,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn templates_are_contiguous_and_phrase_aligned() {
        for t in &TEMPLATES {
            let mut next = 1;
            for s in t.sections {
                assert_eq!(s.0, next, "{} gap before {}", t.genre, s.3);
                assert!(s.1 >= s.0 && (s.1 - s.0 + 1) % 8 == 0, "{} {} is not a multiple of 8 bars", t.genre, s.3);
                next = s.1 + 1;
            }
        }
        let p = plan("melodic_techno", Some(124.0)).unwrap();
        assert_eq!(p["total_bars"], 224);
        assert_eq!(p["duration"], "7:14"); // 224 bars × 240/124 s
        assert_eq!(p["sections"][7]["energy"], 10);
        assert_eq!(plan("house", None).unwrap()["genre"], "deep_house");
        assert!(plan("polka", None).is_none());
    }
}
