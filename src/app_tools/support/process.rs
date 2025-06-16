use std::fs::File;
use std::path::Path;
use std::process::{Command, Stdio};

use rmcp::Error as McpError;

/// Launch a detached process with proper setup
pub fn launch_detached_process(
    mut cmd: Command,
    working_dir: &Path,
    log_file: File,
    process_name: &str,
) -> Result<u32, McpError> {
    // Clone the log file handle for stderr
    let log_file_for_stderr = log_file.try_clone()
        .map_err(|e| McpError::internal_error(
            format!("Failed to clone log file handle: {}", e),
            None
        ))?;
    
    // Set up the command
    cmd.current_dir(working_dir)
        .env("CARGO_MANIFEST_DIR", working_dir)
        .stdin(Stdio::null())  // Important: detach stdin so the child doesn't inherit it
        .stdout(Stdio::from(log_file))
        .stderr(Stdio::from(log_file_for_stderr));
    
    // Spawn the process
    match cmd.spawn() {
        Ok(child) => {
            // Get the process ID
            Ok(child.id())
        }
        Err(e) => {
            Err(McpError::invalid_params(
                format!("Failed to launch '{}': {}", process_name, e),
                None
            ))
        }
    }
}