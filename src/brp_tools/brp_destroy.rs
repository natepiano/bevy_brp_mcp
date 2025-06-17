use rmcp::model::{CallToolResult, Tool};
use rmcp::service::RequestContext;
use rmcp::{Error as McpError, RoleServer};
use serde_json::{Value, json};

use super::constants::{
    BRP_METHOD_DESTROY, DEFAULT_BRP_PORT, JSON_FIELD_DESTROYED_ENTITY, JSON_FIELD_MESSAGE,
    JSON_FIELD_PORT, JSON_FIELD_STATUS, RESPONSE_STATUS_SUCCESS,
};
use super::support::generic_handler::{
    BrpHandlerConfig, EntityParamExtractor, FormatterContext, FormatterFactory, handle_generic,
};
use super::support::response_processor::{BrpError, BrpMetadata, BrpResponseFormatter};
use super::support::serialization::json_tool_result;
use crate::BrpMcpService;
use crate::brp_tools::constants::JSON_FIELD_ENTITY;
use crate::constants::{DESC_BRP_DESTROY, TOOL_BRP_DESTROY};
use crate::support::schema;

pub fn register_tool() -> Tool {
    Tool {
        name:         TOOL_BRP_DESTROY.into(),
        description:  DESC_BRP_DESTROY.into(),
        input_schema: schema::SchemaBuilder::new()
            .add_number_property(JSON_FIELD_ENTITY, "The entity ID to destroy", true)
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
        method:            BRP_METHOD_DESTROY,
        param_extractor:   Box::new(EntityParamExtractor { required: true }),
        formatter_factory: Box::new(DestroyFormatterFactory),
    };

    handle_generic(service, request, context, &config).await
}

/// Factory for creating DestroyFormatter
struct DestroyFormatterFactory;

impl FormatterFactory for DestroyFormatterFactory {
    fn create(&self, context: FormatterContext) -> Box<dyn BrpResponseFormatter> {
        // Extract entity from the context params
        let entity_id = context
            .params
            .as_ref()
            .and_then(|p| p.get(JSON_FIELD_ENTITY))
            .and_then(|v| v.as_u64())
            .unwrap_or(0);

        Box::new(DestroyFormatter { entity_id })
    }
}

/// Formatter for bevy/destroy responses
struct DestroyFormatter {
    entity_id: u64,
}

impl BrpResponseFormatter for DestroyFormatter {
    fn format_success(&self, _data: Value, _metadata: BrpMetadata) -> CallToolResult {
        let formatted_data = json!({
            JSON_FIELD_STATUS: RESPONSE_STATUS_SUCCESS,
            JSON_FIELD_MESSAGE: format!("Successfully destroyed entity {}", self.entity_id),
            JSON_FIELD_DESTROYED_ENTITY: self.entity_id,
        });

        json_tool_result(&formatted_data)
    }

    fn format_error(&self, error: BrpError, metadata: BrpMetadata) -> CallToolResult {
        use super::constants::{
            JSON_FIELD_ERROR_CODE, JSON_FIELD_METADATA, JSON_FIELD_METHOD, JSON_FIELD_PORT,
        };

        let formatted_error = json!({
            JSON_FIELD_STATUS: super::constants::RESPONSE_STATUS_ERROR,
            JSON_FIELD_MESSAGE: error.message,
            JSON_FIELD_ERROR_CODE: error.code,
            JSON_FIELD_METADATA: {
                JSON_FIELD_METHOD: metadata.method,
                JSON_FIELD_PORT: metadata.port,
                JSON_FIELD_ENTITY: self.entity_id
            }
        });

        json_tool_result(&formatted_error)
    }
}
