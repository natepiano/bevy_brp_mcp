use rmcp::model::{CallToolRequestParam, CallToolResult, ListToolsResult};
use rmcp::service::RequestContext;
use rmcp::{Error as McpError, RoleServer};

use crate::BrpMcpService;
use crate::app_tools::{launch_app, launch_example, list_apps, list_examples};
use crate::brp_tools::{
    bevy_list_active_watches, bevy_stop_watch, brp_destroy, brp_execute, brp_get, brp_get_resource,
    brp_get_watch, brp_insert, brp_insert_resource, brp_list, brp_list_resources, brp_list_watch,
    brp_mutate_component, brp_mutate_resource, brp_query, brp_registry_schema, brp_remove,
    brp_remove_resource, brp_reparent, brp_rpc_discover, brp_spawn, brp_status,
};
use crate::constants::*;
use crate::log_tools::{cleanup_logs, list_logs, read_log};

pub async fn register_tools() -> ListToolsResult {
    let tools = vec![
        // Core BRP tools
        brp_execute::register_tool(),
        brp_status::register_tool(),
        // Entity component tools
        brp_list::register_tool(),
        brp_query::register_tool(),
        brp_get::register_tool(),
        brp_destroy::register_tool(),
        brp_spawn::register_tool(),
        brp_insert::register_tool(),
        brp_remove::register_tool(),
        brp_mutate_component::register_tool(),
        brp_reparent::register_tool(),
        // Resource tools
        brp_list_resources::register_tool(),
        brp_get_resource::register_tool(),
        brp_insert_resource::register_tool(),
        brp_remove_resource::register_tool(),
        brp_mutate_resource::register_tool(),
        // Discovery/schema tools
        brp_registry_schema::register_tool(),
        brp_rpc_discover::register_tool(),
        // App management tools
        list_apps::register_tool(),
        list_examples::register_tool(),
        launch_app::register_tool(),
        launch_example::register_tool(),
        // Log management tools
        list_logs::register_tool(),
        read_log::register_tool(),
        cleanup_logs::register_tool(),
        // Streaming/watch tools
        brp_get_watch::register_tool(),
        brp_list_watch::register_tool(),
        bevy_stop_watch::register_tool(),
        bevy_list_active_watches::register_tool(),
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
        // Core BRP tools
        TOOL_BRP_EXECUTE => brp_execute::handle_brp_execute(request, context).await,
        TOOL_BRP_STATUS => brp_status::handle(service, request, context).await,

        // Entity component tools
        TOOL_BRP_LIST => brp_list::handle(service, request, context).await,
        TOOL_BRP_QUERY => brp_query::handle(service, request, context).await,
        TOOL_BRP_GET => brp_get::handle(service, request, context).await,
        TOOL_BRP_DESTROY => brp_destroy::handle(service, request, context).await,
        TOOL_BRP_SPAWN => brp_spawn::handle(service, request, context).await,
        TOOL_BRP_INSERT => brp_insert::handle(service, request, context).await,
        TOOL_BRP_REMOVE => brp_remove::handle(service, request, context).await,
        TOOL_BRP_MUTATE_COMPONENT => brp_mutate_component::handle(service, request, context).await,
        TOOL_BRP_REPARENT => brp_reparent::handle(service, request, context).await,

        // Resource tools
        TOOL_BRP_LIST_RESOURCES => brp_list_resources::handle(service, request, context).await,
        TOOL_BRP_GET_RESOURCE => brp_get_resource::handle(service, request, context).await,
        TOOL_BRP_INSERT_RESOURCE => brp_insert_resource::handle(service, request, context).await,
        TOOL_BRP_REMOVE_RESOURCE => brp_remove_resource::handle(service, request, context).await,
        TOOL_BRP_MUTATE_RESOURCE => brp_mutate_resource::handle(service, request, context).await,

        // Discovery/schema tools
        TOOL_BRP_REGISTRY_SCHEMA => brp_registry_schema::handle(service, request, context).await,
        TOOL_BRP_RPC_DISCOVER => brp_rpc_discover::handle(service, request, context).await,

        // App management tools
        TOOL_LIST_BEVY_APPS => list_apps::handle(service, context).await,
        TOOL_LIST_BEVY_EXAMPLES => list_examples::handle(service, context).await,
        TOOL_LAUNCH_BEVY_APP => launch_app::handle(service, request, context).await,
        TOOL_LAUNCH_BEVY_EXAMPLE => launch_example::handle(service, request, context).await,

        // Log management tools
        TOOL_LIST_LOGS => list_logs::handle(service, request, context).await,
        TOOL_READ_LOG => read_log::handle(service, request, context).await,
        TOOL_CLEANUP_LOGS => cleanup_logs::handle(service, request, context).await,

        // Streaming/watch tools
        TOOL_BRP_GET_WATCH => brp_get_watch::handle(service, request, context).await,
        TOOL_BRP_LIST_WATCH => brp_list_watch::handle(service, request, context).await,
        TOOL_BEVY_STOP_WATCH => bevy_stop_watch::handle(service, request, context).await,
        TOOL_BEVY_LIST_ACTIVE_WATCHES => {
            bevy_list_active_watches::handle(service, request, context).await
        }

        _ => Err(McpError::from(rmcp::model::ErrorData::invalid_params(
            format!("Unknown tool: {}", request.name),
            None,
        ))),
    }
}
