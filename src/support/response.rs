use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::app_tools::support::scanning::extract_workspace_name;
use crate::error::{Error, Result};

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
    /// Convert to JSON string with error-stack context
    pub fn to_json(&self) -> Result<String> {
        use error_stack::ResultExt;

        serde_json::to_string_pretty(self).change_context(Error::General(
            "Failed to serialize JSON response".to_string(),
        ))
    }

    /// Convert to JSON string with fallback on error
    pub fn to_json_fallback(&self) -> String {
        self.to_json().unwrap_or_else(|_| {
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
    pub const fn success() -> Self {
        Self {
            status:  ResponseStatus::Success,
            message: String::new(),
            data:    None,
        }
    }

    pub const fn error() -> Self {
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

    pub fn data(mut self, data: impl Serialize) -> Result<Self> {
        use error_stack::ResultExt;

        self.data = Some(serde_json::to_value(data).change_context(Error::General(
            "Failed to serialize response data".to_string(),
        ))?);
        Ok(self)
    }

    /// Add a field to the data object. Creates a new object if data is None.
    pub fn add_field(mut self, key: &str, value: impl Serialize) -> Result<Self> {
        use error_stack::ResultExt;

        let value_json = serde_json::to_value(value)
            .change_context(Error::General(format!("Failed to serialize field '{key}'")))?;

        if let Some(Value::Object(map)) = &mut self.data {
            map.insert(key.to_string(), value_json);
        } else {
            let mut map = serde_json::Map::new();
            map.insert(key.to_string(), value_json);
            self.data = Some(Value::Object(map));
        }

        Ok(self)
    }

    /// Add data with fallback to error indicator on serialization failure
    /// This method preserves error context in the fallback value and never fails
    pub fn data_with_fallback(mut self, data: impl Serialize) -> Self {
        match serde_json::to_value(&data) {
            Ok(value) => self.data = Some(value),
            Err(e) => {
                use std::any::type_name_of_val;

                // Create an error-stack Report for better error context
                let error_report = error_stack::Report::new(e)
                    .change_context(Error::General(
                        "Serialization failed in data_with_fallback".to_string(),
                    ))
                    .attach_printable(format!("Data type: {}", type_name_of_val(&data)));

                // Preserve rich error information in response
                self.data = Some(serde_json::json!({
                    "error": "serialization_failed",
                    "message": error_report.to_string(),
                    "data_type": type_name_of_val(&data),
                    "context": "data_with_fallback method encountered serialization error"
                }));
            }
        }
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

/// Helper function to create a successful `CallToolResult` with JSON response
pub fn success_json_response(
    message: impl Into<String>,
    data: impl Serialize,
) -> rmcp::model::CallToolResult {
    let response = ResponseBuilder::success()
        .message(message)
        .data(data)
        .map_or_else(
            |_| {
                ResponseBuilder::error()
                    .message("Failed to serialize response data")
                    .data_with_fallback(serde_json::json!({
                        "error": "serialization_failed",
                        "context": "success_json_response"
                    }))
                    .build()
            },
            ResponseBuilder::build,
        );

    rmcp::model::CallToolResult::success(vec![rmcp::model::Content::text(
        response.to_json_fallback(),
    )])
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
