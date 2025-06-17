use rmcp::model::{CallToolResult, Tool};
use rmcp::service::RequestContext;
use rmcp::{Error as McpError, RoleServer};
use serde_json::{Value, json};

use super::constants::{
    BRP_METHOD_RPC_DISCOVER, DEFAULT_BRP_PORT, JSON_FIELD_DATA, JSON_FIELD_MESSAGE,
    JSON_FIELD_PORT, JSON_FIELD_STATUS, RESPONSE_STATUS_SUCCESS,
};
use super::support::generic_handler::{
    BrpHandlerConfig, FormatterContext, FormatterFactory, SimplePortExtractor, handle_generic,
};
use super::support::response_processor::{
    BrpError, BrpMetadata, BrpResponseFormatter, format_error_default,
};
use super::support::serialization::json_tool_result;
use crate::BrpMcpService;
use crate::constants::{DESC_BRP_RPC_DISCOVER, TOOL_BRP_RPC_DISCOVER};
use crate::support::schema;

pub fn register_tool() -> Tool {
    Tool {
        name:         TOOL_BRP_RPC_DISCOVER.into(),
        description:  DESC_BRP_RPC_DISCOVER.into(),
        input_schema: schema::SchemaBuilder::new()
            .add_number_property(
                JSON_FIELD_PORT,
                &format!("The BRP port (default: {})", DEFAULT_BRP_PORT),
                false,
            )
            .build(),
    }
}

pub async fn handle(
    service: &BrpMcpService,
    request: rmcp::model::CallToolRequestParam,
    context: RequestContext<RoleServer>,
) -> Result<CallToolResult, McpError> {
    let config = BrpHandlerConfig {
        method:            BRP_METHOD_RPC_DISCOVER,
        param_extractor:   Box::new(SimplePortExtractor),
        formatter_factory: Box::new(RpcDiscoverFormatterFactory),
    };

    handle_generic(service, request, context, &config).await
}

/// Factory for creating RpcDiscoverFormatter
struct RpcDiscoverFormatterFactory;

impl FormatterFactory for RpcDiscoverFormatterFactory {
    fn create(&self, _context: FormatterContext) -> Box<dyn BrpResponseFormatter> {
        Box::new(RpcDiscoverFormatter)
    }
}

/// Formatter for rpc.discover responses
struct RpcDiscoverFormatter;

impl BrpResponseFormatter for RpcDiscoverFormatter {
    fn format_success(&self, data: Value, _metadata: BrpMetadata) -> CallToolResult {
        // Count the number of methods discovered
        let method_count = if let Some(methods) = data.get("methods").and_then(|m| m.as_array()) {
            methods.len()
        } else {
            0
        };

        let formatted_data = json!({
            JSON_FIELD_STATUS: RESPONSE_STATUS_SUCCESS,
            JSON_FIELD_MESSAGE: format!("Discovered {} BRP method(s)", method_count),
            JSON_FIELD_DATA: data,
        });

        json_tool_result(&formatted_data)
    }

    fn format_error(&self, error: BrpError, metadata: BrpMetadata) -> CallToolResult {
        format_error_default(error, metadata)
    }
}
