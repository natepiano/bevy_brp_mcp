//! Tool generation from declarative definitions.
//!
//! This module provides the runtime code generation logic that converts declarative
//! tool definitions into actual MCP tool registrations and request handlers. It acts
//! as the bridge between the static tool definitions in `tool_definitions.rs` and
//! the runtime tool handling system.
//!
//! # Architecture
//!
//! The generator provides two main functions:
//! - **`generate_tool_registration()`**: Converts a `BrpToolDef` into an MCP `Tool` for
//!   registration
//! - **`generate_tool_handler()`**: Creates a request handler that processes tool calls
//!
//! # Code Generation Process
//!
//! ## Tool Registration Generation
//! ```
//! BrpToolDef → generate_tool_registration() → rmcp::model::Tool
//! ```
//!
//! The registration generator:
//! 1. Extracts tool name and description
//! 2. Builds JSON schema from parameter definitions
//! 3. Creates the MCP Tool structure for discovery
//!
//! ## Handler Generation
//! ```
//! BrpToolDef + Request → generate_tool_handler() → CallToolResult
//! ```
//!
//! The handler generator:
//! 1. Selects appropriate parameter extractor based on tool definition
//! 2. Builds response formatter from formatter definition
//! 3. Configures BRP request handling pipeline
//! 4. Executes the BRP request and formats the response
//!
//! # Parameter Extractors
//!
//! Different tool types use different parameter extraction strategies:
//!
//! - **`Passthrough`**: Passes all parameters unchanged to BRP
//! - **`Entity`**: Extracts entity ID (required or optional)
//! - **`Resource`**: Extracts resource name parameter
//! - **`EmptyParams`**: No parameters (e.g., list operations)
//! - **`BrpExecute`**: Dynamic method selection from parameters
//! - **`RegistrySchema`**: Complex parameter transformation for registry queries
//!
//! # Response Formatters
//!
//! Response formatting is configured based on the operation type:
//!
//! - **`EntityOperation`**: Operations on specific entities with dynamic field extraction
//! - **`ResourceOperation`**: Operations on specific resources
//! - **`PassThrough`**: Direct response forwarding
//! - **`ListOperation`**: List-based responses
//!
//! # Example Usage
//!
//! ```rust
//! // Generate tool registration for discovery
//! let tool = generate_tool_registration(&brp_tool_def);
//!
//! // Handle incoming request
//! let result = generate_tool_handler(&brp_tool_def, service, request, context).await?;
//! ```
//!
//! # Error Handling
//!
//! The generator handles errors at multiple levels:
//! - Parameter validation errors
//! - BRP communication errors
//! - Response formatting errors
//! - Invalid tool definition errors
//!
//! All errors are converted to appropriate MCP error responses with helpful messages.

use rmcp::model::{CallToolRequestParam, CallToolResult, Tool};
use rmcp::service::RequestContext;
use rmcp::{Error as McpError, RoleServer};

use super::support::{
    BrpExecuteExtractor, BrpHandlerConfig, EntityParamExtractor, FormatterContext, ParamExtractor,
    PassthroughExtractor, RegistrySchemaParamExtractor, ResourceParamExtractor,
    ResponseFormatterFactory, SimplePortExtractor, extractors, handle_brp_request,
};
use super::tool_definitions::{
    BrpToolDef, ExtractorType, FormatterType, ParamExtractorType, ParamType,
};
use crate::BrpMcpService;
use crate::support::schema;
use crate::tools::{
    JSON_FIELD_COMPONENTS, JSON_FIELD_ENTITIES, JSON_FIELD_ENTITY, JSON_FIELD_PARENT,
};

/// Generate tool registration from a declarative definition
pub fn generate_tool_registration(def: &BrpToolDef) -> Tool {
    let mut builder = schema::SchemaBuilder::new();

    // Add all parameters to the schema
    for param in &def.params {
        builder = match param.param_type {
            ParamType::Number => {
                builder.add_number_property(param.name, param.description, param.required)
            }
            ParamType::String => {
                builder.add_string_property(param.name, param.description, param.required)
            }
            ParamType::Boolean => {
                builder.add_boolean_property(param.name, param.description, param.required)
            }
            ParamType::StringArray => {
                builder.add_string_array_property(param.name, param.description, param.required)
            }
            ParamType::Any => {
                builder.add_any_property(param.name, param.description, param.required)
            }
        };
    }

    Tool {
        name:         def.name.into(),
        description:  def.description.into(),
        input_schema: builder.build(),
    }
}

/// Generate a handler function for a declarative tool definition
pub async fn generate_tool_handler(
    def: &BrpToolDef,
    service: &BrpMcpService,
    request: CallToolRequestParam,
    context: RequestContext<RoleServer>,
) -> Result<CallToolResult, McpError> {
    // Create the parameter extractor based on the definition
    let param_extractor: Box<dyn ParamExtractor> = match &def.param_extractor {
        ParamExtractorType::Passthrough => Box::new(PassthroughExtractor),
        ParamExtractorType::Entity { required } => Box::new(EntityParamExtractor {
            required: *required,
        }),
        ParamExtractorType::Resource => Box::new(ResourceParamExtractor),
        ParamExtractorType::EmptyParams => Box::new(SimplePortExtractor),
        ParamExtractorType::BrpExecute => Box::new(BrpExecuteExtractor),
        ParamExtractorType::RegistrySchema => Box::new(RegistrySchemaParamExtractor),
    };

    // Create the formatter factory based on the definition
    let mut formatter_builder = match &def.formatter.formatter_type {
        FormatterType::EntityOperation(field) => ResponseFormatterFactory::entity_operation(field),
        FormatterType::ResourceOperation => ResponseFormatterFactory::resource_operation(""),
        FormatterType::Simple => ResponseFormatterFactory::list_operation(),
    };

    // Set the template if provided
    if !def.formatter.template.is_empty() {
        formatter_builder = formatter_builder.with_template(def.formatter.template);
    }

    // Add response fields
    for field in &def.formatter.response_fields {
        formatter_builder = formatter_builder
            .with_response_field(field.name, convert_extractor_type(&field.extractor));
    }

    // Always add default error handling
    formatter_builder = formatter_builder.with_default_error();

    let config = BrpHandlerConfig {
        method: Some(def.method),
        param_extractor,
        formatter_factory: formatter_builder.build(),
    };

    handle_brp_request(service, request, context, &config).await
}

/// Convert our `ExtractorType` enum to the actual extractor function
fn convert_extractor_type(extractor_type: &ExtractorType) -> super::support::FieldExtractor {
    match extractor_type {
        ExtractorType::EntityFromParams => extractors::entity_from_params,
        ExtractorType::ResourceFromParams => extractors::resource_from_params,
        ExtractorType::PassThroughData => extractors::pass_through_data,
        ExtractorType::PassThroughResult => |data, _| data.clone(),
        ExtractorType::EntityCountFromData | ExtractorType::ComponentCountFromData => {
            extractors::array_count
        }
        ExtractorType::EntityFromResponse => extract_entity_from_response,
        ExtractorType::QueryComponentCount => extract_query_component_count,
        ExtractorType::QueryParamsFromContext => extract_query_params_from_context,
        ExtractorType::ParamFromContext(param_name) => match *param_name {
            "components" => {
                |data, context| extract_field_from_context(JSON_FIELD_COMPONENTS, data, context)
            }
            "entities" => {
                |data, context| extract_field_from_context(JSON_FIELD_ENTITIES, data, context)
            }
            "parent" => {
                |data, context| extract_field_from_context(JSON_FIELD_PARENT, data, context)
            }
            _ => |_data, _context| serde_json::Value::Null,
        },
    }
}

/// Extract entity ID from response data (for spawn operation)
fn extract_entity_from_response(
    data: &serde_json::Value,
    _context: &FormatterContext,
) -> serde_json::Value {
    data.get(JSON_FIELD_ENTITY)
        .cloned()
        .unwrap_or_else(|| serde_json::Value::Number(serde_json::Number::from(0)))
}

/// Extract total component count from nested query results
fn extract_query_component_count(
    data: &serde_json::Value,
    _context: &FormatterContext,
) -> serde_json::Value {
    let total = data.as_array().map_or(0, |entities| {
        entities
            .iter()
            .filter_map(|e| e.as_object())
            .map(serde_json::Map::len)
            .sum::<usize>()
    });
    serde_json::Value::Number(serde_json::Number::from(total))
}

/// Extract query parameters from request context
fn extract_query_params_from_context(
    _data: &serde_json::Value,
    context: &FormatterContext,
) -> serde_json::Value {
    context.params.clone().unwrap_or(serde_json::Value::Null)
}

/// Generic field extraction from context parameters
fn extract_field_from_context(
    field_name: &str,
    _data: &serde_json::Value,
    context: &FormatterContext,
) -> serde_json::Value {
    context
        .params
        .as_ref()
        .and_then(|p| p.get(field_name))
        .cloned()
        .unwrap_or(serde_json::Value::Null)
}
