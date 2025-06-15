use rmcp::model::{CallToolRequestParam, CallToolResult};
use rmcp::service::RequestContext;
use rmcp::{Error as McpError, RoleServer};

use crate::BrpMcpService;

mod brp_execute;
mod check_brp;
mod cleanup_logs;
mod launch_app;
mod launch_example;
mod list_apps;
mod list_examples;
mod list_logs;
mod read_log;
mod registry;
mod support;

pub use registry::register_tools;

pub async fn handle_tool_call(
    service: &BrpMcpService,
    request: CallToolRequestParam,
    context: RequestContext<RoleServer>,
) -> Result<CallToolResult, McpError> {
    match request.name.as_ref() {
        "brp_execute" => brp_execute::handle_brp_execute(request, context).await,
        "check_brp" => check_brp::handle(service, request, context).await,
        "cleanup_logs" => cleanup_logs::handle(service, request, context).await,
        "list_bevy_apps" => list_apps::handle(service, context).await,
        "list_bevy_examples" => list_examples::handle(service, context).await,
        "launch_bevy_app" => launch_app::handle(service, request, context).await,
        "launch_bevy_example" => launch_example::handle(service, request, context).await,
        "list_logs" => list_logs::handle(service, request, context).await,
        "read_log" => read_log::handle(service, request, context).await,
        _ => Err(McpError::from(rmcp::model::ErrorData::invalid_params(
            format!("Unknown tool: {}", request.name),
            None,
        ))),
    }
}
