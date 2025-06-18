use rmcp::model::{CallToolRequestParam, CallToolResult, Tool};
use rmcp::service::RequestContext;
use rmcp::{Error as McpError, RoleServer};

use super::constants::{
    BRP_METHOD_GET, DEFAULT_BRP_PORT, JSON_FIELD_COMPONENTS, JSON_FIELD_ENTITY, JSON_FIELD_PORT,
};
use super::support::{
    BrpHandlerConfig, PassthroughExtractor, ResponseFormatterFactory, extractors, handle_request,
};
use crate::BrpMcpService;
use crate::constants::{DESC_BRP_GET, TOOL_BRP_GET};
use crate::support::schema;

pub fn register_tool() -> Tool {
    Tool {
        name: TOOL_BRP_GET.into(),
        description: DESC_BRP_GET.into(),
        input_schema: schema::SchemaBuilder::new()
            .add_number_property(JSON_FIELD_ENTITY, "The entity ID to get component data from", true)
            .add_any_property(
                JSON_FIELD_COMPONENTS,
                "Array of component types to retrieve. Each component must be a fully-qualified type name",
                true
            )
            .add_number_property(JSON_FIELD_PORT, &format!("The BRP port (default: {DEFAULT_BRP_PORT})"), false)
            .build(),
    }
}

pub async fn handle(
    service: &BrpMcpService,
    request: CallToolRequestParam,
    context: RequestContext<RoleServer>,
) -> Result<CallToolResult, McpError> {
    let config = BrpHandlerConfig {
        method:            BRP_METHOD_GET,
        param_extractor:   Box::new(PassthroughExtractor),
        formatter_factory: ResponseFormatterFactory::entity_operation(JSON_FIELD_ENTITY)
            .with_template("Retrieved component data from entity {entity}")
            .with_response_field(JSON_FIELD_ENTITY, extractors::entity_from_params)
            .with_response_field(JSON_FIELD_COMPONENTS, extractors::pass_through_data)
            .with_default_error()
            .build(),
    };

    handle_request(service, request, context, &config).await
}
