use rmcp::model::{CallToolRequestParam, CallToolResult, Tool};
use rmcp::service::RequestContext;
use rmcp::{Error as McpError, RoleServer};
use serde_json::Value;

use super::constants::{
    BRP_METHOD_REGISTRY_SCHEMA, DEFAULT_BRP_PORT, JSON_FIELD_DATA, JSON_FIELD_PORT,
};
use super::support::{
    BrpHandlerConfig, ExtractedParams, ParamExtractor, ResponseFormatterFactory, extractors, handle_brp_request,
};
use crate::BrpMcpService;
use crate::constants::{DESC_BRP_REGISTRY_SCHEMA, TOOL_BRP_REGISTRY_SCHEMA};
use crate::support::params::{extract_optional_number, extract_optional_string_array_from_request};
use crate::support::schema;

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
            .add_number_property(JSON_FIELD_PORT, &format!("The BRP port (default: {DEFAULT_BRP_PORT})" ), false)
            .build(),
    }
}

pub async fn handle(
    service: &BrpMcpService,
    request: CallToolRequestParam,
    context: RequestContext<RoleServer>,
) -> Result<CallToolResult, McpError> {
    let config = BrpHandlerConfig {
        method:            Some(BRP_METHOD_REGISTRY_SCHEMA),
        param_extractor:   Box::new(RegistrySchemaParamExtractor),
        formatter_factory: ResponseFormatterFactory::pass_through()
            .with_template("Retrieved schema information")
            .with_response_field(JSON_FIELD_DATA, extractors::pass_through_data)
            .with_default_error()
            .build(),
    };

    handle_brp_request(service, request, context, &config).await
}

/// Parameter extractor for registry/schema method
///
/// Transforms individual filter parameters into the query structure expected by the BRP method:
/// - `with_crates`: Include only types from specified crates
/// - `without_crates`: Exclude types from specified crates
/// - `with_types`: Include only types with specified reflect traits
/// - `without_types`: Exclude types with specified reflect traits
struct RegistrySchemaParamExtractor;

impl ParamExtractor for RegistrySchemaParamExtractor {
    fn extract(
        &self,
        request: &rmcp::model::CallToolRequestParam,
    ) -> Result<ExtractedParams, McpError> {
        use serde_json::json;

        let port = u16::try_from(extract_optional_number(
            request,
            JSON_FIELD_PORT,
            u64::from(DEFAULT_BRP_PORT),
        )?)
        .map_err(|_| {
            McpError::invalid_params("Port number must be a valid u16".to_string(), None)
        })?;

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

        Ok(ExtractedParams {
            method: None,
            params,
            port,
        })
    }
}
