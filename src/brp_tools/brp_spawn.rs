use rmcp::model::{CallToolResult, Tool};
use rmcp::service::RequestContext;
use rmcp::{Error as McpError, RoleServer};
use serde_json::{Value, json};

use super::constants::{
    BRP_METHOD_SPAWN, DEFAULT_BRP_PORT, JSON_FIELD_COMPONENTS, JSON_FIELD_DATA,
    JSON_FIELD_ERROR_CODE, JSON_FIELD_MESSAGE, JSON_FIELD_METADATA, JSON_FIELD_METHOD,
    JSON_FIELD_PORT, JSON_FIELD_SPAWNED_ENTITY, JSON_FIELD_STATUS, RESPONSE_STATUS_ERROR,
    RESPONSE_STATUS_SUCCESS,
};
use super::support::generic_handler::{
    BrpHandlerConfig, FormatterContext, FormatterFactory, PassthroughExtractor, handle_generic,
};
use super::support::response_processor::{BrpMetadata, BrpResponseFormatter};
use super::support::serialization::json_tool_result;
use crate::BrpMcpService;
use crate::constants::{DESC_BRP_SPAWN, TOOL_BRP_SPAWN};
use crate::support::schema;

pub fn register_tool() -> Tool {
    Tool {
        name: TOOL_BRP_SPAWN.into(),
        description: DESC_BRP_SPAWN.into(),
        input_schema: schema::SchemaBuilder::new()
            .add_any_property(
                JSON_FIELD_COMPONENTS,
                "Object containing component data to spawn with. Keys are component types, values are component data",
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
        method:            BRP_METHOD_SPAWN,
        param_extractor:   Box::new(PassthroughExtractor),
        formatter_factory: Box::new(SpawnFormatterFactory),
    };

    handle_generic(service, request, context, &config).await
}

/// Factory for creating SpawnFormatter
struct SpawnFormatterFactory;

impl FormatterFactory for SpawnFormatterFactory {
    fn create(&self, context: FormatterContext) -> Box<dyn BrpResponseFormatter> {
        // Extract components from the context params
        let components = context.params.unwrap_or_else(|| json!({}));

        Box::new(SpawnFormatter { components })
    }
}

/// Formatter for bevy/spawn responses
struct SpawnFormatter {
    components: Value,
}

impl BrpResponseFormatter for SpawnFormatter {
    fn format_success(&self, data: Value, _metadata: BrpMetadata) -> CallToolResult {
        // Extract spawned entity ID from response
        let entity_id = data.as_u64().unwrap_or(0);

        let component_count = if let Some(obj) = self.components.as_object() {
            obj.len()
        } else {
            0
        };

        let formatted_data = json!({
            JSON_FIELD_STATUS: RESPONSE_STATUS_SUCCESS,
            JSON_FIELD_MESSAGE: format!(
                "Successfully spawned entity {} with {} component(s)",
                entity_id,
                component_count
            ),
            JSON_FIELD_DATA: {
                JSON_FIELD_SPAWNED_ENTITY: entity_id,
                JSON_FIELD_COMPONENTS: self.components,
            }
        });

        json_tool_result(&formatted_data)
    }

    fn format_error(
        &self,
        error: super::support::response_processor::BrpError,
        metadata: BrpMetadata,
    ) -> CallToolResult {
        let formatted_error = json!({
            JSON_FIELD_STATUS: RESPONSE_STATUS_ERROR,
            JSON_FIELD_MESSAGE: error.message,
            JSON_FIELD_ERROR_CODE: error.code,
            JSON_FIELD_DATA: error.data,
            JSON_FIELD_METADATA: {
                JSON_FIELD_METHOD: metadata.method,
                JSON_FIELD_PORT: metadata.port,
                "requested_components": self.components
            }
        });

        json_tool_result(&formatted_error)
    }
}
