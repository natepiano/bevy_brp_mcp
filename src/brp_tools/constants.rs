// Network/Port Constants
pub const DEFAULT_BRP_PORT: u16 = 15702;

// BRP protocol method
pub const BRP_METHOD_LIST: &str = "bevy/list";
pub const BRP_METHOD_QUERY: &str = "bevy/query";

// Response Status Constants
pub const RESPONSE_STATUS_SUCCESS: &str = "success";
pub const RESPONSE_STATUS_ERROR: &str = "error";

// JSON Field Name Constants
pub const JSON_FIELD_STATUS: &str = "status";
pub const JSON_FIELD_MESSAGE: &str = "message";
pub const JSON_FIELD_DATA: &str = "data";
pub const JSON_FIELD_METADATA: &str = "metadata";
pub const JSON_FIELD_ERROR_CODE: &str = "error_code";

// Query Parameter Field Constants
pub const QUERY_FIELD_COMPONENTS: &str = "components";
pub const QUERY_FIELD_OPTION: &str = "option";
pub const QUERY_FIELD_HAS: &str = "has";
pub const QUERY_FIELD_WITH: &str = "with";
pub const QUERY_FIELD_WITHOUT: &str = "without";

// Additional Response Field Constants
pub const JSON_FIELD_METHOD: &str = "method";
pub const JSON_FIELD_PORT: &str = "port";
pub const JSON_FIELD_ENTITY: &str = "entity";
pub const JSON_FIELD_ENTITY_ID: &str = "entity_id";
pub const JSON_FIELD_COUNT: &str = "count";
pub const JSON_FIELD_ENTITY_COUNT: &str = "entity_count";
pub const JSON_FIELD_QUERY_PARAMS: &str = "query_params";
pub const JSON_FIELD_CODE: &str = "code";
pub const JSON_FIELD_HINT: &str = "hint";
pub const JSON_FIELD_STRICT: &str = "strict";
pub const JSON_FIELD_DATA_LOWERCASE: &str = "data"; // for filter/data params
pub const JSON_FIELD_FILTER: &str = "filter";

// JSON-RPC Constants
pub const JSONRPC_VERSION: &str = "2.0";
pub const JSONRPC_DEFAULT_ID: u64 = 1;
pub const JSONRPC_FIELD: &str = "jsonrpc";
pub const JSONRPC_FIELD_ID: &str = "id";
pub const JSONRPC_FIELD_METHOD: &str = "method";
pub const JSONRPC_FIELD_PARAMS: &str = "params";

// Error Handling Constants
pub const ERROR_CODE_THRESHOLD: i64 = 0;
pub const FALLBACK_JSON: &str = "{}";
pub const DEFAULT_ENTITY_COUNT: usize = 0;
