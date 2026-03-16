use crate::{BeadId, BulletId, RunId, SessionOutcome, Timestamp};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BulletScope {
    Global,
    Workspace,
    Language,
    Framework,
    Bead,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BulletType {
    Rule,
    AntiPattern,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BulletState {
    Draft,
    Active,
    Retired,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum BulletMaturity {
    Candidate,
    Established,
    Proven,
    Deprecated,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FeedbackKind {
    Helpful,
    Harmful,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeedbackEventRecord {
    pub kind: FeedbackKind,
    pub timestamp: Timestamp,
    pub bead_id: Option<BeadId>,
    pub run_id: Option<RunId>,
    pub context: Option<String>,
    pub weight: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlaybookBulletRecord {
    pub id: BulletId,
    pub scope: BulletScope,
    pub scope_key: Option<String>,
    pub category: String,
    pub text: String,
    pub bullet_type: BulletType,
    pub state: BulletState,
    pub maturity: BulletMaturity,
    pub helpful_count: u32,
    pub harmful_count: u32,
    pub feedback_events: Vec<FeedbackEventRecord>,
    pub confidence_decay_half_life_days: u32,
    pub pinned: bool,
    pub deprecated: bool,
    pub replaced_by: Option<BulletId>,
    pub deprecation_reason: Option<String>,
    pub source_bead_ids: Vec<BeadId>,
    pub source_run_ids: Vec<RunId>,
    pub tags: Vec<String>,
    pub effective_score: Option<f32>,
    pub created_at: Timestamp,
    pub updated_at: Timestamp,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryDiaryRecord {
    pub bead_id: BeadId,
    pub run_id: RunId,
    pub outcome: SessionOutcome,
    pub summary: String,
    pub accomplishments: Vec<String>,
    pub decisions: Vec<String>,
    pub challenges: Vec<String>,
    pub key_learnings: Vec<String>,
}
