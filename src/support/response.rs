use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::app_tools::support::scanning::extract_workspace_name;

/// Standard JSON response structure for all tools
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonResponse {
    pub status:  ResponseStatus,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data:    Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub debug_info: Option<Vec<String>>,
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
    debug_info: Option<Vec<String>>,
}

impl ResponseBuilder {
    pub const fn success() -> Self {
        Self {
            status:  ResponseStatus::Success,
            message: String::new(),
            data:    None,
            debug_info: None,
        }
    }

    pub const fn error() -> Self {
        Self {
            status:  ResponseStatus::Error,
            message: String::new(),
            data:    None,
            debug_info: None,
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

    /// Add a field to the data object. Creates a new object if data is None.
    pub fn add_field(mut self, key: &str, value: impl Serialize) -> Self {
        let value_json = serde_json::to_value(value).unwrap_or(Value::Null);

        if let Some(Value::Object(map)) = &mut self.data {
            map.insert(key.to_string(), value_json);
        } else {
            let mut map = serde_json::Map::new();
            map.insert(key.to_string(), value_json);
            self.data = Some(Value::Object(map));
        }

        self
    }

    pub fn debug_info(mut self, debug_info: Vec<String>) -> Self {
        self.debug_info = Some(debug_info);
        self
    }

    pub fn build(self) -> JsonResponse {
        JsonResponse {
            status:  self.status,
            message: self.message,
            data:    self.data,
            debug_info: self.debug_info,
        }
    }
}

/// Helper function to create a successful `CallToolResult` with JSON response
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

/// Add workspace information to response data if available
pub fn add_workspace_info_to_response(
    response_data: &mut serde_json::Value,
    workspace_root: Option<&PathBuf>,
) {
    if let Some(root) = workspace_root {
        if let Some(workspace_name) = extract_workspace_name(root) {
            response_data["workspace"] = serde_json::Value::String(workspace_name);
        }
    }
}
