use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Represents a prompt that can be used to guide users
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Prompt {
    pub name:        String,
    pub description: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub arguments:   Option<Vec<PromptArgument>>,
}

/// Represents an argument for a prompt
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptArgument {
    pub name:        String,
    pub description: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub required:    Option<bool>,
}

/// The response returned when a prompt is executed
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptResponse {
    pub messages: Vec<PromptMessage>,
}

/// A message within a prompt response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptMessage {
    pub role:    String,
    pub content: PromptContent,
}

/// Content of a prompt message
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum PromptContent {
    #[serde(rename = "text")]
    Text { text: String },
}

/// Parameters for BRP execute tool
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BrpExecuteParams {
    pub method: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub params: Option<Value>,
    #[serde(default = "default_port")]
    pub port:   u16,
}

fn default_port() -> u16 {
    15702
}
