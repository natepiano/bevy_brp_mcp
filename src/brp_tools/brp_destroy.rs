use rmcp::model::{CallToolResult, Tool};
use rmcp::service::RequestContext;
use rmcp::{Error as McpError, RoleServer};

use super::constants::{
    BRP_METHOD_DESTROY, DEFAULT_BRP_PORT, JSON_FIELD_DESTROYED_ENTITY, JSON_FIELD_PORT,
};
use super::support::configurable_formatter::{ConfigurableFormatterFactory, extractors};
use super::support::generic_handler::{BrpHandlerConfig, EntityParamExtractor, handle_generic};
use crate::BrpMcpService;
use crate::brp_tools::constants::JSON_FIELD_ENTITY;
use crate::constants::{DESC_BRP_DESTROY, TOOL_BRP_DESTROY};
use crate::support::schema;

pub fn register_tool() -> Tool {
    Tool {
        name:         TOOL_BRP_DESTROY.into(),
        description:  DESC_BRP_DESTROY.into(),
        input_schema: schema::SchemaBuilder::new()
            .add_number_property(JSON_FIELD_ENTITY, "The entity ID to destroy", true)
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
    let config = BrpHandlerConfig {
        method:            BRP_METHOD_DESTROY,
        param_extractor:   Box::new(EntityParamExtractor { required: true }),
        formatter_factory: ConfigurableFormatterFactory::entity_operation(
            JSON_FIELD_DESTROYED_ENTITY,
        )
        .with_template("Successfully destroyed entity {entity}")
        .with_response_field(JSON_FIELD_DESTROYED_ENTITY, extractors::entity_from_params)
        .build(),
    };

    handle_generic(service, request, context, &config).await
}
