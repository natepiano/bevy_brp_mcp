//! Tool constants and descriptions for the Bevy BRP MCP server.
//!
//! This module consolidates all tool names, descriptions, and help text for the MCP server.
//! It provides a single source of truth for all tool-related constants.
//!
//! # Naming Conventions
//!
//! Tool names follow a consistent pattern based on their origin:
//!
//! ## Bevy Remote Protocol (BRP) Tools
//! - **`mcp__brp__bevy_*`** - Direct BRP methods (e.g., `bevy/list` → `mcp__brp__bevy_list`)
//! - **`mcp__brp__brp_extras_*`** - Methods from `bevy_brp_extras` plugin (e.g.,
//!   `brp_extras/shutdown` → `mcp__brp__brp_extras_shutdown`)
//! - **`mcp__brp__brp_*`** - Server-only functionality (e.g., `mcp__brp__brp_status`)
//!
//! ## Application Management Tools
//! - **`mcp__brp__*`** - App discovery and launch tools (e.g., `mcp__brp__list_bevy_apps`)
//!
//! ## Help Text Organization
//!
//! Help text files are organized by category and use simplified names (without the `mcp__brp__`
//! prefix):
//! - `help_text/brp_tools/bevy_list.txt` for `mcp__brp__bevy_list`
//! - `help_text/app_tools/list_bevy_apps.txt` for `mcp__brp__list_bevy_apps`
//! - `help_text/log_tools/list_logs.txt` for `mcp__brp__list_logs`

// Macro to include help text files
macro_rules! include_help_text {
    ($file:expr) => {
        include_str!(concat!("../help_text/", $file))
    };
}

// ============================================================================
// BEVY REMOTE PROTOCOL (BRP) CONSTANTS
// ============================================================================

/// Network/Port Constants
pub const DEFAULT_BRP_PORT: u16 = 15702;

/// Response size limits
pub const MAX_RESPONSE_TOKENS: usize = 20_000;

/// Documentation/Help Constants
pub const PORT_DESCRIPTION: &str = "The BRP port (default: 15702)";

// ============================================================================
// BRP PROTOCOL METHODS
// ============================================================================

/// BRP protocol methods (used internally for JSON-RPC calls)
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
pub const BRP_METHOD_QUERY: &str = "bevy/query";
pub const BRP_METHOD_REPARENT: &str = "bevy/reparent";
pub const BRP_METHOD_GET_WATCH: &str = "bevy/get+watch";
pub const BRP_METHOD_LIST_WATCH: &str = "bevy/list+watch";

/// `bevy_brp_extras` methods
pub const BRP_METHOD_EXTRAS_SHUTDOWN: &str = "brp_extras/shutdown";
pub const BRP_METHOD_EXTRAS_SCREENSHOT: &str = "brp_extras/screenshot";
pub const BRP_METHOD_EXTRAS_DISCOVER_FORMAT: &str = "brp_extras/discover_format";

/// `bevy_brp_extras` prefix
pub const BRP_EXTRAS_PREFIX: &str = "brp_extras/";

// ============================================================================
// MCP TOOL NAMES
// ============================================================================

// -----------------------------------------------------------------------------
// Core BRP Tools (Direct protocol methods)
// -----------------------------------------------------------------------------

pub const TOOL_BEVY_LIST: &str = "mcp__brp__bevy_list";
pub const TOOL_BEVY_GET: &str = "mcp__brp__bevy_get";
pub const TOOL_BEVY_DESTROY: &str = "mcp__brp__bevy_destroy";
pub const TOOL_BEVY_INSERT: &str = "mcp__brp__bevy_insert";
pub const TOOL_BEVY_REMOVE: &str = "mcp__brp__bevy_remove";
pub const TOOL_BEVY_LIST_RESOURCES: &str = "mcp__brp__bevy_list_resources";
pub const TOOL_BEVY_GET_RESOURCE: &str = "mcp__brp__bevy_get_resource";
pub const TOOL_BEVY_INSERT_RESOURCE: &str = "mcp__brp__bevy_insert_resource";
pub const TOOL_BEVY_REMOVE_RESOURCE: &str = "mcp__brp__bevy_remove_resource";
pub const TOOL_BEVY_MUTATE_RESOURCE: &str = "mcp__brp__bevy_mutate_resource";
pub const TOOL_BEVY_MUTATE_COMPONENT: &str = "mcp__brp__bevy_mutate_component";
pub const TOOL_BEVY_RPC_DISCOVER: &str = "mcp__brp__bevy_rpc_discover";
pub const TOOL_BEVY_QUERY: &str = "mcp__brp__bevy_query";
pub const TOOL_BEVY_SPAWN: &str = "mcp__brp__bevy_spawn";
pub const TOOL_BRP_EXECUTE: &str = "mcp__brp__brp_execute";
pub const TOOL_BEVY_REGISTRY_SCHEMA: &str = "mcp__brp__bevy_registry_schema";
pub const TOOL_BEVY_REPARENT: &str = "mcp__brp__bevy_reparent";

// -----------------------------------------------------------------------------
// BRP Extras Tools (bevy_brp_extras plugin methods)
// -----------------------------------------------------------------------------

pub const TOOL_BRP_EXTRAS_SHUTDOWN: &str = "mcp__brp__brp_extras_shutdown";
pub const TOOL_BRP_EXTRAS_SCREENSHOT: &str = "mcp__brp__brp_extras_screenshot";
pub const TOOL_BRP_EXTRAS_DISCOVER_FORMAT: &str = "mcp__brp__brp_extras_discover_format";

// -----------------------------------------------------------------------------
// Server-Only BRP Tools (not direct protocol methods)
// -----------------------------------------------------------------------------

pub const TOOL_BRP_STATUS: &str = "mcp__brp__brp_status";
pub const TOOL_BRP_GET_WATCH: &str = "mcp__brp__brp_get_watch";
pub const TOOL_BRP_LIST_WATCH: &str = "mcp__brp__brp_list_watch";
pub const TOOL_BRP_STOP_WATCH: &str = "mcp__brp__bevy_stop_watch";
pub const TOOL_BRP_LIST_ACTIVE_WATCHES: &str = "mcp__brp__bevy_list_active_watches";
pub const TOOL_BRP_SET_DEBUG_MODE: &str = "mcp__brp__set_debug_mode";

// -----------------------------------------------------------------------------
// Application Management Tools
// -----------------------------------------------------------------------------

pub const TOOL_LIST_BEVY_APPS: &str = "mcp__brp__list_bevy_apps";
pub const TOOL_LIST_BEVY_EXAMPLES: &str = "mcp__brp__list_bevy_examples";
pub const TOOL_LIST_BRP_APPS: &str = "mcp__brp__list_brp_apps";
pub const TOOL_LAUNCH_BEVY_APP: &str = "mcp__brp__launch_bevy_app";
pub const TOOL_LAUNCH_BEVY_EXAMPLE: &str = "mcp__brp__launch_bevy_example";

// -----------------------------------------------------------------------------
// Log Management Tools
// -----------------------------------------------------------------------------

pub const TOOL_LIST_LOGS: &str = "mcp__brp__list_logs";
pub const TOOL_READ_LOG: &str = "mcp__brp__read_log";
pub const TOOL_CLEANUP_LOGS: &str = "mcp__brp__cleanup_logs";

// ============================================================================
// TOOL DESCRIPTIONS
// ============================================================================

// -----------------------------------------------------------------------------
// Core BRP Tool Descriptions
// -----------------------------------------------------------------------------

pub const DESC_BEVY_LIST: &str = include_help_text!("brp_tools/brp_list.txt");
pub const DESC_BEVY_GET: &str = include_help_text!("brp_tools/brp_get.txt");
pub const DESC_BEVY_DESTROY: &str = include_help_text!("brp_tools/brp_destroy.txt");
pub const DESC_BEVY_INSERT: &str = include_help_text!("brp_tools/brp_insert.txt");
pub const DESC_BEVY_REMOVE: &str = include_help_text!("brp_tools/brp_remove.txt");
pub const DESC_BEVY_LIST_RESOURCES: &str = include_help_text!("brp_tools/brp_list_resources.txt");
pub const DESC_BEVY_GET_RESOURCE: &str = include_help_text!("brp_tools/brp_get_resource.txt");
pub const DESC_BEVY_INSERT_RESOURCE: &str = include_help_text!("brp_tools/brp_insert_resource.txt");
pub const DESC_BEVY_REMOVE_RESOURCE: &str = include_help_text!("brp_tools/brp_remove_resource.txt");
pub const DESC_BEVY_MUTATE_RESOURCE: &str = include_help_text!("brp_tools/brp_mutate_resource.txt");
pub const DESC_BEVY_MUTATE_COMPONENT: &str =
    include_help_text!("brp_tools/brp_mutate_component.txt");
pub const DESC_BEVY_RPC_DISCOVER: &str = include_help_text!("brp_tools/brp_rpc_discover.txt");

// -----------------------------------------------------------------------------
// BRP Extras Tool Descriptions
// -----------------------------------------------------------------------------

pub const DESC_BRP_EXTRAS_SHUTDOWN: &str = include_help_text!("brp_tools/brp_extras_shutdown.txt");
pub const DESC_BRP_EXTRAS_SCREENSHOT: &str =
    include_help_text!("brp_tools/brp_extras_screenshot.txt");
pub const DESC_BRP_EXTRAS_DISCOVER_FORMAT: &str =
    include_help_text!("brp_tools/brp_extras_discover_format.txt");

// -----------------------------------------------------------------------------
// Server-Only BRP Tool Descriptions
// -----------------------------------------------------------------------------

pub const DESC_BRP_GET_WATCH: &str = include_help_text!("brp_tools/brp_get_watch.txt");
pub const DESC_BRP_LIST_WATCH: &str = include_help_text!("brp_tools/brp_list_watch.txt");
pub const DESC_BRP_STOP_WATCH: &str = include_help_text!("brp_tools/bevy_stop_watch.txt");
pub const DESC_BRP_LIST_ACTIVE_WATCHES: &str =
    include_help_text!("brp_tools/bevy_list_active_watches.txt");

// -----------------------------------------------------------------------------
// Application Management Tool Descriptions
// -----------------------------------------------------------------------------

pub const DESC_LIST_BEVY_APPS: &str = include_help_text!("app_tools/list_bevy_apps.txt");
pub const DESC_LIST_BEVY_EXAMPLES: &str = include_help_text!("app_tools/list_bevy_examples.txt");
pub const DESC_LIST_BRP_APPS: &str = include_help_text!("app_tools/list_brp_apps.txt");
pub const DESC_LAUNCH_BEVY_APP: &str = include_help_text!("app_tools/launch_bevy_app.txt");
pub const DESC_LAUNCH_BEVY_EXAMPLE: &str = include_help_text!("app_tools/launch_bevy_example.txt");

// -----------------------------------------------------------------------------
// Log Management Tool Descriptions
// -----------------------------------------------------------------------------

pub const DESC_LIST_LOGS: &str = include_help_text!("log_tools/list_logs.txt");
pub const DESC_READ_LOG: &str = include_help_text!("log_tools/read_log.txt");
pub const DESC_CLEANUP_LOGS: &str = include_help_text!("log_tools/cleanup_logs.txt");

// ============================================================================
// PARAMETER CONSTANTS
// ============================================================================

/// Common parameter names
pub const PARAM_PORT: &str = "port";
pub const PARAM_APP_NAME: &str = "app_name";
pub const PARAM_EXAMPLE_NAME: &str = "example_name";

// ============================================================================
// JSON-RPC CONSTANTS
// ============================================================================

/// JSON-RPC protocol constants
pub const JSONRPC_VERSION: &str = "2.0";
pub const JSONRPC_DEFAULT_ID: u64 = 1;
pub const JSONRPC_FIELD: &str = "jsonrpc";
pub const JSONRPC_FIELD_ID: &str = "id";
pub const JSONRPC_FIELD_METHOD: &str = "method";
pub const JSONRPC_FIELD_PARAMS: &str = "params";

// ============================================================================
// LOG TOOL CONSTANTS
// ============================================================================

/// Log tool specific constants
pub const FILE_PATH: &str = "path";

// ============================================================================
// BACKWARD COMPATIBILITY ALIASES
// ============================================================================

// Aliases for old naming conventions to maintain compatibility during migration
pub const DESC_BRP_DESTROY: &str = DESC_BEVY_DESTROY;
pub const DESC_BRP_GET: &str = DESC_BEVY_GET;
pub const DESC_BRP_GET_RESOURCE: &str = DESC_BEVY_GET_RESOURCE;
pub const DESC_BRP_INSERT: &str = DESC_BEVY_INSERT;
pub const DESC_BRP_INSERT_RESOURCE: &str = DESC_BEVY_INSERT_RESOURCE;
pub const DESC_BRP_LIST: &str = DESC_BEVY_LIST;
pub const DESC_BRP_LIST_RESOURCES: &str = DESC_BEVY_LIST_RESOURCES;
pub const DESC_BRP_MUTATE_COMPONENT: &str = DESC_BEVY_MUTATE_COMPONENT;
pub const DESC_BRP_MUTATE_RESOURCE: &str = DESC_BEVY_MUTATE_RESOURCE;
pub const DESC_BRP_REMOVE: &str = DESC_BEVY_REMOVE;
pub const DESC_BRP_REMOVE_RESOURCE: &str = DESC_BEVY_REMOVE_RESOURCE;
pub const DESC_BRP_RPC_DISCOVER: &str = DESC_BEVY_RPC_DISCOVER;
// DESC_BRP_GET_WATCH and DESC_BRP_LIST_WATCH already exist with correct names
pub const DESC_BEVY_STOP_WATCH: &str = DESC_BRP_STOP_WATCH;
pub const DESC_BEVY_LIST_ACTIVE_WATCHES: &str = DESC_BRP_LIST_ACTIVE_WATCHES;
pub const DESC_BEVY_SHUTDOWN: &str = DESC_BRP_EXTRAS_SHUTDOWN;
pub const DESC_BEVY_SCREENSHOT: &str = DESC_BRP_EXTRAS_SCREENSHOT;

// Tool name aliases
pub const TOOL_BRP_DESTROY: &str = TOOL_BEVY_DESTROY;
pub const TOOL_BRP_GET: &str = TOOL_BEVY_GET;
pub const TOOL_BRP_GET_RESOURCE: &str = TOOL_BEVY_GET_RESOURCE;
pub const TOOL_BRP_INSERT: &str = TOOL_BEVY_INSERT;
pub const TOOL_BRP_INSERT_RESOURCE: &str = TOOL_BEVY_INSERT_RESOURCE;
pub const TOOL_BRP_LIST: &str = TOOL_BEVY_LIST;
pub const TOOL_BRP_LIST_RESOURCES: &str = TOOL_BEVY_LIST_RESOURCES;
pub const TOOL_BRP_MUTATE_COMPONENT: &str = TOOL_BEVY_MUTATE_COMPONENT;
pub const TOOL_BRP_MUTATE_RESOURCE: &str = TOOL_BEVY_MUTATE_RESOURCE;
pub const TOOL_BRP_REMOVE: &str = TOOL_BEVY_REMOVE;
pub const TOOL_BRP_REMOVE_RESOURCE: &str = TOOL_BEVY_REMOVE_RESOURCE;
pub const TOOL_BRP_RPC_DISCOVER: &str = TOOL_BEVY_RPC_DISCOVER;
// TOOL_BRP_GET_WATCH and TOOL_BRP_LIST_WATCH already exist with correct names
pub const TOOL_BEVY_STOP_WATCH: &str = TOOL_BRP_STOP_WATCH;
pub const TOOL_BEVY_LIST_ACTIVE_WATCHES: &str = TOOL_BRP_LIST_ACTIVE_WATCHES;
pub const TOOL_SET_DEBUG_MODE: &str = TOOL_BRP_SET_DEBUG_MODE;
