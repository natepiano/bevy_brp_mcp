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
    /// Create a successful response with optional data
    #[cfg(test)]
    pub fn success(message: impl Into<String>) -> Self {
        Self {
            status:  ResponseStatus::Success,
            message: message.into(),
            data:    None,
        }
    }

    /// Create a successful response with data
    #[cfg(test)]
    pub fn success_with_data(message: impl Into<String>, data: impl Serialize) -> Self {
        Self {
            status:  ResponseStatus::Success,
            message: message.into(),
            data:    Some(serde_json::to_value(data).unwrap_or(Value::Null)),
        }
    }

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

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    #[test]
    fn test_success_response() {
        let response = JsonResponse::success("Operation successful");
        let json = serde_json::to_value(&response).unwrap();

        assert_eq!(json["status"], "success");
        assert_eq!(json["message"], "Operation successful");
        assert!(json["data"].is_null());
    }

    #[test]
    fn test_success_with_data() {
        let data = json!({"key": "value"});
        let response = JsonResponse::success_with_data("Data retrieved", data);
        let json = serde_json::to_value(&response).unwrap();

        assert_eq!(json["status"], "success");
        assert_eq!(json["message"], "Data retrieved");
        assert_eq!(json["data"]["key"], "value");
    }

    #[test]
    fn test_response_builder() {
        let response = ResponseBuilder::success()
            .message("Custom message")
            .data(json!({"count": 42}))
            .build();

        let json = serde_json::to_value(&response).unwrap();
        assert_eq!(json["status"], "success");
        assert_eq!(json["message"], "Custom message");
        assert_eq!(json["data"]["count"], 42);
    }
}
