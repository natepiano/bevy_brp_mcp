use rmcp::model::CallToolResult;
use rmcp::service::RequestContext;
use rmcp::{Error as McpError, RoleServer};
use serde_json::Value;

use super::brp_client::execute_brp_method;
use super::response_processor::{BrpMetadata, BrpResponseFormatter, process_brp_response};
use crate::BrpMcpService;
use crate::brp_tools::constants::{DEFAULT_BRP_PORT, JSON_FIELD_ENTITY, JSON_FIELD_PORT};
use crate::support::params::extract_optional_number;

/// Configuration for a BRP handler
pub struct BrpHandlerConfig {
    /// The BRP method to call
    pub method:            &'static str,
    /// Function to extract and validate parameters
    pub param_extractor:   Box<dyn ParamExtractor>,
    /// Function to create the appropriate formatter
    pub formatter_factory: Box<dyn FormatterFactory>,
}

/// Trait for extracting parameters from a request
pub trait ParamExtractor: Send + Sync {
    /// Extract parameters and return (params, port)
    fn extract(
        &self,
        request: &rmcp::model::CallToolRequestParam,
    ) -> Result<(Option<Value>, u16), McpError>;
}

/// Trait for creating response formatters
pub trait FormatterFactory: Send + Sync {
    /// Create a formatter with the given context
    fn create(&self, context: FormatterContext) -> Box<dyn BrpResponseFormatter>;
}

/// Context passed to formatter factory
#[derive(Debug, Clone)]
pub struct FormatterContext {
    pub params: Option<Value>,
}

/// Generic handler for BRP methods
pub async fn handle_generic(
    _service: &BrpMcpService,
    request: rmcp::model::CallToolRequestParam,
    _context: RequestContext<RoleServer>,
    config: &BrpHandlerConfig,
) -> Result<CallToolResult, McpError> {
    // Extract parameters using the configured extractor
    let (params, port) = config.param_extractor.extract(&request)?;

    // Call BRP directly using the new client
    let brp_result = execute_brp_method(config.method, params.clone(), Some(port)).await?;

    // Create formatter and metadata
    let formatter_context = FormatterContext {
        params: params.clone(),
    };
    let formatter = config.formatter_factory.create(formatter_context);
    let metadata = BrpMetadata::new(config.method, port);

    // Process response using the new BrpResult directly
    process_brp_response(brp_result, formatter, metadata)
}

/// Simple parameter extractor that just extracts port
pub struct SimplePortExtractor;

impl ParamExtractor for SimplePortExtractor {
    fn extract(
        &self,
        request: &rmcp::model::CallToolRequestParam,
    ) -> Result<(Option<Value>, u16), McpError> {
        let port =
            extract_optional_number(request, JSON_FIELD_PORT, DEFAULT_BRP_PORT as u64)? as u16;
        Ok((None, port))
    }
}

/// Parameter extractor that extracts port and passes through all arguments as params
pub struct PassthroughExtractor;

impl ParamExtractor for PassthroughExtractor {
    fn extract(
        &self,
        request: &rmcp::model::CallToolRequestParam,
    ) -> Result<(Option<Value>, u16), McpError> {
        let port =
            extract_optional_number(request, JSON_FIELD_PORT, DEFAULT_BRP_PORT as u64)? as u16;
        let params = request.arguments.clone().map(serde_json::Value::Object);
        Ok((params, port))
    }
}

/// Parameter extractor for entity-based operations (destroy, list)
pub struct EntityParamExtractor {
    /// Whether entity is required
    pub required: bool,
}

impl ParamExtractor for EntityParamExtractor {
    fn extract(
        &self,
        request: &rmcp::model::CallToolRequestParam,
    ) -> Result<(Option<Value>, u16), McpError> {
        use serde_json::json;

        use crate::support::params::{extract_any_value, extract_required_number};

        let port =
            extract_optional_number(request, JSON_FIELD_PORT, DEFAULT_BRP_PORT as u64)? as u16;

        let params = if self.required {
            let entity = extract_required_number(request, JSON_FIELD_ENTITY)?;
            Some(json!({ JSON_FIELD_ENTITY: entity }))
        } else {
            // For optional entity (like in list)
            extract_any_value(request, JSON_FIELD_ENTITY)
                .and_then(serde_json::Value::as_u64)
                .map(|id| json!({ JSON_FIELD_ENTITY: id }))
        };

        Ok((params, port))
    }
}
