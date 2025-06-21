use rmcp::model::{CallToolRequestParam, CallToolResult, ListToolsResult};
use rmcp::service::RequestContext;
use rmcp::{Error as McpError, RoleServer};

use crate::BrpMcpService;
use crate::app_tools::{launch_app, launch_example, list_apps, list_brp_apps, list_examples};
use crate::brp_tools::{
    bevy_list_active_watches, bevy_shutdown, bevy_stop_watch, brp_get_watch, brp_list_watch,
    brp_status, tool_definitions, tool_generator,
};
// Imports removed - using fully qualified paths in match statement to avoid naming conflicts
use crate::error::BrpMcpError;
use crate::log_tools::{cleanup_logs, list_logs, read_log};
use crate::support::debug_tools;

pub fn register_tools() -> ListToolsResult {
    let mut tools = vec![];

    // Generate tools from declarative definitions
    for def in tool_definitions::get_standard_tools() {
        tools.push(tool_generator::generate_tool_registration(&def));
    }

    // Generate tools with minor variations
    for def in tool_definitions::get_special_tools() {
        tools.push(tool_generator::generate_tool_registration(&def));
    }

    // Add remaining tools that aren't migrated yet
    tools.extend(vec![
        // Core BRP tools (with custom logic)
        brp_status::register_tool(),
        // bevy_brp_extras tools
        bevy_shutdown::register_tool(),
        // App management tools
        list_apps::register_tool(),
        list_brp_apps::register_tool(),
        list_examples::register_tool(),
        launch_app::register_tool(),
        launch_example::register_tool(),
        // Log management tools
        list_logs::register_tool(),
        read_log::register_tool(),
        cleanup_logs::register_tool(),
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
    // Check if this is one of the standard tools
    let standard_tools = tool_definitions::get_standard_tools();
    if let Some(def) = standard_tools.iter().find(|d| d.name == request.name) {
        return tool_generator::generate_tool_handler(def, service, request, context).await;
    }

    // Check if this is one of the special tools
    let special_tools = tool_definitions::get_special_tools();
    if let Some(def) = special_tools.iter().find(|d| d.name == request.name) {
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

        // App management tools
        name if name == crate::tools::TOOL_LIST_BEVY_APPS => {
            list_apps::handle(service, context).await
        }
        name if name == crate::tools::TOOL_LIST_BRP_APPS => {
            list_brp_apps::handle(service, context).await
        }
        name if name == crate::tools::TOOL_LIST_BEVY_EXAMPLES => {
            list_examples::handle(service, context).await
        }
        name if name == crate::tools::TOOL_LAUNCH_BEVY_APP => {
            launch_app::handle(service, request, context).await
        }
        name if name == crate::tools::TOOL_LAUNCH_BEVY_EXAMPLE => {
            launch_example::handle(service, request, context).await
        }

        // Log management tools
        name if name == crate::tools::TOOL_LIST_LOGS => {
            list_logs::handle(service, &request, context)
        }
        name if name == crate::tools::TOOL_READ_LOG => read_log::handle(service, &request, context),
        name if name == crate::tools::TOOL_CLEANUP_LOGS => {
            cleanup_logs::handle(service, &request, context)
        }

        // Streaming/watch tools (custom logic)
        name if name == crate::tools::TOOL_BRP_GET_WATCH => {
            brp_get_watch::handle(service, request, context).await
        }
        name if name == crate::tools::TOOL_BRP_LIST_WATCH => {
            brp_list_watch::handle(service, request, context).await
        }
        name if name == crate::tools::TOOL_BEVY_STOP_WATCH => {
            bevy_stop_watch::handle(service, request, context).await
        }
        name if name == crate::tools::TOOL_BEVY_LIST_ACTIVE_WATCHES => {
            bevy_list_active_watches::handle(service, request, context).await
        }

        // Debug tools
        name if name == crate::tools::TOOL_SET_DEBUG_MODE => {
            debug_tools::handle_set_debug_mode(service, request, context)
        }

        _ => Err(BrpMcpError::invalid("tool", format!("unknown: {}", request.name)).into()),
    }
}
