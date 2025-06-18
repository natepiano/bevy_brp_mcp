use rmcp::model::{CallToolResult, Tool};
use rmcp::service::RequestContext;
use rmcp::{Error as McpError, RoleServer};

use super::constants::{
    BRP_METHOD_MUTATE_COMPONENT, DEFAULT_BRP_PORT, JSON_FIELD_COMPONENT, JSON_FIELD_ENTITY,
    JSON_FIELD_PATH, JSON_FIELD_PORT,
};
use super::support::configurable_formatter::{
    ConfigurableFormatterFactory, FieldExtractor, extractors,
};
use super::support::generic_handler::{BrpHandlerConfig, PassthroughExtractor, handle_generic};
use crate::BrpMcpService;
use crate::constants::{DESC_BRP_MUTATE_COMPONENT, TOOL_BRP_MUTATE_COMPONENT};
use crate::support::schema;

pub fn register_tool() -> Tool {
    Tool {
        name:         TOOL_BRP_MUTATE_COMPONENT.into(),
        description:  DESC_BRP_MUTATE_COMPONENT.into(),
        input_schema: schema::SchemaBuilder::new()
            .add_number_property(
                JSON_FIELD_ENTITY,
                "The entity ID containing the component to mutate",
                true,
            )
            .add_string_property(
                JSON_FIELD_COMPONENT,
                "The fully-qualified type name of the component to mutate",
                true,
            )
            .add_string_property(
                JSON_FIELD_PATH,
                "The path to the field within the component (e.g., 'translation.x')",
                true,
            )
            .add_any_property("value", "The new value for the field", true)
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
    // Custom extractors for component and path
    let component_extractor: FieldExtractor = |_data, context| {
        context
            .params
            .as_ref()
            .and_then(|p| p.get(JSON_FIELD_COMPONENT))
            .cloned()
            .unwrap_or(serde_json::Value::Null)
    };

    let path_extractor: FieldExtractor = |_data, context| {
        context
            .params
            .as_ref()
            .and_then(|p| p.get(JSON_FIELD_PATH))
            .cloned()
            .unwrap_or(serde_json::Value::Null)
    };

    let config = BrpHandlerConfig {
        method:            BRP_METHOD_MUTATE_COMPONENT,
        param_extractor:   Box::new(PassthroughExtractor),
        formatter_factory: ConfigurableFormatterFactory::entity_operation(JSON_FIELD_ENTITY)
            .with_template(
                "Successfully mutated field '{path}' in component '{component}' on entity {entity}",
            )
            .with_response_field(JSON_FIELD_ENTITY, extractors::entity_from_params)
            .with_response_field(JSON_FIELD_COMPONENT, component_extractor)
            .with_response_field(JSON_FIELD_PATH, path_extractor)
            .with_error_metadata_field(JSON_FIELD_COMPONENT, component_extractor)
            .with_error_metadata_field(JSON_FIELD_PATH, path_extractor)
            .build(),
    };

    handle_generic(service, request, context, &config).await
}
