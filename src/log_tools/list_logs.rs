use rmcp::model::CallToolResult;
use rmcp::service::RequestContext;
use rmcp::{Error as McpError, RoleServer};
use serde_json::json;

use super::support::LogFileEntry;
use crate::BrpMcpService;
use crate::log_tools::support;
use crate::support::{params, response};

pub fn handle(
    _service: &BrpMcpService,
    request: &rmcp::model::CallToolRequestParam,
    _context: RequestContext<RoleServer>,
) -> Result<CallToolResult, McpError> {
    // Extract optional app name filter
    let app_name_filter = params::extract_optional_string(request, "app_name", "");

    let logs = list_log_files(app_name_filter)?;

    Ok(response::success_json_response(
        format!("Found {} log files", logs.len()),
        json!({
            "logs": logs,
            "temp_directory": support::get_log_directory().display().to_string(),
        }),
    ))
}

fn list_log_files(app_name_filter: &str) -> Result<Vec<serde_json::Value>, McpError> {
    // Use the iterator to get all log files with optional filter
    let filter = |entry: &LogFileEntry| -> bool {
        app_name_filter.is_empty() || entry.app_name == app_name_filter
    };

    let mut log_entries = support::iterate_log_files(filter)?;

    // Sort by timestamp (newest first)
    log_entries.sort_by(|a, b| {
        let ts_a = a.timestamp.parse::<u128>().unwrap_or(0);
        let ts_b = b.timestamp.parse::<u128>().unwrap_or(0);
        ts_b.cmp(&ts_a)
    });

    // Convert to JSON values
    let json_entries: Vec<serde_json::Value> = log_entries
        .into_iter()
        .map(|entry| entry.to_json())
        .collect();

    Ok(json_entries)
}
