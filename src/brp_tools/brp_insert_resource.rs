use rmcp::model::{CallToolResult, Tool};
use rmcp::service::RequestContext;
use rmcp::{Error as McpError, RoleServer};
use serde_json::{Value, json};

use super::constants::{
    BRP_METHOD_INSERT_RESOURCE, DEFAULT_BRP_PORT, JSON_FIELD_DATA, JSON_FIELD_ERROR_CODE,
    JSON_FIELD_MESSAGE, JSON_FIELD_METADATA, JSON_FIELD_METHOD, JSON_FIELD_PORT,
    JSON_FIELD_RESOURCE, JSON_FIELD_STATUS, RESPONSE_STATUS_ERROR, RESPONSE_STATUS_SUCCESS,
};
use super::support::generic_handler::{
    BrpHandlerConfig, FormatterContext, FormatterFactory, PassthroughExtractor, handle_generic,
};
use super::support::response_processor::{BrpMetadata, BrpResponseFormatter};
use crate::BrpMcpService;
use crate::constants::{DESC_BRP_INSERT_RESOURCE, TOOL_BRP_INSERT_RESOURCE};
use crate::support::schema;
use crate::support::serialization::json_tool_result;

pub fn register_tool() -> Tool {
    Tool {
        name:         TOOL_BRP_INSERT_RESOURCE.into(),
        description:  DESC_BRP_INSERT_RESOURCE.into(),
        input_schema: schema::SchemaBuilder::new()
            .add_string_property(
                JSON_FIELD_RESOURCE,
                "The fully-qualified type name of the resource to insert or update",
                true,
            )
            .add_any_property("value", "The resource value to insert", true)
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
        method:            BRP_METHOD_INSERT_RESOURCE,
        param_extractor:   Box::new(PassthroughExtractor),
        formatter_factory: Box::new(InsertResourceFormatterFactory),
    };

    handle_generic(service, request, context, &config).await
}

/// Factory for creating InsertResourceFormatter
struct InsertResourceFormatterFactory;

impl FormatterFactory for InsertResourceFormatterFactory {
    fn create(&self, context: FormatterContext) -> Box<dyn BrpResponseFormatter> {
        // Extract resource from the context params
        let resource = context
            .params
            .as_ref()
            .and_then(|p| p.get(JSON_FIELD_RESOURCE))
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        Box::new(InsertResourceFormatter { resource })
    }
}

/// Formatter for bevy/insert_resource responses
struct InsertResourceFormatter {
    resource: String,
}

impl BrpResponseFormatter for InsertResourceFormatter {
    fn format_success(&self, _data: Value, _metadata: BrpMetadata) -> CallToolResult {
        let formatted_data = json!({
            JSON_FIELD_STATUS: RESPONSE_STATUS_SUCCESS,
            JSON_FIELD_MESSAGE: format!("Successfully inserted/updated resource '{}'", self.resource),
            JSON_FIELD_DATA: {
                JSON_FIELD_RESOURCE: self.resource
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
                JSON_FIELD_RESOURCE: self.resource
            }
        });

        json_tool_result(&formatted_error)
    }
}
