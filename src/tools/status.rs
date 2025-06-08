//! Status tool implementation

use std::time::SystemTime;

use rmcp::Error as McpError;
use rmcp::model::{CallToolResult, Content};

use crate::app::{AppInfo, AppManager};
use crate::constants::DEFAULT_BRP_PORT;
use crate::detached;

/// Check the status of a Bevy app by name
pub async fn status(app_name: String) -> Result<CallToolResult, McpError> {
    // First check if the app is running
    let app_info = AppManager::resolve(&app_name)
        .await
        .map_err(|e| McpError::internal_error(format!("Failed to resolve app: {}", e), None))?;

    match app_info {
        AppInfo::Running { port } => {
            // App is running, get session info if available
            match detached::get_session_info(&app_name, port).await {
                Ok(Some(session_info)) => {
                    // Calculate uptime
                    let uptime_seconds = SystemTime::now()
                        .duration_since(session_info.start_time)
                        .map(|d| d.as_secs())
                        .unwrap_or(0);

                    let uptime_formatted = format_duration(uptime_seconds);

                    let info = format!(
                        "App Status: RUNNING\n\
                        App Name: {}\n\
                        Process ID: {}\n\
                        Port: {}\n\
                        Log File: {:?}\n\
                        Start Time: {:?}\n\
                        Uptime: {}\n\
                        Process Alive: {}",
                        app_name,
                        session_info.pid,
                        session_info.port,
                        session_info.log_file,
                        session_info.start_time,
                        uptime_formatted,
                        if detached::is_process_alive(session_info.pid) {
                            "Yes"
                        } else {
                            "No"
                        }
                    );

                    Ok(CallToolResult::success(vec![Content::text(info)]))
                }
                Ok(None) => {
                    // App is running but no session info (started manually)
                    let info = format!(
                        "App Status: RUNNING\n\
                        App Name: {}\n\
                        Port: {}\n\
                        Note: No session info found (app may have been started manually)",
                        app_name, port
                    );

                    Ok(CallToolResult::success(vec![Content::text(info)]))
                }
                Err(e) => Err(McpError::internal_error(
                    format!("Failed to get session info: {}", e),
                    None,
                )),
            }
        }
        AppInfo::NotRunning => {
            // Check if we have stale session info
            match detached::get_session_info(&app_name, DEFAULT_BRP_PORT).await {
                Ok(Some(session_info)) => {
                    // We have session info but app is not responding
                    let info = format!(
                        "App Status: NOT RUNNING (stale session found)\n\
                        App Name: {}\n\
                        Last PID: {}\n\
                        Last Port: {}\n\
                        Log File: {:?}\n\
                        Note: Session info exists but app is not responding",
                        app_name, session_info.pid, session_info.port, session_info.log_file
                    );

                    Ok(CallToolResult::success(vec![Content::text(info)]))
                }
                _ => {
                    let info = format!(
                        "App Status: NOT RUNNING\n\
                        App Name: {}\n\
                        No active session found",
                        app_name
                    );

                    Ok(CallToolResult::success(vec![Content::text(info)]))
                }
            }
        }
    }
}

/// Format duration in human-readable format
fn format_duration(seconds: u64) -> String {
    let hours = seconds / 3600;
    let minutes = (seconds % 3600) / 60;
    let secs = seconds % 60;

    if hours > 0 {
        format!("{}h {}m {}s", hours, minutes, secs)
    } else if minutes > 0 {
        format!("{}m {}s", minutes, secs)
    } else {
        format!("{}s", secs)
    }
}
