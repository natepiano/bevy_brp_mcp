// Profile constants
pub const PROFILE_DEBUG: &str = "debug";
pub const PROFILE_RELEASE: &str = "release";
pub const DEFAULT_PROFILE: &str = PROFILE_DEBUG;

// Parameter name constants
pub const PARAM_PROFILE: &str = "profile";
pub const PARAM_APP_NAME: &str = "app_name";
pub const PARAM_EXAMPLE_NAME: &str = "example_name";
pub const PARAM_PORT: &str = "port";

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

// BRP Registration Requirements
pub const BRP_REGISTRATION_REQUIREMENTS: &str =
    include_help_text!("brp_tools/brp_registration_requirements.txt");

// Log tool descriptions
pub const DESC_LIST_LOGS: &str = include_help_text!("log_tools/list_logs.txt");
pub const DESC_READ_LOG: &str = include_help_text!("log_tools/read_log.txt");
pub const DESC_CLEANUP_LOGS: &str = include_help_text!("log_tools/cleanup_logs.txt");

// Tool name constants
pub const TOOL_BRP_EXECUTE: &str = "brp_execute";
pub const TOOL_BRP_LIST: &str = "bevy_list";
pub const TOOL_BRP_QUERY: &str = "bevy_query";
pub const TOOL_BRP_STATUS: &str = "brp_status";
pub const TOOL_CLEANUP_LOGS: &str = "cleanup_logs";
pub const TOOL_LIST_BEVY_APPS: &str = "list_bevy_apps";
pub const TOOL_LIST_BEVY_EXAMPLES: &str = "list_bevy_examples";
pub const TOOL_LAUNCH_BEVY_APP: &str = "launch_bevy_app";
pub const TOOL_LAUNCH_BEVY_EXAMPLE: &str = "launch_bevy_example";
pub const TOOL_LIST_LOGS: &str = "list_logs";
pub const TOOL_READ_LOG: &str = "read_log";

// BRP protocol method
pub const BRP_LIST: &str = "bevy/list";
pub const BRP_QUERY: &str = "bevy/query";
