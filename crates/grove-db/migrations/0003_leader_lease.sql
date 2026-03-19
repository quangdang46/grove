CREATE TABLE IF NOT EXISTS leader_leases (
    slot INTEGER PRIMARY KEY CHECK (slot = 1),
    owner_label TEXT NOT NULL,
    run_id TEXT,
    acquired_at TEXT NOT NULL,
    heartbeat_at TEXT NOT NULL,
    expires_at TEXT NOT NULL,
    released_at TEXT,
    FOREIGN KEY (run_id) REFERENCES task_runs(id) ON DELETE SET NULL
);

CREATE INDEX IF NOT EXISTS idx_leader_leases_active ON leader_leases(released_at, expires_at);
