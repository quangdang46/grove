use crate::{BeadId, SessionId, Timestamp};
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EventKind {
    BeadCacheSynced,
    DependencySnapshotSynced,
    GroveStatusUpdated,
    RunStarted,
    SessionStarted,
    SessionCheckpointed,
    SessionSucceeded,
    SessionFailed,
    HandoffWritten,
    ReservationGranted,
    ReservationConflictDetected,
    ReservationExpired,
    RecoveryActionTaken,
    LeaseAcquired,
    LeaseHeartbeat,
    LeaseReleased,
    ArchiveIngested,
    PlaybookBulletAdded,
    PlaybookBulletPromoted,
    PlaybookBulletDeprecated,
    BrMirrorRequested,
    BrMirrorSucceeded,
    BrMirrorFailed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventLogRecord {
    pub id: i64,
    pub kind: EventKind,
    pub bead_id: Option<BeadId>,
    pub run_id: Option<crate::RunId>,
    pub session_id: Option<SessionId>,
    pub payload: Value,
    pub created_at: Timestamp,
}
