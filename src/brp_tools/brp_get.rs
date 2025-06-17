use rmcp::model::{CallToolResult, Tool};
use rmcp::service::RequestContext;
use rmcp::{Error as McpError, RoleServer};
use serde_json::{Value, json};

use super::constants::{
    BRP_METHOD_GET, DEFAULT_BRP_PORT, JSON_FIELD_COMPONENTS, JSON_FIELD_DATA, JSON_FIELD_ENTITY,
    JSON_FIELD_PORT, JSON_FIELD_STATUS, RESPONSE_STATUS_SUCCESS,
};
use super::support::generic_handler::{
    BrpHandlerConfig, FormatterContext, FormatterFactory, PassthroughExtractor, handle_generic,
};
use super::support::response_processor::{BrpMetadata, BrpResponseFormatter};
use super::support::serialization::json_tool_result;
use crate::BrpMcpService;
use crate::brp_tools::constants::JSON_FIELD_REQUESTED_COMPONENTS;
use crate::constants::{DESC_BRP_GET, TOOL_BRP_GET};
use crate::support::schema;

pub fn register_tool() -> Tool {
    Tool {
        name: TOOL_BRP_GET.into(),
        description: DESC_BRP_GET.into(),
        input_schema: schema::SchemaBuilder::new()
            .add_number_property(JSON_FIELD_ENTITY, "The entity ID to get component data from", true)
            .add_any_property(
                JSON_FIELD_COMPONENTS,
                "Array of component types to retrieve. Each component must be a fully-qualified type name",
                true
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
        method:            BRP_METHOD_GET,
        param_extractor:   Box::new(PassthroughExtractor),
        formatter_factory: Box::new(GetFormatterFactory),
    };

    handle_generic(service, request, context, &config).await
}

/// Factory for creating GetFormatter
struct GetFormatterFactory;

impl FormatterFactory for GetFormatterFactory {
    fn create(&self, context: FormatterContext) -> Box<dyn BrpResponseFormatter> {
        // Extract entity and components from the context params
        let entity_id = context
            .params
            .as_ref()
            .and_then(|p| p.get(JSON_FIELD_ENTITY))
            .and_then(|v| v.as_u64())
            .unwrap_or(0);

        let requested_components = context
            .params
            .as_ref()
            .and_then(|p| p.get(JSON_FIELD_COMPONENTS))
            .and_then(|v| v.as_array())
            .cloned()
            .unwrap_or_default();

        Box::new(GetFormatter {
            entity_id,
            requested_components,
        })
    }
}

/// Formatter for bevy/get responses
struct GetFormatter {
    entity_id:            u64,
    requested_components: Vec<Value>,
}

impl BrpResponseFormatter for GetFormatter {
    fn format_success(&self, data: Value, _metadata: BrpMetadata) -> CallToolResult {
        // Extract component data
        let component_count = if data.is_object() {
            data.as_object().unwrap().len()
        } else {
            0
        };

        let formatted_data = json!({
            JSON_FIELD_STATUS: RESPONSE_STATUS_SUCCESS,
            JSON_FIELD_MESSAGE: format!(
                "Retrieved {} component(s) from entity {}",
                component_count,
                self.entity_id
            ),
            JSON_FIELD_DATA: {
                JSON_FIELD_ENTITY: self.entity_id,
                JSON_FIELD_COMPONENTS: data,
            }
        });

        json_tool_result(&formatted_data)
    }

    fn format_error(
        &self,
        error: super::support::response_processor::BrpError,
        metadata: BrpMetadata,
    ) -> CallToolResult {
        use super::support::response_processor::format_error_default;
        format_error_default(error, metadata)
    }
}
