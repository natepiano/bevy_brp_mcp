//! Cleanup tool implementation

use std::collections::HashSet;
use std::env;

use rmcp::Error as McpError;
use rmcp::model::{CallToolResult, Content};

use crate::detached::{SessionInfo, get_session_prefix, is_process_alive};

/// Clean up session log files and info files
pub async fn clear_logs() -> Result<CallToolResult, McpError> {
    let temp_dir = env::temp_dir();
    let mut cleaned_count = 0;
    let mut preserved_count = 0;
    let mut error_count = 0;
    let session_prefix = get_session_prefix();
    let mut active_session_files = HashSet::new();

    let mut output = String::new();
    output.push_str("=== Clearing Bevy BRP MCP Log Files ===\n\n");

    // First pass: identify active sessions by reading all JSON files
    match std::fs::read_dir(&temp_dir) {
        Ok(entries) => {
            for entry in entries.flatten() {
                let path = entry.path();
                if let Some(file_name) = path.file_name() {
                    let file_name_str = file_name.to_string_lossy();

                    // Check if it's a session info JSON file
                    if file_name_str.starts_with(session_prefix) && file_name_str.ends_with(".json")
                    {
                        // Try to read and parse the session info
                        match std::fs::read_to_string(&path) {
                            Ok(contents) => {
                                match serde_json::from_str::<SessionInfo>(&contents) {
                                    Ok(session_info) => {
                                        // Check if the process is still alive
                                        if is_process_alive(session_info.pid) {
                                            // This is an active session - preserve its files
                                            active_session_files.insert(path.clone());
                                            active_session_files
                                                .insert(session_info.log_file.clone());
                                            output.push_str(&format!(
                                                "Found active session: {} (PID: {}, Port: {})\n",
                                                session_info.app_name,
                                                session_info.pid,
                                                session_info.port
                                            ));
                                        }
                                    }
                                    Err(e) => {
                                        eprintln!(
                                            "Failed to parse session info from {}: {}",
                                            file_name_str, e
                                        );
                                    }
                                }
                            }
                            Err(e) => {
                                eprintln!("Failed to read {}: {}", file_name_str, e);
                            }
                        }
                    }
                }
            }
        }
        Err(e) => {
            return Err(McpError::internal_error(
                format!("Failed to read temp directory: {}", e),
                None,
            ));
        }
    }

    output.push('\n');

    // Second pass: clean up files that don't belong to active sessions
    match std::fs::read_dir(&temp_dir) {
        Ok(entries) => {
            for entry in entries.flatten() {
                let path = entry.path();
                if let Some(file_name) = path.file_name() {
                    let file_name_str = file_name.to_string_lossy();

                    // Check if it's one of our session files (either .log or .json)
                    if file_name_str.starts_with(session_prefix)
                        && (file_name_str.ends_with(".log") || file_name_str.ends_with(".json"))
                    {
                        if active_session_files.contains(&path) {
                            // This file belongs to an active session - preserve it
                            let file_type = if file_name_str.ends_with(".log") {
                                "log file"
                            } else {
                                "session info"
                            };

                            // Get file size
                            let size = match path.metadata() {
                                Ok(metadata) => format_file_size(metadata.len()),
                                Err(_) => "unknown size".to_string(),
                            };

                            output.push_str(&format!(
                                "Preserving active {}: {} ({})\n",
                                file_type, file_name_str, size
                            ));
                            preserved_count += 1;
                        } else {
                            // This file doesn't belong to an active session - remove it
                            match std::fs::remove_file(&path) {
                                Ok(_) => {
                                    let file_type = if file_name_str.ends_with(".log") {
                                        "log file"
                                    } else {
                                        "session info"
                                    };
                                    output.push_str(&format!(
                                        "Removed inactive {}: {}\n",
                                        file_type, file_name_str
                                    ));
                                    cleaned_count += 1;
                                }
                                Err(e) => {
                                    output.push_str(&format!(
                                        "Failed to remove {}: {}\n",
                                        file_name_str, e
                                    ));
                                    error_count += 1;
                                }
                            }
                        }
                    }
                }
            }
        }
        Err(e) => {
            return Err(McpError::internal_error(
                format!("Failed to read temp directory: {}", e),
                None,
            ));
        }
    }

    output.push_str("\n=== Cleanup Summary ===\n");

    if cleaned_count == 0 && error_count == 0 && preserved_count == 0 {
        output.push_str("No Bevy BRP MCP session files found.\n");
    } else {
        if cleaned_count > 0 {
            output.push_str(&format!("Removed {} inactive files\n", cleaned_count));
        }
        if preserved_count > 0 {
            output.push_str(&format!(
                "Preserved {} active session files\n",
                preserved_count
            ));
        }
        if error_count > 0 {
            output.push_str(&format!(
                "Failed to remove {} files (see errors above)\n",
                error_count
            ));
        }
    }

    Ok(CallToolResult::success(vec![Content::text(output)]))
}

/// Format file size in human-readable format
fn format_file_size(bytes: u64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB"];
    let mut size = bytes as f64;
    let mut unit_index = 0;

    while size >= 1024.0 && unit_index < UNITS.len() - 1 {
        size /= 1024.0;
        unit_index += 1;
    }

    if unit_index == 0 {
        format!("{} {}", size as u64, UNITS[unit_index])
    } else {
        format!("{:.1} {}", size, UNITS[unit_index])
    }
}
