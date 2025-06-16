use serde_json::{Value, json};

use crate::brp_tools::constants::{
    DEFAULT_BRP_PORT, JSON_FIELD_DATA_LOWERCASE, JSON_FIELD_ENTITY, JSON_FIELD_FILTER,
    JSON_FIELD_STRICT, JSONRPC_DEFAULT_ID, JSONRPC_FIELD, JSONRPC_FIELD_ID, JSONRPC_FIELD_METHOD,
    JSONRPC_FIELD_PARAMS, JSONRPC_VERSION,
};
use crate::types::BrpExecuteParams;

/// Builder for constructing BrpExecuteParams used by wrapper tools
///
/// This builder provides a fluent API for creating BRP requests following
/// the exact parameter names from the BRP specification.
pub struct BrpRequestBuilder {
    method: String,
    params: Option<Value>,
    port:   u16,
}

impl BrpRequestBuilder {
    /// Create a new builder for the specified BRP method
    pub fn new(method: impl Into<String>) -> Self {
        Self {
            method: method.into(),
            params: None,
            port:   DEFAULT_BRP_PORT,
        }
    }

    /// Helper method to set a parameter value
    fn set_param(mut self, key: &str, value: Value) -> Self {
        let mut params = self.params.take().unwrap_or_else(|| json!({}));
        params[key] = value;
        self.params = Some(params);
        self
    }

    /// Set the port (default: 15702)
    pub fn port(mut self, port: u16) -> Self {
        self.port = port;
        self
    }

    /// Set an entity parameter
    pub fn entity(self, entity_id: u64) -> Self {
        self.set_param(JSON_FIELD_ENTITY, json!(entity_id))
    }

    /// Set strict parameter
    pub fn strict(self, strict: bool) -> Self {
        self.set_param(JSON_FIELD_STRICT, json!(strict))
    }

    /// Set data parameter for bevy/query
    pub fn data(self, data: Value) -> Self {
        self.set_param(JSON_FIELD_DATA_LOWERCASE, data)
    }

    /// Set filter parameter for bevy/query
    pub fn filter(self, filter: Value) -> Self {
        self.set_param(JSON_FIELD_FILTER, filter)
    }

    /// Build the final BrpExecuteParams
    pub fn build(self) -> BrpExecuteParams {
        BrpExecuteParams {
            method: self.method,
            params: self.params,
            port:   self.port,
        }
    }
}

/// Builder for constructing raw JSON-RPC 2.0 requests
///
/// This builder is used for direct HTTP communication with BRP,
/// primarily by the check_brp tool.
pub struct BrpJsonRpcBuilder {
    method: String,
    params: Option<Value>,
    id:     u64,
}

impl BrpJsonRpcBuilder {
    /// Create a new JSON-RPC request builder
    pub fn new(method: impl Into<String>) -> Self {
        Self {
            method: method.into(),
            params: None,
            id:     JSONRPC_DEFAULT_ID,
        }
    }

    /// Set raw params
    pub fn params(mut self, params: Value) -> Self {
        self.params = Some(params);
        self
    }

    /// Build the final JSON-RPC request
    pub fn build(self) -> Value {
        let mut request = json!({
            JSONRPC_FIELD: JSONRPC_VERSION,
            JSONRPC_FIELD_METHOD: self.method,
            JSONRPC_FIELD_ID: self.id
        });

        if let Some(params) = self.params {
            request[JSONRPC_FIELD_PARAMS] = params;
        } else {
            request[JSONRPC_FIELD_PARAMS] = json!(null);
        }

        request
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_brp_request_builder_basic() {
        let params = BrpRequestBuilder::new("bevy/list").port(8080).build();

        assert_eq!(params.method, "bevy/list");
        assert_eq!(params.port, 8080);
        assert!(params.params.is_none());
    }

    #[test]
    fn test_brp_request_builder_with_entity() {
        let params = BrpRequestBuilder::new("bevy/list").entity(123).build();

        assert_eq!(params.method, "bevy/list");
        assert_eq!(params.params, Some(json!({JSON_FIELD_ENTITY: 123})));
    }

    #[test]
    fn test_brp_json_rpc_builder() {
        let request = BrpJsonRpcBuilder::new("bevy/list")
            .params(json!({JSON_FIELD_ENTITY: 123}))
            .build();

        assert_eq!(request[JSONRPC_FIELD], JSONRPC_VERSION);
        assert_eq!(request[JSONRPC_FIELD_METHOD], "bevy/list");
        assert_eq!(request[JSONRPC_FIELD_ID], JSONRPC_DEFAULT_ID);
        assert_eq!(request[JSONRPC_FIELD_PARAMS][JSON_FIELD_ENTITY], 123);
    }
}
