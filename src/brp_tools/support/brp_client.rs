//! Low-level BRP (Bevy Remote Protocol) client for JSON-RPC communication
//!
//! This module provides a clean interface for communicating with BRP servers
//! without the MCP-specific formatting concerns. It handles raw BRP protocol
//! communication and returns structured results that can be formatted by
//! higher-level tools.

use std::time::Duration;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::BrpJsonRpcBuilder;
use crate::brp_tools::constants::{
    BRP_DEFAULT_HOST, BRP_HTTP_PROTOCOL, BRP_JSONRPC_PATH, DEFAULT_BRP_PORT,
};
use crate::error::{Error, Result};
use crate::tools::BRP_EXTRAS_PREFIX;

/// Result of a BRP operation
#[derive(Debug, Clone)]
pub enum BrpResult {
    /// Successful operation with optional data
    Success(Option<Value>),
    /// Error with code, message and optional data
    Error(BrpError),
}

/// Error information from BRP operations
#[derive(Debug, Clone)]
pub struct BrpError {
    pub code:    i32,
    pub message: String,
    pub data:    Option<Value>,
}

/// Raw BRP JSON-RPC response structure
#[derive(Debug, Serialize, Deserialize)]
struct BrpResponse {
    jsonrpc: String,
    id:      u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    result:  Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error:   Option<JsonRpcError>,
}

/// Raw BRP error structure from JSON-RPC response
#[derive(Debug, Serialize, Deserialize)]
struct JsonRpcError {
    code:    i32,
    message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    data:    Option<Value>,
}

/// Execute a BRP method and return structured result
pub async fn execute_brp_method(
    method: &str,
    params: Option<Value>,
    port: Option<u16>,
) -> Result<BrpResult> {
    use error_stack::ResultExt;

    let port = port.unwrap_or(DEFAULT_BRP_PORT);
    let url = format!("{BRP_HTTP_PROTOCOL}://{BRP_DEFAULT_HOST}:{port}{BRP_JSONRPC_PATH}");

    // Build JSON-RPC request
    let mut builder = BrpJsonRpcBuilder::new(method);
    if let Some(params) = params {
        builder = builder.params(params);
    }
    let request_body = builder.build().to_string();

    // Send HTTP request using shared client
    let client = super::http_client::get_client();
    let response = client
        .post(&url)
        .header("Content-Type", "application/json")
        .body(request_body)
        .timeout(Duration::from_secs(30))
        .send()
        .await
        .change_context(Error::JsonRpc("HTTP request failed".to_string()))
        .attach_printable(format!("Method: {method}, Port: {port}, URL: {url}"))?;

    // Check HTTP status
    if !response.status().is_success() {
        return Err(
            error_stack::Report::new(Error::JsonRpc("HTTP error".to_string()))
                .attach_printable(format!(
                    "BRP server returned HTTP error {}: {}",
                    response.status(),
                    response
                        .status()
                        .canonical_reason()
                        .unwrap_or("Unknown error")
                ))
                .attach_printable(format!("Method: {method}, Port: {port}")),
        );
    }

    // Parse JSON-RPC response
    let brp_response: BrpResponse = response
        .json()
        .await
        .change_context(Error::JsonRpc("JSON parsing failed".to_string()))
        .attach_printable("Failed to parse BRP response JSON")
        .attach_printable(format!("Method: {method}, Port: {port}"))?;

    // Convert to structured result
    if let Some(error) = brp_response.error {
        // Check if this is a bevy_brp_extras method that's not found
        let enhanced_message = if error.code == -32601 && method.starts_with(BRP_EXTRAS_PREFIX) {
            format!(
                "{}. This method requires the bevy_brp_extras crate to be added to your Bevy app with the BrpExtrasPlugin",
                error.message
            )
        } else {
            error.message
        };

        Ok(BrpResult::Error(BrpError {
            code:    error.code,
            message: enhanced_message,
            data:    error.data,
        }))
    } else {
        Ok(BrpResult::Success(brp_response.result))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_brp_error_creation() {
        let error = BrpError {
            code:    -32600,
            message: "Invalid Request".to_string(),
            data:    None,
        };
        assert_eq!(error.code, -32600);
        assert_eq!(error.message, "Invalid Request");
        assert!(error.data.is_none());
    }

    #[test]
    fn test_brp_result_success() {
        let result = BrpResult::Success(Some(serde_json::json!({"test": "value"})));
        matches!(result, BrpResult::Success(_));
    }

    #[test]
    fn test_brp_result_error() {
        let error = BrpError {
            code:    -1,
            message: "Test error".to_string(),
            data:    None,
        };
        let result = BrpResult::Error(error);
        matches!(result, BrpResult::Error(_));
    }
}
