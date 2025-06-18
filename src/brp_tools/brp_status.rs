use std::time::Duration;

use rmcp::model::{CallToolResult, Tool};
use rmcp::service::RequestContext;
use rmcp::{Error as McpError, RoleServer};
use serde_json::json;
use sysinfo::System;
use tokio::time::timeout;

use super::constants::{DEFAULT_BRP_PORT, JSON_FIELD_PORT, JSON_FIELD_STATUS};
use super::support::BrpJsonRpcBuilder;
use crate::BrpMcpService;
use crate::constants::{PARAM_APP_NAME, PARAM_PORT, TOOL_BRP_STATUS};
use crate::support::{params, response, schema};

pub fn register_tool() -> Tool {
    Tool {
        name: TOOL_BRP_STATUS.into(),
        description: "Check if a process is running with BRP (Bevy Remote Protocol) enabled. This tool helps diagnose whether a process is running and properly configured with RemotePlugin.".into(),
        input_schema: schema::SchemaBuilder::new()
            .add_string_property(PARAM_APP_NAME, "Name of the process to check for", true)
            .add_number_property(PARAM_PORT, &format!("Port to check for BRP (default: {DEFAULT_BRP_PORT})"), false)
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

    // Check the app
    check_brp_for_app(
        app_name,
        u16::try_from(port).map_err(|_| {
            McpError::invalid_params("Port number must be a valid u16".to_string(), None)
        })?,
    )
    .await
}

async fn check_brp_for_app(app_name: &str, port: u16) -> Result<CallToolResult, McpError> {
    // Check if a process with this name is running using sysinfo
    let mut system = System::new_all();
    system.refresh_processes(sysinfo::ProcessesToUpdate::All, true);

    let running_process = system.processes().values().find(|process| {
        let process_name = process.name().to_string_lossy();
        // Match exact name or with common variations (.exe suffix, etc.)
        process_name == app_name
            || process_name == format!("{app_name}.exe")
            || process_name.strip_suffix(".exe").unwrap_or(&process_name) == app_name
    });

    // Check BRP connectivity
    let brp_responsive = check_brp_on_port(port).await?;

    // Build response based on findings
    let (status, message, app_running, app_pid) = match (running_process, brp_responsive) {
        (Some(process), true) => {
            let pid = process.pid().as_u32();
            (
                "running_with_brp",
                format!(
                    "Process '{app_name}' (PID: {pid}) is running with BRP enabled on port {port}"
                ),
                true,
                Some(pid),
            )
        }
        (Some(process), false) => {
            let pid = process.pid().as_u32();
            (
                "running_no_brp",
                format!(
                    "Process '{app_name}' (PID: {pid}) is running but not responding to BRP on port {port}. Make sure RemotePlugin is added to your Bevy app."
                ),
                true,
                Some(pid),
            )
        }
        (None, true) => {
            // BRP is responding but our specific process isn't found
            (
                "brp_found_process_not_detected",
                format!(
                    "BRP is responding on port {port} but process '{app_name}' not detected. Another process may be using BRP."
                ),
                false,
                None,
            )
        }
        (None, false) => (
            "not_running",
            format!("Process '{app_name}' is not currently running"),
            false,
            None,
        ),
    };

    Ok(response::success_json_response(
        message,
        json!({
            JSON_FIELD_STATUS: status,
            "app_name": app_name,
            JSON_FIELD_PORT: port,
            "app_running": app_running,
            "brp_responsive": brp_responsive,
            "app_pid": app_pid
        }),
    ))
}

/// Check if BRP is responding on the given port
async fn check_brp_on_port(port: u16) -> Result<bool, McpError> {
    // Try a simple BRP request to check connectivity
    let client = reqwest::Client::new();
    let url = format!("http://localhost:{port}");

    // Use bevy/list as a lightweight command using the builder
    let request_body = BrpJsonRpcBuilder::new("bevy/list").build();

    // Set a reasonable timeout
    let response = timeout(
        Duration::from_secs(2),
        client.post(&url).json(&request_body).send(),
    )
    .await;

    match response {
        Ok(Ok(resp)) => {
            // Check if we got a valid JSON-RPC response
            if let Ok(json) = resp.json::<serde_json::Value>().await {
                // A valid BRP response should have jsonrpc field
                Ok(json.get("jsonrpc").is_some())
            } else {
                Ok(false)
            }
        }
        _ => Ok(false),
    }
}
