use rmcp::model::{CallToolRequestParam, CallToolResult, ListToolsResult};
use rmcp::service::RequestContext;
use rmcp::{Error as McpError, RoleServer};

use crate::BrpMcpService;
use crate::app_tools::{launch_app, launch_example, list_apps, list_examples};
use crate::brp_tools::{brp_execute, brp_list, brp_query, brp_status};
use crate::constants::*;
use crate::log_tools::{cleanup_logs, list_logs, read_log};

pub async fn register_tools() -> ListToolsResult {
    let tools = vec![
        brp_execute::register_tool(),
        brp_list::register_tool(),
        brp_query::register_tool(),
        brp_status::register_tool(),
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
        TOOL_BRP_EXECUTE => brp_execute::handle_brp_execute(request, context).await,
        TOOL_BRP_LIST => brp_list::handle(service, request, context).await,
        TOOL_BRP_QUERY => brp_query::handle(service, request, context).await,
        TOOL_BRP_STATUS => brp_status::handle(service, request, context).await,
        TOOL_CLEANUP_LOGS => cleanup_logs::handle(service, request, context).await,
        TOOL_LIST_BEVY_APPS => list_apps::handle(service, context).await,
        TOOL_LIST_BEVY_EXAMPLES => list_examples::handle(service, context).await,
        TOOL_LAUNCH_BEVY_APP => launch_app::handle(service, request, context).await,
        TOOL_LAUNCH_BEVY_EXAMPLE => launch_example::handle(service, request, context).await,
        TOOL_LIST_LOGS => list_logs::handle(service, request, context).await,
        TOOL_READ_LOG => read_log::handle(service, request, context).await,
        _ => Err(McpError::from(rmcp::model::ErrorData::invalid_params(
            format!("Unknown tool: {}", request.name),
            None,
        ))),
    }
}
