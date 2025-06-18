use rmcp::model::CallToolResult;
use rmcp::service::RequestContext;
use rmcp::{Error as McpError, RoleServer};
use serde_json::Value;

use super::config::{BrpHandlerConfig, FormatterContext};
use crate::BrpMcpService;
use crate::brp_tools::support::brp_client::{BrpResult, execute_brp_method};
use crate::brp_tools::support::response_formatter::BrpMetadata;

/// Unified handler for all BRP methods (both static and dynamic)
pub async fn handle_brp_request(
    _service: &BrpMcpService,
    request: rmcp::model::CallToolRequestParam,
    _context: RequestContext<RoleServer>,
    config: &BrpHandlerConfig,
) -> Result<CallToolResult, McpError> {
    // Extract parameters using the configured extractor
    let extracted = config.param_extractor.extract(&request)?;

    // Determine the actual method to call
    let method_name = extracted.method.as_deref().or(config.method).ok_or_else(|| {
        McpError::invalid_params("No method specified for BRP call".to_string(), None)
    })?;

    // Call BRP directly using the new client
    let brp_result = execute_brp_method(method_name, extracted.params.clone(), Some(extracted.port)).await?;

    // Create formatter and metadata
    let formatter_context = FormatterContext {
        params: extracted.params.clone(),
    };
    let formatter = config.formatter_factory.create(formatter_context);

    // Use "brp_execute" for dynamic methods for special error formatting
    let metadata_method = if extracted.method.is_some() {
        "brp_execute"
    } else {
        method_name
    };
    let metadata = BrpMetadata::new(metadata_method, extracted.port);

    // Process response using ResponseFormatter directly
    match brp_result {
        BrpResult::Success(data) => {
            let response_data = data.unwrap_or(Value::Null);
            Ok(formatter.format_success(&response_data, metadata))
        }
        BrpResult::Error(error_info) => Ok(formatter.format_error(error_info, &metadata)),
    }
}
