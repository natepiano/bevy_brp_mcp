// This file contains constants specific to log tool operations
// General MCP server constants are in src/constants.rs

// Macro to include help text files
macro_rules! include_help_text {
    ($file:expr) => {
        include_str!(concat!("../../help_text/", $file))
    };
}

pub const FILE_PATH: &str = "path";

// Tool name constants
pub const TOOL_LIST_LOGS: &str = "list_logs";
pub const TOOL_READ_LOG: &str = "read_log";
pub const TOOL_CLEANUP_LOGS: &str = "cleanup_logs";

// Log tool descriptions
pub const DESC_LIST_LOGS: &str = include_help_text!("log_tools/list_logs.txt");
pub const DESC_READ_LOG: &str = include_help_text!("log_tools/read_log.txt");
pub const DESC_CLEANUP_LOGS: &str = include_help_text!("log_tools/cleanup_logs.txt");
