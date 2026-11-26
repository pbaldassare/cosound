//! D. Music Theory (the Music Intelligence agent): a thin layer over the Music Intelligence Engine (`crate::music`):
//! theory lookups, progressions, composition from words, MIDI editing with versions, MIDI analysis, validation.
use super::{s, unknown, Agent, AgentError, Ctx, OpResult, Outcome};
use crate::music::{api as mapi, caps, styles::Styles, theory};
use serde_json::{json, Value};

pub struct MusicTheory;

impl Agent for MusicTheory {
    fn id(&self) -> &'static str {
        "music_theory"
    }
    fn operations(&self) -> &'static [&'static str] {
        &[
            "key_info", "progression", "generate_chords", "generate_bassline", "generate_melody", "detect_key", "check_coherence",
            "theory", "generate_progressions", "analyze_progression", "substitutions", "compose", "vary_melody", "edit_midi",
            "get_clip", "undo_edit", "compare_versions", "export_midi", "analyze_midi", "validate", "style_profiles",
        ]
    }
    fn run(&self, op: &str, input: &Value, cx: &mut Ctx) -> OpResult {
        cx.tool("music.theory");
        let gen = |part| {
            caps::generate(input, part).and_then(|v| if v["all_checks_passed"] == true { Ok(v) } else { Err("the generated part did not pass the musical checks".into()) })
        };
        if let Some(out) = self.intelligence(op, input, cx) {
            return out;
        }
        let v = match op {
            "key_info" => caps::key_info(input),
            "progression" => caps::progression(input),
            "generate_chords" | "generate_bassline" | "generate_melody" => {
                cx.tool("music.generator");
                cx.tool("music.smf_writer");
                gen(match op {
                    "generate_chords" => "chords",
                    "generate_bassline" => "bass",
                    _ => "melody",
                })
            }
            "check_coherence" => {
                cx.tool("music.generator");
                caps::check_parts(input)
            }
            "detect_key" => {
                cx.tool("music.key_detection");
                let mut w = [0.0f64; 12];
                if let Some(ws) = input["weights"].as_array() {
                    for (i, x) in ws.iter().take(12).enumerate() {
                        w[i] = x.as_f64().unwrap_or(0.0).max(0.0);
                    }
                } else {
                    for n in input["notes"].as_array().into_iter().flatten() {
                        if let Some(p) = n["pitch"].as_u64() {
                            w[(p % 12) as usize] += n["dur"].as_f64().unwrap_or(1.0).max(0.0);
                        }
                    }
                }
                if w.iter().all(|x| *x == 0.0) {
                    return Err(AgentError::input("give notes (pitch, dur) or 12 pitch-class weights"));
                }
                let ranked = theory::detect_key(&w);
                let r2 = |x: f64| (x * 1000.0).round() / 1000.0;
                cx.limit("key finding is an estimate (Krumhansl–Kessler profiles), not a measurement");
                return Ok(Outcome::new(
                    json!({
                        "best": ranked[0].0.name(),
                        "correlation": r2(ranked[0].1),
                        "margin": r2(ranked[0].1 - ranked[1].1),
                        "relative_of_best": ranked[0].0.relative().map(|k| k.name()),
                        "candidates": ranked.iter().take(5).map(|(k, c)| json!({ "key": k.name(), "correlation": r2(*c) })).collect::<Vec<_>>(),
                    }),
                    "estimated",
                ));
            }
            _ => return Err(unknown(self.id(), op)),
        }
        .map_err(AgentError::input)?;
        Ok(Outcome::new(v, "rule_based"))
    }
}

impl MusicTheory {
    /// Prompt 09 operations. None = not one of them (handled by the M2 operations above).
    fn intelligence(&self, op: &str, input: &Value, cx: &mut Ctx) -> Option<OpResult> {
        let err = |e: String| AgentError::input(e);
        let clip = |input: &Value| s(input, "clip_id").map(String::from).ok_or_else(|| AgentError::input("clip_id is required"));
        let out: OpResult = match op {
            "theory" => {
                let what = s(input, "what").unwrap_or("");
                cx.tool(match what {
                    "note" | "interval" => "music.notes",
                    "scale" | "scales" => "music.scales",
                    "key" | "circle" | "modulation" | "estimate_key" => "music.keys",
                    _ => "music.chords",
                });
                let basis = if what == "estimate_key" || what == "recognize_chord" { "estimated" } else { "rule_based" };
                mapi::theory(input).map(|v| Outcome::new(v, basis)).map_err(err)
            }
            "generate_progressions" | "analyze_progression" | "substitutions" => {
                cx.tool("music.harmony");
                let mut i = input.clone();
                i["what"] = json!(match op { "generate_progressions" => "generate", "analyze_progression" => "analyze", _ => "substitutions" });
                mapi::harmony_api(&i).map(|v| Outcome::new(v, "rule_based")).map_err(err)
            }
            "compose" => {
                for t in ["music.composer", "music.styles", "music.harmony", "music.melody", "music.bass", "music.arpeggio", "music.generator", "music.validation"] {
                    cx.tool(t);
                }
                let styles = Styles::load(Some(cx.data_dir));
                let allow = cx.granted("memory.music");
                if allow {
                    cx.tool("memory.facts");
                }
                let r = mapi::compose_api(&cx.db.lock().unwrap(), &styles, input, allow).map_err(err);
                r.map(|v| {
                    if let Some(u) = v["request_understood"]["unknown"].as_array().filter(|a| !a.is_empty()) {
                        cx.limit(format!("words not understood: {}", u.iter().filter_map(|x| x.as_str()).collect::<Vec<_>>().join(", ")));
                    }
                    if v["validation"]["passed"] != true {
                        cx.limit("the composition has technical errors: see validation");
                    }
                    Outcome::new(v, "rule_based")
                })
            }
            "vary_melody" => {
                cx.tool("music.melody");
                cx.tool("music.midi_editor");
                clip(input).and_then(|id| mapi::vary_clip(&cx.db.lock().unwrap(), &id, s(input, "kind").unwrap_or("sequence"), input["seed"].as_u64().unwrap_or(1)).map_err(err)).map(|v| Outcome::new(v, "rule_based"))
            }
            "edit_midi" => {
                cx.tool("music.midi_editor");
                clip(input).and_then(|id| mapi::edit_clip(&cx.db.lock().unwrap(), &id, s(input, "track").unwrap_or(""), s(input, "text").unwrap_or(""), input["seed"].as_u64().unwrap_or(1)).map_err(err)).map(|v| Outcome::new(v, "rule_based"))
            }
            "get_clip" | "undo_edit" | "compare_versions" => {
                cx.tool("music.midi_editor");
                clip(input)
                    .and_then(|id| {
                        let c = cx.db.lock().unwrap();
                        match op {
                            "get_clip" => mapi::clip(&c, &id),
                            "undo_edit" => mapi::undo(&c, &id),
                            _ => mapi::compare(&c, &id, input["from"].as_i64().unwrap_or(1), input["to"].as_i64().unwrap_or(2)),
                        }
                        .map_err(err)
                    })
                    .map(|v| Outcome::new(v, "measured"))
            }
            "export_midi" => {
                cx.tool("music.smf_writer");
                clip(input).and_then(|id| mapi::export(&cx.db.lock().unwrap(), cx.data_dir, &id, input["version"].as_i64()).map_err(err)).map(|v| Outcome::new(v, "measured"))
            }
            "analyze_midi" => {
                cx.tool("music.midi_analysis");
                let r = match s(input, "path") {
                    Some(p) => cx.authorized_file(p).and_then(|f| mapi::analyze_file(&f).map_err(err)),
                    None => mapi::analyze_notes(input).map_err(err),
                };
                r.map(|v| {
                    cx.limit("keys, chords and phrases are estimates from the notes");
                    Outcome::new(v, "estimated")
                })
            }
            "validate" => {
                cx.tool("music.validation");
                (|| -> Result<Value, String> {
                    let tracks: Vec<crate::music::smf::Track> = serde_json::from_value(input["tracks"].clone()).map_err(|e| format!("tracks: {e}"))?;
                    let key = crate::music::keys::parse_key(s(input, "key").ok_or("key is required")?)?;
                    let len = tracks.iter().flat_map(|t| t.notes.iter()).map(|n| n.start + n.dur).max().unwrap_or(0).div_ceil(crate::music::generate::BAR) * crate::music::generate::BAR;
                    let tl = match input["chords"].as_array() {
                        Some(a) => Some(crate::music::harmony::timeline(key, &a.iter().filter_map(|x| x.as_str().map(String::from)).collect::<Vec<_>>(), input["bars_per_chord"].as_f64().unwrap_or(1.0), len / crate::music::generate::BAR)?),
                        None => None,
                    };
                    let moods: Vec<&str> = input["moods"].as_array().map(|a| a.iter().filter_map(|x| x.as_str()).collect()).unwrap_or_default();
                    Ok(crate::music::validate::check(&tracks, &crate::music::validate::Context { key, tl: tl.as_ref(), length: len.max(crate::music::generate::BAR), dissonance_ok: moods.iter().any(|m| ["dark", "tense", "aggressive"].contains(m)) }))
                })()
                .map(|v| Outcome::new(v, "rule_based"))
                .map_err(err)
            }
            "style_profiles" => {
                cx.tool("music.styles");
                let styles = Styles::load(Some(cx.data_dir));
                let blend: Vec<(String, f64)> = input["blend"].as_array().map(|a| a.iter().filter_map(|x| Some((x["style"].as_str()?.to_string(), x["weight"].as_f64().unwrap_or(1.0)))).collect()).unwrap_or_default();
                if blend.is_empty() {
                    Ok(Outcome::new(json!({ "styles": styles.all() }), "convention"))
                } else {
                    styles.blend(&blend).map(|v| Outcome::new(v, "convention")).map_err(err)
                }
            }
            _ => return None,
        };
        Some(out)
    }
}
