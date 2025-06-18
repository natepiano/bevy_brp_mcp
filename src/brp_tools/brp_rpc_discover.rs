use rmcp::model::{CallToolResult, Tool};
use rmcp::service::RequestContext;
use rmcp::{Error as McpError, RoleServer};

use super::constants::{
    BRP_METHOD_RPC_DISCOVER, DEFAULT_BRP_PORT, JSON_FIELD_DATA, JSON_FIELD_PORT,
};
use super::support::configurable_formatter::{ConfigurableFormatterFactory, extractors};
use super::support::generic_handler::{BrpHandlerConfig, SimplePortExtractor, handle_generic};
use crate::BrpMcpService;
use crate::constants::{DESC_BRP_RPC_DISCOVER, TOOL_BRP_RPC_DISCOVER};
use crate::support::schema;

pub fn register_tool() -> Tool {
    Tool {
        name:         TOOL_BRP_RPC_DISCOVER.into(),
        description:  DESC_BRP_RPC_DISCOVER.into(),
        input_schema: schema::SchemaBuilder::new()
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
        method:            BRP_METHOD_RPC_DISCOVER,
        param_extractor:   Box::new(SimplePortExtractor),
        formatter_factory: ConfigurableFormatterFactory::pass_through()
            .with_template("Discovered BRP methods")
            .with_response_field(JSON_FIELD_DATA, extractors::pass_through_data)
            .with_default_error()
            .build(),
    };

    handle_generic(service, request, context, &config).await
}
