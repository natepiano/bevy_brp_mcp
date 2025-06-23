use rmcp::model::{CallToolRequestParam, CallToolResult, ListToolsResult};
use rmcp::service::RequestContext;
use rmcp::{Error as McpError, RoleServer};

use crate::BrpMcpService;
use crate::brp_tools::{
    bevy_list_active_watches, bevy_shutdown, bevy_stop_watch, brp_get_watch, brp_list_watch,
    brp_status,
};
use crate::{tool_definitions, tool_generator};
// Imports removed - using fully qualified paths in match statement to avoid naming conflicts
use crate::error::BrpMcpError;
use crate::support::debug_tools;

pub fn register_tools() -> ListToolsResult {
    let mut tools = vec![];

    // Generate tools from declarative definitions
    for def in tool_definitions::get_all_tools() {
        tools.push(tool_generator::generate_tool_registration(&def));
    }

    // Add remaining tools that aren't migrated yet
    tools.extend(vec![
        // Core BRP tools (with custom logic)
        brp_status::register_tool(),
        // bevy_brp_extras tools
        bevy_shutdown::register_tool(),
        // Streaming/watch tools (custom logic)
        brp_get_watch::register_tool(),
        brp_list_watch::register_tool(),
        bevy_stop_watch::register_tool(),
        bevy_list_active_watches::register_tool(),
        // Debug tools
        debug_tools::register_tool(),
    ]);

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
    // Check if this is one of the declaratively defined tools
    let all_tools = tool_definitions::get_all_tools();
    if let Some(def) = all_tools.iter().find(|d| d.name == request.name) {
        return tool_generator::generate_tool_handler(def, service, request, context).await;
    }

    // Handle remaining tools
    match request.name.as_ref() {
        // Core BRP tools (with custom logic)
        name if name == crate::tools::TOOL_BRP_STATUS => {
            brp_status::handle(service, request, context).await
        }

        // bevy_brp_extras tools
        name if name == crate::tools::TOOL_BRP_EXTRAS_SHUTDOWN => {
            bevy_shutdown::handle(service, request, context).await
        }

        // Streaming/watch tools (custom logic)
        name if name == crate::tools::TOOL_BEVY_GET_WATCH => {
            brp_get_watch::handle(service, request, context).await
        }
        name if name == crate::tools::TOOL_BEVY_LIST_WATCH => {
            brp_list_watch::handle(service, request, context).await
        }
        name if name == crate::tools::TOOL_BRP_STOP_WATCH => {
            bevy_stop_watch::handle(service, request, context).await
        }
        name if name == crate::tools::TOOL_BRP_LIST_ACTIVE_WATCHES => {
            bevy_list_active_watches::handle(service, request, context).await
        }

        // Debug tools
        name if name == crate::tools::TOOL_BRP_SET_DEBUG_MODE => {
            debug_tools::handle_set_debug_mode(service, request, context)
        }

        _ => {
            let tool_name = &request.name;
            Err(BrpMcpError::invalid("tool", format!("unknown: {tool_name}")).into())
        }
    }
}
