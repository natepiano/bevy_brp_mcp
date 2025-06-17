use rmcp::Error as McpError;
use rmcp::model::{CallToolResult, Content};
use serde::Serialize;
use serde_json::Value;

use crate::brp_tools::constants::FALLBACK_JSON;
use crate::constants::TOOL_BRP_EXECUTE;

/// Serializes a value to JSON string with fallback on error
pub fn serialize_with_fallback<T: Serialize>(value: &T) -> String {
    serde_json::to_string(value).unwrap_or_else(|_| FALLBACK_JSON.to_string())
}

/// Creates a CallToolResult with serialized JSON content
pub fn json_tool_result<T: Serialize>(value: &T) -> CallToolResult {
    CallToolResult::success(vec![Content::text(serialize_with_fallback(value))])
}

/// Parse a BRP response from JSON string
pub fn parse_brp_response(text: &str) -> Result<Value, McpError> {
    serde_json::from_str(text)
        .map_err(|e| McpError::internal_error(format!("Failed to parse BRP response: {}", e), None))
}

/// Convert BRP parameters to CallToolRequestParam for brp_execute
pub fn to_execute_request(
    brp_params: impl serde::Serialize,
) -> Result<rmcp::model::CallToolRequestParam, McpError> {
    let arguments = serde_json::to_value(brp_params)
        .map_err(|e| {
            McpError::internal_error(format!("Failed to serialize BRP parameters: {}", e), None)
        })?
        .as_object()
        .cloned();

    Ok(rmcp::model::CallToolRequestParam {
        name: TOOL_BRP_EXECUTE.into(),
        arguments,
    })
}
