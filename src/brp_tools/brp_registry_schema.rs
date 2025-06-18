use rmcp::model::{CallToolResult, Tool};
use rmcp::service::RequestContext;
use rmcp::{Error as McpError, RoleServer};
use serde_json::{Value, json};

use super::constants::{
    BRP_METHOD_REGISTRY_SCHEMA, DEFAULT_BRP_PORT, JSON_FIELD_DATA, JSON_FIELD_MESSAGE,
    JSON_FIELD_PORT, JSON_FIELD_STATUS, RESPONSE_STATUS_SUCCESS,
};
use super::support::generic_handler::{
    BrpHandlerConfig, FormatterContext, FormatterFactory, ParamExtractor, handle_generic,
};
use super::support::response_processor::{
    BrpError, BrpMetadata, BrpResponseFormatter, format_error_default,
};
use crate::BrpMcpService;
use crate::constants::{DESC_BRP_REGISTRY_SCHEMA, TOOL_BRP_REGISTRY_SCHEMA};
use crate::support::params::{extract_optional_number, extract_optional_string_array_from_request};
use crate::support::schema;
use crate::support::serialization::json_tool_result;

pub fn register_tool() -> Tool {
    Tool {
        name: TOOL_BRP_REGISTRY_SCHEMA.into(),
        description: DESC_BRP_REGISTRY_SCHEMA.into(),
        input_schema: schema::SchemaBuilder::new()
            .add_string_array_property(
                "with_crates",
                "Include only types from these crates (e.g., [\"bevy_transform\", \"my_game\"])",
                false
            )
            .add_string_array_property(
                "without_crates", 
                "Exclude types from these crates (e.g., [\"bevy_render\", \"bevy_pbr\"])",
                false
            )
            .add_string_array_property(
                "with_types",
                "Include only types with these reflect traits (e.g., [\"Component\", \"Resource\"])",
                false
            )
            .add_string_array_property(
                "without_types",
                "Exclude types with these reflect traits (e.g., [\"RenderResource\"])",
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
        method:            BRP_METHOD_REGISTRY_SCHEMA,
        param_extractor:   Box::new(RegistrySchemaParamExtractor),
        formatter_factory: Box::new(RegistrySchemaFormatterFactory),
    };

    handle_generic(service, request, context, &config).await
}

/// Parameter extractor for registry/schema method
///
/// Transforms individual filter parameters into the query structure expected by the BRP method:
/// - with_crates: Include only types from specified crates
/// - without_crates: Exclude types from specified crates
/// - with_types: Include only types with specified reflect traits
/// - without_types: Exclude types with specified reflect traits
struct RegistrySchemaParamExtractor;

impl ParamExtractor for RegistrySchemaParamExtractor {
    fn extract(
        &self,
        request: &rmcp::model::CallToolRequestParam,
    ) -> Result<(Option<Value>, u16), McpError> {
        use serde_json::json;

        let port =
            extract_optional_number(request, JSON_FIELD_PORT, DEFAULT_BRP_PORT as u64)? as u16;

        // Extract the individual filter parameters
        let with_crates = extract_optional_string_array_from_request(request, "with_crates")?;
        let without_crates = extract_optional_string_array_from_request(request, "without_crates")?;
        let with_types = extract_optional_string_array_from_request(request, "with_types")?;
        let without_types = extract_optional_string_array_from_request(request, "without_types")?;

        // Build the query object if any filters are provided
        // The BRP method expects a JSON object with filter fields
        let params = if with_crates.is_some()
            || without_crates.is_some()
            || with_types.is_some()
            || without_types.is_some()
        {
            let mut query = serde_json::Map::new();

            // Add crate filters
            if let Some(crates) = with_crates {
                query.insert("with_crates".to_string(), json!(crates));
            }
            if let Some(crates) = without_crates {
                query.insert("without_crates".to_string(), json!(crates));
            }

            // Add reflect trait filters
            if let Some(types) = with_types {
                query.insert("with_types".to_string(), json!(types));
            }
            if let Some(types) = without_types {
                query.insert("without_types".to_string(), json!(types));
            }

            // Return the query object directly as a Value::Object
            Some(Value::Object(query))
        } else {
            // No filters provided, return None to get all schemas
            None
        };

        Ok((params, port))
    }
}

/// Factory for creating RegistrySchemaFormatter
struct RegistrySchemaFormatterFactory;

impl FormatterFactory for RegistrySchemaFormatterFactory {
    fn create(&self, context: FormatterContext) -> Box<dyn BrpResponseFormatter> {
        // Check if any filters were applied based on the parameters
        let filters_applied = context.params.is_some();
        Box::new(RegistrySchemaFormatter::new(filters_applied))
    }
}

/// Formatter for bevy/registry/schema responses
///
/// Provides detailed feedback about the number of schemas returned and any applied filters
struct RegistrySchemaFormatter {
    filters_applied: bool,
}

impl RegistrySchemaFormatter {
    fn new(filters_applied: bool) -> Self {
        Self { filters_applied }
    }
}

impl BrpResponseFormatter for RegistrySchemaFormatter {
    fn format_success(&self, data: Value, _metadata: BrpMetadata) -> CallToolResult {
        // Count the number of schemas returned
        let schema_count = if data.is_object() {
            data.as_object().unwrap().len()
        } else if data.is_array() {
            data.as_array().unwrap().len()
        } else {
            0
        };

        let message = if self.filters_applied {
            format!(
                "Retrieved schema information for {} filtered type(s)",
                schema_count
            )
        } else {
            format!(
                "Retrieved schema information for {} type(s) (all registered types)",
                schema_count
            )
        };

        let formatted_data = json!({
            JSON_FIELD_STATUS: RESPONSE_STATUS_SUCCESS,
            JSON_FIELD_MESSAGE: message,
            JSON_FIELD_DATA: data,
        });

        json_tool_result(&formatted_data)
    }

    fn format_error(&self, error: BrpError, metadata: BrpMetadata) -> CallToolResult {
        format_error_default(error, metadata)
    }
}
