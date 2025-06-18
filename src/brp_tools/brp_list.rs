use rmcp::model::{CallToolRequestParam, CallToolResult, Tool};
use rmcp::service::RequestContext;
use rmcp::{Error as McpError, RoleServer};

use super::constants::{
    BRP_METHOD_LIST, DEFAULT_BRP_PORT, JSON_FIELD_COMPONENT_COUNT, JSON_FIELD_DATA,
    JSON_FIELD_ENTITY, JSON_FIELD_PORT,
};
use super::support::{
    BrpHandlerConfig, EntityParamExtractor, ResponseFormatterFactory, extractors,
    handle_brp_request,
};
use crate::BrpMcpService;
use crate::constants::{DESC_BRP_LIST, TOOL_BRP_LIST};
use crate::support::schema;

pub fn register_tool() -> Tool {
    Tool {
        name:         TOOL_BRP_LIST.into(),
        description:  DESC_BRP_LIST.into(),
        input_schema: schema::SchemaBuilder::new()
            .add_number_property(
                JSON_FIELD_ENTITY,
                "Optional entity ID to list components for",
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
    request: CallToolRequestParam,
    context: RequestContext<RoleServer>,
) -> Result<CallToolResult, McpError> {
    // Use common array_count extractor for component count

    let config = BrpHandlerConfig {
        method:            Some(BRP_METHOD_LIST),
        param_extractor:   Box::new(EntityParamExtractor { required: false }),
        formatter_factory: ResponseFormatterFactory::list_operation()
            .with_template("Listed components")
            .with_response_field(JSON_FIELD_DATA, extractors::pass_through_data)
            .with_response_field(JSON_FIELD_COMPONENT_COUNT, extractors::array_count)
            .with_response_field(JSON_FIELD_ENTITY, extractors::entity_from_params)
            .with_default_error()
            .build(),
    };

    handle_brp_request(service, request, context, &config).await
}
