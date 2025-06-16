use std::time::Duration;

use rmcp::model::{CallToolRequestParam, CallToolResult, Content, Tool};
use rmcp::service::RequestContext;
use rmcp::{Error as McpError, RoleServer};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::support::builder::BrpJsonRpcBuilder;
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
            .add_string_property("method", "The BRP method to execute (e.g., 'rpc.discover', 'bevy/get', 'bevy/query')", true)
            .add_any_property("params", "Optional parameters for the method, as a JSON object or array", false)
            .add_number_property("port", "The BRP port (default: 15702)", false)
            .build(),
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct BrpResponse {
    jsonrpc: String,
    id:      u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    result:  Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error:   Option<BrpError>,
}

#[derive(Debug, Serialize, Deserialize)]
struct BrpError {
    code:    i32,
    message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    data:    Option<Value>,
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
            format!("Invalid parameters: {}", e),
            None,
        ))
    })?;

    // Create BRP request using builder
    let mut builder = BrpJsonRpcBuilder::new(params.method.clone());
    if let Some(p) = params.params {
        builder = builder.params(p);
    }
    let request = builder.build();

    // Connect to BRP server
    let url = format!("http://localhost:{}", params.port);

    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(10))
        .build()
        .map_err(|e| {
            McpError::from(rmcp::model::ErrorData::internal_error(
                format!("Failed to create HTTP client: {}", e),
                None,
            ))
        })?;

    // Send request
    let response = client
        .post(&url)
        .json(&request)
        .send()
        .await
        .map_err(|e| {
            if e.is_connect() {
                McpError::from(rmcp::model::ErrorData::internal_error(
                    format!("Failed to connect to BRP server on port {}. Make sure a Bevy app with RemotePlugin is running.", params.port),
                    None
                ))
            } else {
                McpError::from(rmcp::model::ErrorData::internal_error(
                    format!("HTTP request failed: {}", e),
                    None
                ))
            }
        })?;

    let brp_response: BrpResponse = response.json().await.map_err(|e| {
        McpError::from(rmcp::model::ErrorData::internal_error(
            format!("Failed to parse BRP response: {}", e),
            None,
        ))
    })?;

    // Handle BRP response
    if let Some(error) = brp_response.error {
        let response = ResponseBuilder::error()
            .message(format!("BRP error: {}", error.message))
            .data(serde_json::json!({
                "code": error.code,
                "data": error.data
            }))
            .build();

        Ok(CallToolResult::success(vec![Content::text(
            response.to_json(),
        )]))
    } else if let Some(result) = brp_response.result {
        let response = ResponseBuilder::success()
            .message(format!(
                "Successfully executed BRP method: {}",
                params.method
            ))
            .data(result)
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
