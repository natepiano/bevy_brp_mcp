use rmcp::model::{CallToolRequestParam, CallToolResult, Tool};
use rmcp::service::RequestContext;
use rmcp::{Error as McpError, RoleServer};

use super::constants::{
    BRP_METHOD_QUERY, DEFAULT_BRP_PORT, JSON_FIELD_COMPONENT_COUNT, JSON_FIELD_DATA,
    JSON_FIELD_ENTITY_COUNT, JSON_FIELD_PORT, JSON_FIELD_QUERY_PARAMS, JSON_FIELD_STRICT,
};
use super::support::{
    BrpHandlerConfig, FieldExtractor, PassthroughExtractor, ResponseFormatterFactory, extractors,
    handle_request,
};
use crate::BrpMcpService;
use crate::constants::{DESC_BRP_QUERY, TOOL_BRP_QUERY};
use crate::support::schema;

pub fn register_tool() -> Tool {
    Tool {
        name: TOOL_BRP_QUERY.into(),
        description: DESC_BRP_QUERY.into(),
        input_schema: schema::SchemaBuilder::new()
            .add_any_property(
                JSON_FIELD_DATA,
                "Object specifying what component data to retrieve. Properties: components (array), option (array), has (array)",
                true
            )
            .add_any_property(
                "filter",
                "Object specifying which entities to query. Properties: with (array), without (array)",
                true
            )
            .add_boolean_property(
                JSON_FIELD_STRICT,
                "If true, returns error on unknown component types (default: false)",
                false
            )
            .add_number_property(JSON_FIELD_PORT, &format!("The BRP port (default: {DEFAULT_BRP_PORT})" ), false)
            .build(),
    }
}

pub async fn handle(
    service: &BrpMcpService,
    request: CallToolRequestParam,
    context: RequestContext<RoleServer>,
) -> Result<CallToolResult, McpError> {
    // Use common array_count extractor for entity count

    // Custom extractor for total component count
    let component_count_extractor: FieldExtractor = |data, _context| {
        let total = data
            .as_array()
            .map(|entities| {
                entities
                    .iter()
                    .filter_map(|e| e.as_object())
                    .map(|obj| obj.len())
                    .sum::<usize>()
            })
            .unwrap_or(0);
        serde_json::Value::Number(serde_json::Number::from(total))
    };

    // Custom extractor for query params
    let query_params_extractor: FieldExtractor =
        |_data, context| context.params.clone().unwrap_or(serde_json::Value::Null);

    let config = BrpHandlerConfig {
        method:            BRP_METHOD_QUERY,
        param_extractor:   Box::new(PassthroughExtractor),
        formatter_factory: ResponseFormatterFactory::pass_through()
            .with_template("Query completed successfully")
            .with_response_field(JSON_FIELD_DATA, extractors::pass_through_data)
            .with_response_field(JSON_FIELD_ENTITY_COUNT, extractors::array_count)
            .with_response_field(JSON_FIELD_COMPONENT_COUNT, component_count_extractor)
            .with_response_field(JSON_FIELD_QUERY_PARAMS, query_params_extractor)
            .with_default_error()
            .build(),
    };

    handle_request(service, request, context, &config).await
}
