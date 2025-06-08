//! Logs tool implementation

use std::env;
use std::fs::File;
use std::io::{BufRead, BufReader, Seek, SeekFrom};

use rmcp::Error as McpError;
use rmcp::model::{CallToolResult, Content};

use crate::constants::DEFAULT_BRP_PORT;
use crate::detached;

/// Get recent log output for a Bevy app
pub async fn get_logs(app_name: String, lines: Option<u32>) -> Result<CallToolResult, McpError> {
    let lines_to_read = lines.unwrap_or(50); // Default to 50 lines

    // Get session info to find log file
    match detached::get_session_info(&app_name, DEFAULT_BRP_PORT).await {
        Ok(Some(session_info)) => {
            // Validate that the log file path is within the temp directory
            let temp_dir = env::temp_dir();
            let canonical_temp = temp_dir.canonicalize().map_err(|e| {
                McpError::internal_error(
                    format!("Failed to canonicalize temp directory: {}", e),
                    None,
                )
            })?;

            let canonical_log = session_info.log_file.canonicalize().map_err(|e| {
                McpError::internal_error(
                    format!("Failed to canonicalize log file path: {}", e),
                    None,
                )
            })?;

            // Ensure the log file is within the temp directory
            if !canonical_log.starts_with(&canonical_temp) {
                return Err(McpError::internal_error(
                    "Log file path is outside of temp directory",
                    None,
                ));
            }

            // Read the log file
            match read_last_lines(&session_info.log_file, lines_to_read as usize) {
                Ok(log_content) => {
                    let mut response = format!(
                        "=== Logs for app '{}' (last {} lines) ===\n",
                        app_name, lines_to_read
                    );
                    response.push_str(&format!("Log file: {:?}\n", session_info.log_file));
                    response.push_str("=====================================\n");
                    response.push_str(&log_content);

                    Ok(CallToolResult::success(vec![Content::text(response)]))
                }
                Err(e) => Err(McpError::internal_error(
                    format!("Failed to read log file: {}", e),
                    None,
                )),
            }
        }
        Ok(None) => Ok(CallToolResult::success(vec![Content::text(format!(
            "No session found for app '{}'. The app may not be running or was started manually.",
            app_name
        ))])),
        Err(e) => Err(McpError::internal_error(
            format!("Failed to get session info: {}", e),
            None,
        )),
    }
}

/// Read the last N lines from a file efficiently
fn read_last_lines(path: &std::path::Path, num_lines: usize) -> std::io::Result<String> {
    use std::collections::VecDeque;

    let mut file = File::open(path)?;
    let file_len = file.metadata()?.len();

    // If the file is small (< 64KB), just read it all
    const SMALL_FILE_THRESHOLD: u64 = 65536;
    if file_len < SMALL_FILE_THRESHOLD {
        let reader = BufReader::new(file);
        let all_lines: Vec<String> = reader.lines().collect::<Result<Vec<_>, _>>()?;
        let start_index = all_lines.len().saturating_sub(num_lines);
        return Ok(all_lines[start_index..].join("\n"));
    }

    // For larger files, use a more efficient approach
    // Read from the end of the file in chunks
    const CHUNK_SIZE: u64 = 8192;
    let mut result_lines = VecDeque::new();
    let mut remaining_bytes = vec![];
    let mut pos = file_len;

    while pos > 0 && result_lines.len() < num_lines {
        // Calculate chunk size and position
        let chunk_size = std::cmp::min(CHUNK_SIZE, pos);
        pos = pos.saturating_sub(chunk_size);

        // Read chunk
        file.seek(SeekFrom::Start(pos))?;
        let mut buffer = vec![0u8; chunk_size as usize];
        std::io::Read::read_exact(&mut file, &mut buffer)?;

        // Prepend any remaining bytes from previous iteration
        if !remaining_bytes.is_empty() {
            buffer.extend_from_slice(&remaining_bytes);
            remaining_bytes.clear();
        }

        // Find lines in the buffer (from end to start)
        let text = String::from_utf8_lossy(&buffer);
        let mut lines: Vec<&str> = text.split('\n').collect();

        // If we're not at the beginning of the file, the first part might be incomplete
        if pos > 0 && !lines.is_empty() {
            remaining_bytes = lines[0].as_bytes().to_vec();
            lines.remove(0);
        }

        // Add lines to result (in reverse order since we're reading backwards)
        for line in lines.into_iter().rev() {
            if result_lines.len() >= num_lines {
                break;
            }
            result_lines.push_front(line.to_string());
        }
    }

    // Convert to Vec and return the last N lines
    let lines: Vec<String> = result_lines.into_iter().collect();
    let start_index = lines.len().saturating_sub(num_lines);
    Ok(lines[start_index..].join("\n"))
}
