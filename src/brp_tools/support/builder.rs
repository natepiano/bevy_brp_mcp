use serde_json::{Value, json};

use crate::brp_tools::constants::{
    DEFAULT_BRP_PORT, JSONRPC_DEFAULT_ID, JSONRPC_FIELD, JSONRPC_FIELD_ID, JSONRPC_FIELD_METHOD,
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

    /// Set the port (default: 15702)
    pub fn port(mut self, port: u16) -> Self {
        self.port = port;
        self
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
