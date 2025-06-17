use rmcp::model::{CallToolResult, Tool};
use rmcp::service::RequestContext;
use rmcp::{Error as McpError, RoleServer};
use serde_json::{Value, json};

use super::constants::{
    BRP_METHOD_MUTATE_COMPONENT, DEFAULT_BRP_PORT, JSON_FIELD_COMPONENT, JSON_FIELD_DATA,
    JSON_FIELD_ENTITY, JSON_FIELD_ERROR_CODE, JSON_FIELD_MESSAGE, JSON_FIELD_METADATA,
    JSON_FIELD_METHOD, JSON_FIELD_PATH, JSON_FIELD_PORT, JSON_FIELD_STATUS, RESPONSE_STATUS_ERROR,
    RESPONSE_STATUS_SUCCESS,
};
use super::support::generic_handler::{
    BrpHandlerConfig, FormatterContext, FormatterFactory, PassthroughExtractor, handle_generic,
};
use super::support::response_processor::{BrpMetadata, BrpResponseFormatter};
use super::support::serialization::json_tool_result;
use crate::BrpMcpService;
use crate::constants::{DESC_BRP_MUTATE_COMPONENT, TOOL_BRP_MUTATE_COMPONENT};
use crate::support::schema;

pub fn register_tool() -> Tool {
    Tool {
        name:         TOOL_BRP_MUTATE_COMPONENT.into(),
        description:  DESC_BRP_MUTATE_COMPONENT.into(),
        input_schema: schema::SchemaBuilder::new()
            .add_number_property(
                JSON_FIELD_ENTITY,
                "The entity ID containing the component to mutate",
                true,
            )
            .add_string_property(
                JSON_FIELD_COMPONENT,
                "The fully-qualified type name of the component to mutate",
                true,
            )
            .add_string_property(
                JSON_FIELD_PATH,
                "The path to the field within the component (e.g., 'translation.x')",
                true,
            )
            .add_any_property("value", "The new value for the field", true)
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
        method:            BRP_METHOD_MUTATE_COMPONENT,
        param_extractor:   Box::new(PassthroughExtractor),
        formatter_factory: Box::new(MutateComponentFormatterFactory),
    };

    handle_generic(service, request, context, &config).await
}

/// Factory for creating MutateComponentFormatter
struct MutateComponentFormatterFactory;

impl FormatterFactory for MutateComponentFormatterFactory {
    fn create(&self, context: FormatterContext) -> Box<dyn BrpResponseFormatter> {
        // Extract entity, component, and path from the context params
        let entity = context
            .params
            .as_ref()
            .and_then(|p| p.get(JSON_FIELD_ENTITY))
            .and_then(|v| v.as_u64())
            .unwrap_or(0);

        let component = context
            .params
            .as_ref()
            .and_then(|p| p.get(JSON_FIELD_COMPONENT))
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        let path = context
            .params
            .as_ref()
            .and_then(|p| p.get(JSON_FIELD_PATH))
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        Box::new(MutateComponentFormatter {
            entity,
            component,
            path,
        })
    }
}

/// Formatter for bevy/mutate_component responses
struct MutateComponentFormatter {
    entity:    u64,
    component: String,
    path:      String,
}

impl BrpResponseFormatter for MutateComponentFormatter {
    fn format_success(&self, _data: Value, _metadata: BrpMetadata) -> CallToolResult {
        let formatted_data = json!({
            JSON_FIELD_STATUS: RESPONSE_STATUS_SUCCESS,
            JSON_FIELD_MESSAGE: format!(
                "Successfully mutated field '{}' in component '{}' on entity {}",
                self.path, self.component, self.entity
            ),
            JSON_FIELD_DATA: {
                JSON_FIELD_ENTITY: self.entity,
                JSON_FIELD_COMPONENT: self.component,
                JSON_FIELD_PATH: self.path
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
                JSON_FIELD_ENTITY: self.entity,
                JSON_FIELD_COMPONENT: self.component,
                JSON_FIELD_PATH: self.path
            }
        });

        json_tool_result(&formatted_error)
    }
}
