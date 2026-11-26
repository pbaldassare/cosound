//! Background service: keeps the index current without getting in the way of music production.
//! - recovers scans interrupted by a previous session;
//! - notices drives that come back online and rescans their folders incrementally;
//! - optional scheduled rescan (setting `library_rescan_minutes`, off by default);
//! - analyzes pending files in small batches with a short pause between files, and pauses while a
//!   DAW is running (setting `library_pause_during_daw`, on by default) or when the user pauses it.
use super::{analyze, scan};
use crate::db::{self, Db};
use crate::library;
use serde_json::{json, Value};
use std::collections::{HashSet, VecDeque};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

pub type Notify = Box<dyn Fn(&str, Value) + Send + Sync>;

#[derive(Default)]
struct State {
    queue: VecDeque<Option<String>>,
    activity: String,
    paused_reason: Option<String>,
    last_scans: Vec<scan::ScanSummary>,
    analyzed: usize,
}

pub struct Service {
    db: Arc<Db>,
    state: Mutex<State>,
    pub cancel_scan: AtomicBool,
    cancel_analysis: AtomicBool,
    notify: Notify,
}

const DAWS: [&str; 8] = ["logic pro", "ableton live", "live", "fl studio", "fl64", "pro tools", "cubase", "reaper"];

/// Name of a running DAW, if any.
pub fn daw_running() -> Option<String> {
    let Ok(mut s) = sysinfo::System::new() else { return None };
    s.refresh_processes(sysinfo::ProcessesToUpdate::All, true);
    s.processes().values().find_map(|p| {
        let n = p.name().to_string_lossy().to_lowercase();
        DAWS.iter().find(|d| n == **d || n.starts_with(&format!("{d} "))).map(|_| p.name().to_string_lossy().to_string())
    })
}

impl Service {
    pub fn start(db: Arc<Db>, notify: Notify) -> Arc<Service> {
        let s = Arc::new(Service { db, state: Mutex::new(State { activity: "idle".into(), ..Default::default() }), cancel_scan: AtomicBool::new(false), cancel_analysis: AtomicBool::new(false), notify });
        let me = s.clone();
        std::thread::spawn(move || me.run());
        s
    }

    fn setting(&self, k: &str) -> Option<String> {
        db::setting(&self.db.lock().unwrap(), k).ok().flatten()
    }

    pub fn request_scan(&self, root: Option<String>) {
        self.cancel_scan.store(false, Ordering::SeqCst);
        let mut st = self.state.lock().unwrap();
        if !st.queue.contains(&root) {
            st.queue.push_back(root);
        }
    }

    pub fn cancel(&self) {
        self.cancel_scan.store(true, Ordering::SeqCst);
        self.cancel_analysis.store(true, Ordering::SeqCst);
        self.state.lock().unwrap().queue.clear();
    }

    pub fn status(&self) -> Value {
        let st = self.state.lock().unwrap();
        json!({ "activity": st.activity, "paused_reason": st.paused_reason, "queued_scans": st.queue.len(), "last_scans": st.last_scans, "analyzed_this_session": st.analyzed })
    }

    fn set(&self, activity: &str, paused: Option<String>) {
        let mut st = self.state.lock().unwrap();
        st.activity = activity.into();
        st.paused_reason = paused;
    }

    fn run(self: Arc<Self>) {
        for r in scan::recover(&self.db.lock().unwrap()) {
            self.request_scan(Some(r));
        }
        let mut online: HashSet<String> = HashSet::new();
        let mut last_volumes = Instant::now() - Duration::from_secs(60);
        let mut last_scheduled = Instant::now();
        loop {
            // drives that came back: rescan their folders (incremental: unchanged files are not read)
            if last_volumes.elapsed() > Duration::from_secs(15) {
                last_volumes = Instant::now();
                let disks = library::current_disks();
                let roots = {
                    let c = self.db.lock().unwrap();
                    let _ = library::refresh_volumes(&c, &disks);
                    library::roots(&c).unwrap_or_default()
                };
                let now: HashSet<String> = roots.iter().filter(|r| r.online).map(|r| r.id.clone()).collect();
                // at start every online folder gets an incremental scan (files changed while the app was closed);
                // later, only the folders whose drive came back
                for r in now.difference(&online) {
                    self.request_scan(Some(r.clone()));
                }
                online = now;
            }
            if let Some(m) = self.setting("library_rescan_minutes").and_then(|v| v.parse::<u64>().ok()).filter(|m| *m > 0) {
                if last_scheduled.elapsed() > Duration::from_secs(m * 60) {
                    last_scheduled = Instant::now();
                    self.request_scan(None);
                }
            }
            let next = self.state.lock().unwrap().queue.pop_front();
            if let Some(root) = next {
                let roots: Vec<String> = match root {
                    Some(r) => vec![r],
                    None => library::roots(&self.db.lock().unwrap()).unwrap_or_default().into_iter().filter(|r| r.online).map(|r| r.id).collect(),
                };
                for r in roots {
                    self.set(&format!("scanning {r}"), None);
                    let notify = &self.notify;
                    let s = scan::scan_root(&self.db, &r, &self.cancel_scan, &|n| notify("library-scan", json!({ "root_id": r, "count": n, "done": false, "error": null })));
                    (self.notify)("library-scan", json!({ "root_id": r, "count": s.present, "done": true, "error": s.error, "summary": s }));
                    let mut st = self.state.lock().unwrap();
                    st.last_scans.insert(0, s);
                    st.last_scans.truncate(10);
                }
                self.set("idle", None);
                continue;
            }
            // analysis in small batches, never while the user's DAW is running (unless allowed)
            let mode = self.setting("library_analysis").unwrap_or_default();
            if mode == "paused" {
                self.set("idle", Some("paused by the user".into()));
            } else if self.setting("library_pause_during_daw").as_deref() != Some("0") && daw_running().is_some() {
                self.set("idle", Some(format!("paused while {} is running", daw_running().unwrap_or_default())));
            } else {
                self.cancel_analysis.store(false, Ordering::SeqCst);
                self.set("analyzing", None);
                let pause = self.setting("library_analysis_pause_ms").and_then(|v| v.parse().ok()).unwrap_or(15);
                let n = analyze::run_batch(&self.db, 16, &self.cancel_analysis, pause);
                if n > 0 {
                    self.state.lock().unwrap().analyzed += n;
                    (self.notify)("library-analysis", json!({ "analyzed": n }));
                    continue;
                }
                self.set("idle", None);
            }
            std::thread::sleep(Duration::from_secs(3));
        }
    }
}
