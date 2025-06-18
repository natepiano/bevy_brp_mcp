use rmcp::model::{CallToolResult, Tool};
use rmcp::service::RequestContext;
use rmcp::{Error as McpError, RoleServer};

use super::constants::{
    BRP_METHOD_REPARENT, DEFAULT_BRP_PORT, JSON_FIELD_ENTITIES, JSON_FIELD_PARENT, JSON_FIELD_PORT,
};
use super::support::configurable_formatter::{ConfigurableFormatterFactory, FieldExtractor};
use super::support::generic_handler::{BrpHandlerConfig, PassthroughExtractor, handle_generic};
use crate::BrpMcpService;
use crate::constants::{DESC_BRP_REPARENT, TOOL_BRP_REPARENT};
use crate::support::schema;

pub fn register_tool() -> Tool {
    Tool {
        name:         TOOL_BRP_REPARENT.into(),
        description:  DESC_BRP_REPARENT.into(),
        input_schema: schema::SchemaBuilder::new()
            .add_any_property(JSON_FIELD_ENTITIES, "Array of entity IDs to reparent", true)
            .add_number_property(
                JSON_FIELD_PARENT,
                "The new parent entity ID (omit to remove parent)",
                false,
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
    request: rmcp::model::CallToolRequestParam,
    context: RequestContext<RoleServer>,
) -> Result<CallToolResult, McpError> {
    // Custom extractors for entities and parent
    let entities_extractor: FieldExtractor = |_data, context| {
        context
            .params
            .as_ref()
            .and_then(|p| p.get(JSON_FIELD_ENTITIES))
            .cloned()
            .unwrap_or(serde_json::Value::Null)
    };

    let parent_extractor: FieldExtractor = |_data, context| {
        context
            .params
            .as_ref()
            .and_then(|p| p.get(JSON_FIELD_PARENT))
            .cloned()
            .unwrap_or(serde_json::Value::Null)
    };

    let config = BrpHandlerConfig {
        method:            BRP_METHOD_REPARENT,
        param_extractor:   Box::new(PassthroughExtractor),
        formatter_factory: ConfigurableFormatterFactory::pass_through()
            .with_template("Successfully reparented entities")
            .with_response_field(JSON_FIELD_ENTITIES, entities_extractor)
            .with_response_field(JSON_FIELD_PARENT, parent_extractor)
            .with_error_metadata_field(JSON_FIELD_ENTITIES, entities_extractor)
            .with_error_metadata_field(JSON_FIELD_PARENT, parent_extractor)
            .build(),
    };

    handle_generic(service, request, context, &config).await
}
