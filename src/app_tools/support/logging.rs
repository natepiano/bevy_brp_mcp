use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};

use rmcp::Error as McpError;

use crate::error::BrpMcpError;

/// Helper function to create a `BrpMcpError` for log file write failures
fn log_write_error<E: std::fmt::Display>(err: E) -> BrpMcpError {
    BrpMcpError::io_failed("write to log file", std::path::Path::new("<log>"), err)
}

/// Helper function to create a `BrpMcpError` for log file sync failures
fn log_sync_error<E: std::fmt::Display>(err: E) -> BrpMcpError {
    BrpMcpError::io_failed("sync log file", std::path::Path::new("<log>"), err)
}

/// Create a log file for a Bevy app launch
pub fn create_log_file(
    name: &str,
    launch_type: &str,
    profile: &str,
    binary_path: &Path,
    working_dir: &Path,
) -> Result<(PathBuf, File), McpError> {
    // Generate unique log file name in temp directory
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_err(|e| BrpMcpError::failed_to("get timestamp", e))?
        .as_millis();
    let log_file_path = std::env::temp_dir().join(format!("bevy_brp_mcp_{name}_{timestamp}.log"));

    // Create log file
    let mut log_file = File::create(&log_file_path)
        .map_err(|e| BrpMcpError::io_failed("create", &log_file_path, e))?;

    // Write header
    writeln!(log_file, "=== Bevy BRP MCP Launch Log ===")
        .map_err(|e| McpError::from(log_write_error(e)))?;
    writeln!(log_file, "Started at: {:?}", std::time::SystemTime::now())
        .map_err(|e| McpError::from(log_write_error(e)))?;
    writeln!(log_file, "{launch_type}: {name}").map_err(|e| McpError::from(log_write_error(e)))?;
    writeln!(log_file, "Profile: {profile}").map_err(|e| McpError::from(log_write_error(e)))?;
    writeln!(log_file, "Binary: {}", binary_path.display())
        .map_err(|e| McpError::from(log_write_error(e)))?;
    writeln!(log_file, "Working directory: {}", working_dir.display())
        .map_err(|e| McpError::from(log_write_error(e)))?;
    writeln!(log_file, "============================================\n")
        .map_err(|e| McpError::from(log_write_error(e)))?;
    log_file
        .sync_all()
        .map_err(|e| McpError::from(log_sync_error(e)))?;

    Ok((log_file_path, log_file))
}

/// Open an existing log file for appending (for stdout/stderr redirection)
pub fn open_log_file_for_redirect(log_file_path: &Path) -> Result<File, McpError> {
    File::options()
        .append(true)
        .open(log_file_path)
        .map_err(|e| {
            McpError::from(BrpMcpError::io_failed(
                "open log file for redirect",
                log_file_path,
                e,
            ))
        })
}

/// Appends additional text to an existing log file
pub fn append_to_log_file(log_file_path: &Path, content: &str) -> Result<(), McpError> {
    let mut file = File::options()
        .append(true)
        .open(log_file_path)
        .map_err(|e| {
            McpError::from(BrpMcpError::io_failed(
                "open log file for appending",
                log_file_path,
                e,
            ))
        })?;

    write!(file, "{content}").map_err(|e| McpError::from(log_write_error(e)))?;

    file.sync_all()
        .map_err(|e| McpError::from(log_sync_error(e)))?;

    Ok(())
}
