use rmcp::model::{CallToolRequestParam, CallToolResult, Tool};
use rmcp::service::RequestContext;
use rmcp::{Error as McpError, RoleServer};

use super::constants::{
    BRP_METHOD_REMOVE, DEFAULT_BRP_PORT, JSON_FIELD_COMPONENTS, JSON_FIELD_ENTITY, JSON_FIELD_PORT,
};
use super::support::{
    BrpHandlerConfig, PassthroughExtractor, ResponseFormatterFactory, extractors,
    handle_brp_request,
};
use crate::BrpMcpService;
use crate::constants::{DESC_BRP_REMOVE, TOOL_BRP_REMOVE};
use crate::support::schema;

pub fn register_tool() -> Tool {
    Tool {
        name:         TOOL_BRP_REMOVE.into(),
        description:  DESC_BRP_REMOVE.into(),
        input_schema: schema::SchemaBuilder::new()
            .add_number_property(
                JSON_FIELD_ENTITY,
                "The entity ID to remove components from",
                true,
            )
            .add_any_property(
                JSON_FIELD_COMPONENTS,
                "Array of component type names to remove",
                true,
            )
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
    // Use common components_from_params extractor

    let config = BrpHandlerConfig {
        method:            Some(BRP_METHOD_REMOVE),
        param_extractor:   Box::new(PassthroughExtractor),
        formatter_factory: ResponseFormatterFactory::entity_operation(JSON_FIELD_ENTITY)
            .with_template("Successfully removed components from entity {entity}")
            .with_response_field(JSON_FIELD_ENTITY, extractors::entity_from_params)
            .with_response_field("removed_components", extractors::components_from_params)
            .with_error_metadata_field("requested_components", extractors::components_from_params)
            .build(),
    };

    handle_brp_request(service, request, context, &config).await
}
