use rmcp::model::{CallToolResult, Tool};
use rmcp::service::RequestContext;
use rmcp::{Error as McpError, RoleServer};
use serde_json::{Value, json};

use super::constants::{
    BRP_METHOD_LIST, DEFAULT_BRP_PORT, JSON_FIELD_COMPONENT_COUNT, JSON_FIELD_DATA,
    JSON_FIELD_ENTITY, JSON_FIELD_MESSAGE, JSON_FIELD_PORT, JSON_FIELD_STATUS,
    RESPONSE_STATUS_SUCCESS,
};
use super::support::generic_handler::{
    BrpHandlerConfig, EntityParamExtractor, FormatterContext, FormatterFactory, handle_generic,
};
use super::support::response_processor::{
    BrpError, BrpMetadata, BrpResponseFormatter, format_error_default,
};
use crate::BrpMcpService;
use crate::constants::{DESC_BRP_LIST, TOOL_BRP_LIST};
use crate::support::schema;
use crate::support::serialization::json_tool_result;

pub fn register_tool() -> Tool {
    Tool {
        name:         TOOL_BRP_LIST.into(),
        description:  DESC_BRP_LIST.into(),
        input_schema: schema::SchemaBuilder::new()
            .add_number_property(
                JSON_FIELD_ENTITY,
                "Optional entity ID to list components for",
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
        method:            BRP_METHOD_LIST,
        param_extractor:   Box::new(EntityParamExtractor { required: false }),
        formatter_factory: Box::new(ListFormatterFactory),
    };

    handle_generic(service, request, context, &config).await
}

/// Factory for creating ListFormatter
struct ListFormatterFactory;

impl FormatterFactory for ListFormatterFactory {
    fn create(&self, context: FormatterContext) -> Box<dyn BrpResponseFormatter> {
        let entity_id = context
            .params
            .as_ref()
            .and_then(|p| p.get(JSON_FIELD_ENTITY))
            .and_then(serde_json::Value::as_u64);

        Box::new(ListFormatter { entity_id })
    }
}

/// Formatter for bevy/list responses
struct ListFormatter {
    entity_id: Option<u64>,
}

impl BrpResponseFormatter for ListFormatter {
    fn format_success(&self, data: Value, _metadata: BrpMetadata) -> CallToolResult {
        let components = data.as_array().cloned().unwrap_or_default();

        let message = if let Some(entity_id) = self.entity_id {
            format!(
                "Found {} component(s) on entity {}",
                components.len(),
                entity_id
            )
        } else {
            format!("Found {} registered component type(s)", components.len())
        };

        let mut result = json!({
            JSON_FIELD_STATUS: RESPONSE_STATUS_SUCCESS,
            JSON_FIELD_MESSAGE: message,
            JSON_FIELD_DATA: components,
            JSON_FIELD_COMPONENT_COUNT: components.len(),
        });

        if let Some(entity_id) = self.entity_id {
            result
                .as_object_mut()
                .unwrap()
                .insert(JSON_FIELD_ENTITY.to_string(), json!(entity_id));
        }

        json_tool_result(&result)
    }

    fn format_error(&self, error: BrpError, metadata: BrpMetadata) -> CallToolResult {
        format_error_default(error, metadata)
    }
}
