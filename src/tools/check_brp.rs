use std::path::PathBuf;
use std::time::Duration;

use rmcp::model::{CallToolResult, Tool};
use rmcp::service::RequestContext;
use rmcp::{Error as McpError, RoleServer};
use serde_json::json;
use sysinfo::System;
use tokio::time::timeout;

use crate::BrpMcpService;
use crate::constants::{PARAM_APP_NAME, PARAM_PORT, DEFAULT_PROFILE};

use super::support;

pub fn register_tool() -> Tool {
    Tool {
        name: "check_brp".into(),
        description: "Check if a specific Bevy app is running and has BRP (Bevy Remote Protocol) enabled. This tool helps diagnose whether your app is running and properly configured with RemotePlugin.".into(),
        input_schema: support::schema::SchemaBuilder::new()
            .add_string_property(PARAM_APP_NAME, "Name of the Bevy app to check", true)
            .add_number_property(PARAM_PORT, "Port to check for BRP (default: 15702)", false)
            .build(),
    }
}

pub async fn handle(
    service: &BrpMcpService,
    request: rmcp::model::CallToolRequestParam,
    context: RequestContext<RoleServer>,
) -> Result<CallToolResult, McpError> {
    support::service::handle_with_request_and_paths(service, request, context, |req, search_paths| async move {
        // Get parameters
        let app_name = support::params::extract_required_string(&req, PARAM_APP_NAME)?;
        let port = support::params::extract_optional_number(&req, PARAM_PORT, 15702)?;
        
        // Check the app
        check_brp_for_app(app_name, port as u16, &search_paths).await
    }).await
}

async fn check_brp_for_app(
    app_name: &str,
    port: u16,
    search_paths: &[PathBuf],
) -> Result<CallToolResult, McpError> {
    // Find the app info
    let app = support::scanning::find_required_app(app_name, search_paths)?;
    
    // Get the binary path for the default profile 
    let binary_path = app.get_binary_path(DEFAULT_PROFILE);
    
    // Check if this specific app is running using sysinfo
    let mut system = System::new_all();
    system.refresh_processes(sysinfo::ProcessesToUpdate::All, true);
    
    let running_process = system.processes().values().find(|process| {
        process.exe().map(|p| p.to_path_buf()).unwrap_or_default() == binary_path
    });
    
    // Check BRP connectivity
    let brp_responsive = check_brp_on_port(port).await?;
    
    // Build response based on findings
    let (status, message, app_running, app_pid) = match (running_process, brp_responsive) {
        (Some(process), true) => {
            let pid = process.pid().as_u32();
            (
                "running_with_brp",
                format!("App '{}' (PID: {}) is running with BRP enabled on port {}", app_name, pid, port),
                true,
                Some(pid)
            )
        },
        (Some(process), false) => {
            let pid = process.pid().as_u32();
            (
                "running_no_brp",
                format!("App '{}' (PID: {}) is running but not responding to BRP on port {}. Make sure RemotePlugin is added to your Bevy app.", app_name, pid, port),
                true,
                Some(pid)
            )
        },
        (None, true) => {
            // BRP is responding but our specific app isn't found - might be running with different profile
            (
                "brp_found_app_not_detected",
                format!("BRP is responding on port {} but app '{}' process not detected. It may be running with a different build profile.", port, app_name),
                false,
                None
            )
        },
        (None, false) => {
            (
                "not_running",
                format!("App '{}' is not currently running", app_name),
                false,
                None
            )
        }
    };
    
    Ok(support::response::success_json_response(
        message,
        json!({
            "status": status,
            "app_name": app_name,
            "port": port,
            "app_running": app_running,
            "brp_responsive": brp_responsive,
            "app_pid": app_pid
        })
    ))
}

/// Check if BRP is responding on the given port
async fn check_brp_on_port(port: u16) -> Result<bool, McpError> {
    // Try a simple BRP request to check connectivity
    let client = reqwest::Client::new();
    let url = format!("http://localhost:{}", port);
    
    // Use bevy/list as a lightweight command
    let request_body = json!({
        "jsonrpc": "2.0",
        "method": "bevy/list",
        "id": 1,
        "params": null
    });
    
    // Set a reasonable timeout
    let response = timeout(
        Duration::from_secs(2),
        client.post(&url)
            .json(&request_body)
            .send()
    ).await;
    
    match response {
        Ok(Ok(resp)) => {
            // Check if we got a valid JSON-RPC response
            if let Ok(json) = resp.json::<serde_json::Value>().await {
                // A valid BRP response should have jsonrpc field
                Ok(json.get("jsonrpc").is_some())
            } else {
                Ok(false)
            }
        },
        _ => Ok(false)
    }
}