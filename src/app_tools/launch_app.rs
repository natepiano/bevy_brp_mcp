use std::path::PathBuf;
use std::process::Command;

use rmcp::model::{CallToolResult, Tool};
use rmcp::service::RequestContext;
use rmcp::{Error as McpError, RoleServer};
use serde_json::json;

use super::constants::{DESC_LAUNCH_BEVY_APP, PARAM_APP_NAME, TOOL_LAUNCH_BEVY_APP};
use super::support::{logging, process, scanning};
use crate::BrpMcpService;
use crate::constants::{DEFAULT_PROFILE, PARAM_PROFILE, PROFILE_RELEASE};
use crate::support::{params, response, schema, service};

pub fn register_tool() -> Tool {
    Tool {
        name:         TOOL_LAUNCH_BEVY_APP.into(),
        description:  DESC_LAUNCH_BEVY_APP.into(),
        input_schema: schema::SchemaBuilder::new()
            .add_string_property(PARAM_APP_NAME, "Name of the Bevy app to launch", true)
            .add_profile_property()
            .build(),
    }
}

pub async fn handle(
    service: &BrpMcpService,
    request: rmcp::model::CallToolRequestParam,
    context: RequestContext<RoleServer>,
) -> Result<CallToolResult, McpError> {
    service::handle_with_request_and_paths(
        service,
        request,
        context,
        |req, search_paths| async move {
            // Get parameters
            let app_name = params::extract_required_string(&req, PARAM_APP_NAME)?;
            let profile = params::extract_optional_string(&req, PARAM_PROFILE, DEFAULT_PROFILE);

            // Launch the app
            launch_bevy_app(app_name, profile, &search_paths)
        },
    )
    .await
}

pub fn launch_bevy_app(
    app_name: &str,
    profile: &str,
    search_paths: &[PathBuf],
) -> Result<CallToolResult, McpError> {
    // Find the app
    let app = scanning::find_required_app(app_name, search_paths)?;

    // Build the binary path
    let binary_path = app.get_binary_path(profile);

    // Check if the binary exists
    if !binary_path.exists() {
        return Err(McpError::invalid_params(
            format!(
                "Binary not found at {}. Please build the app with 'cargo build{}' first.",
                binary_path.display(),
                if profile == PROFILE_RELEASE {
                    " --release"
                } else {
                    ""
                }
            ),
            None,
        ));
    }

    // Get the manifest directory (parent of Cargo.toml)
    let manifest_dir = app
        .manifest_path
        .parent()
        .ok_or_else(|| McpError::invalid_params("Invalid manifest path", None))?;

    eprintln!("Launching {} from {}", app_name, manifest_dir.display());
    eprintln!("Binary path: {}", binary_path.display());
    eprintln!("Working directory: {}", manifest_dir.display());
    eprintln!("CARGO_MANIFEST_DIR: {}", manifest_dir.display());

    // Create log file
    let (log_file_path, _) =
        logging::create_log_file(app_name, profile, &binary_path, manifest_dir)?;

    // Open log file for stdout/stderr redirection
    let log_file_for_redirect = logging::open_log_file_for_redirect(&log_file_path)?;

    // Launch the binary
    let cmd = Command::new(&binary_path);
    let pid = process::launch_detached_process(cmd, manifest_dir, log_file_for_redirect, app_name)?;

    Ok(response::success_json_response(
        format!("Successfully launched '{app_name}' (PID: {pid})"),
        json!({
            "app_name": app_name,
            "pid": pid,
            "working_directory": manifest_dir.display().to_string(),
            "binary_path": binary_path.display().to_string(),
            "profile": profile,
            "log_file": log_file_path.display().to_string(),
            "status": "running_in_background"
        }),
    ))
}
