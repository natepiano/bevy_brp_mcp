// This file contains constants for the main MCP server and cross-cutting concerns
// BRP-specific constants are in src/brp_tools/constants.rs

// Profile constants
pub const PROFILE_DEBUG: &str = "debug";
pub const PROFILE_RELEASE: &str = "release";
pub const DEFAULT_PROFILE: &str = PROFILE_DEBUG;

// Re-export DEFAULT_BRP_PORT from brp_tools for use by types.rs
pub use crate::brp_tools::constants::DEFAULT_BRP_PORT;
use crate::brp_tools::constants::JSON_FIELD_PORT;

// Parameter name constants
pub const PARAM_PROFILE: &str = "profile";
pub const PARAM_APP_NAME: &str = "app_name";
pub const PARAM_EXAMPLE_NAME: &str = "example_name";
pub const PARAM_PORT: &str = JSON_FIELD_PORT;

// Macro to include help text files
macro_rules! include_help_text {
    ($file:expr) => {
        include_str!(concat!("../help_text/", $file))
    };
}

// Server info
pub const BEVY_BRP_MCP_INFO: &str = include_help_text!("bevy_brp_mcp_info.txt");

// App tool descriptions
pub const DESC_LIST_BEVY_APPS: &str = include_help_text!("app_tools/list_bevy_apps.txt");
pub const DESC_LIST_BEVY_EXAMPLES: &str = include_help_text!("app_tools/list_bevy_examples.txt");
pub const DESC_LAUNCH_BEVY_APP: &str = include_help_text!("app_tools/launch_bevy_app.txt");
pub const DESC_LAUNCH_BEVY_EXAMPLE: &str = include_help_text!("app_tools/launch_bevy_example.txt");

// BRP tool descriptions
pub const DESC_BRP_LIST: &str = include_help_text!("brp_tools/brp_list.txt");
pub const DESC_BRP_QUERY: &str = include_help_text!("brp_tools/brp_query.txt");
pub const DESC_BRP_GET: &str = include_help_text!("brp_tools/brp_get.txt");
pub const DESC_BRP_DESTROY: &str = include_help_text!("brp_tools/brp_destroy.txt");
pub const DESC_BRP_SPAWN: &str = include_help_text!("brp_tools/brp_spawn.txt");
pub const DESC_BRP_INSERT: &str = include_help_text!("brp_tools/brp_insert.txt");
pub const DESC_BRP_REMOVE: &str = include_help_text!("brp_tools/brp_remove.txt");
pub const DESC_BRP_LIST_RESOURCES: &str = include_help_text!("brp_tools/brp_list_resources.txt");
pub const DESC_BRP_GET_RESOURCE: &str = include_help_text!("brp_tools/brp_get_resource.txt");
pub const DESC_BRP_INSERT_RESOURCE: &str = include_help_text!("brp_tools/brp_insert_resource.txt");
pub const DESC_BRP_REMOVE_RESOURCE: &str = include_help_text!("brp_tools/brp_remove_resource.txt");
pub const DESC_BRP_MUTATE_RESOURCE: &str = include_help_text!("brp_tools/brp_mutate_resource.txt");
pub const DESC_BRP_MUTATE_COMPONENT: &str =
    include_help_text!("brp_tools/brp_mutate_component.txt");
pub const DESC_BRP_REPARENT: &str = include_help_text!("brp_tools/brp_reparent.txt");

// Log tool descriptions
pub const DESC_LIST_LOGS: &str = include_help_text!("log_tools/list_logs.txt");
pub const DESC_READ_LOG: &str = include_help_text!("log_tools/read_log.txt");
pub const DESC_CLEANUP_LOGS: &str = include_help_text!("log_tools/cleanup_logs.txt");

// Tool name constants
pub const TOOL_BRP_EXECUTE: &str = "brp_execute";
pub const TOOL_BRP_LIST: &str = "bevy_list";
pub const TOOL_BRP_QUERY: &str = "bevy_query";
pub const TOOL_BRP_GET: &str = "bevy_get";
pub const TOOL_BRP_DESTROY: &str = "bevy_destroy";
pub const TOOL_BRP_SPAWN: &str = "bevy_spawn";
pub const TOOL_BRP_INSERT: &str = "bevy_insert";
pub const TOOL_BRP_REMOVE: &str = "bevy_remove";
pub const TOOL_BRP_LIST_RESOURCES: &str = "bevy_list_resources";
pub const TOOL_BRP_GET_RESOURCE: &str = "bevy_get_resource";
pub const TOOL_BRP_INSERT_RESOURCE: &str = "bevy_insert_resource";
pub const TOOL_BRP_REMOVE_RESOURCE: &str = "bevy_remove_resource";
pub const TOOL_BRP_MUTATE_RESOURCE: &str = "bevy_mutate_resource";
pub const TOOL_BRP_MUTATE_COMPONENT: &str = "bevy_mutate_component";
pub const TOOL_BRP_REPARENT: &str = "bevy_reparent";
pub const TOOL_BRP_STATUS: &str = "brp_status";
pub const TOOL_CLEANUP_LOGS: &str = "cleanup_logs";
pub const TOOL_LIST_BEVY_APPS: &str = "list_bevy_apps";
pub const TOOL_LIST_BEVY_EXAMPLES: &str = "list_bevy_examples";
pub const TOOL_LAUNCH_BEVY_APP: &str = "launch_bevy_app";
pub const TOOL_LAUNCH_BEVY_EXAMPLE: &str = "launch_bevy_example";
pub const TOOL_LIST_LOGS: &str = "list_logs";
pub const TOOL_READ_LOG: &str = "read_log";
