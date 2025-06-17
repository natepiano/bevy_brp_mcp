use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

use rmcp::model::{CallToolResult, Tool};
use rmcp::service::RequestContext;
use rmcp::{Error as McpError, RoleServer};
use serde_json::json;

use super::support::log_utils;
use crate::BrpMcpService;
use crate::constants::{DESC_READ_LOG, TOOL_READ_LOG};
use crate::log_tools::constants::FILE_PATH;
use crate::support::{params, response, schema};

pub fn register_tool() -> Tool {
    Tool {
        name:         TOOL_READ_LOG.into(),
        description:  DESC_READ_LOG.into(),
        input_schema: schema::SchemaBuilder::new()
            .add_string_property(
                "filename",
                "The log filename (e.g., bevy_brp_mcp_myapp_1234567890.log)",
                true,
            )
            .add_string_property(
                "keyword",
                "Optional keyword to filter lines (case-insensitive)",
                false,
            )
            .add_number_property(
                "tail_lines",
                "Optional number of lines to read from the end of file",
                false,
            )
            .build(),
    }
}

pub async fn handle(
    _service: &BrpMcpService,
    request: rmcp::model::CallToolRequestParam,
    _context: RequestContext<RoleServer>,
) -> Result<CallToolResult, McpError> {
    // Extract parameters
    let filename = params::extract_required_string(&request, "filename")?;
    let keyword = params::extract_optional_string(&request, "keyword", "");
    let tail_lines = params::extract_optional_number(&request, "tail_lines", 0)? as usize;

    // Validate filename format for security
    if !log_utils::is_valid_log_filename(filename) {
        return Err(McpError::invalid_params(
            "Invalid log filename. Only bevy_brp_mcp log files can be read.",
            None,
        ));
    }

    // Build full path
    let log_path = log_utils::get_log_file_path(filename);

    // Check if file exists
    if !log_path.exists() {
        return Err(McpError::invalid_params(
            format!("Log file not found: {}", filename),
            None,
        ));
    }

    // Read the log file
    let (content, metadata) = read_log_file(&log_path, keyword, tail_lines)?;

    Ok(response::success_json_response(
        format!("Successfully read log file: {}", filename),
        json!({
            "filename": filename,
            FILE_PATH: log_path.display().to_string(),
            "size_bytes": metadata.len(),
            "size_human": log_utils::format_bytes(metadata.len()),
            "lines_read": content.lines().count(),
            "content": content,
            "filtered_by_keyword": !keyword.is_empty(),
            "tail_mode": tail_lines > 0,
        }),
    ))
}

fn read_log_file(
    path: &Path,
    keyword: &str,
    tail_lines: usize,
) -> Result<(String, std::fs::Metadata), McpError> {
    // Get file metadata
    let metadata = std::fs::metadata(path).map_err(|e| {
        McpError::internal_error(format!("Failed to get file metadata: {}", e), None)
    })?;

    // Open the file
    let file = File::open(path)
        .map_err(|e| McpError::internal_error(format!("Failed to open log file: {}", e), None))?;

    let reader = BufReader::new(file);
    let mut lines: Vec<String> = Vec::new();

    // Read lines with optional keyword filtering
    for line_result in reader.lines() {
        let line = line_result
            .map_err(|e| McpError::internal_error(format!("Failed to read line: {}", e), None))?;

        // Apply keyword filter if provided
        if keyword.is_empty() || line.to_lowercase().contains(&keyword.to_lowercase()) {
            lines.push(line);
        }
    }

    // Apply tail mode if requested
    let final_lines = if tail_lines > 0 && tail_lines < lines.len() {
        let skip_amount = lines.len() - tail_lines;
        lines.into_iter().skip(skip_amount).collect()
    } else {
        lines
    };

    let content = final_lines.join("\n");
    Ok((content, metadata))
}
