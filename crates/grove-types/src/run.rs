use crate::{errors::InvalidTransition, BeadId, CheckpointId, Timestamp};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RunStatus {
    Active,
    WaitingToRetry,
    Checkpointed,
    Succeeded,
    Failed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FailureClass {
    Timeout,
    RateLimit,
    PermissionDenied,
    CircuitOpen,
    NoProgress,
    RepeatedError,
    ProtocolMalformed,
    ClaudeCrashed,
    BrMirrorFailed,
    Interrupted,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskRunRecord {
    pub id: crate::RunId,
    pub bead_id: BeadId,
    pub attempt_no: i32,
    pub status: RunStatus,
    pub failure_class: Option<FailureClass>,
    pub failure_detail: Option<String>,
    pub started_at: Timestamp,
    pub ended_at: Option<Timestamp>,
    pub session_count: i32,
    pub checkpoint_count: i32,
    pub last_checkpoint_id: Option<CheckpointId>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetryPolicy {
    pub max_attempts: u32,
    pub timeout_backoff_secs: u64,
    pub rate_limit_backoff_secs: u64,
    pub crash_backoff_secs: u64,
    pub no_progress_backoff_secs: u64,
    pub permission_denied_requires_manual_retry: bool,
}

impl TaskRunRecord {
    #[must_use]
    pub fn can_transition_to(&self, next: RunStatus) -> bool {
        use RunStatus::{Active, Checkpointed, Failed, Succeeded, WaitingToRetry};

        matches!(
            (self.status, next),
            (Active, Checkpointed)
                | (Active, WaitingToRetry)
                | (Active, Succeeded)
                | (Active, Failed)
                | (WaitingToRetry, Active)
                | (Checkpointed, Active)
        )
    }

    pub fn ensure_transition(self, next: RunStatus) -> Result<Self, InvalidTransition> {
        if self.can_transition_to(next) {
            Ok(Self {
                status: next,
                ..self
            })
        } else {
            Err(InvalidTransition::new(
                "run",
                format!("{:?}", self.status),
                format!("{:?}", next),
            ))
        }
    }
}
