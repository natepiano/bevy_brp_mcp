pub const PROFILE_DEBUG: &str = "debug";
pub const PROFILE_RELEASE: &str = "release";
pub const DEFAULT_PROFILE: &str = PROFILE_DEBUG;

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