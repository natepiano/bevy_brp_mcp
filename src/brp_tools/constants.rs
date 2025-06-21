// This file contains constants specific to BRP (Bevy Remote Protocol) operations
// General MCP server constants are in src/constants.rs

// Network/Port Constants
pub const DEFAULT_BRP_PORT: u16 = 15702;

// Response size limits
// Claude's MCP token limit - responses exceeding this will be saved to temp files
// This threshold was determined through testing where Claude hits its processing limit
// around 25,000 tokens and returns "exceeds maximum allowed tokens" errors
pub const MAX_RESPONSE_TOKENS: usize = 25_000;

// Token estimation heuristic - approximately 4 characters per token
// Used to proactively detect when responses would exceed Claude's token limit
pub const CHARS_PER_TOKEN: usize = 4;

// BRP protocol method
pub const BRP_METHOD_LIST: &str = "bevy/list";
pub const BRP_METHOD_GET: &str = "bevy/get";
pub const BRP_METHOD_DESTROY: &str = "bevy/destroy";
pub const BRP_METHOD_SPAWN: &str = "bevy/spawn";
pub const BRP_METHOD_INSERT: &str = "bevy/insert";
pub const BRP_METHOD_REMOVE: &str = "bevy/remove";
pub const BRP_METHOD_LIST_RESOURCES: &str = "bevy/list_resources";
pub const BRP_METHOD_GET_RESOURCE: &str = "bevy/get_resource";
pub const BRP_METHOD_INSERT_RESOURCE: &str = "bevy/insert_resource";
pub const BRP_METHOD_REMOVE_RESOURCE: &str = "bevy/remove_resource";
pub const BRP_METHOD_MUTATE_RESOURCE: &str = "bevy/mutate_resource";
pub const BRP_METHOD_MUTATE_COMPONENT: &str = "bevy/mutate_component";
pub const BRP_METHOD_REGISTRY_SCHEMA: &str = "bevy/registry/schema";
pub const BRP_METHOD_RPC_DISCOVER: &str = "rpc.discover";

// bevy_brp_extras methods
pub const BRP_METHOD_EXTRAS_SHUTDOWN: &str = "bevy_brp_extras/shutdown";
pub const BRP_METHOD_EXTRAS_SCREENSHOT: &str = "bevy_brp_extras/screenshot";

// Response Status Constants
// pub const RESPONSE_STATUS_SUCCESS: &str = "success";
// pub const RESPONSE_STATUS_ERROR: &str = "error";

// JSON Field Name Constants
pub const JSON_FIELD_CODE: &str = "code";
pub const JSON_FIELD_COMPONENT: &str = "component";
pub const JSON_FIELD_COMPONENTS: &str = "components";
pub const JSON_FIELD_COUNT: &str = "count";
pub const JSON_FIELD_DATA: &str = "data";
pub const JSON_FIELD_DEBUG_INFO: &str = "debug_info";
pub const JSON_FIELD_DESTROYED_ENTITY: &str = "destroyed_entity";
pub const JSON_FIELD_ENTITIES: &str = "entities";
pub const JSON_FIELD_ENTITY: &str = "entity";
pub const JSON_FIELD_ERROR_CODE: &str = "error_code";
pub const JSON_FIELD_FORMAT_CORRECTIONS: &str = "format_corrections";
pub const JSON_FIELD_LOG_PATH: &str = "log_path";
pub const JSON_FIELD_METADATA: &str = "metadata";
pub const JSON_FIELD_METHOD: &str = "method";
pub const JSON_FIELD_ORIGINAL_ERROR: &str = "original_error";
pub const JSON_FIELD_PARENT: &str = "parent";
pub const JSON_FIELD_PATH: &str = "path";
pub const JSON_FIELD_PORT: &str = "port";
pub const JSON_FIELD_RESOURCE: &str = "resource";
pub const JSON_FIELD_RESOURCES: &str = "resources";
pub const JSON_FIELD_STATUS: &str = "status";
pub const JSON_FIELD_VALUE: &str = "value";
pub const JSON_FIELD_WATCH_ID: &str = "watch_id";
pub const JSON_FIELD_WATCHES: &str = "watches";

// JSON-RPC Constants
pub const JSONRPC_VERSION: &str = "2.0";
pub const JSONRPC_DEFAULT_ID: u64 = 1;
pub const JSONRPC_FIELD: &str = "jsonrpc";
pub const JSONRPC_FIELD_ID: &str = "id";
pub const JSONRPC_FIELD_METHOD: &str = "method";
pub const JSONRPC_FIELD_PARAMS: &str = "params";

// Macro to include help text files
macro_rules! include_help_text {
    ($file:expr) => {
        include_str!(concat!("../../help_text/", $file))
    };
}

// Parameter name constants
pub const PARAM_PORT: &str = JSON_FIELD_PORT;

// Tool name constants
pub const TOOL_BRP_LIST: &str = "bevy_list";
pub const TOOL_BRP_GET: &str = "bevy_get";
pub const TOOL_BRP_DESTROY: &str = "bevy_destroy";
pub const TOOL_BRP_INSERT: &str = "bevy_insert";
pub const TOOL_BRP_REMOVE: &str = "bevy_remove";
pub const TOOL_BRP_LIST_RESOURCES: &str = "bevy_list_resources";
pub const TOOL_BRP_GET_RESOURCE: &str = "bevy_get_resource";
pub const TOOL_BRP_INSERT_RESOURCE: &str = "bevy_insert_resource";
pub const TOOL_BRP_REMOVE_RESOURCE: &str = "bevy_remove_resource";
pub const TOOL_BRP_MUTATE_RESOURCE: &str = "bevy_mutate_resource";
pub const TOOL_BRP_MUTATE_COMPONENT: &str = "bevy_mutate_component";
pub const TOOL_BRP_RPC_DISCOVER: &str = "bevy_rpc_discover";
pub const TOOL_BRP_STATUS: &str = "brp_status";

// Streaming/watch tool names
pub const TOOL_BRP_GET_WATCH: &str = "brp_get_watch";
pub const TOOL_BRP_LIST_WATCH: &str = "brp_list_watch";
pub const TOOL_BEVY_STOP_WATCH: &str = "bevy_stop_watch";
pub const TOOL_BEVY_LIST_ACTIVE_WATCHES: &str = "bevy_list_active_watches";

// Debug tool name
pub const TOOL_SET_DEBUG_MODE: &str = "set_debug_mode";

// bevy_brp_extras tool names
pub const TOOL_BRP_EXTRAS_SHUTDOWN: &str = "brp_extras_shutdown";
pub const TOOL_BRP_EXTRAS_SCREENSHOT: &str = "brp_extras_screenshot";

// BRP tool descriptions
pub const DESC_BRP_LIST: &str = include_help_text!("brp_tools/brp_list.txt");
pub const DESC_BRP_GET: &str = include_help_text!("brp_tools/brp_get.txt");
pub const DESC_BRP_DESTROY: &str = include_help_text!("brp_tools/brp_destroy.txt");
pub const DESC_BRP_INSERT: &str = include_help_text!("brp_tools/brp_insert.txt");
pub const DESC_BRP_REMOVE: &str = include_help_text!("brp_tools/brp_remove.txt");
pub const DESC_BRP_LIST_RESOURCES: &str = include_help_text!("brp_tools/brp_list_resources.txt");
pub const DESC_BRP_GET_RESOURCE: &str = include_help_text!("brp_tools/brp_get_resource.txt");
pub const DESC_BRP_INSERT_RESOURCE: &str = include_help_text!("brp_tools/brp_insert_resource.txt");
pub const DESC_BRP_REMOVE_RESOURCE: &str = include_help_text!("brp_tools/brp_remove_resource.txt");
pub const DESC_BRP_MUTATE_RESOURCE: &str = include_help_text!("brp_tools/brp_mutate_resource.txt");
pub const DESC_BRP_MUTATE_COMPONENT: &str =
    include_help_text!("brp_tools/brp_mutate_component.txt");
pub const DESC_BRP_RPC_DISCOVER: &str = include_help_text!("brp_tools/brp_rpc_discover.txt");

// Watch tool descriptions
pub const DESC_BRP_GET_WATCH: &str = include_help_text!("brp_tools/brp_get_watch.txt");
pub const DESC_BRP_LIST_WATCH: &str = include_help_text!("brp_tools/brp_list_watch.txt");
pub const DESC_BEVY_STOP_WATCH: &str = include_help_text!("brp_tools/bevy_stop_watch.txt");
pub const DESC_BEVY_LIST_ACTIVE_WATCHES: &str =
    include_help_text!("brp_tools/bevy_list_active_watches.txt");

// bevy_brp_extras tool descriptions
pub const DESC_BEVY_SHUTDOWN: &str = include_help_text!("brp_tools/bevy_shutdown.txt");
pub const DESC_BEVY_SCREENSHOT: &str = include_help_text!("brp_tools/bevy_screenshot.txt");

// Documentation/Help Constants
pub const PORT_DESCRIPTION: &str = "The BRP port (default: 15702)";
