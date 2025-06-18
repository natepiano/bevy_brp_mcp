use rmcp::model::{CallToolResult, Tool};
use rmcp::service::RequestContext;
use rmcp::{Error as McpError, RoleServer};

use super::constants::{
    BRP_METHOD_INSERT, DEFAULT_BRP_PORT, JSON_FIELD_COMPONENTS, JSON_FIELD_ENTITY, JSON_FIELD_PORT,
};
use super::support::configurable_formatter::{ConfigurableFormatterFactory, extractors};
use super::support::generic_handler::{BrpHandlerConfig, PassthroughExtractor, handle_generic};
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
                "Object containing component data to insert. Keys are component types, values are component data",
                true
            )
            .add_number_property(JSON_FIELD_PORT, &format!("The BRP port (default: {DEFAULT_BRP_PORT})" ), false)
            .build(),
    }
}

pub async fn handle(
    service: &BrpMcpService,
    request: rmcp::model::CallToolRequestParam,
    context: RequestContext<RoleServer>,
) -> Result<CallToolResult, McpError> {
    // Use common components_from_params extractor

    let config = BrpHandlerConfig {
        method:            BRP_METHOD_INSERT,
        param_extractor:   Box::new(PassthroughExtractor),
        formatter_factory: ConfigurableFormatterFactory::entity_operation(JSON_FIELD_ENTITY)
            .with_template("Successfully inserted components into entity {entity}")
            .with_response_field(JSON_FIELD_ENTITY, extractors::entity_from_params)
            .with_response_field(JSON_FIELD_COMPONENTS, extractors::components_from_params)
            .with_error_metadata_field("requested_components", extractors::components_from_params)
            .build(),
    };

    handle_generic(service, request, context, &config).await
}
