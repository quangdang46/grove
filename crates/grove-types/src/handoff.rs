use crate::{BeadId, Timestamp};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HandoffRecord {
    pub bead_id: BeadId,
    pub run_id: crate::RunId,
    pub summary: String,
    pub artifacts: Vec<String>,
    pub lessons: Vec<String>,
    pub decisions: Vec<String>,
    pub warnings: Vec<String>,
    pub completed_at: Timestamp,
}
