-- Migration: 0006_observability.sql
-- Purpose: Add structured observability fields to the event log for post-mortem analysis.

ALTER TABLE event_log ADD COLUMN correlation_id TEXT;
ALTER TABLE event_log ADD COLUMN operation TEXT;
ALTER TABLE event_log ADD COLUMN outcome TEXT;
ALTER TABLE event_log ADD COLUMN duration_ms INTEGER;
ALTER TABLE event_log ADD COLUMN error_json TEXT;
ALTER TABLE event_log ADD COLUMN context_snapshot_json TEXT;

-- Index for searching structured events by operation and outcome
CREATE INDEX IF NOT EXISTS idx_event_log_operation_outcome
  ON event_log(operation, outcome);

-- Index for stitching cross-operation correlation chains
CREATE INDEX IF NOT EXISTS idx_event_log_correlation_id
  ON event_log(correlation_id) WHERE correlation_id IS NOT NULL;
