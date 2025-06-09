use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::fs::File;
use std::io::Write;

use rmcp::model::{CallToolResult, Content, Tool};
use rmcp::service::RequestContext;
use rmcp::{Error as McpError, RoleServer};

use crate::BrpMcpService;
use crate::cargo_detector::{BinaryInfo, CargoDetector};
use crate::constants::{PROFILE_DEBUG, PROFILE_RELEASE, DEFAULT_PROFILE, LAUNCH_BEVY_APP_DESC};

pub fn register_tool() -> Tool {
    let mut schema = serde_json::Map::new();
    schema.insert("type".to_string(), "object".into());
    
    let mut properties = serde_json::Map::new();
    
    let mut app_name_schema = serde_json::Map::new();
    app_name_schema.insert("type".to_string(), "string".into());
    app_name_schema.insert("description".to_string(), "Name of the Bevy app to launch".into());
    properties.insert("app_name".to_string(), app_name_schema.into());
    
    let mut profile_schema = serde_json::Map::new();
    profile_schema.insert("type".to_string(), "string".into());
    profile_schema.insert("enum".to_string(), vec![PROFILE_DEBUG, PROFILE_RELEASE].into());
    profile_schema.insert("default".to_string(), DEFAULT_PROFILE.into());
    profile_schema.insert("description".to_string(), "Build profile to use (debug or release)".into());
    properties.insert("profile".to_string(), profile_schema.into());
    
    schema.insert("properties".to_string(), properties.into());
    schema.insert("required".to_string(), vec!["app_name"].into());

    Tool {
        name: "launch_bevy_app".into(),
        description: LAUNCH_BEVY_APP_DESC.into(),
        input_schema: std::sync::Arc::new(schema),
    }
}

pub async fn handle(
    service: &BrpMcpService,
    request: rmcp::model::CallToolRequestParam,
    context: RequestContext<RoleServer>,
) -> Result<CallToolResult, McpError> {
    // Get parameters
    let app_name = request
        .arguments
        .as_ref()
        .and_then(|args| args.get("app_name"))
        .and_then(|v| v.as_str())
        .ok_or_else(|| McpError::invalid_params("Missing required parameter: app_name", None))?;
    
    let profile = request
        .arguments
        .as_ref()
        .and_then(|args| args.get("profile"))
        .and_then(|v| v.as_str())
        .unwrap_or(DEFAULT_PROFILE);
    
    // Fetch current roots
    eprintln!("Fetching current roots from client...");
    if let Err(e) = service.fetch_roots_from_client(context.peer.clone()).await {
        eprintln!("Failed to fetch roots: {}", e);
    }
    let search_paths = service.roots.lock().unwrap().clone();
    
    // Launch the app
    launch_bevy_app(app_name, profile, &search_paths).await
}

/// Find a specific Bevy app by name
pub fn find_app(app_name: &str, search_paths: &[PathBuf]) -> Option<BinaryInfo> {
    for root in search_paths {
        // Check the root itself
        if root.join("Cargo.toml").exists() {
            if let Ok(detector) = CargoDetector::from_path(root) {
                let apps = detector.find_bevy_apps();
                if let Some(app) = apps.into_iter().find(|a| a.name == app_name) {
                    return Some(app);
                }
            }
        }
        
        // Check immediate subdirectories
        if let Ok(entries) = std::fs::read_dir(root) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() && path.join("Cargo.toml").exists() {
                    // Skip hidden directories and target
                    if let Some(name) = path.file_name() {
                        let name_str = name.to_string_lossy();
                        if name_str.starts_with('.') || name_str == "target" {
                            continue;
                        }
                    }
                    
                    if let Ok(detector) = CargoDetector::from_path(&path) {
                        let apps = detector.find_bevy_apps();
                        if let Some(app) = apps.into_iter().find(|a| a.name == app_name) {
                            return Some(app);
                        }
                    }
                }
            }
        }
    }
    None
}

pub async fn launch_bevy_app(
    app_name: &str,
    profile: &str,
    search_paths: &[PathBuf],
) -> Result<CallToolResult, McpError> {
    // Find the app
    let app = find_app(app_name, search_paths)
        .ok_or_else(|| McpError::invalid_params(
            format!("Bevy app '{}' not found in any search path", app_name),
            None
        ))?;
    
    // Build the binary path
    let binary_path = app.workspace_root.join("target").join(profile).join(&app.name);
    
    // Check if the binary exists
    if !binary_path.exists() {
        return Err(McpError::invalid_params(
            format!(
                "Binary not found at {}. Please build the app with 'cargo build{}' first.",
                binary_path.display(),
                if profile == PROFILE_RELEASE { " --release" } else { "" }
            ),
            None
        ));
    }
    
    // Get the manifest directory (parent of Cargo.toml)
    let manifest_dir = app.manifest_path.parent()
        .ok_or_else(|| McpError::invalid_params(
            "Invalid manifest path",
            None
        ))?;
    
    eprintln!("Launching {} from {}", app_name, manifest_dir.display());
    eprintln!("Binary path: {}", binary_path.display());
    eprintln!("Working directory: {}", manifest_dir.display());
    eprintln!("CARGO_MANIFEST_DIR: {}", manifest_dir.display());
    
    // Generate unique log file name in temp directory
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_err(|e| McpError::internal_error(
            format!("Failed to get timestamp: {}", e),
            None
        ))?
        .as_millis();
    let log_file_path = std::env::temp_dir().join(format!("bevy_brp_mcp_{}_{}.log", app_name, timestamp));
    
    // Create log file
    let mut log_file = File::create(&log_file_path)
        .map_err(|e| McpError::internal_error(
            format!("Failed to create log file: {}", e),
            None
        ))?;
    
    writeln!(log_file, "=== Bevy BRP MCP Launch Log ===")
        .map_err(|e| McpError::internal_error(format!("Failed to write to log file: {}", e), None))?;
    writeln!(log_file, "Started at: {:?}", std::time::SystemTime::now())
        .map_err(|e| McpError::internal_error(format!("Failed to write to log file: {}", e), None))?;
    writeln!(log_file, "App: {}", app_name)
        .map_err(|e| McpError::internal_error(format!("Failed to write to log file: {}", e), None))?;
    writeln!(log_file, "Profile: {}", profile)
        .map_err(|e| McpError::internal_error(format!("Failed to write to log file: {}", e), None))?;
    writeln!(log_file, "Binary: {}", binary_path.display())
        .map_err(|e| McpError::internal_error(format!("Failed to write to log file: {}", e), None))?;
    writeln!(log_file, "Working directory: {}", manifest_dir.display())
        .map_err(|e| McpError::internal_error(format!("Failed to write to log file: {}", e), None))?;
    writeln!(log_file, "============================================\n")
        .map_err(|e| McpError::internal_error(format!("Failed to write to log file: {}", e), None))?;
    log_file.sync_all()
        .map_err(|e| McpError::internal_error(format!("Failed to sync log file: {}", e), None))?;
    
    // Open log file for stdout/stderr redirection
    let log_file_for_redirect = File::options()
        .append(true)
        .open(&log_file_path)
        .map_err(|e| McpError::internal_error(
            format!("Failed to open log file for redirect: {}", e),
            None
        ))?;
    
    // Launch the binary with proper working directory and environment
    let mut cmd = Command::new(&binary_path);
    cmd.current_dir(manifest_dir)
        .env("CARGO_MANIFEST_DIR", manifest_dir)
        .stdin(Stdio::null())  // Important: detach stdin so the child doesn't inherit it
        .stdout(Stdio::from(log_file_for_redirect.try_clone().map_err(|e| 
            McpError::internal_error(format!("Failed to clone log file handle: {}", e), None)
        )?))
        .stderr(Stdio::from(log_file_for_redirect));
    
    match cmd.spawn() {
        Ok(child) => {
            // Get the process ID
            let pid = child.id();
            
            // Don't wait for the process - let it run in the background
            let output = format!(
                "Successfully launched '{}' (PID: {})\n\
                Working directory: {}\n\
                Binary: {}\n\
                Profile: {}\n\
                Log file: {}\n\n\
                The application is now running in the background.",
                app_name,
                pid,
                manifest_dir.display(),
                binary_path.display(),
                profile,
                log_file_path.display()
            );
            
            Ok(CallToolResult::success(vec![Content::text(output)]))
        }
        Err(e) => {
            Err(McpError::invalid_params(
                format!("Failed to launch '{}': {}", app_name, e),
                None
            ))
        }
    }
}