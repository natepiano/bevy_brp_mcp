use std::path::PathBuf;

use rmcp::model::CallToolResult;

use crate::brp_tools::constants::{JSON_FIELD_LOG_PATH, JSON_FIELD_WATCH_ID};
use crate::error::BrpMcpError;
use crate::support::response::ResponseBuilder;
use crate::support::serialization::json_response_to_result;

pub fn format_watch_start_response(
    result: Result<(u32, PathBuf), BrpMcpError>,
    operation_name: &str,
    entity_id: u64,
) -> CallToolResult {
    match result {
        Ok((watch_id, log_path)) => {
            let response = ResponseBuilder::success()
                .message(format!(
                    "Started {operation_name} {watch_id} for entity {entity_id}"
                ))
                .add_field(JSON_FIELD_WATCH_ID, watch_id)
                .add_field(JSON_FIELD_LOG_PATH, log_path.to_string_lossy())
                .build();
            json_response_to_result(&response)
        }
        Err(e) => {
            let response = ResponseBuilder::error().message(e.to_string()).build();
            json_response_to_result(&response)
        }
    }
}

pub fn format_watch_stop_response(
    result: Result<(), BrpMcpError>,
    watch_id: u32,
) -> CallToolResult {
    match result {
        Ok(()) => {
            let response = ResponseBuilder::success()
                .message(format!("Stopped watch {watch_id}"))
                .build();
            json_response_to_result(&response)
        }
        Err(e) => {
            let response = ResponseBuilder::error().message(e.to_string()).build();
            json_response_to_result(&response)
        }
    }
}
