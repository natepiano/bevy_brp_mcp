use rmcp::model::{CallToolRequestParam, CallToolResult, Tool};
use rmcp::service::RequestContext;
use rmcp::{Error as McpError, RoleServer};

use super::constants::{
    BRP_METHOD_MUTATE_RESOURCE, DEFAULT_BRP_PORT, JSON_FIELD_PATH, JSON_FIELD_PORT,
    JSON_FIELD_RESOURCE,
};
use super::support::{
    BrpHandlerConfig, FieldExtractor, PassthroughExtractor, ResponseFormatterFactory, extractors,
    handle_request,
};
use crate::BrpMcpService;
use crate::constants::{DESC_BRP_MUTATE_RESOURCE, TOOL_BRP_MUTATE_RESOURCE};
use crate::support::schema;

pub fn register_tool() -> Tool {
    Tool {
        name:         TOOL_BRP_MUTATE_RESOURCE.into(),
        description:  DESC_BRP_MUTATE_RESOURCE.into(),
        input_schema: schema::SchemaBuilder::new()
            .add_string_property(
                JSON_FIELD_RESOURCE,
                "The fully-qualified type name of the resource to mutate",
                true,
            )
            .add_string_property(
                JSON_FIELD_PATH,
                "The path to the field within the resource (e.g., 'settings.volume')",
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
    request: CallToolRequestParam,
    context: RequestContext<RoleServer>,
) -> Result<CallToolResult, McpError> {
    // Custom extractor for path
    let path_extractor: FieldExtractor = |_data, context| {
        context
            .params
            .as_ref()
            .and_then(|p| p.get(JSON_FIELD_PATH))
            .cloned()
            .unwrap_or(serde_json::Value::Null)
    };

    let config = BrpHandlerConfig {
        method:            BRP_METHOD_MUTATE_RESOURCE,
        param_extractor:   Box::new(PassthroughExtractor),
        formatter_factory: ResponseFormatterFactory::resource_operation(JSON_FIELD_RESOURCE)
            .with_template("Successfully mutated field '{path}' in resource '{resource}'")
            .with_response_field(JSON_FIELD_RESOURCE, extractors::resource_from_params)
            .with_response_field(JSON_FIELD_PATH, path_extractor)
            .with_error_metadata_field(JSON_FIELD_PATH, path_extractor)
            .build(),
    };

    handle_request(service, request, context, &config).await
}
