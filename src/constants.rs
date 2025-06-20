// This file contains constants for the main MCP server and cross-cutting concerns
// Module-specific constants are in their respective constants.rs files:
// - App tools: src/app_tools/constants.rs
// - BRP tools: src/brp_tools/constants.rs
// - Log tools: src/log_tools/constants.rs

// Profile constants (used across multiple modules)
pub const PROFILE_DEBUG: &str = "debug";
pub const PROFILE_RELEASE: &str = "release";
pub const DEFAULT_PROFILE: &str = PROFILE_DEBUG;

// Parameter name constants (used across multiple modules)
pub const PARAM_PROFILE: &str = "profile";

// Macro to include help text files (used across all modules)
macro_rules! include_help_text {
    ($file:expr) => {
        include_str!(concat!("../help_text/", $file))
    };
}

// Server info (used by main server)
pub const BEVY_BRP_MCP_INFO: &str = include_help_text!("bevy_brp_mcp_info.txt");
