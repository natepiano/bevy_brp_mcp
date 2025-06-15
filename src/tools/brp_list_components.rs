use rmcp::model::{CallToolResult, Tool};
use rmcp::service::RequestContext;
use rmcp::{Error as McpError, RoleServer};
use serde_json::{json, Value};

use crate::BrpMcpService;
use crate::types::BrpExecuteParams;
use crate::constants::BRP_LIST_COMPONENTS_DESC;

use super::support;
use super::support::formatting::generate_empty_components_hint;

pub fn register_tool() -> Tool {
    Tool {
        name: "brp_list_components".into(),
        description: BRP_LIST_COMPONENTS_DESC.into(),
        input_schema: support::schema::SchemaBuilder::new()
            .add_number_property(
                "entity_id", 
                "Optional entity ID to list components for. If not provided, lists all registered \
                 components in the app. Internally calls bevy/list with or without entity parameter.", 
                false
            )
            .add_number_property("port", "The BRP port (default: 15702)", false)
            .build(),
    }
}

pub async fn handle(
    _service: &BrpMcpService,
    request: rmcp::model::CallToolRequestParam,
    context: RequestContext<RoleServer>,
) -> Result<CallToolResult, McpError> {
    // Extract parameters
    let entity_id = request.arguments
        .as_ref()
        .and_then(|args| args.get("entity_id"))
        .and_then(|v| v.as_u64());
    
    let port = request.arguments
        .as_ref()
        .and_then(|args| args.get("port"))
        .and_then(|v| v.as_u64())
        .map(|v| v as u16)
        .unwrap_or(15702);
    
    // Build params for bevy/list
    let params = entity_id.map(|entity| json!({
        "entity": entity
    }));
    
    // Create BRP execute params
    let brp_params = BrpExecuteParams {
        method: "bevy/list".to_string(),
        params,
        port,
    };
    
    // Convert to request format expected by brp_execute
    let execute_request = rmcp::model::CallToolRequestParam {
        name: "brp_execute".into(),
        arguments: serde_json::to_value(brp_params)
            .ok()
            .and_then(|v| v.as_object().cloned()),
    };
    
    // Call brp_execute
    let result = super::brp_execute::handle_brp_execute(execute_request, context).await?;
    
    // Extract and format the response
    if let Some(content) = result.content.first() {
        if let Some(text) = content.as_text() {
            // Parse the response from brp_execute
            let response: Value = serde_json::from_str(&text.text).map_err(|e| {
                McpError::internal_error(
                    format!("Failed to parse BRP response: {}", e),
                    None
                )
            })?;
            
            // Extract the component list from the data field
            let components = response
                .get("data")
                .and_then(|v| v.as_array())
                .ok_or_else(|| {
                    McpError::internal_error(
                        "Invalid response format from bevy/list",
                        None
                    )
                })?;
            
            // Format the response
            let formatted_data = if let Some(entity) = entity_id {
                let mut response = json!({
                    "status": "success",
                    "message": format!("Found {} components on entity {}", components.len(), entity),
                    "data": {
                        "entity": entity,
                        "components": components,
                        "count": components.len()
                    }
                });
                
                // Add hint if no components found
                if components.is_empty() {
                    response["hint"] = json!(generate_empty_components_hint(Some(entity)));
                }
                
                response
            } else {
                let mut response = json!({
                    "status": "success",
                    "message": format!("Found {} registered component types", components.len()),
                    "data": {
                        "components": components,
                        "count": components.len()
                    }
                });
                
                // Add hint if no components found
                if components.is_empty() {
                    response["hint"] = json!(generate_empty_components_hint(None));
                }
                
                response
            };
            
            Ok(CallToolResult::success(vec![rmcp::model::Content::text(
                serde_json::to_string(&formatted_data).unwrap_or_else(|_| "{}".to_string())
            )]))
        } else {
            Err(McpError::internal_error(
                "No text content in BRP response",
                None
            ))
        }
    } else {
        Err(McpError::internal_error(
            "No content in BRP response",
            None
        ))
    }
}