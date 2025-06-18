use rmcp::model::{CallToolRequestParam, CallToolResult, Tool};
use rmcp::service::RequestContext;
use rmcp::{Error as McpError, RoleServer};

use super::constants::{
    BRP_METHOD_INSERT_RESOURCE, DEFAULT_BRP_PORT, JSON_FIELD_PORT, JSON_FIELD_RESOURCE,
    MATH_TYPE_FORMAT_NOTE,
};
use super::support::{
    BrpHandlerConfig, PassthroughExtractor, ResponseFormatterFactory, extractors,
    handle_brp_request,
};
use crate::BrpMcpService;
use crate::constants::{DESC_BRP_INSERT_RESOURCE, TOOL_BRP_INSERT_RESOURCE};
use crate::support::schema;

pub fn register_tool() -> Tool {
    Tool {
        name:         TOOL_BRP_INSERT_RESOURCE.into(),
        description:  DESC_BRP_INSERT_RESOURCE.into(),
        input_schema: schema::SchemaBuilder::new()
            .add_string_property(
                JSON_FIELD_RESOURCE,
                "The fully-qualified type name of the resource to insert or update",
                true,
            )
            .add_any_property(
                "value",
                &format!("The resource value to insert.{MATH_TYPE_FORMAT_NOTE}"),
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
    let config = BrpHandlerConfig {
        method:            Some(BRP_METHOD_INSERT_RESOURCE),
        param_extractor:   Box::new(PassthroughExtractor),
        formatter_factory: ResponseFormatterFactory::resource_operation(JSON_FIELD_RESOURCE)
            .with_template("Successfully inserted/updated resource '{resource}'")
            .with_response_field(JSON_FIELD_RESOURCE, extractors::resource_from_params)
            .build(),
    };

    handle_brp_request(service, request, context, &config).await
}
