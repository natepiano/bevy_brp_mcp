use std::time::Duration;

use rmcp::model::{CallToolResult, Tool};
use rmcp::service::RequestContext;
use rmcp::{Error as McpError, RoleServer};
use serde_json::json;
use sysinfo::{Signal, System};
use tokio::time::timeout;

use super::constants::{
    BRP_METHOD_EXTRAS_SHUTDOWN, DEFAULT_BRP_PORT, DESC_BEVY_SHUTDOWN, JSON_FIELD_PORT,
    JSON_FIELD_STATUS, PARAM_PORT, TOOL_BRP_EXTRAS_SHUTDOWN,
};
use super::support::BrpJsonRpcBuilder;
use crate::BrpMcpService;
use crate::app_tools::constants::PARAM_APP_NAME;
use crate::support::{params, response, schema};

pub fn register_tool() -> Tool {
    Tool {
        name:         TOOL_BRP_EXTRAS_SHUTDOWN.into(),
        description:  DESC_BEVY_SHUTDOWN.into(),
        input_schema: schema::SchemaBuilder::new()
            .add_string_property(PARAM_APP_NAME, "Name of the Bevy app to shutdown", true)
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
    let app_name = params::extract_required_string(&request, PARAM_APP_NAME)?;
    let port = params::extract_optional_number(&request, PARAM_PORT, u64::from(DEFAULT_BRP_PORT))?;

    // Shutdown the app
    shutdown_bevy_app(
        app_name,
        u16::try_from(port).map_err(|_| {
            McpError::invalid_params("Port number must be a valid u16".to_string(), None)
        })?,
    )
    .await
}

async fn shutdown_bevy_app(app_name: &str, port: u16) -> Result<CallToolResult, McpError> {
    // First, try graceful shutdown via bevy_brp_extras
    if try_graceful_shutdown(port).await == Ok(true) {
        // Graceful shutdown succeeded
        let message = format!(
            "Successfully initiated graceful shutdown for '{app_name}' via bevy_brp_extras on port {port}"
        );
        return Ok(response::success_json_response(
            message.clone(),
            json!({
                JSON_FIELD_STATUS: "success",
                "method": "clean_shutdown",
                "app_name": app_name,
                JSON_FIELD_PORT: port,
                "message": message
            }),
        ));
    }
    // BRP responded but bevy_brp_extras not available, or BRP not responsive - fall back to kill

    // Fall back to process termination
    match kill_process(app_name) {
        Ok(Some(pid)) => {
            let message = format!(
                "Terminated process '{app_name}' (PID: {pid}) using kill. Consider adding bevy_brp_extras for clean shutdown."
            );
            Ok(response::success_json_response(
                message.clone(),
                json!({
                    JSON_FIELD_STATUS: "success",
                    "method": "process_kill",
                    "app_name": app_name,
                    JSON_FIELD_PORT: port,
                    "pid": pid,
                    "message": message
                }),
            ))
        }
        Ok(None) => {
            let message = format!("Process '{app_name}' is not currently running");
            Ok(response::success_json_response(
                message.clone(),
                json!({
                    JSON_FIELD_STATUS: "error",
                    "method": "none",
                    "app_name": app_name,
                    JSON_FIELD_PORT: port,
                    "message": message
                }),
            ))
        }
        Err(e) => {
            let message = format!("Failed to terminate process '{app_name}': {e}");
            Ok(response::success_json_response(
                message.clone(),
                json!({
                    JSON_FIELD_STATUS: "error",
                    "method": "process_kill_failed",
                    "app_name": app_name,
                    JSON_FIELD_PORT: port,
                    "message": message
                }),
            ))
        }
    }
}

/// Try to gracefully shutdown via `bevy_brp_extras`
async fn try_graceful_shutdown(port: u16) -> Result<bool, McpError> {
    let client = reqwest::Client::new();
    let url = format!("http://localhost:{port}");

    // Create shutdown request
    let request_body = BrpJsonRpcBuilder::new(BRP_METHOD_EXTRAS_SHUTDOWN).build();

    // Set a reasonable timeout
    let response = timeout(
        Duration::from_secs(5),
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
                        // Check if it's an error response indicating method not found
                        if let Some(error) = json.get("error") {
                            if let Some(code) = error.get("code") {
                                // Method not found typically returns -32601
                                if code.as_i64() == Some(-32601) {
                                    return Ok(false); // bevy_brp_extras not available
                                }
                            }
                        }
                        // Assume success if no error or different error
                        Ok(true)
                    } else {
                        Ok(false)
                    }
                }
                Err(_) => Ok(false),
            }
        }
        _ => Err(McpError::internal_error(
            "BRP not responsive".to_string(),
            None,
        )),
    }
}

/// Kill the process using the system signal
fn kill_process(app_name: &str) -> Result<Option<u32>, String> {
    let mut system = System::new_all();
    system.refresh_processes(sysinfo::ProcessesToUpdate::All, true);

    let running_process = system.processes().values().find(|process| {
        let process_name = process.name().to_string_lossy();
        // Match exact name or with common variations (.exe suffix, etc.)
        process_name == app_name
            || process_name == format!("{app_name}.exe")
            || process_name.strip_suffix(".exe").unwrap_or(&process_name) == app_name
    });

    running_process.map_or(Ok(None), |process| {
        let pid = process.pid().as_u32();

        // Try to kill the process
        if process.kill_with(Signal::Term).unwrap_or(false) {
            Ok(Some(pid))
        } else {
            Err(format!("Failed to send SIGTERM to process {pid}"))
        }
    })
}
