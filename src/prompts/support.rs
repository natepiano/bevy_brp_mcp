use crate::types::{PromptContent, PromptMessage, PromptResponse};

/// Helper to create a simple text response for a prompt
pub fn text_response(text: impl Into<String>) -> PromptResponse {
    PromptResponse {
        messages: vec![PromptMessage {
            role:    "assistant".to_string(),
            content: PromptContent::Text { text: text.into() },
        }],
    }
}
