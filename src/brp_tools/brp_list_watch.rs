//! Start watching an entity for component list changes

use rmcp::model::{CallToolRequestParam, CallToolResult, Tool};
use rmcp::service::RequestContext;
use rmcp::{Error as McpError, RoleServer};
use serde_json::Value;

use super::support;
use crate::BrpMcpService;
use crate::brp_tools::constants::{DEFAULT_BRP_PORT, JSON_FIELD_ENTITY, JSON_FIELD_PORT};
use crate::constants::{DESC_BRP_LIST_WATCH, TOOL_BRP_LIST_WATCH};
use crate::support::{params, schema};

pub fn register_tool() -> Tool {
    Tool {
        name:         TOOL_BRP_LIST_WATCH.into(),
        description:  DESC_BRP_LIST_WATCH.into(),
        input_schema: schema::SchemaBuilder::new()
            .add_number_property(
                JSON_FIELD_ENTITY,
                "The entity ID to watch for component list changes",
                true,
            )
            .add_number_property(
                JSON_FIELD_PORT,
                &format!("The BRP port (default: {DEFAULT_BRP_PORT})"),
                false,
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

    // Extract parameters
    let entity_id = params::extract_required_u64(&arguments, JSON_FIELD_ENTITY, "entity")?;
    let port = params::extract_optional_u16(&arguments, JSON_FIELD_PORT, DEFAULT_BRP_PORT);

    // Start the watch task
    let result = support::start_list_watch_task(entity_id, port).await;
    Ok(support::format_watch_start_response(
        result,
        "list watch",
        entity_id,
    ))
}
