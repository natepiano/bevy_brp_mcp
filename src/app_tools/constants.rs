// This file contains constants specific to app tool operations
// General MCP server constants are in src/constants.rs

// Macro to include help text files
macro_rules! include_help_text {
    ($file:expr) => {
        include_str!(concat!("../../help_text/", $file))
    };
}

// Parameter name constants
pub const PARAM_APP_NAME: &str = "app_name";
pub const PARAM_EXAMPLE_NAME: &str = "example_name";

// Tool name constants
pub const TOOL_LIST_BEVY_APPS: &str = "list_bevy_apps";
pub const TOOL_LIST_BEVY_EXAMPLES: &str = "list_bevy_examples";
pub const TOOL_LAUNCH_BEVY_APP: &str = "launch_bevy_app";
pub const TOOL_LAUNCH_BEVY_EXAMPLE: &str = "launch_bevy_example";

// App tool descriptions
pub const DESC_LIST_BEVY_APPS: &str = include_help_text!("app_tools/list_bevy_apps.txt");
pub const DESC_LIST_BEVY_EXAMPLES: &str = include_help_text!("app_tools/list_bevy_examples.txt");
pub const DESC_LAUNCH_BEVY_APP: &str = include_help_text!("app_tools/launch_bevy_app.txt");
pub const DESC_LAUNCH_BEVY_EXAMPLE: &str = include_help_text!("app_tools/launch_bevy_example.txt");
