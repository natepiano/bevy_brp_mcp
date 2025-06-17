use rmcp::model::{CallToolResult, Tool};
use rmcp::service::RequestContext;
use rmcp::{Error as McpError, RoleServer};
use serde_json::{Value, json};

use super::constants::{
    BRP_METHOD_LIST_RESOURCES, DEFAULT_BRP_PORT, JSON_FIELD_COMPONENT_COUNT, JSON_FIELD_DATA,
    JSON_FIELD_MESSAGE, JSON_FIELD_PORT, JSON_FIELD_RESOURCES, JSON_FIELD_STATUS,
    RESPONSE_STATUS_SUCCESS,
};
use super::support::generic_handler::{
    BrpHandlerConfig, FormatterContext, FormatterFactory, SimplePortExtractor, handle_generic,
};
use super::support::response_processor::{
    BrpError, BrpMetadata, BrpResponseFormatter, format_error_default,
};
use super::support::serialization::json_tool_result;
use crate::BrpMcpService;
use crate::constants::{DESC_BRP_LIST_RESOURCES, TOOL_BRP_LIST_RESOURCES};
use crate::support::schema;

pub fn register_tool() -> Tool {
    Tool {
        name:         TOOL_BRP_LIST_RESOURCES.into(),
        description:  DESC_BRP_LIST_RESOURCES.into(),
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
        method:            BRP_METHOD_LIST_RESOURCES,
        param_extractor:   Box::new(SimplePortExtractor),
        formatter_factory: Box::new(ListResourcesFormatterFactory),
    };

    handle_generic(service, request, context, &config).await
}

/// Factory for creating ListResourcesFormatter
struct ListResourcesFormatterFactory;

impl FormatterFactory for ListResourcesFormatterFactory {
    fn create(&self, _context: FormatterContext) -> Box<dyn BrpResponseFormatter> {
        Box::new(ListResourcesFormatter)
    }
}

/// Formatter for bevy/list_resources responses
struct ListResourcesFormatter;

impl BrpResponseFormatter for ListResourcesFormatter {
    fn format_success(&self, data: Value, _metadata: BrpMetadata) -> CallToolResult {
        // Extract resources array
        let resources = data.as_array().cloned().unwrap_or_default();

        let resource_count = resources.len();

        let formatted_data = json!({
            JSON_FIELD_STATUS: RESPONSE_STATUS_SUCCESS,
            JSON_FIELD_MESSAGE: format!("Found {} resource(s)", resource_count),
            JSON_FIELD_DATA: {
                JSON_FIELD_RESOURCES: resources,
                JSON_FIELD_COMPONENT_COUNT: resource_count,
            }
        });

        json_tool_result(&formatted_data)
    }

    fn format_error(&self, error: BrpError, metadata: BrpMetadata) -> CallToolResult {
        format_error_default(error, metadata)
    }
}
