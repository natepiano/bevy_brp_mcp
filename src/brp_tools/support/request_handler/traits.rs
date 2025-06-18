use rmcp::Error as McpError;
use serde_json::Value;

/// Result of parameter extraction
pub struct ExtractedParams {
    /// The method name for dynamic handlers, None for static
    pub method: Option<String>,
    /// The extracted parameters
    pub params: Option<Value>,
    /// The BRP port to use
    pub port:   u16,
}

/// Unified trait for extracting parameters from a request
pub trait ParamExtractor: Send + Sync {
    /// Extract parameters from the request
    fn extract(
        &self,
        request: &rmcp::model::CallToolRequestParam,
    ) -> Result<ExtractedParams, McpError>;
}
