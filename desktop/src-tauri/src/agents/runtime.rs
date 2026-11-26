//! Agent runtime: executes the tasks it is given, nothing more. It does not plan, choose agents
//! or combine results (that is the central Agent's job). It validates requests, enforces
//! enable/disable, permissions and user approvals (Safety Gate), runs tasks (inline or in the
//! background, with cancellation), keeps the status of every task, emits events and writes the
//! activity log (`ail_tasks`).
use super::protocol::*;
use super::registry::Registry;
use super::schema::validate;
use super::{Agent, Ctx};
use crate::db::{self, Db};
use serde_json::{json, Value};
use std::collections::{HashMap, VecDeque};
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

const MAX_TASKS: usize = 500;
const MAX_EVENTS: usize = 2000;

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Actor {
    User,
    CentralAgent,
}

struct Rec {
    req: TaskRequest,
    res: TaskResult,
    cancel: Arc<AtomicBool>,
}

#[derive(Default)]
struct State {
    tasks: VecDeque<Rec>,
    events: VecDeque<Event>,
    seq: u64,
}

struct Inner {
    db: Arc<Db>,
    data_dir: PathBuf,
    registry: Registry,
    agents: Vec<Box<dyn Agent>>,
    state: Mutex<State>,
}

#[derive(Clone)]
pub struct Runtime {
    inner: Arc<Inner>,
}

fn err(code: &str, message: impl Into<String>) -> ErrorInfo {
    ErrorInfo { code: code.into(), message: message.into(), retryable: false }
}

impl Runtime {
    pub fn new(db: Arc<Db>, data_dir: PathBuf) -> Runtime {
        let registry = Registry::load().expect("agent manifests are valid (checked by the test suite)");
        let rt = Runtime { inner: Arc::new(Inner { db, data_dir, registry, agents: super::all_agents(), state: Mutex::new(State::default()) }) };
        rt.restore();
        rt
    }

    fn state(&self) -> std::sync::MutexGuard<'_, State> {
        self.inner.state.lock().unwrap_or_else(|e| e.into_inner())
    }

    /// Permission id → granted, from the user's settings (defaults from permissions.json).
    pub fn granted(&self) -> HashMap<String, bool> {
        let settings = db::settings_all(&self.inner.db.lock().unwrap()).unwrap_or_default();
        self.inner
            .registry
            .permissions
            .iter()
            .map(|p| {
                let on = p.grantable && p.setting.as_ref().and_then(|s| settings.get(s)).map_or(p.default, |v| v == "1");
                (p.id.clone(), on)
            })
            .collect()
    }

    pub fn enabled(&self, agent: &str) -> bool {
        db::setting(&self.inner.db.lock().unwrap(), &format!("agent_enabled.{agent}")).ok().flatten().as_deref() != Some("0")
    }

    /// Agent Registry + Tool Registry + permissions, with live state (enabled, granted).
    pub fn registry_view(&self) -> Value {
        let reg = &self.inner.registry;
        let granted = self.granted();
        let agents: Vec<Value> = reg
            .manifests
            .iter()
            .map(|m| {
                let mut v = serde_json::to_value(m).unwrap_or(Value::Null);
                let avail = m.operations.iter().filter(|o| o.status == "available").count();
                v["enabled"] = json!(self.enabled(&m.id));
                let imp = self.inner.agents.iter().find(|a| a.id() == m.id);
                v["implemented"] = json!(imp.is_some());
                v["implemented_operations"] = json!(imp.map(|a| a.operations()).unwrap_or_default());
                v["state"] = json!(if avail == m.operations.len() { "active" } else if avail > 0 { "partial" } else { "planned" });
                v
            })
            .collect();
        let tools: Vec<Value> = reg
            .tools
            .iter()
            .map(|t| {
                let mut v = serde_json::to_value(t).unwrap_or(Value::Null);
                v["used_by"] = json!(reg.manifests.iter().filter(|m| m.tools.contains(&t.id)).map(|m| m.id.clone()).collect::<Vec<_>>());
                v
            })
            .collect();
        let perms: Vec<Value> = reg
            .permissions
            .iter()
            .map(|p| {
                let mut v = serde_json::to_value(p).unwrap_or(Value::Null);
                v["granted"] = json!(granted.get(&p.id).copied().unwrap_or(false));
                v
            })
            .collect();
        json!({ "contract_version": CONTRACT_VERSION, "agents": agents, "tools": tools, "permissions": perms })
    }

    fn emit(st: &mut State, kind: &str, res: Option<&TaskResult>, detail: Value) {
        st.seq += 1;
        let ev = Event {
            seq: st.seq,
            at_ms: now_ms(),
            kind: kind.into(),
            task_id: res.map(|r| r.task_id.clone()),
            agent: res.map(|r| r.agent.clone()),
            operation: res.map(|r| r.operation.clone()),
            status: res.map(|r| r.status),
            detail,
        };
        st.events.push_back(ev);
        while st.events.len() > MAX_EVENTS {
            st.events.pop_front();
        }
    }

    fn persist(&self, req: &TaskRequest, res: &TaskResult) {
        let c = self.inner.db.lock().unwrap();
        let _ = c.execute(
            "insert into ail_tasks(task_id, request_id, trace_id, session_id, project_id, agent, operation, capability, requested_by, request, status, result, error)
             values (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)
             on conflict(task_id) do update set status = excluded.status, result = excluded.result, error = excluded.error, updated_at = datetime('now')",
            rusqlite::params![
                res.task_id,
                res.request_id,
                res.trace_id,
                res.session_id,
                res.project_id,
                res.agent,
                res.operation,
                format!("{}.{}", res.agent, res.operation),
                res.requested_by,
                serde_json::to_string(req).unwrap_or_default(),
                res.status.as_str(),
                serde_json::to_string(res).unwrap_or_default(),
                res.errors.first().map(|e| format!("{}: {}", e.code, e.message)),
            ],
        );
    }

    /// Reload recent tasks; tasks left unfinished by a previous run are marked CANCELLED.
    fn restore(&self) {
        let rows: Vec<(String, String)> = {
            let c = self.inner.db.lock().unwrap();
            let mut st = match c.prepare("select request, result from ail_tasks order by created_at desc, rowid desc limit 200") {
                Ok(s) => s,
                Err(_) => return,
            };
            st.query_map([], |r| Ok((r.get(0)?, r.get::<_, Option<String>>(1)?.unwrap_or_default()))).map(|m| m.filter_map(Result::ok).collect()).unwrap_or_default()
        };
        for (rq, rs) in rows.into_iter().rev() {
            let (Ok(req), Ok(mut res)) = (serde_json::from_str::<TaskRequest>(&rq), serde_json::from_str::<TaskResult>(&rs)) else { continue };
            if !res.status.is_final() {
                res.status = Status::Cancelled;
                res.approval = None;
                res.errors.push(err("INTERRUPTED", "the app was closed before this task finished"));
                res.finished_at_ms = Some(now_ms());
                self.persist(&req, &res);
            }
            self.state().tasks.push_back(Rec { req, res, cancel: Arc::new(AtomicBool::new(false)) });
        }
    }

    fn record(&self, req: &TaskRequest, res: &TaskResult) {
        {
            let mut st = self.state();
            st.tasks.push_back(Rec { req: req.clone(), res: res.clone(), cancel: Arc::new(AtomicBool::new(false)) });
            while st.tasks.len() > MAX_TASKS {
                st.tasks.pop_front();
            }
            Self::emit(&mut st, "task.created", Some(res), json!({ "requested_by": req.requested_by }));
            if res.status.is_final() {
                Self::emit(&mut st, "task.status", Some(res), json!({}));
            }
        }
        self.persist(req, res);
    }

    /// Update a task in place and return a copy (with the request, to persist).
    fn update(&self, task_id: &str, f: impl FnOnce(&mut TaskResult)) -> Option<(TaskRequest, TaskResult)> {
        let mut st = self.state();
        let rec = st.tasks.iter_mut().find(|r| r.res.task_id == task_id)?;
        let before = rec.res.status;
        f(&mut rec.res);
        let (req, res) = (rec.req.clone(), rec.res.clone());
        if res.status != before {
            Self::emit(&mut st, "task.status", Some(&res), json!({ "from": before }));
        }
        Some((req, res))
    }

    pub fn submit(&self, req: TaskRequest) -> TaskResult {
        let now = now_ms();
        let task_id = req.task_id.clone().filter(|t| !t.trim().is_empty() && t.len() <= 100).unwrap_or_else(db::new_id);
        let mut res = TaskResult::new(&req, task_id.clone(), now);
        let reg = &self.inner.registry;
        let manifest = reg.manifest(&req.agent);
        res.agent_version = manifest.map(|m| m.version.clone());
        let checked = (|| {
            if req.contract_version != CONTRACT_VERSION {
                return Err(err("PROTOCOL_VERSION", format!("contract_version {} is not supported (this runtime speaks {CONTRACT_VERSION})", req.contract_version)));
            }
            if req.request_id.trim().is_empty() {
                return Err(err("SCHEMA_INVALID", "request_id is required"));
            }
            if self.state().tasks.iter().any(|r| r.res.task_id == task_id) {
                return Err(err("DUPLICATE_TASK", format!("task_id {task_id} already exists")));
            }
            let m = manifest.ok_or_else(|| err("AGENT_UNKNOWN", format!("no agent {:?} in the registry", req.agent)))?;
            let op = m.operations.iter().find(|o| o.name == req.operation).ok_or_else(|| err("OPERATION_UNKNOWN", format!("{} has no operation {:?}", m.id, req.operation)))?;
            if op.status != "available" {
                return Err(err("NOT_IMPLEMENTED", format!("{}.{} is designed but not implemented yet", m.id, op.name)));
            }
            if !self.enabled(&m.id) {
                return Err(err("AGENT_DISABLED", format!("{} is disabled by the user", m.id)));
            }
            let input = if req.input.is_null() { json!({}) } else { req.input.clone() };
            validate(&op.input, &input, "").map_err(|e| err("SCHEMA_INVALID", e))?;
            let granted = self.granted();
            let missing: Vec<&str> = op.permissions.iter().filter(|p| !granted.get(*p).copied().unwrap_or(false)).map(|p| p.as_str()).collect();
            if !missing.is_empty() {
                return Err(err("PERMISSION_DENIED", format!("permission not granted: {} (the user can change it in Settings)", missing.join(", "))));
            }
            Ok(op)
        })();
        let op = match checked {
            Ok(op) => op,
            Err(e) => {
                if e.code == "NOT_IMPLEMENTED" {
                    if let Some(r) = manifest.and_then(|m| m.operations.iter().find(|o| o.name == req.operation)).and_then(|o| o.requires.clone()) {
                        res.limitations.push(format!("requires: {r}"));
                    }
                }
                res.status = Status::Failed;
                res.errors.push(e);
                res.finished_at_ms = Some(now);
                if res.errors[0].code != "DUPLICATE_TASK" {
                    self.record(&req, &res);
                }
                return res;
            }
        };
        self.record(&req, &res);
        if op.requires_approval || matches!(op.risk.as_str(), "medium" | "high") {
            let approval = ApprovalRequest {
                approval_id: db::new_id(),
                task_id: task_id.clone(),
                agent: req.agent.clone(),
                operation: req.operation.clone(),
                risk: op.risk.clone(),
                summary: op.summary.en.clone(),
                approvable_by: "user".into(),
                requested_at_ms: now_ms(),
            };
            let (rq, rs) = self
                .update(&task_id, |r| {
                    r.status = Status::WaitingApproval;
                    r.approval = Some(approval.clone());
                })
                .expect("task just recorded");
            Self::emit(&mut self.state(), "approval.requested", Some(&rs), json!({ "approval_id": approval.approval_id, "risk": approval.risk }));
            self.persist(&rq, &rs);
            return rs;
        }
        self.start(&task_id)
    }

    fn start(&self, task_id: &str) -> TaskResult {
        let (req, res) = self
            .update(task_id, |r| {
                r.status = Status::Running;
                r.started_at_ms = Some(now_ms());
            })
            .expect("task exists");
        self.persist(&req, &res);
        if req.run_async {
            let rt = self.clone();
            let id = task_id.to_string();
            std::thread::spawn(move || {
                rt.execute(&id);
            });
            res
        } else {
            self.execute(task_id)
        }
    }

    fn execute(&self, task_id: &str) -> TaskResult {
        let Some((req, cancel)) = self.state().tasks.iter().find(|r| r.res.task_id == task_id).map(|r| (r.req.clone(), r.cancel.clone())) else {
            return TaskResult::new(&TaskRequest { contract_version: CONTRACT_VERSION.into(), request_id: String::new(), task_id: None, trace_id: None, session_id: None, project_id: None, agent: String::new(), operation: String::new(), input: Value::Null, requested_by: None, run_async: false }, task_id.into(), now_ms());
        };
        let granted = self.granted();
        let input = if req.input.is_null() { json!({}) } else { req.input.clone() };
        let mut ctx = Ctx::new(&self.inner.db, &self.inner.data_dir, req.project_id.clone(), &granted, &cancel);
        let out = match self.inner.agents.iter().find(|a| a.id() == req.agent) {
            None => Err(AgentError::new("NOT_IMPLEMENTED", "no implementation is registered for this agent")),
            Some(a) => std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| a.run(&req.operation, &input, &mut ctx))).unwrap_or_else(|_| Err(AgentError::new("EXECUTION_FAILED", "internal error in the agent"))),
        };
        let Ctx { tools_used, files, limitations, .. } = ctx;
        let (req, res) = self
            .update(task_id, |r| {
                r.tools_used = tools_used;
                r.files = files;
                r.limitations.extend(limitations);
                match out {
                    Ok(o) => {
                        r.status = Status::Completed;
                        r.result = Some(o.result);
                        r.basis = Some(o.basis.into());
                        r.needs = o.needs;
                        r.proposed_daw_ops = o.proposed_daw_ops;
                    }
                    Err(e) => {
                        r.status = if e.code == "CANCELLED" { Status::Cancelled } else { Status::Failed };
                        r.errors.push(ErrorInfo { code: e.code.into(), message: e.message, retryable: e.retryable });
                    }
                }
                r.finished_at_ms = Some(now_ms());
            })
            .expect("task exists");
        self.persist(&req, &res);
        res
    }

    pub fn status(&self, task_id: &str) -> Option<TaskResult> {
        self.state().tasks.iter().find(|r| r.res.task_id == task_id).map(|r| r.res.clone())
    }

    /// Most recent first.
    pub fn tasks(&self, limit: usize) -> Vec<TaskResult> {
        self.state().tasks.iter().rev().take(limit).map(|r| r.res.clone()).collect()
    }

    pub fn pending_approvals(&self) -> Vec<TaskResult> {
        self.state().tasks.iter().filter(|r| r.res.status == Status::WaitingApproval).map(|r| r.res.clone()).collect()
    }

    /// Wait for a background task to finish (polling).
    #[cfg(test)]
    pub fn wait(&self, task_id: &str, timeout_ms: u64) -> Option<TaskResult> {
        let end = now_ms() + timeout_ms;
        loop {
            let r = self.status(task_id)?;
            if r.status.is_final() || r.status == Status::WaitingApproval || now_ms() > end {
                return Some(r);
            }
            std::thread::sleep(std::time::Duration::from_millis(5));
        }
    }

    pub fn cancel(&self, task_id: &str, actor: Actor) -> Result<TaskResult, ErrorInfo> {
        let who = if actor == Actor::User { "the user" } else { "the central Agent" };
        let (status, flag) = {
            let st = self.state();
            let r = st.tasks.iter().find(|r| r.res.task_id == task_id).ok_or_else(|| err("TASK_UNKNOWN", format!("no task {task_id}")))?;
            (r.res.status, r.cancel.clone())
        };
        match status {
            s if s.is_final() => Err(err("ALREADY_FINISHED", format!("task is already {}", s.as_str()))),
            Status::Running => {
                flag.store(true, Ordering::SeqCst);
                let res = self.status(task_id).expect("task exists");
                Self::emit(&mut self.state(), "task.cancel_requested", Some(&res), json!({ "by": who }));
                Ok(res)
            }
            _ => {
                flag.store(true, Ordering::SeqCst);
                let (req, res) = self
                    .update(task_id, |r| {
                        r.status = Status::Cancelled;
                        r.approval = None;
                        r.errors.push(err("CANCELLED", format!("cancelled by {who}")));
                        r.finished_at_ms = Some(now_ms());
                    })
                    .expect("task exists");
                self.persist(&req, &res);
                Ok(res)
            }
        }
    }

    /// Answer an approval request. Only the user can approve (Safety Gate, AIL §5): the central
    /// Agent may reject, never approve, so even a faulty Agent cannot skip the user's consent.
    pub fn respond(&self, approval_id: &str, approve: bool, actor: Actor) -> Result<TaskResult, ErrorInfo> {
        let task_id = self
            .state()
            .tasks
            .iter()
            .find(|r| r.res.status == Status::WaitingApproval && r.res.approval.as_ref().is_some_and(|a| a.approval_id == approval_id))
            .map(|r| r.res.task_id.clone())
            .ok_or_else(|| err("APPROVAL_UNKNOWN", format!("no pending approval {approval_id}")))?;
        if approve && actor != Actor::User {
            return Err(err("PERMISSION_DENIED", "only the user can approve this operation (Safety Gate)"));
        }
        let by = if actor == Actor::User { "user" } else { "central_agent" };
        let res = self.status(&task_id).expect("task exists");
        Self::emit(&mut self.state(), "approval.resolved", Some(&res), json!({ "approval_id": approval_id, "approved": approve, "by": by }));
        let _ = db::log_op(&self.inner.db.lock().unwrap(), res.project_id.as_deref(), if approve { "agent.approve" } else { "agent.reject" }, "low", &json!({"agent": res.agent, "operation": res.operation, "by": by}), None);
        if approve {
            self.update(&task_id, |r| r.approval = None);
            Ok(self.start(&task_id))
        } else {
            let (req, res) = self
                .update(&task_id, |r| {
                    r.status = Status::Cancelled;
                    r.approval = None;
                    r.errors.push(err("APPROVAL_REJECTED", format!("rejected by the {by}")));
                    r.finished_at_ms = Some(now_ms());
                })
                .expect("task exists");
            self.persist(&req, &res);
            Ok(res)
        }
    }

    pub fn events_since(&self, seq: u64) -> Vec<Event> {
        self.state().events.iter().filter(|e| e.seq > seq).cloned().collect()
    }

    /// Enable or disable an agent (user choice, stored in settings and logged).
    pub fn set_enabled(&self, agent: &str, on: bool) -> Result<(), String> {
        if self.inner.registry.manifest(agent).is_none() {
            return Err(format!("no agent {agent}"));
        }
        {
            let c = self.inner.db.lock().unwrap();
            db::set_setting(&c, &format!("agent_enabled.{agent}"), if on { "1" } else { "0" }).map_err(|e| e.to_string())?;
            db::log_op(&c, None, if on { "agent.enable" } else { "agent.disable" }, "low", &json!({ "agent": agent }), None).map_err(|e| e.to_string())?;
        }
        let mut st = self.state();
        st.seq += 1;
        let seq = st.seq;
        st.events.push_back(Event { seq, at_ms: now_ms(), kind: if on { "agent.enabled" } else { "agent.disabled" }.into(), task_id: None, agent: Some(agent.into()), operation: None, status: None, detail: json!({}) });
        Ok(())
    }

    /// Agent API for transports (the adapter toward the partner's central Agent wraps this).
    /// One JSON message in, one JSON message out. See docs/agents/AGENT-PROTOCOL.md.
    pub fn handle(&self, msg: &Value) -> Value {
        let kind = msg["type"].as_str().unwrap_or("");
        let ok = |data: Value| json!({ "contract_version": CONTRACT_VERSION, "type": kind, "ok": true, "data": data });
        let fail = |e: ErrorInfo| json!({ "contract_version": CONTRACT_VERSION, "type": kind, "ok": false, "error": e });
        if let Some(v) = msg["contract_version"].as_str() {
            if v != CONTRACT_VERSION {
                return fail(err("PROTOCOL_VERSION", format!("contract_version {v} is not supported")));
            }
        }
        let task_id = msg["task_id"].as_str().unwrap_or("");
        match kind {
            "registry.get" => ok(self.registry_view()),
            "task.submit" => {
                let mut r = msg["request"].clone();
                if r["requested_by"].is_null() {
                    r["requested_by"] = json!("central_agent");
                }
                match serde_json::from_value::<TaskRequest>(r) {
                    Ok(req) => ok(json!(self.submit(req))),
                    Err(e) => fail(err("SCHEMA_INVALID", format!("request: {e}"))),
                }
            }
            "task.status" => self.status(task_id).map_or_else(|| fail(err("TASK_UNKNOWN", format!("no task {task_id}"))), |r| ok(json!(r))),
            "task.list" => ok(json!(self.tasks(msg["limit"].as_u64().unwrap_or(50) as usize))),
            "task.cancel" => self.cancel(task_id, Actor::CentralAgent).map_or_else(fail, |r| ok(json!(r))),
            "approval.respond" => {
                let approve = match msg["decision"].as_str() {
                    Some("approve") => true,
                    Some("reject") => false,
                    _ => return fail(err("SCHEMA_INVALID", "decision must be \"approve\" or \"reject\"")),
                };
                self.respond(msg["approval_id"].as_str().unwrap_or(""), approve, Actor::CentralAgent).map_or_else(fail, |r| ok(json!(r)))
            }
            "approval.list" => ok(json!(self.pending_approvals())),
            "events.poll" => ok(json!(self.events_since(msg["since"].as_u64().unwrap_or(0)))),
            _ => fail(err("MESSAGE_UNKNOWN", format!("unknown message type {kind:?}"))),
        }
    }
}
