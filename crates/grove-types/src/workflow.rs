use crate::{prompt::ExecutionContract, task::GroveBeadRecord};
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WorkflowPhase {
    Explore,
    Plan,
    Validate,
    Execute,
    Review,
    Compound,
}

impl WorkflowPhase {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Explore => "explore",
            Self::Plan => "plan",
            Self::Validate => "validate",
            Self::Execute => "execute",
            Self::Review => "review",
            Self::Compound => "compound",
        }
    }

    #[must_use]
    pub const fn title_label(self) -> &'static str {
        match self {
            Self::Explore => "EXPLORE",
            Self::Plan => "PLAN",
            Self::Validate => "VALIDATE",
            Self::Execute => "EXECUTE",
            Self::Review => "REVIEW",
            Self::Compound => "COMPOUND",
        }
    }

    #[must_use]
    pub const fn next(self) -> Option<Self> {
        match self {
            Self::Explore => Some(Self::Plan),
            Self::Plan => Some(Self::Validate),
            Self::Validate => Some(Self::Execute),
            Self::Execute => Some(Self::Review),
            Self::Review => Some(Self::Compound),
            Self::Compound => None,
        }
    }

    #[must_use]
    pub const fn execution_contract(self) -> ExecutionContract {
        match self {
            Self::Explore => ExecutionContract::Explore,
            Self::Plan => ExecutionContract::Plan,
            Self::Validate => ExecutionContract::Validate,
            Self::Execute => ExecutionContract::Implement,
            Self::Review => ExecutionContract::Review,
            Self::Compound => ExecutionContract::Compound,
        }
    }

    #[must_use]
    pub const fn description_preamble(self) -> &'static str {
        match self {
            Self::Explore => {
                "Workflow phase: explore. Clarify scope, missing context, constraints, and decision boundaries before implementation."
            }
            Self::Plan => {
                "Workflow phase: plan. Produce a concrete implementation approach and decompose the work into execution-ready steps."
            }
            Self::Validate => {
                "Workflow phase: validate. Stress-test the current plan, identify weak assumptions, and tighten the execution path before coding."
            }
            Self::Execute => {
                "Workflow phase: execute. Implement the validated plan directly and keep changes aligned with the current task contract."
            }
            Self::Review => {
                "Workflow phase: review. Audit the completed implementation critically, verify quality, and fix any obvious defects before closure."
            }
            Self::Compound => {
                "Workflow phase: compound. Extract durable lessons, decisions, and follow-up notes from the finished work without reopening solved code."
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorkflowState {
    pub phase: WorkflowPhase,
}

impl WorkflowState {
    #[must_use]
    pub const fn new(phase: WorkflowPhase) -> Self {
        Self { phase }
    }

    #[must_use]
    pub fn from_metadata(metadata: &Value) -> Option<Self> {
        metadata
            .get("grove")
            .and_then(|grove| grove.get("workflow"))
            .cloned()
            .and_then(|value| serde_json::from_value(value).ok())
    }

    #[must_use]
    pub fn inferred_from_labels(labels: &[String]) -> Option<Self> {
        for label in labels {
            let normalized = label.trim().to_ascii_lowercase();
            if normalized == "grove:workflow" {
                return Some(Self::new(WorkflowPhase::Explore));
            }
            if let Some(raw_phase) = normalized.strip_prefix("grove:workflow:") {
                let phase = match raw_phase {
                    "explore" => WorkflowPhase::Explore,
                    "plan" => WorkflowPhase::Plan,
                    "validate" => WorkflowPhase::Validate,
                    "execute" => WorkflowPhase::Execute,
                    "review" => WorkflowPhase::Review,
                    "compound" => WorkflowPhase::Compound,
                    _ => continue,
                };
                return Some(Self::new(phase));
            }
        }
        None
    }

    #[must_use]
    pub fn inferred_from_issue_type(issue_type: &str) -> Option<Self> {
        match issue_type.trim().to_ascii_lowercase().as_str() {
            "feature" | "epic" => Some(Self::new(WorkflowPhase::Explore)),
            _ => None,
        }
    }

    #[must_use]
    pub fn from_bead(bead: &GroveBeadRecord) -> Option<Self> {
        Self::from_metadata(&bead.metadata)
            .or_else(|| Self::inferred_from_labels(&bead.bead.labels))
            .or_else(|| Self::inferred_from_issue_type(&bead.bead.issue_type))
    }

    #[must_use]
    pub fn apply_to_metadata(&self, metadata: &Value) -> Value {
        let mut next = metadata.clone();
        if !next.is_object() {
            next = serde_json::json!({});
        }
        if let Some(object) = next.as_object_mut() {
            let grove = object
                .entry("grove".to_owned())
                .or_insert_with(|| serde_json::json!({}));
            if !grove.is_object() {
                *grove = serde_json::json!({});
            }
            if let Some(grove_object) = grove.as_object_mut() {
                grove_object.insert(
                    "workflow".to_owned(),
                    serde_json::to_value(self).unwrap_or_default(),
                );
            }
        }
        next
    }
}
