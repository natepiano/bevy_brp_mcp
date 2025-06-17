use rmcp::model::{CallToolResult, Tool};
use rmcp::service::RequestContext;
use rmcp::{Error as McpError, RoleServer};
use serde_json::{Value, json};

use super::constants::{
    BRP_METHOD_REPARENT, DEFAULT_BRP_PORT, JSON_FIELD_DATA, JSON_FIELD_ENTITIES,
    JSON_FIELD_ERROR_CODE, JSON_FIELD_MESSAGE, JSON_FIELD_METADATA, JSON_FIELD_METHOD,
    JSON_FIELD_PARENT, JSON_FIELD_PORT, JSON_FIELD_STATUS, RESPONSE_STATUS_ERROR,
    RESPONSE_STATUS_SUCCESS,
};
use super::support::generic_handler::{
    BrpHandlerConfig, FormatterContext, FormatterFactory, PassthroughExtractor, handle_generic,
};
use super::support::response_processor::{BrpMetadata, BrpResponseFormatter};
use super::support::serialization::json_tool_result;
use crate::BrpMcpService;
use crate::constants::{DESC_BRP_REPARENT, TOOL_BRP_REPARENT};
use crate::support::schema;

pub fn register_tool() -> Tool {
    Tool {
        name:         TOOL_BRP_REPARENT.into(),
        description:  DESC_BRP_REPARENT.into(),
        input_schema: schema::SchemaBuilder::new()
            .add_any_property(JSON_FIELD_ENTITIES, "Array of entity IDs to reparent", true)
            .add_number_property(
                JSON_FIELD_PARENT,
                "The new parent entity ID (omit to remove parent)",
                false,
            )
            .add_number_property(
                JSON_FIELD_PORT,
                &format!("The BRP port (default: {})", DEFAULT_BRP_PORT),
                false,
            )
            .build(),
    }
}

pub async fn handle(
    service: &BrpMcpService,
    request: rmcp::model::CallToolRequestParam,
    context: RequestContext<RoleServer>,
) -> Result<CallToolResult, McpError> {
    let config = BrpHandlerConfig {
        method:            BRP_METHOD_REPARENT,
        param_extractor:   Box::new(PassthroughExtractor),
        formatter_factory: Box::new(ReparentFormatterFactory),
    };

    handle_generic(service, request, context, &config).await
}

/// Factory for creating ReparentFormatter
struct ReparentFormatterFactory;

impl FormatterFactory for ReparentFormatterFactory {
    fn create(&self, context: FormatterContext) -> Box<dyn BrpResponseFormatter> {
        // Extract entities and parent from the context params
        let entities = context
            .params
            .as_ref()
            .and_then(|p| p.get(JSON_FIELD_ENTITIES))
            .and_then(|v| v.as_array())
            .map(|arr| arr.len())
            .unwrap_or(0);

        let parent = context
            .params
            .as_ref()
            .and_then(|p| p.get(JSON_FIELD_PARENT))
            .and_then(|v| v.as_u64());

        Box::new(ReparentFormatter {
            entity_count: entities,
            parent,
        })
    }
}

/// Formatter for bevy/reparent responses
struct ReparentFormatter {
    entity_count: usize,
    parent:       Option<u64>,
}

impl BrpResponseFormatter for ReparentFormatter {
    fn format_success(&self, _data: Value, _metadata: BrpMetadata) -> CallToolResult {
        let message = if let Some(parent_id) = self.parent {
            format!(
                "Successfully reparented {} entities to parent {}",
                self.entity_count, parent_id
            )
        } else {
            format!(
                "Successfully removed parent from {} entities",
                self.entity_count
            )
        };

        let formatted_data = json!({
            JSON_FIELD_STATUS: RESPONSE_STATUS_SUCCESS,
            JSON_FIELD_MESSAGE: message,
            JSON_FIELD_DATA: {
                JSON_FIELD_ENTITIES: self.entity_count,
                JSON_FIELD_PARENT: self.parent
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
                JSON_FIELD_ENTITIES: self.entity_count,
                JSON_FIELD_PARENT: self.parent
            }
        });

        json_tool_result(&formatted_error)
    }
}
