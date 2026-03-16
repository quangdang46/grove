use crate::{BeadId, RunId, SessionId, SourceId, Timestamp};
use camino::Utf8PathBuf;
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum MessageRole {
    User,
    Agent,
    Tool,
    System,
    Other(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceRecord {
    pub id: SourceId,
    pub source_path: Utf8PathBuf,
    pub origin_host: Option<String>,
    pub metadata_json: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversationRecord {
    pub id: Option<i64>,
    pub bead_id: Option<BeadId>,
    pub run_id: Option<RunId>,
    pub session_id: SessionId,
    pub workspace: Option<Utf8PathBuf>,
    pub title: Option<String>,
    pub source_path: Utf8PathBuf,
    pub started_at: Option<Timestamp>,
    pub ended_at: Option<Timestamp>,
    pub approx_tokens: Option<i64>,
    pub metadata_json: Value,
    pub messages: Vec<MessageRecord>,
    pub source_id: SourceId,
    pub origin_host: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageRecord {
    pub id: Option<i64>,
    pub idx: i64,
    pub role: MessageRole,
    pub author: Option<String>,
    pub created_at: Option<Timestamp>,
    pub content: String,
    pub extra_json: Value,
    pub snippets: Vec<SnippetRecord>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SnippetRecord {
    pub id: Option<i64>,
    pub file_path: Option<Utf8PathBuf>,
    pub start_line: Option<i64>,
    pub end_line: Option<i64>,
    pub language: Option<String>,
    pub snippet_text: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RelevantSnippet {
    pub conversation_id: i64,
    pub message_id: i64,
    pub file_path: Option<Utf8PathBuf>,
    pub snippet: String,
    pub score: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetrievalBundle {
    pub snippets: Vec<RelevantSnippet>,
    pub conversations: Vec<i64>,
}
