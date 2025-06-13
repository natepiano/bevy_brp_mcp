use rmcp::model::CallToolRequestParam;
use rmcp::Error as McpError;

/// Extract a required string parameter from the request
pub fn extract_required_string<'a>(
    request: &'a CallToolRequestParam,
    param_name: &str,
) -> Result<&'a str, McpError> {
    request
        .arguments
        .as_ref()
        .and_then(|args| args.get(param_name))
        .and_then(|v| v.as_str())
        .ok_or_else(|| McpError::invalid_params(
            format!("Missing required parameter: {}", param_name),
            None
        ))
}

/// Extract an optional string parameter from the request with a default value
pub fn extract_optional_string<'a>(
    request: &'a CallToolRequestParam,
    param_name: &str,
    default: &'a str,
) -> &'a str {
    request
        .arguments
        .as_ref()
        .and_then(|args| args.get(param_name))
        .and_then(|v| v.as_str())
        .unwrap_or(default)
}

/// Extract an optional number parameter from the request with a default value
pub fn extract_optional_number(
    request: &CallToolRequestParam,
    param_name: &str,
    default: u64,
) -> Result<u64, McpError> {
    request
        .arguments
        .as_ref()
        .and_then(|args| args.get(param_name))
        .map(|v| {
            v.as_u64()
                .or_else(|| v.as_i64().map(|i| i as u64))
                .ok_or_else(|| McpError::invalid_params(
                    format!("Parameter '{}' must be a number", param_name),
                    None
                ))
        })
        .transpose()
        .map(|opt| opt.unwrap_or(default))
}