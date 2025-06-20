// This file contains constants specific to BRP (Bevy Remote Protocol) operations
// General MCP server constants are in src/constants.rs

// Network/Port Constants
pub const DEFAULT_BRP_PORT: u16 = 15702;

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

// Response Status Constants
// pub const RESPONSE_STATUS_SUCCESS: &str = "success";
// pub const RESPONSE_STATUS_ERROR: &str = "error";

// JSON Field Name Constants
pub const JSON_FIELD_CODE: &str = "code";
pub const JSON_FIELD_COMPONENT: &str = "component";
pub const JSON_FIELD_COMPONENTS: &str = "components";
pub const JSON_FIELD_COUNT: &str = "count";
pub const JSON_FIELD_DATA: &str = "data";
pub const JSON_FIELD_DESTROYED_ENTITY: &str = "destroyed_entity";
pub const JSON_FIELD_ENTITY: &str = "entity";
pub const JSON_FIELD_ERROR_CODE: &str = "error_code";
pub const JSON_FIELD_LOG_PATH: &str = "log_path";
pub const JSON_FIELD_METADATA: &str = "metadata";
pub const JSON_FIELD_METHOD: &str = "method";
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

// Documentation/Help Constants
pub const PORT_DESCRIPTION: &str = "The BRP port (default: 15702)";
