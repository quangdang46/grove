use crate::{BeadId, RunId, Timestamp};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ReservationMode {
    Shared,
    Exclusive,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReservationRecord {
    pub id: i64,
    pub bead_id: BeadId,
    pub run_id: Option<RunId>,
    pub path_pattern: String,
    pub mode: ReservationMode,
    pub reason: Option<String>,
    pub expires_at: Timestamp,
    pub released_at: Option<Timestamp>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReservationConflict {
    pub requested_by_bead: BeadId,
    pub conflicting_bead: BeadId,
    pub requested_pattern: String,
    pub held_pattern: String,
    pub conflicting_run_id: Option<RunId>,
}
