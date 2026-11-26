//! Agent Protocol v1 (contract_version "1"): the messages exchanged between the central Agent
//! (or any caller) and the specialized agents. JSON Schemas: packages/contracts/.
use serde::{Deserialize, Serialize};
use serde_json::Value;

pub const CONTRACT_VERSION: &str = "1";

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Status {
    Pending,
    Running,
    WaitingApproval,
    Completed,
    Failed,
    Cancelled,
}

impl Status {
    pub fn is_final(self) -> bool {
        matches!(self, Status::Completed | Status::Failed | Status::Cancelled)
    }
    pub fn as_str(self) -> &'static str {
        match self {
            Status::Pending => "PENDING",
            Status::Running => "RUNNING",
            Status::WaitingApproval => "WAITING_APPROVAL",
            Status::Completed => "COMPLETED",
            Status::Failed => "FAILED",
            Status::Cancelled => "CANCELLED",
        }
    }
}

fn v1() -> String {
    CONTRACT_VERSION.into()
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct TaskRequest {
    #[serde(default = "v1")]
    pub contract_version: String,
    /// Caller's id for this request (idempotency and correlation on the caller side).
    pub request_id: String,
    /// Optional: the caller may choose the task id; otherwise one is generated.
    #[serde(default)]
    pub task_id: Option<String>,
    #[serde(default)]
    pub trace_id: Option<String>,
    #[serde(default)]
    pub session_id: Option<String>,
    #[serde(default)]
    pub project_id: Option<String>,
    pub agent: String,
    pub operation: String,
    #[serde(default)]
    pub input: Value,
    /// central_agent | user | mock | test
    #[serde(default)]
    pub requested_by: Option<String>,
    /// Run in the background: the call returns at once (RUNNING) and the result is polled.
    #[serde(default, rename = "async")]
    pub run_async: bool,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct ErrorInfo {
    pub code: String,
    pub message: String,
    pub retryable: bool,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct FileRef {
    pub path: String,
    pub kind: String,
    pub bytes: u64,
    pub sha256: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct ApprovalRequest {
    pub approval_id: String,
    pub task_id: String,
    pub agent: String,
    pub operation: String,
    pub risk: String,
    pub summary: String,
    /// Only the user can approve (Safety Gate). The central Agent can reject or cancel.
    pub approvable_by: String,
    pub requested_at_ms: u64,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct TaskResult {
    pub contract_version: String,
    pub request_id: String,
    pub task_id: String,
    pub trace_id: Option<String>,
    pub session_id: Option<String>,
    pub project_id: Option<String>,
    pub agent: String,
    pub agent_version: Option<String>,
    pub operation: String,
    pub requested_by: Option<String>,
    pub status: Status,
    pub result: Option<Value>,
    /// measured | rule_based | convention | estimated | web | file_name | mixed | none
    pub basis: Option<String>,
    pub files: Vec<FileRef>,
    pub tools_used: Vec<String>,
    pub errors: Vec<ErrorInfo>,
    pub limitations: Vec<String>,
    /// Work this agent suggests the central Agent may assign next (agents never call each other).
    pub needs: Vec<Value>,
    /// DAW operations proposed, never executed by the proposing agent.
    pub proposed_daw_ops: Vec<Value>,
    pub approval: Option<ApprovalRequest>,
    pub created_at_ms: u64,
    pub started_at_ms: Option<u64>,
    pub finished_at_ms: Option<u64>,
}

impl TaskResult {
    pub fn new(req: &TaskRequest, task_id: String, now: u64) -> TaskResult {
        TaskResult {
            contract_version: CONTRACT_VERSION.into(),
            request_id: req.request_id.clone(),
            task_id,
            trace_id: req.trace_id.clone(),
            session_id: req.session_id.clone(),
            project_id: req.project_id.clone(),
            agent: req.agent.clone(),
            agent_version: None,
            operation: req.operation.clone(),
            requested_by: req.requested_by.clone(),
            status: Status::Pending,
            result: None,
            basis: None,
            files: vec![],
            tools_used: vec![],
            errors: vec![],
            limitations: vec![],
            needs: vec![],
            proposed_daw_ops: vec![],
            approval: None,
            created_at_ms: now,
            started_at_ms: None,
            finished_at_ms: None,
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Event {
    pub seq: u64,
    pub at_ms: u64,
    /// task.created | task.status | approval.requested | approval.resolved | agent.enabled | agent.disabled
    pub kind: String,
    pub task_id: Option<String>,
    pub agent: Option<String>,
    pub operation: Option<String>,
    pub status: Option<Status>,
    pub detail: Value,
}

/// Error raised inside an agent; becomes an `ErrorInfo` in the result.
#[derive(Debug, Clone, PartialEq)]
pub struct AgentError {
    pub code: &'static str,
    pub message: String,
    pub retryable: bool,
}

impl AgentError {
    pub fn new(code: &'static str, message: impl Into<String>) -> Self {
        AgentError { code, message: message.into(), retryable: false }
    }
    pub fn input(message: impl Into<String>) -> Self {
        AgentError::new("SCHEMA_INVALID", message)
    }
}

pub fn now_ms() -> u64 {
    std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_millis() as u64).unwrap_or(0)
}
