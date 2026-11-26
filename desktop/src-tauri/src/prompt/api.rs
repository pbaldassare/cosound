//! Prompt Engine API: one user turn (text or voice transcript) → a validated `PromptInterpretation` (schema 1.0).
//! Pipeline: sanitize → language → constraints (masked) → parameters → intents → context and references →
//! conversation act → ambiguities (default / recover / ask) → safety → instructions → enhancement → approval →
//! reply → optional model refinement → schema validation → stored turn. Nothing is executed.
use super::constraints::{self, Constraint};
use super::context::{self, Ambiguity, Ctx, ProjectCtx, Target};
use super::enhance::{self, element_name};
use super::extract::{self, p, Param};
use super::intent;
use super::model::{self, ModelAdapter};
use super::profile;
use super::safety;
use super::text::{self, normalize};
use rusqlite::{params, Connection, OptionalExtension};
use serde_json::{json, Value};
use std::sync::OnceLock;

pub const SCHEMA: &str = include_str!("../../../../packages/contracts/prompt-interpretation.schema.json");
pub const MAX_CHARS: usize = 4000;

fn inline(v: &Value, defs: &Value) -> Value {
    match v {
        Value::Object(m) => {
            if let Some(r) = m.get("$ref").and_then(|r| r.as_str()).and_then(|r| r.strip_prefix("#/$defs/")) {
                return inline(&defs[r], defs);
            }
            Value::Object(m.iter().map(|(k, x)| (k.clone(), inline(x, defs))).collect())
        }
        Value::Array(a) => Value::Array(a.iter().map(|x| inline(x, defs)).collect()),
        _ => v.clone(),
    }
}

/// COSOUND's validator reads a null value as missing; a required field that may be null is checked by its type instead.
fn nullable_not_required(v: &mut Value) {
    if let Value::Object(m) = v {
        if let (Some(Value::Array(req)), Some(Value::Object(props))) = (m.get("required").cloned(), m.get("properties").cloned()) {
            let keep: Vec<Value> = req.into_iter().filter(|r| !props.get(r.as_str().unwrap_or("")).and_then(|p| p["type"].as_array()).is_some_and(|t| t.iter().any(|x| x == "null"))).collect();
            m.insert("required".into(), Value::Array(keep));
        }
        for x in m.values_mut() {
            nullable_not_required(x);
        }
    } else if let Value::Array(a) = v {
        a.iter_mut().for_each(nullable_not_required);
    }
}

/// The schema with its local `$ref`s resolved, so the validator checks intents and parameters too.
pub fn schema() -> &'static Value {
    static S: OnceLock<Value> = OnceLock::new();
    S.get_or_init(|| {
        let raw: Value = serde_json::from_str(SCHEMA).expect("prompt-interpretation schema is valid JSON");
        let defs = raw["$defs"].clone();
        let mut s = inline(&raw, &defs);
        // primary_intent: null or one of the intents
        s["properties"]["primary_intent"] = json!({ "type": ["string", "null"] });
        nullable_not_required(&mut s);
        s
    })
}

pub fn validate(v: &Value) -> Result<(), String> {
    crate::agents::schema::validate(schema(), v, "interpretation")?;
    if let Some(p) = v["primary_intent"].as_str() {
        if !schema()["$defs"]["intent"]["enum"].as_array().unwrap().iter().any(|x| x == p) {
            return Err(format!("primary_intent {p} is not an intent"));
        }
    }
    Ok(())
}

fn previous(c: &Connection, session: &str) -> (Option<Value>, Option<Value>, i64) {
    let last = |acts: &str| -> Option<Value> {
        c.query_row(&format!("select interpretation from prompt_turns where session_id = ?1 {acts} order by turn desc limit 1"), [session], |r| r.get::<_, String>(0))
            .optional()
            .ok()
            .flatten()
            .and_then(|s| serde_json::from_str(&s).ok())
    };
    let n: i64 = c.query_row("select count(*) from prompt_turns where session_id = ?1", [session], |r| r.get(0)).unwrap_or(0);
    (last(""), last("and act in ('new_request', 'modification', 'correction')"), n)
}

fn count(c: &Connection, sql: &str) -> i64 {
    c.query_row(sql, [], |r| r.get(0)).unwrap_or(0)
}

fn granted(c: &Connection, key: &str) -> bool {
    crate::db::setting(c, key).ok().flatten().is_some_and(|v| v == "1")
}

fn q(it: bool, i: &str, e: &str) -> String {
    (if it { i } else { e }).to_string()
}

fn issue(code: &str, severity: &str, message: String, suggestion: Option<String>) -> Value {
    json!({ "code": code, "severity": severity, "message": message, "suggestion": suggestion })
}

/// `input`: { text, source?: text|voice, transcript_confidence?, session_id?, user_id?, context?: Ctx,
/// external_content?: [{label, content}] }.
pub fn interpret(c: &Connection, input: &Value, adapter: Option<&dyn ModelAdapter>) -> Result<Value, String> {
    let raw = input["text"].as_str().ok_or("text is required")?;
    let (clean, truncated) = text::sanitize(raw, MAX_CHARS);
    if clean.trim().is_empty() {
        return Err("the request is empty".into());
    }
    let source = if input["source"] == "voice" { "voice" } else { "text" };
    let tconf = input["transcript_confidence"].as_f64().map(|x| x.clamp(0.0, 1.0));
    let session = input["session_id"].as_str().filter(|s| !s.is_empty() && s.len() <= 100).unwrap_or("default").to_string();
    let request_id = crate::db::new_id();
    let prof = profile::load(c);
    let app_lang = crate::db::setting(c, "language").ok().flatten().filter(|l| l == "en" || l == "it").unwrap_or_else(|| "it".into());
    let t = normalize(&clean);
    let lang = text::language(&t, &app_lang);
    let supported = lang == "it" || lang == "en";
    let reply_lang = if prof.language != "auto" { prof.language.clone() } else if supported { lang.clone() } else { app_lang.clone() };
    let it = reply_lang == "it";
    let external = safety::external(input["external_content"].as_array().map(|a| a.as_slice()).unwrap_or(&[]));
    let (last, last_action, turns) = previous(c, &session);
    let prev_targets: Vec<Target> = last_action.as_ref().and_then(|p| serde_json::from_value(p["targets"].clone()).ok()).unwrap_or_default();

    // ---- context: what the caller gives, plus COSOUND's own data
    let mut ctx: Ctx = serde_json::from_value(input["context"].clone()).unwrap_or_default();
    if ctx.project.is_none() {
        if let Some(id) = crate::db::setting(c, "current_project").ok().flatten().filter(|s| !s.is_empty()) {
            if let Ok(pj) = crate::projects::get(c, &id) {
                ctx.project = Some(ProjectCtx { id: Some(pj.id), name: Some(pj.name), daw: pj.daw, bpm: pj.bpm, key: pj.musical_key, genre: None });
            }
        }
    }
    let library_files = ctx.library_files.map(|n| n as i64).unwrap_or_else(|| count(c, "select count(*) from files where missing_since is null"));
    let plugins = ctx.plugins_available.as_ref().map(|p| p.len() as i64).unwrap_or_else(|| count(c, "select count(*) from plugin_products where available = 1"));
    let music_prefs = crate::music::api::prefs(c, granted(c, "perm_music_memory"));
    let workflow_mem = granted(c, "perm_workflow_memory");
    let mem_facts: Vec<crate::memory::Fact> = if workflow_mem { crate::memory::list(c, None).unwrap_or_default() } else { vec![] };
    let preferred_daw = mem_facts.iter().find(|f| f.key.to_lowercase().contains("daw")).and_then(|f| f.value.get("value").cloned().or(Some(f.value.clone()))).and_then(|v| v.as_str().map(String::from));

    // ---- constraints first: their words are masked for the rest of the analysis
    let (mut cons, mask) = constraints::parse(&t);
    let masked = constraints::masked(&t, &mask);
    for pc in last_action.as_ref().and_then(|p| serde_json::from_value::<Vec<Constraint>>(p["constraints"].clone()).ok()).unwrap_or_default() {
        if pc.scope != "request" && !cons.iter().any(|x| x.kind == pc.kind && x.subject == pc.subject) {
            cons.push(Constraint { source: "conversation".into(), ..pc });
        }
    }
    for f in mem_facts.iter().filter(|f| f.category == "instruction") {
        let txt = f.value.get("value").and_then(|v| v.as_str()).or(f.value.as_str()).unwrap_or("");
        for mut mc in constraints::parse(&normalize(txt)).0 {
            if !cons.iter().any(|x| x.kind == mc.kind && x.subject == mc.subject) {
                mc.scope = "permanent".into();
                mc.source = "memory".into();
                cons.push(mc);
            }
        }
    }

    // ---- parameters, intents, references, conversation act
    let x = extract::extract(&t);
    let midi_ops = crate::music::edit::parse(&masked.norm).0.len();
    let elements = context::elements(&masked);
    let intents = if supported { intent::classify(&masked, &x, &elements, midi_ops) } else { vec![] };
    let has = |id: &str| intents.iter().any(|i| i.id == id);
    let creating = has("music_creation") || (intent::creating(&masked) && !intent::comparative(&masked));
    let (mut targets, mut amb) = context::resolve(&masked, &ctx, &prev_targets, creating, &reply_lang);
    let implicit = targets.iter().any(|t| t.resolved_by == "conversation") || (context::deictic(&masked) && elements.is_empty());
    let act = if supported { intent::act(&t, &intents, last.is_some(), implicit) } else { "new_request" };
    let refers_to = match act {
        "new_request" | "question" | "smalltalk" => None,
        _ => last_action.as_ref().or(last.as_ref()).and_then(|p| p["request_id"].as_str().map(String::from)),
    };
    // a follow-up without a new element ("adesso abbassa leggermente il volume") is about the same element
    if matches!(act, "correction" | "modification") && !targets.iter().any(|t| t.kind != "section") && !amb.iter().any(|a| a.kind == "unresolved_reference") {
        for pt in prev_targets.iter().filter(|t| t.kind != "section") {
            targets.push(Target { resolved_by: "conversation".into(), confidence: (pt.confidence * 0.9).min(0.85), ..pt.clone() });
        }
    }
    // a change with nothing to change: ask which element
    let modifying = ["sound_design", "mixing", "midi_editing", "track_editing"].iter().any(|i| has(i)) && !creating;
    if modifying && !targets.iter().any(|t| t.kind != "section" || has("arrangement")) && !amb.iter().any(|a| a.kind == "unresolved_reference") && act != "undo" {
        amb.push(Ambiguity { kind: "unresolved_reference", about: q(it, "non è chiaro quale elemento modificare", "it is not clear which element to change"), severity: "ask", options: ctx.tracks.iter().map(|t| t.name.clone()).collect() });
    }

    let mut explicit: Vec<Param> = x.params.clone();
    let mut inferred: Vec<Param> = vec![];
    let mut defaults: Vec<Value> = vec![];
    let said = |n: &str, e: &[Param]| e.iter().any(|p| p.name == n);
    if matches!(act, "modification" | "correction") {
        for prm in last_action.as_ref().and_then(|p| serde_json::from_value::<Vec<Value>>(p["parameters"]["explicit"].clone()).ok()).unwrap_or_default() {
            let name = prm["name"].as_str().unwrap_or("").to_string();
            let group = prm["group"].as_str().unwrap_or("");
            let carry = group == "general" || (act == "correction" && !explicit.iter().any(|e| e.group == "sound") && group == "sound");
            if carry && !said(&name, &explicit) && !inferred.iter().any(|i| i.name == name) {
                let g: &'static str = if group == "sound" { "sound" } else { "general" };
                inferred.push(p(g, &name, prm["value"].clone(), Some(if it { "dalla richiesta precedente" } else { "from the earlier request" }), "context", 0.8));
            }
        }
    }
    if creating {
        let pj = ctx.project.clone().unwrap_or_default();
        let genre = x.params.iter().find(|q| q.name == "genre").and_then(|q| q.value["id"].as_str().map(String::from));
        if !said("bpm", &explicit) {
            if let Some(b) = pj.bpm {
                inferred.push(p("general", "bpm", json!(b), Some(if it { "BPM del progetto aperto" } else { "BPM of the open project" }), "context", 0.9));
                amb.push(Ambiguity { kind: "missing_recoverable", about: "bpm".into(), severity: "resolved_from_context", options: vec![] });
            } else if let Some(b) = music_prefs.as_ref().and_then(|m| m.bpm) {
                inferred.push(p("general", "bpm", json!(b), Some(if it { "preferenza salvata" } else { "saved preference" }), "memory", 0.7));
            } else if let Some(g) = &genre {
                let st = crate::music::styles::Styles::load(None);
                if let Some(s) = st.get(g) {
                    defaults.push(json!({ "name": "bpm", "value": s["bpm"]["default"], "why": format!("typical tempo of {}", s["label"].as_str().unwrap_or(g)) }));
                    amb.push(Ambiguity { kind: "missing_optional", about: "bpm".into(), severity: "default_used", options: vec![] });
                }
            }
        }
        if !said("key", &explicit) {
            if let Some(k) = pj.key.clone() {
                inferred.push(p("general", "key", json!(k), Some(if it { "tonalità del progetto aperto" } else { "key of the open project" }), "context", 0.9));
            } else if let Some(k) = music_prefs.as_ref().and_then(|m| m.key.clone()) {
                inferred.push(p("general", "key", json!(k), Some(if it { "preferenza salvata" } else { "saved preference" }), "memory", 0.7));
            } else if let Some(g) = &genre {
                let st = crate::music::styles::Styles::load(None);
                if let Some(k) = st.get(g).and_then(|s| s["key_default"].as_str()) {
                    defaults.push(json!({ "name": "key", "value": k, "why": "usual starting key of the style; change it if you want another" }));
                    amb.push(Ambiguity { kind: "missing_optional", about: "key".into(), severity: "default_used", options: vec![] });
                }
            }
        }
        if genre.is_none() && !said("genre", &explicit) {
            if let Some(s) = music_prefs.as_ref().and_then(|m| m.style.clone()) {
                inferred.push(p("general", "genre", json!({ "id": s, "weight": 1.0 }), Some(if it { "preferenza salvata" } else { "saved preference" }), "memory", 0.6));
            }
        }
    }

    // ---- issues
    let mut issues: Vec<Value> = vec![];
    if truncated {
        issues.push(issue("input_too_long", "warning", format!("{} {MAX_CHARS}", q(it, "La richiesta è stata accorciata a", "The request was shortened to")), Some(q(it, "Dividila in più messaggi.", "Split it into several messages."))));
    }
    if !supported {
        issues.push(issue("language_not_supported", "error", q(it, &format!("Per ora capisco italiano e inglese (lingua rilevata: {lang})."), &format!("For now I understand Italian and English (detected: {lang}).")), Some(q(it, "Riscrivi la richiesta in italiano o in inglese.", "Please write it in Italian or English."))));
    }
    let unsafe_code = safety::asks_code_execution(&t);
    if unsafe_code {
        issues.push(issue("unsafe_input", "error", q(it, "Non eseguo codice o comandi di sistema scritti in una richiesta.", "I don't run code or system commands written in a request."), Some(q(it, "Descrivi il risultato musicale che vuoi: lo preparo con gli strumenti di COSOUND.", "Describe the musical result you want: COSOUND's tools will prepare it."))));
    }
    if let Some((what, how)) = safety::unsupported(&t) {
        issues.push(issue("unsupported_operation", "error", format!("{}: {what}.", q(it, "COSOUND non può occuparsi di", "COSOUND cannot handle")), Some(how.to_string())));
    }
    for e in external.iter().filter(|e| !e["warnings"].as_array().unwrap().is_empty()) {
        issues.push(issue("external_instructions_ignored", "warning", format!("{} «{}» {}", q(it, "Il contenuto", "The content"), e["label"].as_str().unwrap_or(""), q(it, "contiene frasi che sembrano istruzioni: le tratto come dati e le ignoro.", "contains instruction-like text: treated as data and ignored.")), None));
    }
    for (code, msg) in &x.problems {
        let sev = if *code == "contradictory" { "warning" } else { "error" };
        issues.push(issue(code, sev, msg.clone(), Some(q(it, "Indica un solo valore possibile.", "Give one possible value."))));
        if *code == "contradictory" {
            amb.push(Ambiguity { kind: "contradiction", about: msg.clone(), severity: "ask", options: vec![] });
        }
    }
    if let (Some(b), Some(g)) = (explicit.iter().find(|q| q.name == "bpm").and_then(|q| q.value.as_f64()), explicit.iter().find(|q| q.name == "genre").and_then(|q| q.value["id"].as_str().map(String::from))) {
        if let Some(s) = crate::music::styles::Styles::load(None).get(&g) {
            let (lo, hi) = (s["bpm"]["min"].as_f64().unwrap_or(0.0), s["bpm"]["max"].as_f64().unwrap_or(300.0));
            if b < lo - 10.0 || b > hi + 10.0 {
                issues.push(issue("incompatible_parameters", "warning", format!("{b} BPM {} {} ({lo}–{hi})", q(it, "è insolito per", "is unusual for"), s["label"].as_str().unwrap_or(&g)), Some(q(it, "Va bene se è voluto (per esempio half-time); altrimenti indica un altro tempo.", "Fine if intended (e.g. half-time); otherwise give another tempo."))));
            }
        }
    }
    let own_only = cons.iter().any(|c| c.subject == "own_samples");
    if own_only && library_files == 0 {
        issues.push(issue("missing_resource", "warning", q(it, "La Smart Library non contiene ancora i tuoi sample.", "The Smart Library does not contain your samples yet."), Some(q(it, "Aggiungi le cartelle dei sample nella Smart Library.", "Add your sample folders to the Smart Library."))));
    }
    if (has("plugin_configuration") || (creating && has("sound_design"))) && plugins == 0 {
        issues.push(issue("missing_resource", "warning", q(it, "Non ho ancora l'elenco dei plug-in installati.", "The list of installed plug-ins is not available yet."), Some(q(it, "Apri la pagina Plug-in e avvia la scansione.", "Open the Plug-ins page and run a scan."))));
    }
    // files for analysis or import
    let needs_file = has("audio_analysis") || has("file_import") || (has("reference_analysis") && !explicit.iter().any(|q| q.name == "reference"));
    let path_in_text = t.raw.contains('/') || t.raw.contains('\\') || [".wav", ".mid", ".mp3", ".aif", ".flac"].iter().any(|e| t.norm.contains(e));
    if needs_file && !path_in_text && !has("library_search") {
        match ctx.files.len() {
            0 => amb.push(Ambiguity { kind: "missing_essential", about: q(it, "quale file usare", "which file to use"), severity: "ask", options: vec![] }),
            1 => amb.push(Ambiguity { kind: "missing_recoverable", about: "file".into(), severity: "resolved_from_context", options: vec![] }),
            _ => amb.push(Ambiguity { kind: "unresolved_reference", about: q(it, "ci sono più file", "there are several files"), severity: "ask", options: ctx.files.iter().filter_map(|f| f["name"].as_str().or(f["path"].as_str()).map(String::from)).collect() }),
        }
    }
    let daw_known = ctx.daw.is_some() || ctx.project.as_ref().is_some_and(|p| p.daw.is_some()) || preferred_daw.is_some() || t.any(&["logic", "ableton", "reaper", "fl studio", "cubase", "pro tools"]);
    if has("daw_management") && !daw_known && !has("music_creation") {
        amb.push(Ambiguity { kind: "missing_essential", about: q(it, "quale DAW usare", "which DAW to use"), severity: "ask", options: vec!["Logic Pro".into(), "Ableton Live".into(), "REAPER".into(), "FL Studio".into(), "Cubase".into(), "Pro Tools".into()] });
    } else if has("daw_management") && ctx.daw.is_none() && preferred_daw.is_some() {
        ctx.daw = preferred_daw.clone();
        inferred.push(p("general", "daw", json!(preferred_daw), Some(if it { "DAW che usi di solito" } else { "your usual DAW" }), "memory", 0.8));
    }
    if source == "voice" {
        if let Some(tc) = tconf {
            if tc < 0.5 {
                amb.push(Ambiguity { kind: "low_transcript_confidence", about: format!("«{}»", clean.trim()), severity: "ask", options: vec![] });
            }
        }
    }

    // ---- safety: destructive operations, constraints in conflict with them
    let destructive = safety::destructive(&masked);
    for d in &destructive {
        if cons.iter().any(|c| (c.kind == "no_delete" && d.kind.starts_with("delete")) || (c.kind == "no_overwrite" && (d.kind == "overwrite_file" || d.kind == "export_over")) || (c.kind == "preserve" && c.subject == d.subject)) {
            amb.push(Ambiguity { kind: "contradiction", about: q(it, &format!("chiedi di non toccare ciò che vuoi eliminare ({})", element_name(&d.subject, true)), &format!("you ask not to touch what you want to remove ({})", element_name(&d.subject, false))), severity: "ask", options: vec![] });
        }
    }
    if intents.is_empty() && act == "new_request" && supported && !unsafe_code && issues.iter().all(|i| i["code"] != "unsupported_operation") {
        issues.push(issue("not_understood", "error", q(it, "Non ho capito cosa vuoi ottenere.", "I didn't understand what you want to achieve."), Some(q(it, "Prova a descrivere il risultato, per esempio: «rendi il basso più caldo» o «crea una melodia in La minore a 124 BPM».", "Try describing the result, e.g. \"make the bass warmer\" or \"create a melody in A minor at 124 BPM\"."))));
    }

    // ---- clarification: only for what is essential
    let ask: Vec<&Ambiguity> = amb.iter().filter(|a| a.severity == "ask").collect();
    let clarification = match ask.first() {
        Some(a) => {
            let question = match a.kind {
                "unresolved_reference" if !a.options.is_empty() => format!("{} {}", q(it, "A quale elemento ti riferisci?", "Which element do you mean?"), a.options.join(" · ")),
                "unresolved_reference" => q(it, &format!("A quale elemento ti riferisci ({})?", a.about), &format!("Which element do you mean ({})?", a.about)),
                "missing_essential" => format!("{} {}?", q(it, "Mi serve un'informazione:", "I need one thing:"), a.about),
                "contradiction" => format!("{} {}. {}", q(it, "C'è una contraddizione:", "There is a contradiction:"), a.about, q(it, "Cosa preferisci?", "Which do you prefer?")),
                "low_transcript_confidence" => format!("{} {} — {}", q(it, "Ho capito:", "I heard:"), a.about, q(it, "è corretto?", "is that right?")),
                _ => a.about.clone(),
            };
            json!({ "needed": true, "question": question, "options": a.options })
        }
        None => json!({ "needed": false, "question": null, "options": [] }),
    };
    let blocked = clarification["needed"] == true || issues.iter().any(|i| i["severity"] == "error");

    // ---- instructions, approval, goal, enhancement
    let prev_agent = last_action.as_ref().and_then(|p| p["instructions"].as_array().and_then(|a| a.first()).and_then(|i| i["agent"].as_str().map(String::from)));
    let mut all_params = explicit.clone();
    all_params.extend(inferred.iter().cloned());
    let instructions = if blocked {
        vec![]
    } else {
        super::instructions::generate(&super::instructions::Input { text: &t, intents: &intents, targets: &targets, params: &all_params, constraints: &cons, destructive: &destructive, ctx: &ctx, creating, confirm_first: prof.mode != "proactive", act, previous_agent: prev_agent })
    };
    for ins in instructions.iter().filter(|i| i["suggested_operation"].is_string() && i["operation_available"] == false) {
        issues.push(issue("unsupported_operation", "info", format!("{}.{} {}", ins["agent"].as_str().unwrap_or(""), ins["suggested_operation"].as_str().unwrap_or(""), q(it, "è prevista ma non ancora disponibile: il Central Agent può proporre un'alternativa.", "is planned but not available yet: the Central Agent can propose an alternative.")), None));
    }
    let mut reasons: Vec<String> = destructive.iter().map(|d| format!("{}: {}", d.kind, d.subject)).collect();
    if instructions.iter().any(|i| i["requires_approval"] == true) {
        reasons.push(if prof.mode != "proactive" { "interaction mode: confirm before acting".into() } else { "writes outside COSOUND or to shared memory".into() });
    }
    reasons.dedup();
    let approval = json!({ "required": !destructive.is_empty() || instructions.iter().any(|i| i["requires_approval"] == true), "reasons": reasons, "destructive_operations": destructive });
    let primary = intents.first().map(|i| i.id);
    let main_target = targets.iter().find(|t| t.kind != "section").map(|t| t.track_name.clone().unwrap_or_else(|| element_name(&t.element, it)));
    let section = targets.iter().find(|t| t.kind == "section").map(|t| element_name(&t.element, it));
    let quals = &x.qualities;
    let (goal_id, verb) = match (act, primary) {
        ("undo", _) => ("undo_last_change".to_string(), q(it, "Annullo l'ultima modifica", "I'll undo the last change")),
        ("restore", _) => ("restore_version".into(), q(it, "Ripristino una versione precedente", "I'll restore an earlier version")),
        ("confirm", _) => ("confirm_version".into(), q(it, "Confermo la versione attuale", "I'll keep the current version")),
        ("cancel", _) => ("cancel_request".into(), q(it, "Annullo la richiesta", "I'll cancel the request")),
        ("smalltalk", _) => ("smalltalk".into(), String::new()),
        (_, Some("composition" | "music_creation")) => ("compose".into(), q(it, "Compongo", "I'll compose")),
        (_, Some("midi_editing")) => ("edit_midi".into(), q(it, "Modifico le note di", "I'll edit the notes of")),
        (_, Some("mixing")) => {
            let down = explicit.iter().any(|q| q.name == "gain_change" && (q.value["db"].as_f64().is_some_and(|d| d < 0.0) || q.value["direction"] == "decrease")) || quals.iter().any(|q| q.0 == "quieter");
            (if down { "lower_level" } else { "change_mix" }.into(), q(it, if down { "Abbasso il volume di" } else { "Regolo nel mix" }, if down { "I'll lower the level of" } else { "I'll adjust in the mix" }))
        }
        (_, Some("sound_design")) => (quals.iter().find(|q| !q.2).map(|q| format!("{}_{}", if q.1 < 0.0 { "decrease" } else { "increase" }, enhance::quality_noun(&q.0, false).replace(' ', "_"))).unwrap_or_else(|| "modify_sound".into()), q(it, "Modifico il suono di", "I'll change the sound of")),
        (_, Some("arrangement")) => ("arrange".into(), q(it, "Lavoro sull'arrangiamento di", "I'll work on the arrangement of")),
        (_, Some("track_editing")) if destructive.iter().any(|d| d.kind == "delete_tracks") => ("delete_tracks".into(), q(it, "Elimino la traccia di", "I'll delete the track of")),
        (_, Some(other)) => {
            let (i, e) = match other {
                "track_editing" => ("Modifico le tracce:", "I'll edit the tracks:"),
                "library_search" => ("Cerco nelle tue librerie", "I'll search your libraries"),
                "sample_selection" => ("Cerco sample adatti", "I'll look for suitable samples"),
                "plugin_configuration" => ("Configuro i plug-in di", "I'll set up the plug-ins of"),
                "mastering" => ("Preparo il master", "I'll prepare the master"),
                "audio_analysis" => ("Analizzo l'audio", "I'll analyse the audio"),
                "reference_analysis" => ("Leggo i tratti generali del riferimento", "I'll read the reference's general traits"),
                "file_import" => ("Importo il file", "I'll import the file"),
                "daw_management" => ("Preparo le operazioni nella DAW", "I'll prepare the DAW operations"),
                "project_management" => ("Gestisco il progetto", "I'll handle the project"),
                "workflow_learning" => ("Ricorderò questa abitudine, con il tuo permesso", "I'll remember this habit, with your permission"),
                "preferences" => ("Aggiorno le preferenze", "I'll update the preferences"),
                _ => ("", ""),
            };
            (other.to_string(), q(it, i, e))
        }
        (_, None) => ("unknown".into(), String::new()),
    };
    let target_words = match (&main_target, &section) {
        (Some(t), Some(s)) => Some(format!("{t} in {s}")),
        (Some(t), None) => Some(t.clone()),
        (None, Some(s)) => Some(s.clone()),
        _ => None,
    };
    let what = if primary == Some("composition") || primary == Some("music_creation") {
        let parts: Vec<String> = targets.iter().filter(|t| t.kind != "section").map(|t| element_name(&t.element, it)).collect();
        let g = explicit.iter().find(|q| q.name == "genre").map(|q| q.value["id"].as_str().unwrap_or("").replace('_', " ")).unwrap_or_default();
        let b = explicit.iter().find(|q| q.name == "bpm").map(|q| format!("{} BPM", q.value.as_f64().unwrap_or(0.0))).unwrap_or_default();
        let k = explicit.iter().find(|q| q.name == "key").and_then(|q| q.value.as_str()).map(|k| crate::music::keys::parse_key(k).map(|k| if it { crate::music::keys::key_name_it(k) } else { k.name() }).unwrap_or_default()).unwrap_or_default();
        Some([if parts.is_empty() { q(it, "un brano", "a track") } else { parts.join(", ") }, g, k, b].into_iter().filter(|s| !s.is_empty()).collect::<Vec<_>>().join(" · "))
    } else {
        target_words
    };
    let interpretation_text = if verb.is_empty() {
        match primary {
            Some("information") => q(it, "Rispondo alla domanda.", "I'll answer the question."),
            Some(p) => format!("{}: {p}.", q(it, "Richiesta", "Request")),
            None => String::new(),
        }
    } else if primary == Some("composition") || primary == Some("music_creation") {
        profile::goal_sentence(it, &verb, what, &[], "")
    } else {
        // these verbs already name their object ("Preparo il master")
        let self_contained = matches!(primary, Some("mastering" | "audio_analysis" | "reference_analysis" | "file_import" | "daw_management" | "project_management" | "workflow_learning" | "preferences" | "library_search" | "sample_selection"));
        // the amounts the user gave, in their words' terms
        let gain = explicit.iter().find(|g| g.name == "gain_change").map(|g| match (g.value["db"].as_f64(), g.value["approximate"].as_bool(), g.value["amount"].as_str()) {
            (Some(db), Some(true), _) => format!("{} {} dB", q(it, "di circa", "by about"), db.abs()),
            (Some(db), _, _) => format!("{} {} dB", q(it, "di", "by"), db.abs()),
            (None, _, Some("slight")) => q(it, "leggermente", "slightly"),
            (None, _, Some("large")) => q(it, "di molto", "a lot"),
            _ => String::new(),
        }).unwrap_or_default();
        let lufs = explicit.iter().find(|g| g.name == "loudness_target").and_then(|g| g.value.as_f64()).map(|v| format!("{} {v} LUFS", q(it, "a", "at"))).unwrap_or_default();
        profile::goal_sentence(it, &verb, if self_contained { None } else { what }, if primary == Some("sound_design") { quals } else { &[] }, [gain, lufs].join(" ").trim())
    };
    let mut user_words: Vec<String> = explicit.iter().filter_map(|p| p.evidence.clone()).collect();
    user_words.extend(elements.iter().map(|e| e.0.clone()));
    user_words.dedup();
    let op_word = match primary {
        Some("sound_design") => if creating { "design_sound" } else { "modify_sound" },
        Some("mixing") => "change_level",
        Some("midi_editing") => "edit_midi",
        Some("composition" | "music_creation") => "compose",
        Some("arrangement") => "arrange",
        Some("library_search" | "sample_selection") => "search",
        Some(o) => o,
        None => "",
    };
    let authorized: Vec<String> = if blocked || op_word.is_empty() || !matches!(act, "new_request" | "modification" | "correction") {
        vec![]
    } else if targets.is_empty() {
        vec![op_word.to_string()]
    } else {
        targets.iter().filter(|t| !destructive.iter().any(|d| d.subject == t.element)).map(|t| format!("{op_word}:{}", t.track_id.clone().unwrap_or_else(|| t.element.clone()))).collect()
    };
    let enhancement = enhance::enhance(it, user_words, quals, &targets, authorized);
    let expected = match (act, primary) {
        ("undo" | "restore", _) => ("previous_version", q(it, "la versione precedente, ripristinata", "the earlier version, restored")),
        ("confirm", _) => ("confirmed_version", q(it, "la versione attuale, confermata", "the current version, confirmed")),
        ("cancel" | "smalltalk", _) => ("none", q(it, "nessuna operazione", "no operation")),
        (_, Some("composition" | "music_creation")) => ("midi_clip", q(it, "parti MIDI coerenti e verificate, esportabili nella DAW", "coherent, checked MIDI parts ready for the DAW")),
        (_, Some("midi_editing")) => ("edited_midi_clip", q(it, "una nuova versione delle note, confrontabile con la precedente", "a new version of the notes, comparable with the previous one")),
        (_, Some("sound_design")) => (if creating { "new_sound" } else { "modified_sound" }, q(it, "il suono modificato, da ascoltare prima di applicarlo", "the modified sound, to audition before applying")),
        (_, Some("mixing")) => ("mix_change", q(it, "una modifica di mix sull'elemento indicato", "a mix change on the element named")),
        (_, Some("mastering")) => ("mastered_audio", q(it, "il master al livello richiesto", "the master at the requested level")),
        (_, Some("arrangement")) => ("arrangement_change", q(it, "una proposta di arrangiamento per le sezioni indicate", "an arrangement proposal for the sections named")),
        (_, Some("library_search" | "sample_selection")) => ("search_results", q(it, "file dalle tue librerie", "files from your libraries")),
        (_, Some("audio_analysis" | "reference_analysis")) => ("analysis_report", q(it, "un'analisi con fonti e affidabilità", "an analysis with sources and confidence")),
        (_, Some("information")) => ("answer", q(it, "una risposta", "an answer")),
        (_, Some(o)) => (o, q(it, "il risultato richiesto", "the requested result")),
        (_, None) => ("none", q(it, "nessuna operazione", "no operation")),
    };
    let mut confidence = intents.first().map_or(if matches!(act, "undo" | "restore" | "confirm" | "cancel" | "smalltalk") { 0.85 } else { 0.2 }, |i| i.confidence);
    if !ask.is_empty() {
        confidence *= 0.6;
    }
    if let Some(tc) = tconf {
        confidence *= 0.5 + 0.5 * tc;
    }
    confidence -= 0.15 * issues.iter().filter(|i| i["severity"] == "error").count() as f64;
    let confidence = (confidence.clamp(0.0, 1.0) * 100.0).round() / 100.0;
    let project = ctx.project.as_ref().map(|p| json!({ "id": p.id.clone().unwrap_or_default(), "name": p.name.clone().unwrap_or_default(), "daw": p.daw }));
    explicit.retain(|p| p.source == "explicit");
    let mut out = json!({
        "schema": "cosound.prompt-interpretation", "schema_version": super::SCHEMA_VERSION,
        "request_id": request_id, "session_id": session,
        "user": { "id": input["user_id"].as_str().unwrap_or("local"), "language": lang },
        "project": project,
        "created_at": c.query_row("select strftime('%Y-%m-%dT%H:%M:%SZ', 'now')", [], |r| r.get::<_, String>(0)).unwrap_or_default(),
        "input": { "text": clean, "source": source, "language": lang, "transcript_confidence": tconf, "external_content": external },
        "conversation": { "act": act, "refers_to": refers_to, "turn": turns + 1 },
        "intents": intents, "primary_intent": primary,
        "goal": { "id": goal_id, "interpretation": interpretation_text },
        "targets": targets,
        "parameters": { "explicit": explicit, "inferred": inferred },
        "constraints": cons,
        "resources": resources(&cons, &x, &ctx, library_files, plugins, &intents),
        "expected_output": { "kind": expected.0, "description": expected.1 },
        "instructions": instructions,
        "enhancement": enhancement,
        "approval": approval,
        "ambiguities": amb,
        "clarification": clarification,
        "issues": issues,
        "defaults_used": defaults,
        "confidence": confidence,
        "interpreter": { "adapter": model::RULES_ADAPTER, "template": null, "fallback": false },
        "context": context::summary(&ctx),
        "reply": { "language": reply_lang, "text": "" },
    });
    validate(&out).map_err(|e| format!("internal: the rules produced an invalid interpretation: {e}"))?;
    // ---- optional model refinement (never when the input is unsafe or the request is blocked by an error)
    let cfg = model::load_config(c);
    if let Some(a) = adapter.filter(|_| cfg.provider != "none" && !unsafe_code && supported) {
        let mut usage = model::load_usage(c);
        let vars: Vec<(&str, String)> = vec![
            ("schema_version", super::SCHEMA_VERSION.into()), ("reply_language", reply_lang.clone()), ("source", source.into()), ("language", lang.clone()),
            ("text", clean.clone()), ("context", context::summary(&ctx).to_string()), ("previous", last_action.as_ref().map(|p| p["goal"].to_string()).unwrap_or_default()),
            ("external", input["external_content"].as_array().map(|a| a.iter().map(|e| format!("[{}]\n{}", e["label"].as_str().unwrap_or(""), e["content"].as_str().unwrap_or("").chars().take(4000).collect::<String>())).collect::<Vec<_>>().join("\n\n")).unwrap_or_default()),
            ("rules", out.to_string()),
        ];
        out = model::refine(&out, a, &cfg, &mut usage, &vars);
        let _ = crate::db::set_setting(c, "prompt_model_usage", &serde_json::to_string(&usage).unwrap());
    }
    out["reply"]["text"] = json!(profile::reply(&prof, it, &out));
    validate(&out).map_err(|e| format!("internal: invalid interpretation after refinement: {e}"))?;
    c.execute(
        "insert into prompt_turns(request_id, session_id, turn, source, language, act, primary_intent, text, interpretation) values (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
        params![request_id, session, turns + 1, source, lang, act, primary, clean, out.to_string()],
    )
    .map_err(|e| e.to_string())?;
    Ok(out)
}

fn resources(cons: &[Constraint], x: &extract::Extracted, ctx: &Ctx, library_files: i64, plugins: i64, intents: &[intent::Intent]) -> Vec<Value> {
    let has = |id: &str| intents.iter().any(|i| i.id == id);
    let mut r = vec![];
    if has("sample_selection") || has("library_search") || cons.iter().any(|c| c.subject == "own_samples") {
        r.push(json!({ "kind": "library", "detail": format!("Smart Library: {library_files} files"), "available": library_files > 0 }));
    }
    if has("plugin_configuration") || has("sound_design") {
        r.push(json!({ "kind": "plugin", "detail": format!("{plugins} installed plug-ins known"), "available": plugins > 0 }));
    }
    if has("daw_management") || ctx.daw.is_some() {
        r.push(json!({ "kind": "daw", "detail": ctx.daw.clone().or(ctx.project.as_ref().and_then(|p| p.daw.clone())).unwrap_or_else(|| "unknown".into()), "available": ctx.daw.is_some() }));
    }
    for f in &ctx.files {
        r.push(json!({ "kind": if f["path"].as_str().is_some_and(|p| p.ends_with(".mid")) { "midi" } else { "file" }, "detail": f["name"].as_str().or(f["path"].as_str()).unwrap_or("file"), "available": true }));
    }
    if let Some(rf) = x.params.iter().find(|q| q.name == "reference") {
        r.push(json!({ "kind": "reference", "detail": rf.value, "available": null }));
    }
    r
}

pub fn history(c: &Connection, session: &str, limit: i64) -> Vec<Value> {
    let mut st = match c.prepare("select interpretation from prompt_turns where session_id = ?1 order by turn desc limit ?2") {
        Ok(s) => s,
        Err(_) => return vec![],
    };
    st.query_map(params![session, limit], |r| r.get::<_, String>(0)).map(|rows| rows.filter_map(|r| r.ok()).filter_map(|s| serde_json::from_str(&s).ok()).collect()).unwrap_or_default()
}

pub fn reset(c: &Connection, session: &str) -> Result<usize, String> {
    c.execute("delete from prompt_turns where session_id = ?1", [session]).map_err(|e| e.to_string())
}
