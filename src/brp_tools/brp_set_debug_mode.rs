use std::sync::atomic::{AtomicBool, Ordering};

use rmcp::model::{CallToolResult, Tool};
use rmcp::service::RequestContext;
use rmcp::{Error as McpError, RoleServer};
use serde::Deserialize;

use crate::BrpMcpService;
use crate::error::BrpMcpError;
use crate::support::response::ResponseBuilder;
use crate::support::schema;
use crate::support::serialization::json_response_to_result;

static DEBUG_ENABLED: AtomicBool = AtomicBool::new(false);

/// Check if debug mode is currently enabled
pub fn is_debug_enabled() -> bool {
    DEBUG_ENABLED.load(Ordering::Relaxed)
}

#[derive(Debug, Deserialize)]
pub struct SetDebugModeParams {
    enabled: bool,
}

/// Handle the `set_debug_mode` tool request
pub fn handle_set_debug_mode(
    _service: &BrpMcpService,
    request: rmcp::model::CallToolRequestParam,
    _context: RequestContext<RoleServer>,
) -> Result<CallToolResult, McpError> {
    let args = request.arguments.unwrap_or_default();
    let params: SetDebugModeParams = serde_json::from_value(serde_json::Value::Object(args))
        .map_err(|e| -> McpError { BrpMcpError::validation_failed("parameters", e).into() })?;

    // Update the debug state
    DEBUG_ENABLED.store(params.enabled, Ordering::Relaxed);

    let message = if params.enabled {
        "Debug mode enabled - comprehensive BRP diagnostic information will be included in responses"
    } else {
        "Debug mode disabled - comprehensive BRP diagnostic information will be excluded from responses"
    };

    let response = ResponseBuilder::success()
        .message(message)
        .add_field("debug_enabled", params.enabled)
        .build();

    Ok(json_response_to_result(&response))
}

/// Register the `set_debug_mode` tool
pub fn register_tool() -> Tool {
    use crate::tools::{DESC_BRP_SET_DEBUG_MODE, TOOL_BRP_SET_DEBUG_MODE};

    Tool {
        name:         TOOL_BRP_SET_DEBUG_MODE.into(),
        description:  DESC_BRP_SET_DEBUG_MODE.into(),
        input_schema: schema::SchemaBuilder::new()
            .add_boolean_property(
                "enabled",
                "Set to true to enable debug output, false to disable",
                true,
            )
            .build(),
    }
}
