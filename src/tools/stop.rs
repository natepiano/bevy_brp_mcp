//! Stop tool implementation

use rmcp::Error as McpError;
use rmcp::model::{CallToolResult, Content};

use crate::constants::DEFAULT_BRP_PORT;
use crate::detached;

/// Stop a running Bevy app by name
pub async fn stop(app_name: String) -> Result<CallToolResult, McpError> {
    // Check if we have session info for this app
    match detached::get_session_info(&app_name, DEFAULT_BRP_PORT).await {
        Ok(Some(session_info)) => {
            // Kill the process
            match detached::kill_process(session_info.pid) {
                Ok(()) => {
                    // Clean up session files
                    let session_path =
                        detached::get_session_info_path(&app_name, session_info.port);
                    let _ = std::fs::remove_file(&session_path);

                    Ok(CallToolResult::success(vec![Content::text(format!(
                        "Stopped app '{}' (PID: {}) on port {}",
                        app_name, session_info.pid, session_info.port
                    ))]))
                }
                Err(e) => Err(McpError::internal_error(
                    format!("Failed to stop app '{}': {}", app_name, e),
                    None,
                )),
            }
        }
        Ok(None) => Ok(CallToolResult::success(vec![Content::text(format!(
            "No running session found for app '{}'",
            app_name
        ))])),
        Err(e) => Err(McpError::internal_error(
            format!("Failed to check session info: {}", e),
            None,
        )),
    }
}
