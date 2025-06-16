use rmcp::Error as McpError;
use serde_json::Value;

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
        name: "brp_execute".into(),
        arguments,
    })
}
