use rmcp::model::{CallToolRequestParam, CallToolResult, Tool};
use rmcp::service::RequestContext;
use rmcp::{Error as McpError, RoleServer};

use super::constants::{DEFAULT_BRP_PORT, JSON_FIELD_METHOD, JSON_FIELD_PARAMS, JSON_FIELD_PORT};
use super::support::{
    BrpExecuteExtractor, DynamicBrpHandlerConfig, ResponseFormatterFactory, handle_dynamic,
};
use crate::BrpMcpService;
use crate::constants::TOOL_BRP_EXECUTE;
use crate::support::schema;

/// Execute any BRP method on a running Bevy app
pub fn register_tool() -> Tool {
    Tool {
        name: TOOL_BRP_EXECUTE.into(),
        description: "Execute any Bevy Remote Protocol (BRP) method on a running Bevy app. This tool allows you to send arbitrary BRP commands and receive responses.".into(),
        input_schema: schema::SchemaBuilder::new()
            .add_string_property(JSON_FIELD_METHOD, "The BRP method to execute (e.g., 'rpc.discover', 'bevy/get', 'bevy/query')", true)
            .add_any_property(JSON_FIELD_PARAMS, "Optional parameters for the method, as a JSON object or array", false)
            .add_number_property(JSON_FIELD_PORT, &format!("The BRP port (default: {DEFAULT_BRP_PORT})"), false)
            .build(),
    }
}

pub async fn handle(
    service: &BrpMcpService,
    request: CallToolRequestParam,
    context: RequestContext<RoleServer>,
) -> Result<CallToolResult, McpError> {
    let config = DynamicBrpHandlerConfig {
        param_extractor:   Box::new(BrpExecuteExtractor),
        formatter_factory: ResponseFormatterFactory::method_execution().build(),
    };

    handle_dynamic(service, request, context, &config).await
}
