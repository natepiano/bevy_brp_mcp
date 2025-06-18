use rmcp::model::CallToolResult;
use serde_json::{Value, json};

use super::brp_client::BrpError;
use super::request_handler::FormatterContext;
use crate::support::response::ResponseBuilder;
use crate::support::serialization::json_response_to_result;

/// Metadata about a BRP request for response formatting
#[derive(Debug, Clone)]
pub struct BrpMetadata {
    pub method: String,
    pub port:   u16,
}

impl BrpMetadata {
    pub fn new(method: &str, port: u16) -> Self {
        Self {
            method: method.to_string(),
            port,
        }
    }
}

use crate::brp_tools::constants::{
    JSON_FIELD_CODE, JSON_FIELD_DATA, JSON_FIELD_ERROR_CODE, JSON_FIELD_METADATA, JSON_FIELD_METHOD,
    JSON_FIELD_PORT,
};

/// Default error formatter implementation
pub fn format_error_default(mut error: BrpError, metadata: &BrpMetadata) -> CallToolResult {
    // Enhance error messages for common format issues
    if error.code == -23402 && error.message.contains("expected a sequence of") {
        error.message.push_str("\nHint: Math types like Vec3 use array format [x,y,z], not objects {x:1,y:2,z:3}");
    }

    let response = ResponseBuilder::error()
        .message(&error.message)
        .add_field(JSON_FIELD_ERROR_CODE, error.code)
        .add_field(JSON_FIELD_DATA, error.data)
        .add_field(
            JSON_FIELD_METADATA,
            json!({
                JSON_FIELD_METHOD: metadata.method,
                JSON_FIELD_PORT: metadata.port
            }),
        )
        .build();

    json_response_to_result(&response)
}

/// A configurable formatter that can handle various BRP response formatting needs
pub struct ResponseFormatter {
    config:  FormatterConfig,
    context: FormatterContext,
}

/// Configuration for the configurable formatter
#[derive(Clone)]
pub struct FormatterConfig {
    /// Template for success messages - can include placeholders like {entity}, {resource}, etc.
    pub success_template:      Option<String>,
    /// Additional fields to add to success responses
    pub success_fields:        Vec<(String, FieldExtractor)>,
    /// Additional fields to add to error metadata
    pub error_metadata_fields: Vec<(String, FieldExtractor)>,
    /// Whether to use default error formatting
    pub use_default_error:     bool,
}

/// Function type for extracting field values from context
pub type FieldExtractor = fn(&Value, &FormatterContext) -> Value;

impl ResponseFormatter {
    pub const fn new(config: FormatterConfig, context: FormatterContext) -> Self {
        Self { config, context }
    }

    pub fn format_success(&self, data: &Value, _metadata: BrpMetadata) -> CallToolResult {
        let mut builder = ResponseBuilder::success();

        // Apply success template if provided
        if let Some(template) = &self.config.success_template {
            let message = substitute_template(template, self.context.params.as_ref());
            builder = builder.message(message);
        }

        // Add configured fields
        for (field_name, extractor) in &self.config.success_fields {
            let value = extractor(data, &self.context);
            builder = builder.add_field(field_name, value);
        }

        json_response_to_result(&builder.build())
    }

    pub fn format_error(&self, mut error: BrpError, metadata: &BrpMetadata) -> CallToolResult {
        // Enhance error messages for common format issues
        if error.code == -23402 && error.message.contains("expected a sequence of") {
            error.message.push_str("\nHint: Math types like Vec3 use array format [x,y,z], not objects {x:1,y:2,z:3}");
        }
        
        if self.config.use_default_error {
            format_error_default(error, metadata)
        } else {
            let mut metadata_obj = json!({
                JSON_FIELD_METHOD: metadata.method,
                JSON_FIELD_PORT: metadata.port,
            });

            // Add configured error metadata fields
            if let Some(metadata_map) = metadata_obj.as_object_mut() {
                for (field_name, extractor) in &self.config.error_metadata_fields {
                    let value = extractor(&Value::Null, &self.context);
                    metadata_map.insert(field_name.clone(), value);
                }
            }

            let mut builder = ResponseBuilder::error().message(&error.message);

            // Handle special BRP execution error format
            if metadata.method == "brp_execute" {
                builder = builder
                    .add_field(JSON_FIELD_CODE, error.code)
                    .add_field(JSON_FIELD_DATA, error.data.unwrap_or(Value::Null));
            } else {
                builder = builder
                    .add_field(JSON_FIELD_ERROR_CODE, error.code)
                    .add_field(JSON_FIELD_METADATA, metadata_obj);
            }

            json_response_to_result(&builder.build())
        }
    }
}

/// Factory for creating configurable formatters
pub struct ResponseFormatterFactory {
    config: FormatterConfig,
}

impl ResponseFormatterFactory {
    /// Create a formatter for simple entity operations (destroy, etc.)
    pub fn entity_operation(_entity_field: &str) -> ResponseFormatterBuilder {
        use crate::brp_tools::constants::JSON_FIELD_ENTITY;

        ResponseFormatterBuilder {
            config: FormatterConfig {
                success_template:      None,
                success_fields:        vec![],
                error_metadata_fields: vec![(
                    JSON_FIELD_ENTITY.to_string(),
                    extractors::entity_from_params,
                )],
                use_default_error:     false,
            },
        }
    }

    /// Create a formatter for resource operations
    pub fn resource_operation(_resource_field: &str) -> ResponseFormatterBuilder {
        use crate::brp_tools::constants::JSON_FIELD_RESOURCE;

        ResponseFormatterBuilder {
            config: FormatterConfig {
                success_template:      None,
                success_fields:        vec![],
                error_metadata_fields: vec![(
                    JSON_FIELD_RESOURCE.to_string(),
                    extractors::resource_from_params,
                )],
                use_default_error:     false,
            },
        }
    }

    /// Create a formatter that passes through the response data
    pub fn pass_through() -> ResponseFormatterBuilder {
        use crate::brp_tools::constants::JSON_FIELD_DATA;

        ResponseFormatterBuilder {
            config: FormatterConfig {
                success_template:      Some("Operation completed successfully".to_string()),
                success_fields:        vec![(
                    JSON_FIELD_DATA.to_string(),
                    extractors::pass_through_data,
                )],
                error_metadata_fields: vec![],
                use_default_error:     true,
            },
        }
    }

    /// Create a formatter for generic method execution
    pub fn method_execution() -> ResponseFormatterBuilder {
        use crate::brp_tools::constants::JSON_FIELD_DATA;

        ResponseFormatterBuilder {
            config: FormatterConfig {
                success_template:      Some(
                    "Successfully executed BRP method: {method}".to_string(),
                ),
                success_fields:        vec![(
                    JSON_FIELD_DATA.to_string(),
                    extractors::conditional_data,
                )],
                error_metadata_fields: vec![],
                use_default_error:     false,
            },
        }
    }

    /// Create a formatter for list operations
    pub fn list_operation() -> ResponseFormatterBuilder {
        ResponseFormatterBuilder {
            config: FormatterConfig {
                success_template:      None,
                success_fields:        vec![],
                error_metadata_fields: vec![],
                use_default_error:     true,
            },
        }
    }
}

impl ResponseFormatterFactory {
    pub fn create(&self, context: FormatterContext) -> ResponseFormatter {
        ResponseFormatter::new(self.config.clone(), context)
    }
}

/// Builder for configuring formatters
pub struct ResponseFormatterBuilder {
    config: FormatterConfig,
}

impl ResponseFormatterBuilder {
    /// Set the success message template
    pub fn with_template(mut self, template: impl Into<String>) -> Self {
        self.config.success_template = Some(template.into());
        self
    }

    /// Add a field to the success response
    pub fn with_response_field(
        mut self,
        name: impl Into<String>,
        extractor: FieldExtractor,
    ) -> Self {
        self.config.success_fields.push((name.into(), extractor));
        self
    }

    /// Add a field to error metadata
    pub fn with_error_metadata_field(
        mut self,
        name: impl Into<String>,
        extractor: FieldExtractor,
    ) -> Self {
        self.config
            .error_metadata_fields
            .push((name.into(), extractor));
        self
    }

    /// Use default error formatting
    pub const fn with_default_error(mut self) -> Self {
        self.config.use_default_error = true;
        self
    }

    /// Build the formatter factory
    pub fn build(self) -> ResponseFormatterFactory {
        ResponseFormatterFactory {
            config: self.config,
        }
    }
}

/// Substitute placeholders in a template string with values from params
fn substitute_template(template: &str, params: Option<&Value>) -> String {
    let mut result = template.to_string();

    if let Some(Value::Object(map)) = params {
        for (key, value) in map {
            let placeholder = format!("{{{key}}}");
            let replacement = match value {
                Value::String(s) => s.clone(),
                Value::Number(n) => n.to_string(),
                Value::Bool(b) => b.to_string(),
                _ => value.to_string(),
            };
            result = result.replace(&placeholder, &replacement);
        }
    }

    result
}

// Common field extractors

// Helper functions for common field extractors
pub mod extractors {
    use super::{FormatterContext, Value};
    use crate::brp_tools::constants::{JSON_FIELD_ENTITY, JSON_FIELD_RESOURCE};

    /// Extract entity ID from context params
    pub fn entity_from_params(_data: &Value, context: &FormatterContext) -> Value {
        context
            .params
            .as_ref()
            .and_then(|p| p.get(JSON_FIELD_ENTITY))
            .cloned()
            .unwrap_or(Value::Null)
    }

    /// Extract resource name from context params
    pub fn resource_from_params(_data: &Value, context: &FormatterContext) -> Value {
        context
            .params
            .as_ref()
            .and_then(|p| p.get(JSON_FIELD_RESOURCE))
            .cloned()
            .unwrap_or(Value::Null)
    }

    /// Pass through the BRP response data
    pub fn pass_through_data(data: &Value, _context: &FormatterContext) -> Value {
        data.clone()
    }

    /// Count elements in an array from the response data
    pub fn array_count(data: &Value, _context: &FormatterContext) -> Value {
        data.as_array().map_or(0, std::vec::Vec::len).into()
    }

    /// Create a field extractor that gets components from params
    pub fn components_from_params(_data: &Value, context: &FormatterContext) -> Value {
        context
            .params
            .as_ref()
            .and_then(|p| p.get("components"))
            .cloned()
            .unwrap_or(Value::Null)
    }

    /// Conditionally pass through data only if it's not null
    pub fn conditional_data(data: &Value, _context: &FormatterContext) -> Value {
        if data.is_null() {
            Value::Null
        } else {
            data.clone()
        }
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    #[test]
    fn test_substitute_template() {
        let params = Some(json!({
            "entity": 123,
            "name": "test_resource"
        }));

        let result = substitute_template("Entity {entity} with name {name}", params.as_ref());
        assert_eq!(result, "Entity 123 with name test_resource");

        let result = substitute_template("No substitutions", params.as_ref());
        assert_eq!(result, "No substitutions");

        let result = substitute_template("Missing {missing} placeholder", params.as_ref());
        assert_eq!(result, "Missing {missing} placeholder");
    }

    #[test]
    fn test_configurable_formatter_success() {
        use crate::brp_tools::constants::JSON_FIELD_DESTROYED_ENTITY;

        let config = FormatterConfig {
            success_template:      Some("Successfully destroyed entity {entity}".to_string()),
            success_fields:        vec![(
                JSON_FIELD_DESTROYED_ENTITY.to_string(),
                extractors::entity_from_params,
            )],
            error_metadata_fields: vec![],
            use_default_error:     false,
        };

        let context = FormatterContext {
            params: Some(json!({ "entity": 123 })),
        };

        let formatter = ResponseFormatter::new(config, context);
        let metadata = BrpMetadata::new("bevy/destroy", 15702);
        let result = formatter.format_success(&Value::Null, metadata);

        // Verify result has content
        assert_eq!(result.content.len(), 1);
        // For now, we'll just verify that formatting doesn't panic
        // TODO: Once we understand Content type better, add proper content validation
    }

    #[test]
    fn test_configurable_formatter_error_with_metadata() {
        use crate::brp_tools::constants::JSON_FIELD_ENTITY;

        let config = FormatterConfig {
            success_template:      None,
            success_fields:        vec![],
            error_metadata_fields: vec![(
                JSON_FIELD_ENTITY.to_string(),
                extractors::entity_from_params,
            )],
            use_default_error:     false,
        };

        let context = FormatterContext {
            params: Some(json!({ "entity": 456 })),
        };

        let formatter = ResponseFormatter::new(config, context);
        let metadata = BrpMetadata::new("bevy/destroy", 15702);
        let error = BrpError {
            code:    -32603,
            message: "Entity not found".to_string(),
            data:    None,
        };

        let result = formatter.format_error(error, &metadata);

        // Verify result has content
        assert_eq!(result.content.len(), 1);
        // TODO: Add proper content validation once Content type is understood
    }

    #[test]
    fn test_configurable_formatter_default_error() {
        let config = FormatterConfig {
            success_template:      None,
            success_fields:        vec![],
            error_metadata_fields: vec![],
            use_default_error:     true,
        };

        let context = FormatterContext { params: None };

        let formatter = ResponseFormatter::new(config, context);
        let metadata = BrpMetadata::new("bevy/test", 15702);
        let error = BrpError {
            code:    -32603,
            message: "Test error".to_string(),
            data:    Some(json!({"detail": "extra info"})),
        };

        let result = formatter.format_error(error, &metadata);

        // Verify result has content
        assert_eq!(result.content.len(), 1);
        // TODO: Add proper content validation once Content type is understood
    }

    #[test]
    fn test_entity_operation_builder() {
        use crate::brp_tools::constants::JSON_FIELD_DESTROYED_ENTITY;

        let factory = ResponseFormatterFactory::entity_operation(JSON_FIELD_DESTROYED_ENTITY)
            .with_template("Successfully destroyed entity {entity}")
            .with_response_field(JSON_FIELD_DESTROYED_ENTITY, extractors::entity_from_params)
            .build();

        let context = FormatterContext {
            params: Some(json!({ "entity": 789 })),
        };

        let formatter = factory.create(context);
        let metadata = BrpMetadata::new("bevy/destroy", 15702);
        let result = formatter.format_success(&Value::Null, metadata);

        // Verify result has content
        assert_eq!(result.content.len(), 1);
        // TODO: Add proper content validation once Content type is understood
    }

    #[test]
    fn test_pass_through_builder() {
        let factory = ResponseFormatterFactory::pass_through().build();

        let context = FormatterContext { params: None };

        let formatter = factory.create(context);
        let metadata = BrpMetadata::new("bevy/query", 15702);
        let data = json!([{"entity": 1}, {"entity": 2}]);
        let result = formatter.format_success(&data, metadata);

        // Verify result has content
        assert_eq!(result.content.len(), 1);
        // TODO: Add proper content validation once Content type is understood
    }

    #[test]
    fn test_extractors() {
        let context = FormatterContext {
            params: Some(json!({
                "entity": 100,
                "resource": "TestResource"
            })),
        };

        let data = json!({"result": "success"});

        assert_eq!(extractors::entity_from_params(&data, &context), 100);
        assert_eq!(
            extractors::resource_from_params(&data, &context),
            "TestResource"
        );
        assert_eq!(extractors::pass_through_data(&data, &context), data);

        // Test array_count extractor
        let array_data = json!([1, 2, 3, 4, 5]);
        assert_eq!(extractors::array_count(&array_data, &context), 5);

        let empty_array = json!([]);
        assert_eq!(extractors::array_count(&empty_array, &context), 0);

        let non_array = json!({"not": "array"});
        assert_eq!(extractors::array_count(&non_array, &context), 0);

        // Test components_from_params extractor
        let components_context = FormatterContext {
            params: Some(json!({
                "components": ["Transform", "Sprite"]
            })),
        };
        assert_eq!(
            extractors::components_from_params(&data, &components_context),
            json!(["Transform", "Sprite"])
        );

        // Test with missing components field
        assert_eq!(
            extractors::components_from_params(&data, &context),
            Value::Null
        );
    }
}
