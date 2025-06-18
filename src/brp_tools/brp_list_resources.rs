use rmcp::model::{CallToolRequestParam, CallToolResult, Tool};
use rmcp::service::RequestContext;
use rmcp::{Error as McpError, RoleServer};

use super::constants::{
    BRP_METHOD_LIST_RESOURCES, DEFAULT_BRP_PORT, JSON_FIELD_COMPONENT_COUNT, JSON_FIELD_PORT,
    JSON_FIELD_RESOURCES,
};
use super::support::{
    BrpHandlerConfig, ResponseFormatterFactory, SimplePortExtractor, extractors, handle_brp_request,
};
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
                &format!("The BRP port (default: {DEFAULT_BRP_PORT})"),
                false,
            )
            .build(),
    }
}

pub async fn handle(
    service: &BrpMcpService,
    request: CallToolRequestParam,
    context: RequestContext<RoleServer>,
) -> Result<CallToolResult, McpError> {
    // Use common array_count extractor for resource count

    let config = BrpHandlerConfig {
        method:            Some(BRP_METHOD_LIST_RESOURCES),
        param_extractor:   Box::new(SimplePortExtractor),
        formatter_factory: ResponseFormatterFactory::list_operation()
            .with_template("Listed resources")
            .with_response_field(JSON_FIELD_RESOURCES, extractors::pass_through_data)
            .with_response_field(JSON_FIELD_COMPONENT_COUNT, extractors::array_count)
            .with_default_error()
            .build(),
    };

    handle_brp_request(service, request, context, &config).await
}
