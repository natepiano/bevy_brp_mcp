//! Launch tool implementation

use std::path::PathBuf;
use std::process::Command;
use std::io::{BufRead, BufReader};

use rmcp::Error as McpError;
use rmcp::model::{CallToolResult, Content};

use crate::app::{AppInfo, AppManager};
use crate::cargo;
use crate::constants::DEFAULT_BRP_PORT;

/// Launch a Bevy app by name with optional profile
/// If build_if_missing is true, it will build the app if the binary doesn't exist
pub async fn launch(
    app_name: String,
    profile: Option<String>,
    roots: Vec<PathBuf>,
    build_if_missing: Option<bool>,
) -> Result<CallToolResult, McpError> {
    // First check if already running
    match AppManager::resolve(&app_name).await {
        Ok(AppInfo::Running { port }) => {
            return Ok(CallToolResult::success(vec![Content::text(format!(
                "App '{}' is already running on port {}",
                app_name, port
            ))]));
        }
        Ok(AppInfo::NotRunning) | Err(_) => {
            // Continue to launch
        }
    }

    // Validate and get the profile
    let profile_str = profile
        .as_deref()
        .unwrap_or(crate::constants::DEFAULT_BUILD_PROFILE);

    // Validate profile name to prevent command injection
    if let Err(e) = cargo::validate_profile_name(profile_str) {
        return Err(McpError::invalid_params(
            format!("Invalid profile name: {}", e),
            None,
        ));
    }

    // Find the binary
    let binary_path = match AppManager::find_binary(&app_name, Some(profile_str), &roots) {
        Ok(path) => path,
        Err(e) => {
            let error_msg = e.to_string();

            // Check specific error patterns
            if error_msg.contains("found but not built") {
                // Scenario 2: App found but binary not built
                if build_if_missing == Some(true) {
                    // Build the app
                    let project_path = extract_project_path(&error_msg);
                    build_app(&app_name, profile_str, project_path.as_deref())?;
                    
                    // Try to find the binary again after building
                    match AppManager::find_binary(&app_name, Some(profile_str), &roots) {
                        Ok(path) => path,
                        Err(e) => {
                            return Err(McpError::internal_error(
                                format!("Build succeeded but binary still not found: {}", e),
                                None,
                            ));
                        }
                    }
                } else {
                    // Return error that prompts the client to ask about building
                    return Err(McpError::invalid_params(
                        format!("Binary '{}' not built for profile '{}'. Build and run?", app_name, profile_str),
                        None,
                    ));
                }
            } else if error_msg.contains("does not depend on Bevy") {
                // App exists but is not a Bevy app
                return Err(McpError::invalid_params(
                    format!("App '{}' found but does not depend on Bevy", app_name),
                    None,
                ));
            } else if error_msg.contains("not found in any of the provided roots") || 
                     (error_msg.contains("not found") && !error_msg.contains("found in project") && !error_msg.contains("found but")) {
                // Scenario 1: App not found in workspace
                return Err(McpError::invalid_params(
                    format!("App '{}' not found in workspace", app_name),
                    None,
                ));
            } else {
                // Other errors (including generic "not found")
                return Err(McpError::invalid_params(
                    format!("Failed to find app '{}': {}", app_name, e),
                    None,
                ));
            }
        }
    };

    // Scenario 3: Binary exists → report and launch
    eprintln!(
        "Launching '{}' with profile '{}'...",
        app_name, profile_str
    );

    // Launch the app
    AppManager::launch_app(&app_name, &binary_path)
        .await
        .map_err(|e| McpError::internal_error(format!("Failed to launch app: {}", e), None))?;

    Ok(CallToolResult::success(vec![Content::text(format!(
        "Launched '{}' ({} build) on port {}",
        app_name, profile_str, DEFAULT_BRP_PORT
    ))]))
}

/// Extract project path from error message
fn extract_project_path(error_msg: &str) -> Option<String> {
    error_msg.find("project at ").and_then(|start| {
        let path_start = start + "project at ".len();
        error_msg[path_start..]
            .find(" but")
            .map(|end| error_msg[path_start..path_start + end].to_string())
    })
}

/// Build the app using cargo
fn build_app(app_name: &str, profile: &str, project_path: Option<&str>) -> Result<(), McpError> {
    eprintln!("Building '{}' with profile '{}'...", app_name, profile);
    
    // Prepare the build command
    let mut cmd = Command::new("cargo");
    cmd.arg("build")
        .arg("--profile")
        .arg(profile)
        .arg("--bin")
        .arg(app_name);

    // If we have a project path, run the command there
    if let Some(path) = project_path {
        cmd.current_dir(path);
        eprintln!("Building in directory: {}", path);
    }

    eprintln!("Running: cargo build --profile {} --bin {}", profile, app_name);

    // Execute the build and capture output
    let mut child = cmd
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .map_err(|e| {
            McpError::internal_error(
                format!("Failed to execute cargo build: {}", e),
                None,
            )
        })?;

    // Read output as it comes
    if let Some(stdout) = child.stdout.take() {
        let reader = BufReader::new(stdout);
        for line in reader.lines().map_while(Result::ok) {
            eprintln!("  {}", line);
        }
    }

    // Wait for the process to complete
    let status = child.wait().map_err(|e| {
        McpError::internal_error(
            format!("Failed to wait for cargo build: {}", e),
            None,
        )
    })?;

    if !status.success() {
        // Get stderr if available
        if let Some(mut stderr) = child.stderr.take() {
            let mut stderr_buf = Vec::new();
            use std::io::Read;
            let _ = stderr.read_to_end(&mut stderr_buf);
            eprintln!("Build failed:\n{}", String::from_utf8_lossy(&stderr_buf));
        }
        
        return Err(McpError::internal_error(
            format!("Build failed for '{}' with profile '{}'", app_name, profile),
            None,
        ));
    }

    eprintln!("Build successful!");
    Ok(())
}
