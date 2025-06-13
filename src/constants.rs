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

// Tool descriptions
pub const LIST_BEVY_APPS_DESC: &str = include_help_text!("tools/list_bevy_apps.txt");
pub const LIST_BEVY_EXAMPLES_DESC: &str = include_help_text!("tools/list_bevy_examples.txt");
pub const LAUNCH_BEVY_APP_DESC: &str = include_help_text!("tools/launch_bevy_app.txt");
pub const LAUNCH_BEVY_EXAMPLE_DESC: &str = include_help_text!("tools/launch_bevy_example.txt");