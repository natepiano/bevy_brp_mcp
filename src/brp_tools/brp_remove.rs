use rmcp::model::{CallToolResult, Tool};
use rmcp::service::RequestContext;
use rmcp::{Error as McpError, RoleServer};
use serde_json::{Value, json};

use super::constants::{
    BRP_METHOD_REMOVE, DEFAULT_BRP_PORT, JSON_FIELD_COMPONENTS, JSON_FIELD_DATA, JSON_FIELD_ENTITY,
    JSON_FIELD_ERROR_CODE, JSON_FIELD_MESSAGE, JSON_FIELD_METADATA, JSON_FIELD_METHOD,
    JSON_FIELD_PORT, JSON_FIELD_STATUS, RESPONSE_STATUS_ERROR, RESPONSE_STATUS_SUCCESS,
};
use super::support::generic_handler::{
    BrpHandlerConfig, FormatterContext, FormatterFactory, PassthroughExtractor, handle_generic,
};
use super::support::response_processor::{BrpMetadata, BrpResponseFormatter};
use crate::BrpMcpService;
use crate::constants::{DESC_BRP_REMOVE, TOOL_BRP_REMOVE};
use crate::support::schema;
use crate::support::serialization::json_tool_result;

pub fn register_tool() -> Tool {
    Tool {
        name:         TOOL_BRP_REMOVE.into(),
        description:  DESC_BRP_REMOVE.into(),
        input_schema: schema::SchemaBuilder::new()
            .add_number_property(
                JSON_FIELD_ENTITY,
                "The entity ID to remove components from",
                true,
            )
            .add_any_property(
                JSON_FIELD_COMPONENTS,
                "Array of component type names to remove",
                true,
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
        method:            BRP_METHOD_REMOVE,
        param_extractor:   Box::new(PassthroughExtractor),
        formatter_factory: Box::new(RemoveFormatterFactory),
    };

    handle_generic(service, request, context, &config).await
}

/// Factory for creating RemoveFormatter
struct RemoveFormatterFactory;

impl FormatterFactory for RemoveFormatterFactory {
    fn create(&self, context: FormatterContext) -> Box<dyn BrpResponseFormatter> {
        // Extract entity and components from the context params
        let entity_id = context
            .params
            .as_ref()
            .and_then(|p| p.get(JSON_FIELD_ENTITY))
            .and_then(|v| v.as_u64())
            .unwrap_or(0);

        let components = context
            .params
            .as_ref()
            .and_then(|p| p.get(JSON_FIELD_COMPONENTS))
            .and_then(|v| v.as_array())
            .cloned()
            .unwrap_or_default();

        Box::new(RemoveFormatter {
            entity_id,
            components,
        })
    }
}

/// Formatter for bevy/remove responses
struct RemoveFormatter {
    entity_id:  u64,
    components: Vec<Value>,
}

impl BrpResponseFormatter for RemoveFormatter {
    fn format_success(&self, _data: Value, _metadata: BrpMetadata) -> CallToolResult {
        let formatted_data = json!({
            JSON_FIELD_STATUS: RESPONSE_STATUS_SUCCESS,
            JSON_FIELD_MESSAGE: format!(
                "Successfully removed {} component(s) from entity {}",
                self.components.len(),
                self.entity_id
            ),
            JSON_FIELD_DATA: {
                JSON_FIELD_ENTITY: self.entity_id,
                "removed_components": self.components,
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
                JSON_FIELD_ENTITY: self.entity_id,
                "requested_components": self.components
            }
        });

        json_tool_result(&formatted_error)
    }
}
