use serde_json::{Value, json};

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
            port:   15702,
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
        self.set_param("entity", json!(entity_id))
    }

    /// Set components parameter (array of component type names)
    pub fn components(self, components: Vec<String>) -> Self {
        self.set_param("components", json!(components))
    }

    /// Set strict parameter
    pub fn strict(self, strict: bool) -> Self {
        self.set_param("strict", json!(strict))
    }

    /// Set resource parameter
    pub fn resource(self, resource: impl Into<String>) -> Self {
        self.set_param("resource", json!(resource.into()))
    }

    /// Set path parameter (for mutation methods)
    pub fn path(self, path: impl Into<String>) -> Self {
        self.set_param("path", json!(path.into()))
    }

    /// Set value parameter (for mutation and resource methods)
    pub fn value(self, value: Value) -> Self {
        self.set_param("value", value)
    }

    /// Set entities parameter (for reparent)
    pub fn entities(self, entities: Vec<u64>) -> Self {
        self.set_param("entities", json!(entities))
    }

    /// Set parent parameter (for reparent)
    pub fn parent(self, parent: u64) -> Self {
        self.set_param("parent", json!(parent))
    }

    /// Set component parameter (singular, for mutation)
    pub fn component(self, component: impl Into<String>) -> Self {
        self.set_param("component", json!(component.into()))
    }

    /// Set data parameter for bevy/query
    pub fn data(self, data: Value) -> Self {
        self.set_param("data", data)
    }

    /// Set filter parameter for bevy/query
    pub fn filter(self, filter: Value) -> Self {
        self.set_param("filter", filter)
    }

    /// Helper method to build query data object
    pub fn query_data(
        self,
        components: Option<Vec<String>>,
        option: Option<Vec<String>>,
        has: Option<Vec<String>>,
    ) -> Self {
        let mut data = json!({});
        if let Some(c) = components {
            data["components"] = json!(c);
        }
        if let Some(o) = option {
            data["option"] = json!(o);
        }
        if let Some(h) = has {
            data["has"] = json!(h);
        }
        self.data(data)
    }

    /// Helper method to build query filter object
    pub fn query_filter(self, with: Option<Vec<String>>, without: Option<Vec<String>>) -> Self {
        let mut filter = json!({});
        if let Some(w) = with {
            filter["with"] = json!(w);
        }
        if let Some(wo) = without {
            filter["without"] = json!(wo);
        }
        self.filter(filter)
    }

    /// Set raw params (for methods with complex structures)
    pub fn params(mut self, params: Value) -> Self {
        self.params = Some(params);
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
            id:     1,
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
            "jsonrpc": "2.0",
            "method": self.method,
            "id": self.id
        });

        if let Some(params) = self.params {
            request["params"] = params;
        } else {
            request["params"] = json!(null);
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
        assert_eq!(params.params, Some(json!({"entity": 123})));
    }

    #[test]
    fn test_brp_request_builder_query() {
        let params = BrpRequestBuilder::new("bevy/query")
            .query_data(
                Some(vec!["Transform".to_string()]),
                None,
                Some(vec!["Name".to_string()]),
            )
            .query_filter(Some(vec!["Camera".to_string()]), None)
            .strict(true)
            .build();

        let expected_params = json!({
            "data": {
                "components": ["Transform"],
                "has": ["Name"]
            },
            "filter": {
                "with": ["Camera"]
            },
            "strict": true
        });

        assert_eq!(params.params, Some(expected_params));
    }

    #[test]
    fn test_brp_json_rpc_builder() {
        let request = BrpJsonRpcBuilder::new("bevy/list")
            .id(42)
            .entity(123)
            .build();

        assert_eq!(request["jsonrpc"], "2.0");
        assert_eq!(request["method"], "bevy/list");
        assert_eq!(request["id"], 42);
        assert_eq!(request["params"]["entity"], 123);
    }
}
