//! H. Smart Library: search, stats, probable duplicates and drives of the authorized libraries.
//! Read-only on the user's files: never moves, edits or deletes them.
use super::{s, unknown, Agent, AgentError, Ctx, OpResult, Outcome};
use crate::library;
use crate::music::theory::Key;
use serde_json::{json, Value};

pub struct SmartLibrary;

fn fail(e: impl ToString) -> AgentError {
    AgentError::new("EXECUTION_FAILED", e.to_string())
}

fn same_key(a: &str, b: &str) -> bool {
    match (Key::parse(a), Key::parse(b)) {
        (Ok(x), Ok(y)) => x.tonic.pc() == y.tonic.pc() && x.mode.minor_third() == y.mode.minor_third(),
        _ => false,
    }
}

impl Agent for SmartLibrary {
    fn id(&self) -> &'static str {
        "smart_library"
    }
    fn operations(&self) -> &'static [&'static str] {
        &["search", "stats", "find_duplicates", "volumes"]
    }
    fn run(&self, op: &str, input: &Value, cx: &mut Ctx) -> OpResult {
        cx.tool("library.index");
        let c = cx.db.lock().unwrap();
        let no_roots = library::roots(&c).map_err(fail)?.is_empty();
        drop(c);
        if no_roots {
            cx.limit("no folder is authorized yet: the user adds folders in Smart Library");
        }
        match op {
            "search" => {
                cx.tool("library.filename_meta");
                let limit = input["limit"].as_u64().unwrap_or(50).clamp(1, 500) as usize;
                let (lo, hi) = (input["bpm_min"].as_f64(), input["bpm_max"].as_f64());
                let key = s(input, "key");
                let filtered = lo.is_some() || hi.is_some() || key.is_some();
                let hits = library::search(&cx.db.lock().unwrap(), s(input, "query").unwrap_or(""), s(input, "kind"), if filtered { 5000 } else { limit as i64 }).map_err(fail)?;
                let mut out = Vec::new();
                let mut offline = 0;
                for h in hits {
                    let (bpm, k) = library::filename_meta(&h.name);
                    if lo.is_some_and(|l| bpm.is_none_or(|b| (b as f64) < l)) || hi.is_some_and(|x| bpm.is_none_or(|b| (b as f64) > x)) {
                        continue;
                    }
                    if key.is_some_and(|q| k.as_deref().is_none_or(|k| !same_key(q, k))) {
                        continue;
                    }
                    offline += h.path.is_none() as usize;
                    out.push(json!({ "file": h, "bpm_from_name": bpm, "key_from_name": k }));
                    if out.len() == limit {
                        break;
                    }
                }
                if offline > 0 {
                    cx.limit(format!("{offline} result(s) are on drives that are not connected: path is null until the drive is plugged in"));
                }
                if filtered {
                    cx.limit("BPM and key filters use values written in file names, not measurements");
                }
                Ok(Outcome::new(json!({ "count": out.len(), "hits": out }), "file_name"))
            }
            "stats" => Ok(Outcome::new(library::stats(&cx.db.lock().unwrap()).map_err(fail)?, "measured")),
            "find_duplicates" => {
                let groups = library::duplicates(&cx.db.lock().unwrap(), input["limit"].as_u64().unwrap_or(50).clamp(1, 500) as i64).map_err(fail)?;
                cx.limit("probable duplicates: same name and size, contents not compared; nothing is deleted");
                Ok(Outcome::new(json!({ "groups": groups.len(), "duplicates": groups }), "estimated"))
            }
            "volumes" => {
                let disks = library::current_disks();
                let c = cx.db.lock().unwrap();
                library::refresh_volumes(&c, &disks).map_err(fail)?;
                Ok(Outcome::new(json!({ "volumes": library::volumes(&c, &disks).map_err(fail)?, "roots": library::roots(&c).map_err(fail)? }), "measured"))
            }
            _ => Err(unknown(self.id(), op)),
        }
    }
}
