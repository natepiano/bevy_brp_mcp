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
