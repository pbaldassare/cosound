//! Context Interpretation: what is known about the project (from the Central Agent, the DAW connectors or COSOUND's
//! own data) and how references in a request ("il basso", "questo elemento", "rendilo") are resolved to tracks.
//! When a reference cannot be resolved with enough certainty the engine asks; it never picks a track at random.
use super::lexicon::{CLITIC_ENDINGS, CLITIC_VERBS, DEICTIC, ELEMENTS, SECTIONS};
use super::text::Text;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct TrackCtx {
    #[serde(default)]
    pub id: String,
    #[serde(default)]
    pub name: String,
    /// role id if known (bass, kick, pad …)
    #[serde(default)]
    pub role: Option<String>,
    #[serde(default)]
    pub instrument: Option<String>,
    #[serde(default)]
    pub plugins: Vec<String>,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct ProjectCtx {
    #[serde(default)]
    pub id: Option<String>,
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub daw: Option<String>,
    #[serde(default)]
    pub bpm: Option<f64>,
    #[serde(default)]
    pub key: Option<String>,
    #[serde(default)]
    pub genre: Option<String>,
}

/// Everything the Central Agent (or the app) can tell the Prompt Engine. Every field is optional.
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct Ctx {
    #[serde(default)]
    pub project: Option<ProjectCtx>,
    #[serde(default)]
    pub daw: Option<String>,
    #[serde(default)]
    pub tracks: Vec<TrackCtx>,
    #[serde(default)]
    pub selected_track_id: Option<String>,
    #[serde(default)]
    pub section: Option<String>,
    #[serde(default)]
    pub plugins_available: Option<Vec<String>>,
    #[serde(default)]
    pub library_files: Option<u64>,
    #[serde(default)]
    pub files: Vec<Value>,
    #[serde(default)]
    pub recent_operations: Vec<Value>,
    #[serde(default)]
    pub preferences: Vec<Value>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Target {
    pub element: String,
    pub kind: String,
    pub track_id: Option<String>,
    pub track_name: Option<String>,
    pub resolved_by: String,
    pub confidence: f64,
}

#[derive(Clone, Debug, Serialize, PartialEq)]
pub struct Ambiguity {
    pub kind: &'static str,
    pub about: String,
    pub severity: &'static str,
    pub options: Vec<String>,
}

/// Element ids named in the text, in order of appearance (longest phrase per position).
pub fn elements(t: &Text) -> Vec<(String, &'static str)> {
    let mut found: Vec<(usize, String, &'static str)> = vec![];
    for (id, kind, words) in ELEMENTS {
        for w in words.iter() {
            if let Some(pos) = t.pos(w) {
                if !found.iter().any(|f| f.1 == id) {
                    found.push((pos, id.to_string(), kind));
                }
            }
        }
    }
    // "synth lead" is a lead, "sub bass" a bass: drop a generic element hidden inside a longer one
    found.sort_by_key(|f| f.0);
    if found.iter().any(|f| f.1 == "lead") {
        found.retain(|f| f.1 != "synth" || t.pos("synth lead").is_none());
    }
    found.into_iter().map(|f| (f.1, f.2)).collect()
}

pub fn sections(t: &Text) -> Vec<String> {
    SECTIONS.iter().filter(|(_, ws)| ws.iter().any(|w| t.has(w))).map(|(id, _)| id.to_string()).collect()
}

/// The request points at something without naming it ("questo elemento", "rendilo", "make it").
pub fn deictic(t: &Text) -> bool {
    t.any(&DEICTIC) || t.words.iter().any(|w| CLITIC_VERBS.iter().any(|v| w.starts_with(v) && w.len() > v.len() && w.len() <= v.len() + 3 && CLITIC_ENDINGS.iter().any(|e| w.ends_with(e))))
}

fn matches(track: &TrackCtx, element: &str) -> bool {
    let words: Vec<&str> = ELEMENTS.iter().find(|e| e.0 == element).map(|e| e.2.to_vec()).unwrap_or_default();
    let name = track.name.to_lowercase();
    track.role.as_deref() == Some(element) || words.iter().any(|w| name.split(|c: char| !c.is_alphanumeric()).any(|n| n == *w) || (w.contains(' ') && name.contains(w))) || name == element
}

/// Resolve the request's targets. `creating` = the request makes something new (a missing track is not an error).
pub fn resolve(t: &Text, ctx: &Ctx, previous: &[Target], creating: bool, lang: &str) -> (Vec<Target>, Vec<Ambiguity>) {
    let it = lang == "it";
    let mut out = vec![];
    let mut amb = vec![];
    let named = elements(t);
    for (el, kind) in &named {
        if ctx.tracks.is_empty() || *kind == "mix" || *kind == "project" {
            out.push(Target { element: el.clone(), kind: kind.to_string(), track_id: None, track_name: None, resolved_by: if creating && *kind != "mix" { "new" } else { "explicit" }.into(), confidence: 0.9 });
            continue;
        }
        let hits: Vec<&TrackCtx> = ctx.tracks.iter().filter(|tr| matches(tr, el)).collect();
        match hits.len() {
            1 => out.push(Target { element: el.clone(), kind: "track".into(), track_id: Some(hits[0].id.clone()), track_name: Some(hits[0].name.clone()), resolved_by: "context".into(), confidence: 0.95 }),
            0 if creating => out.push(Target { element: el.clone(), kind: kind.to_string(), track_id: None, track_name: None, resolved_by: "new".into(), confidence: 0.85 }),
            0 => amb.push(Ambiguity { kind: "unresolved_reference", about: if it { format!("nel progetto non c'è un elemento «{el}»") } else { format!("there is no «{el}» in the project") }, severity: "ask", options: ctx.tracks.iter().map(|t| t.name.clone()).collect() }),
            _ => {
                if let Some(sel) = hits.iter().find(|h| ctx.selected_track_id.as_deref() == Some(h.id.as_str())) {
                    out.push(Target { element: el.clone(), kind: "track".into(), track_id: Some(sel.id.clone()), track_name: Some(sel.name.clone()), resolved_by: "selection".into(), confidence: 0.85 });
                } else {
                    amb.push(Ambiguity { kind: "unresolved_reference", about: if it { format!("ci sono {} tracce che possono essere «{el}»", hits.len()) } else { format!("{} tracks could be «{el}»", hits.len()) }, severity: "ask", options: hits.iter().map(|h| h.name.clone()).collect() });
                }
            }
        }
    }
    for s in sections(t) {
        out.push(Target { element: s, kind: "section".into(), track_id: None, track_name: None, resolved_by: "explicit".into(), confidence: 0.9 });
    }
    let has_element = out.iter().any(|x| x.kind != "section") || amb.iter().any(|a| a.kind == "unresolved_reference");
    if !has_element && deictic(t) {
        if let Some(sel) = ctx.selected_track_id.as_ref().and_then(|id| ctx.tracks.iter().find(|tr| &tr.id == id)) {
            out.push(Target { element: sel.role.clone().unwrap_or_else(|| sel.name.to_lowercase()), kind: "track".into(), track_id: Some(sel.id.clone()), track_name: Some(sel.name.clone()), resolved_by: "selection".into(), confidence: 0.9 });
        } else if let Some(prev) = previous.iter().find(|p| p.kind != "section") {
            let mut p = prev.clone();
            p.resolved_by = "conversation".into();
            p.confidence = (prev.confidence * 0.9).min(0.85);
            out.push(p);
        } else {
            amb.push(Ambiguity {
                kind: "unresolved_reference",
                about: if it { "non è chiaro a quale elemento ti riferisci".into() } else { "it is not clear which element you mean".into() },
                severity: "ask",
                options: ctx.tracks.iter().map(|t| t.name.clone()).collect(),
            });
        }
    }
    (out, amb)
}

/// Context as stored in the interpretation (only what was given or found, nothing invented).
pub fn summary(ctx: &Ctx) -> Value {
    json!({
        "project": ctx.project, "daw": ctx.daw, "tracks": ctx.tracks.len(), "selected_track_id": ctx.selected_track_id,
        "section": ctx.section, "plugins_available": ctx.plugins_available.as_ref().map(|p| p.len()), "library_files": ctx.library_files,
        "files": ctx.files.len(), "recent_operations": ctx.recent_operations.len(),
    })
}

#[cfg(test)]
mod tests {
    use super::super::text::normalize;
    use super::*;

    fn ctx() -> Ctx {
        serde_json::from_value(json!({
            "tracks": [
                { "id": "t1", "name": "Kick", "role": "kick" }, { "id": "t2", "name": "Sub Bass" },
                { "id": "t3", "name": "Lead Synth" }, { "id": "t4", "name": "Pad Synth" }, { "id": "t5", "name": "Vox" }
            ]
        })).unwrap()
    }

    #[test]
    fn tracks_and_instruments_are_found() {
        let (t, a) = resolve(&normalize("Rendi il basso più potente"), &ctx(), &[], false, "it");
        assert!(a.is_empty());
        assert_eq!((t[0].element.as_str(), t[0].track_id.as_deref()), ("bass", Some("t2")));
        let (t, _) = resolve(&normalize("abbassa il kick e allarga il pad"), &ctx(), &[], false, "it");
        assert_eq!(t.iter().map(|x| x.track_id.clone().unwrap()).collect::<Vec<_>>(), ["t1", "t4"]);
        // two synths: ask, unless one is selected
        let (t, a) = resolve(&normalize("make the synth brighter"), &ctx(), &[], false, "en");
        assert!(t.is_empty() && a[0].kind == "unresolved_reference" && a[0].options.len() == 2, "{a:?}");
        let mut c = ctx();
        c.selected_track_id = Some("t3".into());
        let (t, a) = resolve(&normalize("make the synth brighter"), &c, &[], false, "en");
        assert!(a.is_empty() && t[0].track_id.as_deref() == Some("t3") && t[0].resolved_by == "selection");
        // an element that is not in the project: ask when modifying, create when creating
        let (_, a) = resolve(&normalize("rendi gli archi più caldi"), &ctx(), &[], false, "it");
        assert_eq!(a[0].severity, "ask");
        let (t, a) = resolve(&normalize("aggiungi degli archi"), &ctx(), &[], true, "it");
        assert!(a.is_empty() && t[0].resolved_by == "new");
    }

    #[test]
    fn this_element_uses_selection_conversation_or_asks() {
        let (t, a) = resolve(&normalize("Rendi questo elemento più aggressivo"), &ctx(), &[], false, "it");
        assert!(t.is_empty() && a[0].kind == "unresolved_reference" && a[0].options.len() == 5);
        let mut c = ctx();
        c.selected_track_id = Some("t5".into());
        let (t, _) = resolve(&normalize("Rendi questo elemento più aggressivo"), &c, &[], false, "it");
        assert_eq!(t[0].track_name.as_deref(), Some("Vox"));
        let prev = vec![Target { element: "bass".into(), kind: "track".into(), track_id: Some("t2".into()), track_name: Some("Sub Bass".into()), resolved_by: "context".into(), confidence: 0.95 }];
        let (t, _) = resolve(&normalize("Rendilo più aggressivo"), &ctx(), &prev, false, "it");
        assert_eq!((t[0].track_id.as_deref(), t[0].resolved_by.as_str()), (Some("t2"), "conversation"));
        assert!(deictic(&normalize("alzalo")) && !deictic(&normalize("alza il kick")));
    }
}
