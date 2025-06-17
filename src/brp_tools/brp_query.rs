use rmcp::model::{CallToolResult, Tool};
use rmcp::service::RequestContext;
use rmcp::{Error as McpError, RoleServer};
use serde_json::{Value, json};

use super::constants::{
    BRP_METHOD_QUERY, DEFAULT_BRP_PORT, JSON_FIELD_COMPONENT_COUNT, JSON_FIELD_DATA,
    JSON_FIELD_ENTITY_COUNT, JSON_FIELD_MESSAGE, JSON_FIELD_PORT, JSON_FIELD_QUERY_PARAMS,
    JSON_FIELD_STATUS, JSON_FIELD_STRICT, RESPONSE_STATUS_SUCCESS,
};
use super::support::generic_handler::{
    BrpHandlerConfig, FormatterContext, FormatterFactory, PassthroughExtractor, handle_generic,
};
use super::support::response_processor::{
    BrpError, BrpMetadata, BrpResponseFormatter, format_error_default,
};
use super::support::serialization::json_tool_result;
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
                false
            )
            .add_any_property(
                "filter",
                "Object specifying which entities to query. Properties: with (array), without (array)",
                false
            )
            .add_boolean_property(
                JSON_FIELD_STRICT,
                "If true, returns error on unknown component types (default: false)",
                false
            )
            .add_number_property(JSON_FIELD_PORT, &format!("The BRP port (default: {})", DEFAULT_BRP_PORT), false)
            .build(),
    }
}

pub async fn handle(
    service: &BrpMcpService,
    request: rmcp::model::CallToolRequestParam,
    context: RequestContext<RoleServer>,
) -> Result<CallToolResult, McpError> {
    let config = BrpHandlerConfig {
        method:            BRP_METHOD_QUERY,
        param_extractor:   Box::new(PassthroughExtractor),
        formatter_factory: Box::new(QueryFormatterFactory),
    };

    handle_generic(service, request, context, &config).await
}

/// Factory for creating QueryFormatter
struct QueryFormatterFactory;

impl FormatterFactory for QueryFormatterFactory {
    fn create(&self, context: FormatterContext) -> Box<dyn BrpResponseFormatter> {
        // Extract query params from context for formatter
        let query_params = context.params.clone().unwrap_or_else(|| json!({}));
        Box::new(QueryFormatter { query_params })
    }
}

/// Formatter for bevy/query responses
struct QueryFormatter {
    query_params: Value,
}

impl BrpResponseFormatter for QueryFormatter {
    fn format_success(&self, data: Value, _metadata: BrpMetadata) -> CallToolResult {
        // Extract entities array
        let entities = data.as_array().cloned().unwrap_or_default();

        let entity_count = entities.len();

        // Count total components across all entities
        let total_components = entities
            .iter()
            .filter_map(|e| e.as_object())
            .map(|obj| obj.len())
            .sum::<usize>();

        let formatted_data = json!({
            JSON_FIELD_STATUS: RESPONSE_STATUS_SUCCESS,
            JSON_FIELD_MESSAGE: format!(
                "Query returned {} entity(ies) with {} total component(s)",
                entity_count,
                total_components
            ),
            JSON_FIELD_DATA: entities,
            JSON_FIELD_ENTITY_COUNT: entity_count,
            JSON_FIELD_COMPONENT_COUNT: total_components,
            JSON_FIELD_QUERY_PARAMS: self.query_params,
        });

        json_tool_result(&formatted_data)
    }

    fn format_error(&self, error: BrpError, metadata: BrpMetadata) -> CallToolResult {
        format_error_default(error, metadata)
    }
}
