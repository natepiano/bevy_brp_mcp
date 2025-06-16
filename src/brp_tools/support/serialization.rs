use rmcp::model::{CallToolResult, Content};
use serde::Serialize;

use crate::brp_tools::constants::FALLBACK_JSON;

/// Serializes a value to JSON string with fallback on error
pub fn serialize_with_fallback<T: Serialize>(value: &T) -> String {
    serde_json::to_string(value).unwrap_or_else(|_| FALLBACK_JSON.to_string())
}

/// Creates a CallToolResult with serialized JSON content
pub fn json_tool_result<T: Serialize>(value: &T) -> CallToolResult {
    CallToolResult::success(vec![Content::text(serialize_with_fallback(value))])
}
