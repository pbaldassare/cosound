//! H. Smart Library Agent: interprets library requests and uses the Smart Library System engine
//! (crate::smartlib) to answer them. The engine scans, indexes, analyzes and searches; the agent
//! chooses the engine call, adds context limits and labels the basis of the answer. Read-only on
//! the user's files.
use super::{s, unknown, Agent, AgentError, Ctx, OpResult, Outcome};
use crate::smartlib::{api, query};
use serde_json::{json, Value};

pub struct SmartLibrary;

fn fail(e: impl ToString) -> AgentError {
    AgentError::new("EXECUTION_FAILED", e.to_string())
}

fn note_results(cx: &mut Ctx, v: &Value) {
    if v["offline_results"].as_u64().unwrap_or(0) > 0 {
        cx.limit(format!("{} result(s) are on drives that are not connected: path is null until the drive is plugged in", v["offline_results"]));
    }
    cx.limit("BPM, key, categories and sound character are estimates: each value carries its source and confidence");
}

impl Agent for SmartLibrary {
    fn id(&self) -> &'static str {
        "smart_library"
    }
    fn operations(&self) -> &'static [&'static str] {
        &["search", "describe_sound", "find_similar", "suggest_loops", "find_presets", "find_midi", "file_info", "find_duplicates", "stats", "volumes", "scan", "analyze_now", "analysis_status"]
    }
    fn run(&self, op: &str, input: &Value, cx: &mut Ctx) -> OpResult {
        cx.tool("library.index");
        let limit = input["limit"].as_u64().unwrap_or(30).clamp(1, 500) as usize;
        let roots = crate::library::roots(&cx.db.lock().unwrap()).map_err(fail)?.len();
        if roots == 0 {
            cx.limit("no folder is authorized yet: the user adds folders in Smart Library");
        }
        match op {
            "search" | "describe_sound" | "find_midi" | "find_presets" => {
                cx.tool("library.search");
                let text = s(input, "text").or_else(|| s(input, "query")).unwrap_or("");
                let v = {
                    let c = cx.db.lock().unwrap();
                    match op {
                        "find_midi" => api::search_text(&c, text, limit, Some("midi")),
                        "find_presets" => {
                            cx.tool("library.preset_reader");
                            let mut q = query::parse(&format!("{text} {}", s(input, "category").unwrap_or("")), &[]);
                            q.kind = Some("preset".into());
                            if let Some(p) = s(input, "plugin") {
                                q.plugin = Some(crate::sound::known_synth(p).map_or_else(|| p.to_lowercase(), |k| k.id.to_string()));
                            }
                            api::search_structured(&c, &q, limit)
                        }
                        _ => {
                            let mut q = query::parse(text, &[]);
                            if let Some(k) = s(input, "kind") {
                                q.kind = Some(k.into());
                            }
                            if let (Some(a), Some(b)) = (input["bpm_min"].as_f64(), input["bpm_max"].as_f64()) {
                                q.bpm_range = Some((a.min(b), a.max(b)));
                            }
                            if let Some(k) = s(input, "key") {
                                q.key = Some(crate::music::theory::Key::parse(k).map_err(AgentError::input)?.name());
                            }
                            if text.is_empty() && q == query::Query::default() {
                                return Err(AgentError::input("give text (a description or keywords)"));
                            }
                            api::search_structured(&c, &q, limit)
                        }
                    }
                };
                if op == "describe_sound" {
                    cx.tool("library.audio_analyzer");
                    cx.limit("sound character is estimated from the audio analysis: a ranking, not a certain match");
                }
                if op == "find_midi" {
                    cx.tool("library.midi_reader");
                }
                if op == "find_presets" && v["count"] == 0 {
                    cx.limit("no matching preset was found in the authorized folders: none is proposed");
                }
                note_results(cx, &v);
                Ok(Outcome::new(v, if op == "describe_sound" { "estimated" } else { "mixed" }))
            }
            "find_similar" => {
                cx.tool("library.search");
                let id = input["file_id"].as_i64().ok_or_else(|| AgentError::input("file_id is required"))?;
                let v = api::similar(&cx.db.lock().unwrap(), id, limit).map_err(|e| AgentError::new("NOT_FOUND", e))?;
                cx.limit("similarity is an estimate of the timbre, not a certain musical match");
                Ok(Outcome::new(v, "estimated"))
            }
            "suggest_loops" => {
                cx.tool("library.search");
                let bpm = input["bpm"].as_f64().ok_or_else(|| AgentError::input("bpm is required"))?;
                let key = match s(input, "key") {
                    Some(k) => Some(crate::music::theory::Key::parse(k).map_err(AgentError::input)?.name()),
                    None => None,
                };
                let v = api::loops(&cx.db.lock().unwrap(), bpm, key.as_deref(), s(input, "category"), limit);
                cx.limit("loops are never modified: the stretch needed is applied in the DAW");
                Ok(Outcome::new(v, "mixed"))
            }
            "file_info" => {
                let ids: Vec<i64> = input["file_ids"].as_array().into_iter().flatten().filter_map(|v| v.as_i64()).collect();
                let c = cx.db.lock().unwrap();
                let files: Vec<Value> = if input["details"].as_bool() == Some(true) {
                    ids.iter().map(|id| api::detail(&c, *id).unwrap_or_else(|e| json!({ "id": id, "error": e }))).collect()
                } else {
                    api::resolve(&c, &ids)
                };
                Ok(Outcome::new(json!({ "files": files }), "measured"))
            }
            "find_duplicates" => {
                cx.tool("library.duplicates");
                let v = api::duplicates(cx.db, &std::sync::atomic::AtomicBool::new(false), input["include_similar"].as_bool().unwrap_or(false));
                cx.limit("nothing is deleted, renamed or moved; similar sounds are not duplicates");
                Ok(Outcome::new(v, "measured"))
            }
            "stats" => Ok(Outcome::new(crate::smartlib::index::stats(&cx.db.lock().unwrap()), "measured")),
            "volumes" => Ok(Outcome::new(api::overview(&cx.db.lock().unwrap()), "measured")),
            "scan" => {
                cx.tool("library.scanner");
                let cancel = std::sync::atomic::AtomicBool::new(cx.cancelled());
                let res = api::scan(cx.db, s(input, "root_id"), &cancel);
                if res.iter().any(|r| r.state == "offline") {
                    cx.limit("folders on drives that are not connected were not scanned; their files are kept, not marked missing");
                }
                Ok(Outcome::new(json!({ "folders": res }), "measured"))
            }
            "analyze_now" => {
                for t in ["library.audio_analyzer", "library.classifier", "library.midi_reader", "library.preset_reader"] {
                    cx.tool(t);
                }
                let cancel = std::sync::atomic::AtomicBool::new(false);
                let mut done = 0;
                while done < limit && !cx.cancelled() {
                    let n = crate::smartlib::analyze::run_batch(cx.db, (limit - done).min(16), &cancel, 0);
                    if n == 0 {
                        break;
                    }
                    done += n;
                }
                if cx.cancelled() {
                    return Err(AgentError::new("CANCELLED", format!("stopped after {done} files")));
                }
                Ok(Outcome::new(json!({ "analyzed": done, "status": api::analysis_status(&cx.db.lock().unwrap()) }), "measured"))
            }
            "analysis_status" => Ok(Outcome::new(api::analysis_status(&cx.db.lock().unwrap()), "measured")),
            _ => Err(unknown(self.id(), op)),
        }
    }
}
