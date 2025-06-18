//! Shared serialization utilities

use rmcp::model::{CallToolResult, Content};

use crate::support::response::JsonResponse;

// const FALLBACK_JSON: &str = "{}";

// /// Serializes a value to JSON string with fallback on error
// pub fn serialize_with_fallback<T: Serialize>(value: &T) -> String {
//     serde_json::to_string(value).unwrap_or_else(|_| FALLBACK_JSON.to_string())
// }

// /// Creates a CallToolResult with serialized JSON content
// pub fn json_tool_result<T: Serialize>(value: &T) -> CallToolResult {
//     CallToolResult::success(vec![Content::text(serialize_with_fallback(value))])
// }

/// Creates a `CallToolResult` from a `JsonResponse`
pub fn json_response_to_result(response: &JsonResponse) -> CallToolResult {
    CallToolResult::success(vec![Content::text(response.to_json())])
}
