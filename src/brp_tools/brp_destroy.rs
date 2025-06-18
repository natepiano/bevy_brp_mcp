use rmcp::model::{CallToolResult, Tool};
use rmcp::service::RequestContext;
use rmcp::{Error as McpError, RoleServer};
use serde_json::{Value, json};

use super::constants::{
    BRP_METHOD_DESTROY, DEFAULT_BRP_PORT, JSON_FIELD_DESTROYED_ENTITY, JSON_FIELD_PORT,
};
use super::support::generic_handler::{
    BrpHandlerConfig, EntityParamExtractor, FormatterContext, FormatterFactory, handle_generic,
};
use super::support::response_processor::{BrpError, BrpMetadata, BrpResponseFormatter};
use crate::BrpMcpService;
use crate::brp_tools::constants::JSON_FIELD_ENTITY;
use crate::constants::{DESC_BRP_DESTROY, TOOL_BRP_DESTROY};
use crate::support::response::ResponseBuilder;
use crate::support::schema;
use crate::support::serialization::json_response_to_result;

pub fn register_tool() -> Tool {
    Tool {
        name:         TOOL_BRP_DESTROY.into(),
        description:  DESC_BRP_DESTROY.into(),
        input_schema: schema::SchemaBuilder::new()
            .add_number_property(JSON_FIELD_ENTITY, "The entity ID to destroy", true)
            .add_number_property(
                JSON_FIELD_PORT,
                &format!("The BRP port (default: {DEFAULT_BRP_PORT})"),
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

/// Factory for creating `DestroyFormatter`
struct DestroyFormatterFactory;

impl FormatterFactory for DestroyFormatterFactory {
    fn create(&self, context: FormatterContext) -> Box<dyn BrpResponseFormatter> {
        // Extract entity from the context params
        let entity_id = context
            .params
            .as_ref()
            .and_then(|p| p.get(JSON_FIELD_ENTITY))
            .and_then(serde_json::Value::as_u64)
            .unwrap_or(0);

        Box::new(DestroyFormatter { entity_id })
    }
}

/// Formatter for `bevy/destroy` responses
struct DestroyFormatter {
    entity_id: u64,
}

impl BrpResponseFormatter for DestroyFormatter {
    fn format_success(&self, _data: Value, _metadata: BrpMetadata) -> CallToolResult {
        let response = ResponseBuilder::success()
            .message(format!("Successfully destroyed entity {}", self.entity_id))
            .add_field(JSON_FIELD_DESTROYED_ENTITY, self.entity_id)
            .build();

        json_response_to_result(response)
    }

    fn format_error(&self, error: BrpError, metadata: BrpMetadata) -> CallToolResult {
        use super::constants::{
            JSON_FIELD_ERROR_CODE, JSON_FIELD_METADATA, JSON_FIELD_METHOD, JSON_FIELD_PORT,
        };

        let response = ResponseBuilder::error()
            .message(&error.message)
            .add_field(JSON_FIELD_ERROR_CODE, error.code)
            .add_field(
                JSON_FIELD_METADATA,
                json!({
                    JSON_FIELD_METHOD: metadata.method,
                    JSON_FIELD_PORT: metadata.port,
                    JSON_FIELD_ENTITY: self.entity_id
                }),
            )
            .build();

        json_response_to_result(response)
    }
}
