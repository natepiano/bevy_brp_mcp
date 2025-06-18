use rmcp::model::{CallToolResult, Tool};
use rmcp::service::RequestContext;
use rmcp::{Error as McpError, RoleServer};

use super::constants::{
    BRP_METHOD_SPAWN, DEFAULT_BRP_PORT, JSON_FIELD_COMPONENTS, JSON_FIELD_PORT,
    JSON_FIELD_SPAWNED_ENTITY,
};
use super::support::configurable_formatter::{ConfigurableFormatterFactory, FieldExtractor};
use super::support::generic_handler::{BrpHandlerConfig, PassthroughExtractor, handle_generic};
use crate::BrpMcpService;
use crate::constants::{DESC_BRP_SPAWN, TOOL_BRP_SPAWN};
use crate::support::schema;

pub fn register_tool() -> Tool {
    Tool {
        name: TOOL_BRP_SPAWN.into(),
        description: DESC_BRP_SPAWN.into(),
        input_schema: schema::SchemaBuilder::new()
            .add_any_property(
                JSON_FIELD_COMPONENTS,
                "Object containing component data to spawn with. Keys are component types, values are component data",
                false
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
    // Custom extractor for spawned entity ID
    let spawned_entity_extractor: FieldExtractor = |data, _context| {
        data.get("entity")
            .cloned()
            .unwrap_or(serde_json::Value::Number(serde_json::Number::from(0)))
    };

    // Custom extractor for components
    let components_extractor: FieldExtractor =
        |_data, context| context.params.clone().unwrap_or(serde_json::Value::Null);

    let config = BrpHandlerConfig {
        method:            BRP_METHOD_SPAWN,
        param_extractor:   Box::new(PassthroughExtractor),
        formatter_factory: ConfigurableFormatterFactory::entity_operation(
            JSON_FIELD_SPAWNED_ENTITY,
        )
        .with_template("Successfully spawned entity")
        .with_response_field(JSON_FIELD_SPAWNED_ENTITY, spawned_entity_extractor)
        .with_response_field(JSON_FIELD_COMPONENTS, components_extractor)
        .with_error_metadata_field("requested_components", components_extractor)
        .build(),
    };

    handle_generic(service, request, context, &config).await
}
