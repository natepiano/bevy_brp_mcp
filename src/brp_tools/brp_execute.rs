use rmcp::model::{CallToolRequestParam, CallToolResult, Content, Tool};
use rmcp::service::RequestContext;
use rmcp::{Error as McpError, RoleServer};

use super::constants::{
    DEFAULT_BRP_PORT, JSON_FIELD_CODE, JSON_FIELD_DATA, JSON_FIELD_METHOD, JSON_FIELD_PARAMS,
    JSON_FIELD_PORT,
};
use super::support::brp_client::{BrpResult, execute_brp_method};
use crate::constants::TOOL_BRP_EXECUTE;
use crate::support::response::ResponseBuilder;
use crate::support::schema;
use crate::types::BrpExecuteParams;

/// Execute any BRP method on a running Bevy app
pub fn register_tool() -> Tool {
    Tool {
        name: TOOL_BRP_EXECUTE.into(),
        description: "Execute any Bevy Remote Protocol (BRP) method on a running Bevy app. This tool allows you to send arbitrary BRP commands and receive responses.".into(),
        input_schema: schema::SchemaBuilder::new()
            .add_string_property(JSON_FIELD_METHOD, "The BRP method to execute (e.g., 'rpc.discover', 'bevy/get', 'bevy/query')", true)
            .add_any_property(JSON_FIELD_PARAMS, "Optional parameters for the method, as a JSON object or array", false)
            .add_number_property(JSON_FIELD_PORT, &format!("The BRP port (default: {DEFAULT_BRP_PORT})"), false)
            .build(),
    }
}

pub async fn handle_brp_execute(
    params: CallToolRequestParam,
    _context: RequestContext<RoleServer>,
) -> Result<CallToolResult, McpError> {
    let params: BrpExecuteParams = serde_json::from_value(serde_json::Value::Object(
        params.arguments.unwrap_or_default(),
    ))
    .map_err(|e| {
        McpError::from(rmcp::model::ErrorData::invalid_params(
            format!("Invalid parameters: {e}"),
            None,
        ))
    })?;

    // Use the new brp_client for BRP communication
    let brp_result = execute_brp_method(&params.method, params.params, Some(params.port)).await?;

    // Format the response using the same approach as before
    match brp_result {
        BrpResult::Success(data) => {
            if let Some(result_data) = data {
                let response = ResponseBuilder::success()
                    .message(format!(
                        "Successfully executed BRP method: {}",
                        params.method
                    ))
                    .data(result_data)
                    .build();

                Ok(CallToolResult::success(vec![Content::text(
                    response.to_json(),
                )]))
            } else {
                let response = ResponseBuilder::success()
                    .message(format!(
                        "BRP method {} executed successfully with no result",
                        params.method
                    ))
                    .build();

                Ok(CallToolResult::success(vec![Content::text(
                    response.to_json(),
                )]))
            }
        }
        BrpResult::Error(error_info) => {
            let response = ResponseBuilder::error()
                .message(format!("BRP error: {}", error_info.message))
                .data(serde_json::json!({
                    JSON_FIELD_CODE: error_info.code,
                    JSON_FIELD_DATA: error_info.data
                }))
                .build();

            Ok(CallToolResult::success(vec![Content::text(
                response.to_json(),
            )]))
        }
    }
}
