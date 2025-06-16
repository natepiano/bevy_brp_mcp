use rmcp::model::{CallToolResult, Tool};
use rmcp::service::RequestContext;
use rmcp::{Error as McpError, RoleServer};
use serde_json::json;

use super::support::builder::BrpRequestBuilder;
use super::support::utils::{parse_brp_response, to_execute_request};
use crate::BrpMcpService;
use crate::constants::BRP_QUERY_DESC;
use crate::support::schema;

pub fn register_tool() -> Tool {
    Tool {
        name: "brp_query".into(),
        description: BRP_QUERY_DESC.into(),
        input_schema: schema::SchemaBuilder::new()
            .add_any_property(
                "data",
                "Object specifying what component data to retrieve. Properties: components (array), option (array), has (array)",
                false
            )
            .add_any_property(
                "filter",
                "Object specifying which entities to query. Properties: with (array), without (array)",
                false
            )
            .add_boolean_property(
                "strict",
                "If true, query fails on unknown component types",
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
    let args = request.arguments.as_ref();

    let port = args
        .and_then(|args| args.get("port"))
        .and_then(|v| v.as_u64())
        .map(|v| v as u16)
        .unwrap_or(15702);

    // Build BRP request using the builder
    let mut builder = BrpRequestBuilder::new("bevy/query").port(port);

    // Handle data parameter
    if let Some(data) = args.and_then(|args| args.get("data")) {
        builder = builder.data(data.clone());
    }

    // Handle filter parameter
    if let Some(filter) = args.and_then(|args| args.get("filter")) {
        builder = builder.filter(filter.clone());
    }

    // Handle strict parameter
    if let Some(strict) = args
        .and_then(|args| args.get("strict"))
        .and_then(|v| v.as_bool())
    {
        builder = builder.strict(strict);
    }

    let brp_params = builder.build();

    // Convert to request format expected by brp_execute
    let execute_request = to_execute_request(brp_params)?;

    // Call brp_execute
    let result = super::brp_execute::handle_brp_execute(execute_request, context).await?;

    // Extract and format the response
    if let Some(content) = result.content.first() {
        if let Some(text) = content.as_text() {
            // Parse the response from brp_execute
            let response = parse_brp_response(&text.text)?;

            // Extract the query results from the data field
            let query_results = response.get("data").ok_or_else(|| {
                McpError::internal_error("Invalid response format from bevy/query", None)
            })?;

            // Count entities in results
            let entity_count = if let Some(obj) = query_results.as_object() {
                obj.len()
            } else {
                0
            };

            // Format the response
            let formatted_data = json!({
                "status": "success",
                "message": format!("Query returned {} entities", entity_count),
                "data": query_results,
                "metadata": {
                    "entity_count": entity_count,
                    "query_params": {
                        "data": args.and_then(|a| a.get("data")),
                        "filter": args.and_then(|a| a.get("filter")),
                        "strict": args.and_then(|a| a.get("strict"))
                    }
                }
            });

            Ok(CallToolResult::success(vec![rmcp::model::Content::text(
                serde_json::to_string(&formatted_data).unwrap_or_else(|_| "{}".to_string()),
            )]))
        } else {
            Err(McpError::internal_error(
                "No text content in BRP response",
                None,
            ))
        }
    } else {
        Err(McpError::internal_error("No content in BRP response", None))
    }
}
