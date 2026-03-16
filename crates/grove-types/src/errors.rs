use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq, Error)]
#[error("invalid state transition: {entity} from {from} to {to}")]
pub struct InvalidTransition {
    pub entity: String,
    pub from: String,
    pub to: String,
}

impl InvalidTransition {
    #[must_use]
    pub fn new(entity: impl Into<String>, from: impl Into<String>, to: impl Into<String>) -> Self {
        Self {
            entity: entity.into(),
            from: from.into(),
            to: to.into(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Error)]
pub enum GroveTypesError {
    #[error("invalid checkpoint payload: {reason}")]
    InvalidCheckpoint { reason: String },

    #[error("reservation conflict: {requested} conflicts with {held} (held by {held_by})")]
    ReservationConflict {
        requested: String,
        held: String,
        held_by: String,
    },

    #[error("permission denied: {detail}")]
    PermissionDenied { detail: String },

    #[error("rate limited: retry after {retry_after_secs}s")]
    RateLimited { retry_after_secs: u64 },

    #[error("invalid state transition: {0}")]
    InvalidTransition(#[from] InvalidTransition),
}
