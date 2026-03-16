-- Grove initial schema
-- Version: 1
-- Applies: core phase-1 runtime tables

CREATE TABLE IF NOT EXISTS _migrations (
    version INTEGER PRIMARY KEY,
    name TEXT NOT NULL,
    applied_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS bead_cache (
    bead_id TEXT PRIMARY KEY,
    title TEXT NOT NULL,
    description TEXT,
    priority INTEGER NOT NULL,
    issue_type TEXT NOT NULL,
    status TEXT NOT NULL,
    assignee TEXT,
    labels_json TEXT NOT NULL DEFAULT '[]',
    parent_ids_json TEXT NOT NULL DEFAULT '[]',
    dependency_ids_json TEXT NOT NULL DEFAULT '[]',
    dependent_ids_json TEXT NOT NULL DEFAULT '[]',
    raw_json TEXT NOT NULL,
    synced_at TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_bead_cache_status ON bead_cache(status);
CREATE INDEX IF NOT EXISTS idx_bead_cache_priority_status ON bead_cache(priority, status);

CREATE TABLE IF NOT EXISTS task_runs (
    id TEXT PRIMARY KEY,
    bead_id TEXT NOT NULL,
    attempt_no INTEGER NOT NULL,
    status TEXT NOT NULL,
    failure_class TEXT,
    failure_detail TEXT,
    started_at TEXT NOT NULL,
    ended_at TEXT,
    session_count INTEGER NOT NULL DEFAULT 0,
    checkpoint_count INTEGER NOT NULL DEFAULT 0,
    last_checkpoint_id TEXT,
    FOREIGN KEY (bead_id) REFERENCES bead_cache(bead_id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_task_runs_bead_attempt ON task_runs(bead_id, attempt_no);
CREATE INDEX IF NOT EXISTS idx_task_runs_status ON task_runs(status);

CREATE TABLE IF NOT EXISTS bead_runtime (
    bead_id TEXT PRIMARY KEY,
    grove_status TEXT NOT NULL,
    declared_paths_json TEXT NOT NULL DEFAULT '[]',
    metadata_json TEXT NOT NULL DEFAULT '{}',
    last_run_id TEXT,
    retry_after TEXT,
    last_failure_class TEXT,
    last_failure_detail TEXT,
    runtime_updated_at TEXT NOT NULL,
    FOREIGN KEY (bead_id) REFERENCES bead_cache(bead_id) ON DELETE CASCADE,
    FOREIGN KEY (last_run_id) REFERENCES task_runs(id) ON DELETE SET NULL
);

CREATE INDEX IF NOT EXISTS idx_bead_runtime_status ON bead_runtime(grove_status);
CREATE INDEX IF NOT EXISTS idx_bead_runtime_retry_after ON bead_runtime(retry_after);

CREATE TABLE IF NOT EXISTS bead_dependencies (
    parent_id TEXT NOT NULL,
    child_id TEXT NOT NULL,
    relation_type TEXT NOT NULL DEFAULT 'blocks',
    synced_at TEXT NOT NULL,
    PRIMARY KEY (parent_id, child_id, relation_type)
);

CREATE INDEX IF NOT EXISTS idx_bead_dependencies_child ON bead_dependencies(child_id);
CREATE INDEX IF NOT EXISTS idx_bead_dependencies_parent ON bead_dependencies(parent_id);

CREATE TABLE IF NOT EXISTS claude_sessions (
    id TEXT PRIMARY KEY,
    run_id TEXT NOT NULL,
    external_session_id TEXT,
    ordinal_in_run INTEGER NOT NULL,
    status TEXT NOT NULL,
    started_at TEXT NOT NULL,
    ended_at TEXT,
    prompt_bytes INTEGER NOT NULL DEFAULT 0,
    estimated_input_tokens INTEGER NOT NULL DEFAULT 0,
    estimated_output_tokens INTEGER NOT NULL DEFAULT 0,
    exit_code INTEGER,
    stop_reason TEXT,
    transcript_path TEXT NOT NULL,
    FOREIGN KEY (run_id) REFERENCES task_runs(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_claude_sessions_run_ordinal ON claude_sessions(run_id, ordinal_in_run);
CREATE INDEX IF NOT EXISTS idx_claude_sessions_external ON claude_sessions(external_session_id);

CREATE TABLE IF NOT EXISTS checkpoints (
    id TEXT PRIMARY KEY,
    bead_id TEXT NOT NULL,
    run_id TEXT NOT NULL,
    session_id TEXT NOT NULL,
    progress TEXT NOT NULL,
    next_step TEXT NOT NULL,
    payload_json TEXT NOT NULL,
    saved_at TEXT NOT NULL,
    resume_generation INTEGER NOT NULL,
    FOREIGN KEY (bead_id) REFERENCES bead_cache(bead_id) ON DELETE CASCADE,
    FOREIGN KEY (run_id) REFERENCES task_runs(id) ON DELETE CASCADE,
    FOREIGN KEY (session_id) REFERENCES claude_sessions(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_checkpoints_bead_saved ON checkpoints(bead_id, saved_at DESC);
CREATE INDEX IF NOT EXISTS idx_checkpoints_run_saved ON checkpoints(run_id, saved_at DESC);

CREATE TABLE IF NOT EXISTS handoffs (
    bead_id TEXT PRIMARY KEY,
    run_id TEXT NOT NULL,
    summary TEXT NOT NULL,
    artifacts_json TEXT NOT NULL,
    lessons_json TEXT NOT NULL,
    decisions_json TEXT NOT NULL,
    warnings_json TEXT NOT NULL,
    completed_at TEXT NOT NULL,
    FOREIGN KEY (bead_id) REFERENCES bead_cache(bead_id) ON DELETE CASCADE,
    FOREIGN KEY (run_id) REFERENCES task_runs(id) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS reservations (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    bead_id TEXT NOT NULL,
    run_id TEXT,
    path_pattern TEXT NOT NULL,
    exclusive INTEGER NOT NULL,
    reason TEXT,
    expires_at TEXT NOT NULL,
    released_at TEXT,
    FOREIGN KEY (bead_id) REFERENCES bead_cache(bead_id) ON DELETE CASCADE,
    FOREIGN KEY (run_id) REFERENCES task_runs(id) ON DELETE SET NULL
);

CREATE INDEX IF NOT EXISTS idx_reservations_active ON reservations(released_at, expires_at);
CREATE INDEX IF NOT EXISTS idx_reservations_bead ON reservations(bead_id);

CREATE TABLE IF NOT EXISTS event_log (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    kind TEXT NOT NULL,
    bead_id TEXT,
    run_id TEXT,
    session_id TEXT,
    payload_json TEXT NOT NULL,
    created_at TEXT NOT NULL,
    FOREIGN KEY (bead_id) REFERENCES bead_cache(bead_id) ON DELETE CASCADE,
    FOREIGN KEY (run_id) REFERENCES task_runs(id) ON DELETE CASCADE,
    FOREIGN KEY (session_id) REFERENCES claude_sessions(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_event_log_bead ON event_log(bead_id, id);
CREATE INDEX IF NOT EXISTS idx_event_log_run ON event_log(run_id, id);
CREATE INDEX IF NOT EXISTS idx_event_log_session ON event_log(session_id, id);
CREATE INDEX IF NOT EXISTS idx_event_log_kind_created ON event_log(kind, created_at);
