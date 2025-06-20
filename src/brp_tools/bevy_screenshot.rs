use std::time::Duration;

use rmcp::model::{CallToolResult, Tool};
use rmcp::service::RequestContext;
use rmcp::{Error as McpError, RoleServer};
use serde_json::json;
use tokio::time::timeout;

use super::constants::{
    BRP_METHOD_EXTRAS_SCREENSHOT, DEFAULT_BRP_PORT, DESC_BEVY_SCREENSHOT, JSON_FIELD_PATH,
    JSON_FIELD_PORT, JSON_FIELD_STATUS, PARAM_PORT, TOOL_BRP_EXTRAS_SCREENSHOT,
};
use super::support::BrpJsonRpcBuilder;
use crate::BrpMcpService;
use crate::support::{params, response, schema};

const PARAM_PATH: &str = "path";

pub fn register_tool() -> Tool {
    Tool {
        name:         TOOL_BRP_EXTRAS_SCREENSHOT.into(),
        description:  DESC_BEVY_SCREENSHOT.into(),
        input_schema: schema::SchemaBuilder::new()
            .add_string_property(
                PARAM_PATH,
                "File path where the screenshot should be saved",
                true,
            )
            .add_number_property(
                PARAM_PORT,
                &format!("BRP port to connect to (default: {DEFAULT_BRP_PORT})"),
                false,
            )
            .build(),
    }
}

pub async fn handle(
    _service: &BrpMcpService,
    request: rmcp::model::CallToolRequestParam,
    _context: RequestContext<RoleServer>,
) -> Result<CallToolResult, McpError> {
    // Get parameters
    let path = params::extract_required_string(&request, PARAM_PATH)?;
    let port = params::extract_optional_number(&request, PARAM_PORT, u64::from(DEFAULT_BRP_PORT))?;

    // Take the screenshot
    take_screenshot(
        path,
        u16::try_from(port).map_err(|_| {
            McpError::invalid_params("Port number must be a valid u16".to_string(), None)
        })?,
    )
    .await
}

async fn take_screenshot(path: &str, port: u16) -> Result<CallToolResult, McpError> {
    // Try to take screenshot via bevy_brp_extras
    match try_screenshot_via_extras(path, port).await {
        Ok(true) => {
            let message = format!("Successfully captured screenshot and saved to '{path}'");
            Ok(response::success_json_response(
                message.clone(),
                json!({
                    JSON_FIELD_STATUS: "success",
                    JSON_FIELD_PATH: path,
                    JSON_FIELD_PORT: port,
                    "message": message
                }),
            ))
        }
        Ok(false) => {
            // bevy_brp_extras not available
            let message = "Screenshot failed: bevy_brp_extras is not available. Add bevy_brp_extras to your Bevy app dependencies and register the BrpExtrasPlugin to enable screenshot functionality.";
            Ok(response::success_json_response(
                message.to_string(),
                json!({
                    JSON_FIELD_STATUS: "error",
                    "error_type": "extras_not_available",
                    JSON_FIELD_PATH: path,
                    JSON_FIELD_PORT: port,
                    "message": message,
                    "help": "To add bevy_brp_extras:\n1. Add to Cargo.toml: bevy_brp_extras = \"*\"\n2. Add to your app: .add_plugins(bevy_brp_extras::BrpExtrasPlugin)"
                }),
            ))
        }
        Err(e) => {
            let message = format!("Screenshot failed: {e}");
            Ok(response::success_json_response(
                message.clone(),
                json!({
                    JSON_FIELD_STATUS: "error",
                    "error_type": "brp_error",
                    JSON_FIELD_PATH: path,
                    JSON_FIELD_PORT: port,
                    "message": message
                }),
            ))
        }
    }
}

/// Try to take screenshot via `bevy_brp_extras`
async fn try_screenshot_via_extras(path: &str, port: u16) -> Result<bool, String> {
    let client = reqwest::Client::new();
    let url = format!("http://localhost:{port}");

    // Create screenshot request with path parameter
    let request_body = BrpJsonRpcBuilder::new(BRP_METHOD_EXTRAS_SCREENSHOT)
        .params(json!({ JSON_FIELD_PATH: path }))
        .build();

    // Set a reasonable timeout for screenshot operations
    let response = timeout(
        Duration::from_secs(10),
        client.post(&url).json(&request_body).send(),
    )
    .await;

    match response {
        Ok(Ok(resp)) => {
            // Check if we got a valid JSON-RPC response
            match resp.json::<serde_json::Value>().await {
                Ok(json) => {
                    // Check if this is a valid JSON-RPC response
                    if json.get("jsonrpc").is_some() {
                        // Check if it's an error response
                        if let Some(error) = json.get("error") {
                            if let Some(code) = error.get("code") {
                                // Method not found typically returns -32601
                                if code.as_i64() == Some(-32601) {
                                    return Ok(false); // bevy_brp_extras not available
                                }
                            }
                            // Other error - extract message if available
                            let error_msg = error
                                .get("message")
                                .and_then(|m| m.as_str())
                                .unwrap_or("Unknown BRP error");
                            return Err(error_msg.to_string());
                        }
                        // Success response
                        Ok(true)
                    } else {
                        Err("Invalid JSON-RPC response".to_string())
                    }
                }
                Err(e) => Err(format!("Failed to parse response: {e}")),
            }
        }
        Ok(Err(e)) => Err(format!("HTTP request failed: {e}")),
        Err(_) => Err("Request timed out - BRP may not be responsive".to_string()),
    }
}
