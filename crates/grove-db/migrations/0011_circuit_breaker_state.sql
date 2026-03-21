-- Migration: 0011_circuit_breaker_state.sql
-- Purpose: Persist durable circuit-breaker snapshot in bead runtime state

ALTER TABLE bead_runtime ADD COLUMN circuit_breaker_json TEXT;
