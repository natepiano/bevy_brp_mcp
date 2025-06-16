use rmcp::model::{CallToolResult, Tool};
use rmcp::service::RequestContext;
use rmcp::{Error as McpError, RoleServer};
use serde_json::{Value, json};

use super::support::builder::BrpRequestBuilder;
use super::support::formatting::generate_empty_components_hint;
use super::support::response_processor::{BrpMetadata, BrpResponseFormatter, process_brp_response};
use super::support::utils::to_execute_request;
use crate::BrpMcpService;
use crate::constants::{BRP_LIST, DESC_BRP_LIST, TOOL_BRP_LIST};
use crate::support::schema;

pub fn register_tool() -> Tool {
    Tool {
        name: TOOL_BRP_LIST.into(),
        description: DESC_BRP_LIST.into(),
        input_schema: schema::SchemaBuilder::new()
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
    let entity_id = request
        .arguments
        .as_ref()
        .and_then(|args| args.get("entity_id"))
        .and_then(|v| v.as_u64());

    let port = request
        .arguments
        .as_ref()
        .and_then(|args| args.get("port"))
        .and_then(|v| v.as_u64())
        .map(|v| v as u16)
        .unwrap_or(15702);

    // Build BRP request using the builder
    let mut builder = BrpRequestBuilder::new(BRP_LIST).port(port);

    if let Some(entity) = entity_id {
        builder = builder.entity(entity);
    }

    let brp_params = builder.build();

    // Convert to request format expected by brp_execute
    let execute_request = to_execute_request(brp_params)?;

    // Call brp_execute
    let result = super::brp_execute::handle_brp_execute(execute_request, context).await?;

    // Create formatter and metadata
    let formatter = ListFormatter::new(entity_id);
    let metadata = BrpMetadata::new(BRP_LIST, port);

    // Use the response processor to handle the result
    process_brp_response(result, formatter, metadata)
}

/// Formatter for bevy/list responses
struct ListFormatter {
    entity_id: Option<u64>,
}

impl ListFormatter {
    fn new(entity_id: Option<u64>) -> Self {
        Self { entity_id }
    }
}

impl BrpResponseFormatter for ListFormatter {
    fn format_success(&self, data: Value, _metadata: BrpMetadata) -> CallToolResult {
        // Extract the component list from the data field
        let empty_vec = vec![];
        let components = data.as_array().unwrap_or(&empty_vec);

        // Format the response based on entity_id
        let formatted_data = if let Some(entity) = self.entity_id {
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

        CallToolResult::success(vec![rmcp::model::Content::text(
            serde_json::to_string(&formatted_data).unwrap_or_else(|_| "{}".to_string()),
        )])
    }

    fn format_error(
        &self,
        error: super::support::response_processor::BrpError,
        metadata: BrpMetadata,
    ) -> CallToolResult {
        let formatted_error = json!({
            "status": "error",
            "message": error.message,
            "error_code": error.code,
            "data": error.data,
            "metadata": {
                "method": metadata.method,
                "port": metadata.port,
                "entity_id": self.entity_id
            }
        });

        CallToolResult::success(vec![rmcp::model::Content::text(
            serde_json::to_string(&formatted_error).unwrap_or_else(|_| "{}".to_string()),
        )])
    }
}
