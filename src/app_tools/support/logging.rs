use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};

use rmcp::Error as McpError;

/// Helper function to create an `McpError` for log file write failures
fn log_write_error<E: std::fmt::Display>(err: E) -> McpError {
    McpError::internal_error(format!("Failed to write to log file: {err}"), None)
}

/// Helper function to create an `McpError` for log file sync failures
fn log_sync_error<E: std::fmt::Display>(err: E) -> McpError {
    McpError::internal_error(format!("Failed to sync log file: {err}"), None)
}

/// Create a log file for a Bevy app launch
pub fn create_log_file(
    app_name: &str,
    profile: &str,
    binary_path: &Path,
    working_dir: &Path,
) -> Result<(PathBuf, File), McpError> {
    // Generate unique log file name in temp directory
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_err(|e| McpError::internal_error(format!("Failed to get timestamp: {e}"), None))?
        .as_millis();
    let log_file_path =
        std::env::temp_dir().join(format!("bevy_brp_mcp_{app_name}_{timestamp}.log"));

    // Create log file
    let mut log_file = File::create(&log_file_path)
        .map_err(|e| McpError::internal_error(format!("Failed to create log file: {e}"), None))?;

    // Write header
    writeln!(log_file, "=== Bevy BRP MCP Launch Log ===").map_err(log_write_error)?;
    writeln!(log_file, "Started at: {:?}", std::time::SystemTime::now())
        .map_err(log_write_error)?;
    writeln!(log_file, "App: {app_name}").map_err(log_write_error)?;
    writeln!(log_file, "Profile: {profile}").map_err(log_write_error)?;
    writeln!(log_file, "Binary: {}", binary_path.display()).map_err(log_write_error)?;
    writeln!(log_file, "Working directory: {}", working_dir.display()).map_err(log_write_error)?;
    writeln!(log_file, "============================================\n")
        .map_err(log_write_error)?;
    log_file.sync_all().map_err(log_sync_error)?;

    Ok((log_file_path, log_file))
}

/// Open an existing log file for appending (for stdout/stderr redirection)
pub fn open_log_file_for_redirect(log_file_path: &Path) -> Result<File, McpError> {
    File::options()
        .append(true)
        .open(log_file_path)
        .map_err(|e| {
            McpError::internal_error(format!("Failed to open log file for redirect: {e}"), None)
        })
}

/// Appends additional text to an existing log file
pub fn append_to_log_file(log_file_path: &Path, content: &str) -> Result<(), McpError> {
    let mut file = File::options()
        .append(true)
        .open(log_file_path)
        .map_err(|e| {
            McpError::internal_error(format!("Failed to open log file for appending: {e}"), None)
        })?;

    write!(file, "{content}").map_err(log_write_error)?;

    file.sync_all().map_err(log_sync_error)?;

    Ok(())
}
