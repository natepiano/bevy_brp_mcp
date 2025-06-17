use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Standard JSON response structure for all tools
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonResponse {
    pub status:  ResponseStatus,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data:    Option<Value>,
}

/// Response status types
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ResponseStatus {
    Success,
    Error,
}

impl JsonResponse {
    /// Convert to JSON string
    pub fn to_json(&self) -> String {
        serde_json::to_string_pretty(self).unwrap_or_else(|_| {
            r#"{"status":"error","message":"Failed to serialize response"}"#.to_string()
        })
    }
}

/// Builder for constructing JSON responses
pub struct ResponseBuilder {
    status:  ResponseStatus,
    message: String,
    data:    Option<Value>,
}

impl ResponseBuilder {
    pub fn success() -> Self {
        Self {
            status:  ResponseStatus::Success,
            message: String::new(),
            data:    None,
        }
    }

    pub fn error() -> Self {
        Self {
            status:  ResponseStatus::Error,
            message: String::new(),
            data:    None,
        }
    }

    pub fn message(mut self, message: impl Into<String>) -> Self {
        self.message = message.into();
        self
    }

    pub fn data(mut self, data: impl Serialize) -> Self {
        self.data = Some(serde_json::to_value(data).unwrap_or(Value::Null));
        self
    }

    pub fn build(self) -> JsonResponse {
        JsonResponse {
            status:  self.status,
            message: self.message,
            data:    self.data,
        }
    }
}

/// Helper function to create a successful CallToolResult with JSON response
pub fn success_json_response(
    message: impl Into<String>,
    data: impl Serialize,
) -> rmcp::model::CallToolResult {
    let response = ResponseBuilder::success()
        .message(message)
        .data(data)
        .build();

    rmcp::model::CallToolResult::success(vec![rmcp::model::Content::text(response.to_json())])
}
