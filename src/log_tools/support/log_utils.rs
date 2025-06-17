use std::fs;
use std::path::PathBuf;
use std::time::SystemTime;

use rmcp::Error as McpError;
use serde_json::json;

use crate::log_tools::constants::FILE_PATH;

// Constants
pub const LOG_PREFIX: &str = "bevy_brp_mcp_";
pub const LOG_EXTENSION: &str = ".log";

/// Validates if a filename follows the bevy_brp_mcp log naming convention
pub fn is_valid_log_filename(filename: &str) -> bool {
    filename.starts_with(LOG_PREFIX) && filename.ends_with(LOG_EXTENSION)
}

/// Parses a log filename into app name and timestamp components
/// Returns Some((app_name, timestamp_str)) if valid, None otherwise
pub fn parse_log_filename(filename: &str) -> Option<(String, String)> {
    if !is_valid_log_filename(filename) {
        return None;
    }

    let parts: Vec<&str> = filename
        .trim_start_matches(LOG_PREFIX)
        .trim_end_matches(LOG_EXTENSION)
        .rsplitn(2, '_')
        .collect();

    if parts.len() != 2 {
        return None;
    }

    // Parts are reversed due to rsplitn
    let timestamp_str = parts[0].to_string();
    let app_name = parts[1].to_string();

    Some((app_name, timestamp_str))
}

/// Formats bytes into human-readable string with appropriate unit
pub fn format_bytes(bytes: u64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB"];
    let mut size = bytes as f64;
    let mut unit_index = 0;

    while size >= 1024.0 && unit_index < UNITS.len() - 1 {
        size /= 1024.0;
        unit_index += 1;
    }

    if unit_index == 0 {
        format!("{} {}", bytes, UNITS[unit_index])
    } else {
        format!("{:.2} {}", size, UNITS[unit_index])
    }
}

/// Gets the log directory (system temp directory)
pub fn get_log_directory() -> PathBuf {
    std::env::temp_dir()
}

/// Gets the full path for a log file given its filename
pub fn get_log_file_path(filename: &str) -> PathBuf {
    get_log_directory().join(filename)
}

/// Represents a log file entry with metadata
#[derive(Debug, Clone)]
pub struct LogFileEntry {
    pub filename:  String,
    pub app_name:  String,
    pub timestamp: String,
    pub path:      PathBuf,
    pub metadata:  fs::Metadata,
}

impl LogFileEntry {
    /// Converts the entry to a JSON value for API responses
    pub fn to_json(&self) -> serde_json::Value {
        let size = self.metadata.len();
        let modified = self
            .metadata
            .modified()
            .ok()
            .and_then(|t| t.duration_since(SystemTime::UNIX_EPOCH).ok())
            .map(|d| d.as_secs())
            .unwrap_or(0);

        let modified_str = self
            .metadata
            .modified()
            .ok()
            .map(|t| {
                chrono::DateTime::<chrono::Local>::from(t)
                    .format("%Y-%m-%d %H:%M:%S")
                    .to_string()
            })
            .unwrap_or_else(|| "Unknown".to_string());

        let timestamp_value = self.timestamp.parse::<u128>().unwrap_or(0);

        json!({
            "filename": self.filename,
            "app_name": self.app_name,
            "timestamp": timestamp_value,
            "size_bytes": size,
            "size_human": format_bytes(size),
            "last_modified": modified_str,
            "last_modified_timestamp": modified,
            FILE_PATH: self.path.display().to_string(),
        })
    }
}

/// Iterates over log files in the temp directory with optional filtering
/// The filter function receives a LogFileEntry and returns true to include it
pub fn iterate_log_files<F>(filter: F) -> Result<Vec<LogFileEntry>, McpError>
where
    F: Fn(&LogFileEntry) -> bool,
{
    let temp_dir = get_log_directory();
    let mut log_entries = Vec::new();

    // Read the temp directory
    let entries = fs::read_dir(&temp_dir).map_err(|e| {
        McpError::internal_error(format!("Failed to read temp directory: {}", e), None)
    })?;

    // Process each entry
    for entry in entries {
        let entry = entry.map_err(|e| {
            McpError::internal_error(format!("Failed to read directory entry: {}", e), None)
        })?;

        let path = entry.path();
        let filename = path.file_name().and_then(|n| n.to_str()).unwrap_or("");

        // Parse the filename
        if let Some((app_name, timestamp)) = parse_log_filename(filename) {
            // Get file metadata
            let metadata = entry.metadata().map_err(|e| {
                McpError::internal_error(format!("Failed to get file metadata: {}", e), None)
            })?;

            let log_entry = LogFileEntry {
                filename: filename.to_string(),
                app_name,
                timestamp,
                path,
                metadata,
            };

            // Apply filter
            if filter(&log_entry) {
                log_entries.push(log_entry);
            }
        }
    }

    Ok(log_entries)
}
