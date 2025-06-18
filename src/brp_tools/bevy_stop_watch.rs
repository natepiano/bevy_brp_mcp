//! Stop an active watch

use rmcp::model::{CallToolRequestParam, CallToolResult, Tool};
use rmcp::service::RequestContext;
use rmcp::{Error as McpError, RoleServer};
use serde_json::Value;

use crate::BrpMcpService;
use crate::brp_tools::constants::JSON_FIELD_WATCH_ID;
use crate::brp_tools::support::watch_response;
use crate::constants::{DESC_BEVY_STOP_WATCH, TOOL_BEVY_STOP_WATCH};
use crate::support::{params, schema};
use crate::watch_manager::WATCH_MANAGER;

pub fn register_tool() -> Tool {
    Tool {
        name:         TOOL_BEVY_STOP_WATCH.into(),
        description:  DESC_BEVY_STOP_WATCH.into(),
        input_schema: schema::SchemaBuilder::new()
            .add_number_property(
                JSON_FIELD_WATCH_ID,
                "The watch ID returned from bevy_start_entity_watch or bevy_start_list_watch",
                true,
            )
            .build(),
    }
}

pub async fn handle(
    _service: &BrpMcpService,
    request: CallToolRequestParam,
    _context: RequestContext<RoleServer>,
) -> Result<CallToolResult, McpError> {
    let arguments = Value::Object(request.arguments.unwrap_or_default());

    // Extract watch ID
    let watch_id = params::extract_required_u32(&arguments, JSON_FIELD_WATCH_ID, "watch_id")?;

    // Stop the watch and release lock immediately
    let result = {
        let mut manager = WATCH_MANAGER.lock().await;
        manager.stop_watch(watch_id).await
    };
    Ok(watch_response::format_watch_stop_response(result, watch_id))
}
