use crate::{BeadId, RunId, Timestamp};
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GroveBeadStatus {
    Idle,
    Ready,
    Running,
    Checkpointed,
    WaitingToRetry,
    Succeeded,
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BeadRef {
    pub id: BeadId,
    pub title: String,
    pub description: Option<String>,
    pub priority: i32,
    pub issue_type: String,
    pub br_status: String,
    pub assignee: Option<String>,
    pub labels: Vec<String>,
    pub created_at: Timestamp,
    pub updated_at: Timestamp,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroveBeadRecord {
    pub bead: BeadRef,
    pub grove_status: GroveBeadStatus,
    pub declared_paths: Vec<String>,
    pub metadata: Value,
    pub last_run_id: Option<RunId>,
    pub retry_after: Option<Timestamp>,
    pub last_failure_class: Option<crate::FailureClass>,
    pub last_failure_detail: Option<String>,
    pub synced_at: Timestamp,
    pub runtime_updated_at: Timestamp,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct BeadRuntimePatch {
    pub grove_status: Option<GroveBeadStatus>,
    pub declared_paths: Option<Vec<String>>,
    pub metadata: Option<Value>,
    pub last_run_id: Option<Option<RunId>>,
    pub retry_after: Option<Option<Timestamp>>,
    pub last_failure_class: Option<Option<crate::FailureClass>>,
    pub last_failure_detail: Option<Option<String>>,
}

impl GroveBeadRecord {
    #[must_use]
    pub fn can_transition_to(&self, next: GroveBeadStatus) -> bool {
        use GroveBeadStatus::{
            Checkpointed, Failed, Idle, Ready, Running, Succeeded, WaitingToRetry,
        };

        matches!(
            (self.grove_status, next),
            (Idle, Ready)
                | (Idle, Running)
                | (Ready, Running)
                | (Running, Checkpointed)
                | (Running, Succeeded)
                | (Running, Failed)
                | (Checkpointed, Running)
                | (Checkpointed, Ready)
                | (Failed, Ready)
                | (WaitingToRetry, Ready)
        )
    }
}
