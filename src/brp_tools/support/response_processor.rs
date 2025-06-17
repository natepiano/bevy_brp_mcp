use rmcp::Error as McpError;
use rmcp::model::CallToolResult;
use serde_json::Value;

use super::serialization::parse_brp_response;
use crate::brp_tools::constants::{JSON_FIELD_CODE, JSON_FIELD_DATA, JSON_FIELD_MESSAGE};

/// Metadata about a BRP request for response formatting
#[derive(Debug, Clone)]
pub struct BrpMetadata {
    pub method: String,
    pub port:   u16,
}

impl BrpMetadata {
    pub fn new(method: &str, port: u16) -> Self {
        Self {
            method: method.to_string(),
            port,
        }
    }
}

/// Structured error information for BRP responses
#[derive(Debug, Clone)]
pub struct BrpError {
    pub code:    Option<i64>,
    pub message: String,
    pub data:    Option<Value>,
}

impl BrpError {
    pub fn new(message: String) -> Self {
        Self {
            code: None,
            message,
            data: None,
        }
    }
}

/// Trait for formatting BRP responses in method-specific ways
pub trait BrpResponseFormatter {
    /// Format a successful BRP response
    fn format_success(&self, data: Value, metadata: BrpMetadata) -> CallToolResult;

    /// Format an error BRP response
    fn format_error(&self, error: BrpError, metadata: BrpMetadata) -> CallToolResult;
}

/// Default error formatter implementation
pub fn format_error_default(error: BrpError, metadata: BrpMetadata) -> CallToolResult {
    use serde_json::json;

    use super::serialization::json_tool_result;
    use crate::brp_tools::constants::{
        JSON_FIELD_DATA, JSON_FIELD_ERROR_CODE, JSON_FIELD_MESSAGE, JSON_FIELD_METADATA,
        JSON_FIELD_METHOD, JSON_FIELD_PORT, JSON_FIELD_STATUS, RESPONSE_STATUS_ERROR,
    };

    let formatted_error = json!({
        JSON_FIELD_STATUS: RESPONSE_STATUS_ERROR,
        JSON_FIELD_MESSAGE: error.message,
        JSON_FIELD_ERROR_CODE: error.code,
        JSON_FIELD_DATA: error.data,
        JSON_FIELD_METADATA: {
            JSON_FIELD_METHOD: metadata.method,
            JSON_FIELD_PORT: metadata.port
        }
    });

    json_tool_result(&formatted_error)
}

/// Generic function to process BRP responses using a formatter
pub fn process_brp_response(
    brp_result: CallToolResult,
    formatter: Box<dyn BrpResponseFormatter>,
    metadata: BrpMetadata,
) -> Result<CallToolResult, McpError> {
    // Extract and format the response
    if let Some(content) = brp_result.content.first() {
        if let Some(text) = content.as_text() {
            // Parse the response from brp_execute
            let response = parse_brp_response(&text.text)?;

            // Check if this is an error response
            if let Some(status) = response.get("status").and_then(|s| s.as_str()) {
                if status == "error" {
                    // Extract error information
                    let message = response
                        .get(JSON_FIELD_MESSAGE)
                        .and_then(|m| m.as_str())
                        .unwrap_or("Unknown error")
                        .to_string();

                    let error_data = response.get(JSON_FIELD_DATA).cloned();
                    let code = error_data
                        .as_ref()
                        .and_then(|d| d.get(JSON_FIELD_CODE))
                        .and_then(|c| c.as_i64());

                    let error = BrpError {
                        code,
                        message,
                        data: error_data,
                    };

                    return Ok(formatter.format_error(error, metadata));
                }
            }

            // Extract the data from the response
            let data = response.get(JSON_FIELD_DATA).ok_or_else(|| {
                McpError::internal_error("Invalid response format from BRP method", None)
            })?;

            // Use the formatter to format the success response
            Ok(formatter.format_success(data.clone(), metadata))
        } else {
            let error = BrpError::new("No text content in BRP response".to_string());
            Ok(formatter.format_error(error, metadata))
        }
    } else {
        let error = BrpError::new("No content in BRP response".to_string());
        Ok(formatter.format_error(error, metadata))
    }
}
