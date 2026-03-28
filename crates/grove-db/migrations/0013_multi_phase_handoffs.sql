-- Migration: 0013_multi_phase_handoffs.sql
-- Purpose: Allow multiple successful handoffs per bead so workflow-managed beads can advance through internal phases.

ALTER TABLE handoffs RENAME TO handoffs_old;

CREATE TABLE handoffs (
    bead_id TEXT NOT NULL,
    run_id TEXT NOT NULL,
    summary TEXT NOT NULL,
    artifacts_json TEXT NOT NULL,
    lessons_json TEXT NOT NULL,
    decisions_json TEXT NOT NULL,
    warnings_json TEXT NOT NULL,
    completed_at TEXT NOT NULL,
    PRIMARY KEY (bead_id, run_id),
    FOREIGN KEY (bead_id) REFERENCES bead_cache(bead_id) ON DELETE CASCADE,
    FOREIGN KEY (run_id) REFERENCES task_runs(id) ON DELETE CASCADE
);

INSERT INTO handoffs (
    bead_id, run_id, summary, artifacts_json, lessons_json, decisions_json, warnings_json, completed_at
)
SELECT
    bead_id, run_id, summary, artifacts_json, lessons_json, decisions_json, warnings_json, completed_at
FROM handoffs_old;

DROP TABLE handoffs_old;

CREATE INDEX IF NOT EXISTS idx_handoffs_bead_completed
  ON handoffs(bead_id, completed_at DESC, run_id DESC);

CREATE INDEX IF NOT EXISTS idx_handoffs_run
  ON handoffs(run_id);
