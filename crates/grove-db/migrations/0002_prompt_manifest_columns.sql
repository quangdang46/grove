ALTER TABLE claude_sessions ADD COLUMN prompt_id TEXT;
ALTER TABLE claude_sessions ADD COLUMN prompt_manifest_path TEXT;

CREATE INDEX IF NOT EXISTS idx_claude_sessions_prompt_id ON claude_sessions(prompt_id);
