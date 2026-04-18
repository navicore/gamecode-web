use crate::notebook::Notebook;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoredConversation {
    pub id: String,
    pub notebook: Notebook,
    pub context_state: ContextState,
    pub metadata: ConversationMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextState {
    pub compressed_summaries: Vec<String>,
    pub active_messages: Vec<crate::api::ChatMessage>,
    pub total_tokens: usize,
    pub compression_count: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversationMetadata {
    pub created_at: DateTime<Utc>,
    pub modified_at: DateTime<Utc>,
    pub title: String,
    pub model: String,
    pub provider: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversationRef {
    pub id: String,
    pub title: String,
    pub modified_at: DateTime<Utc>,
    pub preview: String,
}

// Rough approximation: ~4 characters per token for English.
pub fn estimate_tokens(text: &str) -> usize {
    text.len() / 4
}

pub fn estimate_message_tokens(message: &crate::api::ChatMessage) -> usize {
    estimate_tokens(&message.role) + estimate_tokens(&message.content) + 3
}

pub fn estimate_context_tokens(messages: &[crate::api::ChatMessage]) -> usize {
    messages.iter().map(estimate_message_tokens).sum()
}
