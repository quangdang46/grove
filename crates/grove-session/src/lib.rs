mod analysis;
mod analyzer;
mod backend;
mod exit_policy;
mod materializer;
mod parser;
mod progress;
mod protocol;
mod retry;
mod runner;
mod transcript;

pub use analysis::{analyze_iteration, AnalysisInput};
pub use analyzer::{
    analyze_session_outcome, classify_session_outcome, classify_session_outcome_with_policy,
    evaluate_exit_policy, evaluate_outcome_exit_policy, update_circuit_breaker, ContextMonitor,
    ContextPressure, ContextPressureDecision, SessionAnalysisContext,
};
pub use backend::{ClaudeBackend, CliClaudeBackend, RunningSession, StartSessionRequest};
pub use exit_policy::{ExitDecision, ExitPolicy};
pub use materializer::{
    CheckpointPromptInput, PromptMaterialization, PromptMaterializationInput, materialize_prompt,
};
pub use parser::{ParserLineKind, ProtocolParser, ProtocolWarning};
pub use progress::infer_progress_signal;
pub use protocol::{
    parse_protocol_event, ProtocolMarker, ProtocolParseError, GROVE_ARTIFACTS_PREFIX,
    GROVE_CHECKPOINT_PREFIX, GROVE_DECISIONS_PREFIX, GROVE_EXIT_PREFIX, GROVE_LESSONS_PREFIX,
    GROVE_RESULT_PREFIX, GROVE_WARNINGS_PREFIX,
};
pub use retry::{plan_retry_mutation, RetryMutationPlan};
pub use runner::{
    SessionLifecycleHooks, SingleTaskSessionRequest, SingleTaskSessionResult,
    SingleTaskSessionRunnerError, execute_single_task_session,
    execute_single_task_session_with_hooks,
};
pub use transcript::{replay_transcript, TranscriptError, TranscriptReplay, TranscriptWriter};

pub const CRATE_PURPOSE: &str =
    "Claude session protocol parsing, transcript capture, and session analysis helpers.";
