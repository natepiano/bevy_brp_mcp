use rmcp::model::{CallToolRequestParam, CallToolResult, Tool};
use rmcp::service::RequestContext;
use rmcp::{Error as McpError, RoleServer};

use super::constants::{
    BRP_METHOD_REMOVE_RESOURCE, DEFAULT_BRP_PORT, JSON_FIELD_PORT, JSON_FIELD_RESOURCE,
};
use super::support::{
    BrpHandlerConfig, PassthroughExtractor, ResponseFormatterFactory, extractors, handle_request,
};
use crate::BrpMcpService;
use crate::constants::{DESC_BRP_REMOVE_RESOURCE, TOOL_BRP_REMOVE_RESOURCE};
use crate::support::schema;

pub fn register_tool() -> Tool {
    Tool {
        name:         TOOL_BRP_REMOVE_RESOURCE.into(),
        description:  DESC_BRP_REMOVE_RESOURCE.into(),
        input_schema: schema::SchemaBuilder::new()
            .add_string_property(
                JSON_FIELD_RESOURCE,
                "The fully-qualified type name of the resource to remove",
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
        method:            BRP_METHOD_REMOVE_RESOURCE,
        param_extractor:   Box::new(PassthroughExtractor),
        formatter_factory: ResponseFormatterFactory::resource_operation(JSON_FIELD_RESOURCE)
            .with_template("Successfully removed resource '{resource}'")
            .with_response_field(JSON_FIELD_RESOURCE, extractors::resource_from_params)
            .build(),
    };

    handle_request(service, request, context, &config).await
}
