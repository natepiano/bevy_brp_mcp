use rmcp::Error as McpError;
use rmcp::model::CallToolRequestParam;
use serde_json::Value;

use crate::constants::PARAM_WORKSPACE;
use crate::error::BrpMcpError;

// Value-based extraction functions (lower-level)

/// Extract a required u32 from a JSON value
pub fn extract_required_u32(
    arguments: &Value,
    field_name: &str,
    field_description: &str,
) -> Result<u32, McpError> {
    arguments[field_name]
        .as_u64()
        .ok_or_else(|| BrpMcpError::missing(&format!("{field_description} parameter")).into())
        .and_then(|v| {
            u32::try_from(v).map_err(|_| {
                BrpMcpError::invalid(&format!("{field_description} value"), "too large for u32")
                    .into()
            })
        })
}

/// Extract a required u64 from a JSON value
pub fn extract_required_u64(
    arguments: &Value,
    field_name: &str,
    field_description: &str,
) -> Result<u64, McpError> {
    arguments[field_name]
        .as_u64()
        .ok_or_else(|| BrpMcpError::missing(&format!("{field_description} parameter")).into())
}

/// Extract an optional u16 with a default value
pub fn extract_optional_u16(arguments: &Value, field_name: &str, default_value: u16) -> u16 {
    arguments[field_name]
        .as_u64()
        .and_then(|v| u16::try_from(v).ok())
        .unwrap_or(default_value)
}

/// Extract an optional array of strings from a Value
pub fn extract_optional_string_array(arguments: &Value, field_name: &str) -> Option<Vec<String>> {
    arguments[field_name].as_array().map(|arr| {
        arr.iter()
            .filter_map(|v| v.as_str().map(String::from))
            .collect::<Vec<String>>()
    })
}

// CallToolRequestParam-based extraction functions (higher-level)

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
        .ok_or_else(|| BrpMcpError::missing(&format!("required parameter: {param_name}")).into())
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
        .map_or(Ok(default), |v| {
            v.as_u64().ok_or_else(|| {
                BrpMcpError::invalid(&format!("parameter '{param_name}'"), "must be a number")
                    .into()
            })
        })
}

/// Extract an optional u32 parameter from the request with a default value
pub fn extract_optional_u32(
    request: &CallToolRequestParam,
    param_name: &str,
    default: u32,
) -> Result<u32, McpError> {
    let value = extract_optional_number(request, param_name, u64::from(default))?;
    u32::try_from(value)
        .map_err(|_| BrpMcpError::invalid(param_name, "value too large for u32").into())
}

/// Extract a required number parameter from the request
pub fn extract_required_number(
    request: &CallToolRequestParam,
    param_name: &str,
) -> Result<u64, McpError> {
    request
        .arguments
        .as_ref()
        .and_then(|args| args.get(param_name))
        .and_then(serde_json::Value::as_u64)
        .ok_or_else(|| BrpMcpError::missing(&format!("{param_name} (must be a number)")).into())
}

/// Extract any value parameter from the request (for generic JSON values)
pub fn extract_any_value<'a>(
    request: &'a CallToolRequestParam,
    param_name: &str,
) -> Option<&'a serde_json::Value> {
    request
        .arguments
        .as_ref()
        .and_then(|args| args.get(param_name))
}

/// Extract an optional string array parameter from the request
pub fn extract_optional_string_array_from_request(
    request: &CallToolRequestParam,
    param_name: &str,
) -> Result<Option<Vec<String>>, McpError> {
    match request
        .arguments
        .as_ref()
        .and_then(|args| args.get(param_name))
    {
        Some(v) => {
            if let Some(arr) = v.as_array() {
                let mut result = Vec::new();
                for item in arr {
                    if let Some(s) = item.as_str() {
                        result.push(s.to_string());
                    } else {
                        return Err(BrpMcpError::invalid(
                            &format!("items in '{param_name}' array"),
                            "must be strings",
                        )
                        .into());
                    }
                }
                Ok(Some(result))
            } else {
                Err(
                    BrpMcpError::invalid(&format!("parameter '{param_name}'"), "must be an array")
                        .into(),
                )
            }
        }
        None => Ok(None),
    }
}

/// Extract an optional workspace parameter from the request
/// Returns None if not provided or empty string
pub fn extract_optional_workspace(request: &CallToolRequestParam) -> Option<String> {
    let workspace = extract_optional_string(request, PARAM_WORKSPACE, "");
    if workspace.is_empty() {
        None
    } else {
        Some(workspace.to_string())
    }
}
