use rmcp::model::{CallToolRequestParam, CallToolResult, Tool};
use rmcp::service::RequestContext;
use rmcp::{Error as McpError, RoleServer};

use super::constants::{
    BRP_METHOD_INSERT, DEFAULT_BRP_PORT, JSON_FIELD_COMPONENTS, JSON_FIELD_ENTITY, JSON_FIELD_PORT,
    MATH_TYPE_FORMAT_NOTE,
};
use super::support::{
    BrpHandlerConfig, PassthroughExtractor, ResponseFormatterFactory, extractors,
    handle_brp_request,
};
use crate::BrpMcpService;
use crate::constants::{DESC_BRP_INSERT, TOOL_BRP_INSERT};
use crate::support::schema;

pub fn register_tool() -> Tool {
    Tool {
        name: TOOL_BRP_INSERT.into(),
        description: DESC_BRP_INSERT.into(),
        input_schema: schema::SchemaBuilder::new()
            .add_number_property(JSON_FIELD_ENTITY, "The entity ID to insert components into", true)
            .add_any_property(
                JSON_FIELD_COMPONENTS,
                &format!("Object containing component data to insert. Keys are component types, values are component data.{MATH_TYPE_FORMAT_NOTE}" ),
                true
            )
            .add_number_property(JSON_FIELD_PORT, &format!("The BRP port (default: {DEFAULT_BRP_PORT})" ), false)
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
        method:            Some(BRP_METHOD_INSERT),
        param_extractor:   Box::new(PassthroughExtractor),
        formatter_factory: ResponseFormatterFactory::entity_operation(JSON_FIELD_ENTITY)
            .with_template("Successfully inserted components into entity {entity}")
            .with_response_field(JSON_FIELD_ENTITY, extractors::entity_from_params)
            .with_response_field(JSON_FIELD_COMPONENTS, extractors::components_from_params)
            .with_error_metadata_field("requested_components", extractors::components_from_params)
            .build(),
    };

    handle_brp_request(service, request, context, &config).await
}
