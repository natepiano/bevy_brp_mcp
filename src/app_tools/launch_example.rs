use std::path::PathBuf;
use std::process::Command;

use rmcp::model::{CallToolResult, Tool};
use rmcp::service::RequestContext;
use rmcp::{Error as McpError, RoleServer};
use serde_json::json;

use super::support::{logging, process, scanning};
use crate::BrpMcpService;
use crate::constants::{DEFAULT_PROFILE, PARAM_EXAMPLE_NAME, PARAM_PROFILE, PROFILE_RELEASE};
use crate::error::BrpMcpError;
use crate::support::{params, response, schema, service};
use crate::tools::{DESC_LAUNCH_BEVY_EXAMPLE, TOOL_LAUNCH_BEVY_EXAMPLE};

pub fn register_tool() -> Tool {
    Tool {
        name:         TOOL_LAUNCH_BEVY_EXAMPLE.into(),
        description:  DESC_LAUNCH_BEVY_EXAMPLE.into(),
        input_schema: schema::SchemaBuilder::new()
            .add_string_property(
                PARAM_EXAMPLE_NAME,
                "Name of the Bevy example to launch",
                true,
            )
            .add_profile_property()
            .build(),
    }
}

pub async fn handle(
    service: &BrpMcpService,
    request: rmcp::model::CallToolRequestParam,
    context: RequestContext<RoleServer>,
) -> Result<CallToolResult, McpError> {
    // Get parameters
    let example_name = params::extract_required_string(&request, PARAM_EXAMPLE_NAME)?;
    let profile = params::extract_optional_string(&request, PARAM_PROFILE, DEFAULT_PROFILE);

    // Fetch current roots
    let search_paths = service::fetch_roots_and_get_paths(service, context).await?;

    // Launch the example
    launch_bevy_example(example_name, profile, &search_paths)
}

pub fn launch_bevy_example(
    example_name: &str,
    profile: &str,
    search_paths: &[PathBuf],
) -> Result<CallToolResult, McpError> {
    // Find the example
    let example = scanning::find_required_example(example_name, search_paths)?;

    // Get the manifest directory (parent of Cargo.toml)
    let manifest_dir = example.manifest_path.parent().ok_or_else(|| -> McpError {
        BrpMcpError::invalid("manifest path", "no parent directory").into()
    })?;

    eprintln!(
        "Launching example {} from package {}",
        example_name, example.package_name
    );
    eprintln!("Working directory: {}", manifest_dir.display());
    eprintln!("Profile: {profile}");

    // Create log file for example output (examples use cargo run, so we pass the command string)
    let cargo_command = format!(
        "cargo run --example {example_name} {}",
        if profile == PROFILE_RELEASE {
            "--release"
        } else {
            ""
        }
    )
    .trim()
    .to_string();

    let (log_file_path, _) = logging::create_log_file(
        example_name,
        profile,
        &PathBuf::from(&cargo_command),
        manifest_dir,
    )?;

    // Add extra info to log file
    logging::append_to_log_file(
        &log_file_path,
        &format!("Package: {}\n", example.package_name),
    )?;

    // Open log file for stdout/stderr redirection
    let log_file_for_redirect = logging::open_log_file_for_redirect(&log_file_path)?;

    // Build cargo command
    let mut cmd = Command::new("cargo");
    cmd.arg("run").arg("--example").arg(example_name);

    // Add profile flag if release
    if profile == PROFILE_RELEASE {
        cmd.arg("--release");
    }

    // Launch the process
    let pid =
        process::launch_detached_process(cmd, manifest_dir, log_file_for_redirect, example_name)?;

    Ok(response::success_json_response(
        format!("Successfully launched example '{example_name}' (PID: {pid})"),
        json!({
            "example_name": example_name,
            "pid": pid,
            "package_name": example.package_name,
            "working_directory": manifest_dir.display().to_string(),
            "profile": profile,
            "log_file": log_file_path.display().to_string(),
            "status": "running_in_background",
            "note": "Cargo will build the example if needed before running"
        }),
    ))
}
