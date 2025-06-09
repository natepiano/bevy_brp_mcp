use rmcp::model::{CallToolRequestParam, CallToolResult};
use rmcp::service::RequestContext;
use rmcp::{Error as McpError, RoleServer};

use crate::BrpMcpService;

mod support;
mod launch;
mod list_apps;
mod list_examples;
mod registry;

pub use registry::register_tools;

pub async fn handle_tool_call(
    service: &BrpMcpService,
    request: CallToolRequestParam,
    context: RequestContext<RoleServer>,
) -> Result<CallToolResult, McpError> {
    match request.name.as_ref() {
        "list_bevy_apps" => list_apps::handle(service, context).await,
        "list_bevy_examples" => list_examples::handle(service, context).await,
        "launch_bevy_app" => launch::handle(service, request, context).await,
        _ => Err(McpError::invalid_params(
            format!("Unknown tool: {}", request.name),
            None,
        )),
    }
}