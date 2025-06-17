use rmcp::Error as McpError;
use rmcp::model::CallToolResult;
use serde_json::Value;

use super::brp_client::BrpResult;

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

/// Re-export BrpErrorInfo for compatibility
pub use super::brp_client::BrpErrorInfo as BrpError;

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
    brp_result: BrpResult,
    formatter: Box<dyn BrpResponseFormatter>,
    metadata: BrpMetadata,
) -> Result<CallToolResult, McpError> {
    match brp_result {
        BrpResult::Success(data) => {
            // Use the data directly, handle null case gracefully
            let response_data = data.unwrap_or(Value::Null);
            Ok(formatter.format_success(response_data, metadata))
        }
        BrpResult::Error(error_info) => {
            // Convert BrpErrorInfo to BrpError for compatibility
            let error = BrpError {
                code:    error_info.code,
                message: error_info.message,
                data:    error_info.data,
            };
            Ok(formatter.format_error(error, metadata))
        }
    }
}
