-- Migration: 0007_archive_fts.sql
-- Purpose: Add archive and FTS search capability.

-- Conversations
CREATE TABLE IF NOT EXISTS archive_sources (
    id TEXT PRIMARY KEY,
    source_path TEXT NOT NULL,
    origin_host TEXT,
    metadata_json TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS archive_conversations (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    bead_id TEXT,
    run_id TEXT,
    session_id TEXT NOT NULL,
    workspace TEXT,
    title TEXT,
    source_path TEXT NOT NULL,
    started_at TEXT,
    ended_at TEXT,
    approx_tokens INTEGER,
    metadata_json TEXT NOT NULL,
    source_id TEXT NOT NULL,
    origin_host TEXT,
    FOREIGN KEY(source_id) REFERENCES archive_sources(id)
);

CREATE UNIQUE INDEX IF NOT EXISTS idx_archive_conversations_session 
    ON archive_conversations(session_id) WHERE session_id IS NOT NULL;
CREATE INDEX IF NOT EXISTS idx_archive_conversations_bead 
    ON archive_conversations(bead_id);

-- Messages
CREATE TABLE IF NOT EXISTS archive_messages (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    conversation_id INTEGER NOT NULL,
    idx INTEGER NOT NULL,
    role TEXT NOT NULL,
    author TEXT,
    created_at TEXT,
    content TEXT NOT NULL,
    extra_json TEXT NOT NULL,
    FOREIGN KEY(conversation_id) REFERENCES archive_conversations(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_archive_messages_conversation 
    ON archive_messages(conversation_id);

-- Snippets
CREATE TABLE IF NOT EXISTS archive_snippets (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    message_id INTEGER NOT NULL,
    file_path TEXT,
    start_line INTEGER,
    end_line INTEGER,
    language TEXT,
    snippet_text TEXT,
    FOREIGN KEY(message_id) REFERENCES archive_messages(id) ON DELETE CASCADE
);

-- FTS5 table for fast searches
CREATE VIRTUAL TABLE IF NOT EXISTS archive_fts USING fts5(
    conversation_id UNINDEXED,
    message_id UNINDEXED,
    role UNINDEXED,
    content,
    tokenize="porter unicode61"
);

-- Triggers to keep FTS synchronized with archive_messages
CREATE TRIGGER IF NOT EXISTS archive_messages_ai AFTER INSERT ON archive_messages BEGIN
  INSERT INTO archive_fts(rowid, conversation_id, message_id, role, content) 
  VALUES (new.id, new.conversation_id, new.id, new.role, new.content);
END;

CREATE TRIGGER IF NOT EXISTS archive_messages_ad AFTER DELETE ON archive_messages BEGIN
  DELETE FROM archive_fts WHERE rowid = old.id;
END;

CREATE TRIGGER IF NOT EXISTS archive_messages_au AFTER UPDATE ON archive_messages BEGIN
  DELETE FROM archive_fts WHERE rowid = old.id;
  INSERT INTO archive_fts(rowid, conversation_id, message_id, role, content) 
  VALUES (new.id, new.conversation_id, new.id, new.role, new.content);
END;
