use rmcp::model::{CallToolRequestParam, CallToolResult, ListToolsResult};
use rmcp::service::RequestContext;
use rmcp::{Error as McpError, RoleServer};

use crate::BrpMcpService;
use crate::app_tools::{launch_app, launch_example, list_apps, list_examples};
use crate::brp_tools::{brp_execute, brp_list, brp_query, check_brp};
use crate::log_tools::{cleanup_logs, list_logs, read_log};

pub async fn register_tools() -> ListToolsResult {
    let tools = vec![
        brp_execute::register_tool(),
        brp_list::register_tool(),
        brp_query::register_tool(),
        check_brp::register_tool(),
        list_apps::register_tool(),
        list_examples::register_tool(),
        launch_app::register_tool(),
        launch_example::register_tool(),
        list_logs::register_tool(),
        read_log::register_tool(),
        cleanup_logs::register_tool(),
    ];

    ListToolsResult {
        next_cursor: None,
        tools,
    }
}

pub async fn handle_tool_call(
    service: &BrpMcpService,
    request: CallToolRequestParam,
    context: RequestContext<RoleServer>,
) -> Result<CallToolResult, McpError> {
    match request.name.as_ref() {
        "brp_execute" => brp_execute::handle_brp_execute(request, context).await,
        "brp_list_components" => brp_list::handle(service, request, context).await,
        "brp_query" => brp_query::handle(service, request, context).await,
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
