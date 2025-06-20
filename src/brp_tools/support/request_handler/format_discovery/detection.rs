//! Error detection and pattern matching logic for format discovery

use regex::Regex;
use rmcp::Error as McpError;
use serde_json::Value;

use super::constants::{
    ACCESS_ERROR_REGEX, EXPECTED_TYPE_REGEX, MATH_TYPE_ARRAY_REGEX, MISSING_FIELD_REGEX,
    TRANSFORM_SEQUENCE_REGEX, TUPLE_STRUCT_PATH_REGEX, TYPE_MISMATCH_REGEX,
    UNKNOWN_COMPONENT_REGEX, UNKNOWN_COMPONENT_TYPE_REGEX, VARIANT_TYPE_MISMATCH_REGEX,
};
use crate::brp_tools::constants::BRP_METHOD_REGISTRY_SCHEMA;
use crate::brp_tools::support::brp_client::{BrpError, BrpResult, execute_brp_method};

/// Known error patterns that can be deterministically handled
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ErrorPattern {
    /// Transform expects sequence of f32 values (e.g., "expected a sequence of 4 f32 values")
    TransformSequence { expected_count: usize },
    /// Component expects a specific type (e.g., "expected `bevy_ecs::name::Name`")
    ExpectedType { expected_type: String },
    /// Vec3/Quat math types expect array format
    MathTypeArray { math_type: String },
    /// Enum serialization issue - unknown component type
    UnknownComponentType { component_type: String },
    /// Tuple struct access error (e.g., "found a tuple struct instead")
    TupleStructAccess { field_path: String },
    /// Bevy `AccessError`: Error accessing element with X access
    AccessError {
        access:     String,
        error_type: String,
    },
    /// Type mismatch: Expected X access to access Y, found Z instead
    TypeMismatch {
        expected: String,
        actual:   String,
        access:   String,
    },
    /// Variant type mismatch for enums
    VariantTypeMismatch {
        expected: String,
        actual:   String,
        access:   String,
    },
    /// Missing field in struct/tuple
    MissingField {
        field_name: String,
        type_name:  String,
    },
    /// Unknown component type from BRP
    UnknownComponent { component_path: String },
}

/// Result of error pattern analysis
#[derive(Debug, Clone)]
pub struct ErrorAnalysis {
    pub pattern: Option<ErrorPattern>,
}

/// Result of registry checking for serialization support
#[derive(Debug, Clone)]
pub struct SerializationCheck {
    pub diagnostic_message: String,
}

/// Tier information for debugging
#[derive(Debug, Clone)]
pub struct TierInfo {
    pub tier:      u8,
    pub tier_name: String,
    pub action:    String,
    pub success:   bool,
}

/// Check for access error pattern
fn check_access_error(message: &str) -> Option<ErrorPattern> {
    let access_error_regex = ACCESS_ERROR_REGEX.get_or_init(|| {
        Regex::new(r"Error accessing element with `([^`]+)` access(?:\s*\(offset \d+\))?: (.+)")
            .unwrap()
    });

    access_error_regex.captures(message).map(|captures| {
        let access = captures[1].to_string();
        let error_type = captures[2].to_string();
        ErrorPattern::AccessError { access, error_type }
    })
}

/// Check for type mismatch pattern
fn check_type_mismatch(message: &str) -> Option<ErrorPattern> {
    let type_mismatch_regex = TYPE_MISMATCH_REGEX.get_or_init(|| {
        Regex::new(r"Expected ([a-zA-Z0-9_\[\]]+) access to access a ([a-zA-Z0-9_]+), found a ([a-zA-Z0-9_]+) instead\.")
            .unwrap()
    });

    type_mismatch_regex.captures(message).map(|captures| {
        let access = captures[1].to_string();
        let expected = captures[2].to_string();
        let actual = captures[3].to_string();
        ErrorPattern::TypeMismatch {
            expected,
            actual,
            access,
        }
    })
}

/// Check for variant type mismatch pattern
fn check_variant_type_mismatch(message: &str) -> Option<ErrorPattern> {
    let variant_type_mismatch_regex = VARIANT_TYPE_MISMATCH_REGEX.get_or_init(|| {
        Regex::new(r"Expected variant ([a-zA-Z0-9_\[\]]+) access to access a ([a-zA-Z0-9_]+) variant, found a ([a-zA-Z0-9_]+) variant instead\.")
            .unwrap()
    });

    variant_type_mismatch_regex
        .captures(message)
        .map(|captures| {
            let access = captures[1].to_string();
            let expected = captures[2].to_string();
            let actual = captures[3].to_string();
            ErrorPattern::VariantTypeMismatch {
                expected,
                actual,
                access,
            }
        })
}

/// Check for missing field pattern
fn check_missing_field(message: &str) -> Option<ErrorPattern> {
    let missing_field_regex = MISSING_FIELD_REGEX.get_or_init(|| {
        Regex::new(r#"The ([a-zA-Z0-9_]+) accessed doesn't have (?:an? )?[`"]([^`"]+)[`"] field"#)
            .unwrap()
    });

    missing_field_regex.captures(message).map(|captures| {
        let type_name = captures[1].to_string();
        let field_name = captures[2].to_string();
        ErrorPattern::MissingField {
            field_name,
            type_name,
        }
    })
}

/// Check for unknown component pattern
fn check_unknown_component(message: &str) -> Option<ErrorPattern> {
    let unknown_component_regex = UNKNOWN_COMPONENT_REGEX
        .get_or_init(|| Regex::new(r"Unknown component type: `([^`]+)`").unwrap());

    unknown_component_regex.captures(message).map(|captures| {
        let component_path = captures[1].to_string();
        ErrorPattern::UnknownComponent { component_path }
    })
}

/// Check for transform sequence pattern
fn check_transform_sequence(message: &str) -> Option<ErrorPattern> {
    let transform_regex = TRANSFORM_SEQUENCE_REGEX
        .get_or_init(|| Regex::new(r"expected a sequence of (\d+) f32 values").unwrap());

    transform_regex.captures(message).and_then(|captures| {
        captures[1]
            .parse::<usize>()
            .ok()
            .map(|count| ErrorPattern::TransformSequence {
                expected_count: count,
            })
    })
}

/// Check for expected type pattern
fn check_expected_type(message: &str) -> Option<ErrorPattern> {
    let expected_type_regex = EXPECTED_TYPE_REGEX
        .get_or_init(|| Regex::new(r"expected `([a-zA-Z_:]+(?::[a-zA-Z_:]+)*)`").unwrap());

    expected_type_regex.captures(message).map(|captures| {
        let expected_type = captures[1].to_string();
        ErrorPattern::ExpectedType { expected_type }
    })
}

/// Check for math type array pattern
fn check_math_type_array(message: &str) -> Option<ErrorPattern> {
    let math_type_array_regex = MATH_TYPE_ARRAY_REGEX.get_or_init(|| {
        Regex::new(r"(Vec2|Vec3|Vec4|Quat)\s+(?:expects?|requires?|needs?)\s+array").unwrap()
    });

    math_type_array_regex.captures(message).map(|captures| {
        let math_type = captures[1].to_string();
        ErrorPattern::MathTypeArray { math_type }
    })
}

/// Check for tuple struct path pattern
fn check_tuple_struct_path(message: &str) -> Option<ErrorPattern> {
    let tuple_struct_path_regex = TUPLE_STRUCT_PATH_REGEX
        .get_or_init(|| Regex::new(r#"(?:at path|path)\s+[`"]?([^`"\s]+)[`"]?"#).unwrap());

    tuple_struct_path_regex.captures(message).map(|captures| {
        let field_path = captures[1].to_string();
        ErrorPattern::TupleStructAccess { field_path }
    })
}

/// Check for unknown component type pattern
fn check_unknown_component_type(message: &str) -> Option<ErrorPattern> {
    let unknown_component_type_regex = UNKNOWN_COMPONENT_TYPE_REGEX.get_or_init(|| {
        Regex::new(r"Unknown component type(?::\s*)?[`']?([^`'\s]+)[`']?").unwrap()
    });

    unknown_component_type_regex
        .captures(message)
        .map(|captures| {
            let component_type = captures[1].to_string();
            ErrorPattern::UnknownComponentType { component_type }
        })
}

/// Analyze error message to identify known patterns using exact regex matching
pub fn analyze_error_pattern(error: &BrpError) -> ErrorAnalysis {
    let message = &error.message;

    // Pattern 1: Access errors
    if let Some(pattern) = check_access_error(message) {
        return ErrorAnalysis {
            pattern: Some(pattern),
        };
    }

    // Check all patterns
    if let Some(pattern) = check_type_mismatch(message)
        .or_else(|| check_variant_type_mismatch(message))
        .or_else(|| check_missing_field(message))
        .or_else(|| check_unknown_component(message))
        .or_else(|| check_transform_sequence(message))
        .or_else(|| check_expected_type(message))
        .or_else(|| check_math_type_array(message))
        .or_else(|| check_tuple_struct_path(message))
        .or_else(|| check_unknown_component_type(message))
    {
        return ErrorAnalysis {
            pattern: Some(pattern),
        };
    }

    // No pattern matched
    ErrorAnalysis { pattern: None }
}

/// Check if a type supports serialization by querying the registry schema
pub async fn check_type_serialization(
    type_name: &str,
    port: Option<u16>,
) -> Result<SerializationCheck, McpError> {
    // Query the registry schema for this specific type
    let schema_params = serde_json::json!({
        "with_types": ["Component", "Resource"],
        "with_crates": [extract_crate_name(type_name)]
    });

    let schema_result =
        execute_brp_method(BRP_METHOD_REGISTRY_SCHEMA, Some(schema_params), port).await?;

    match schema_result {
        BrpResult::Success(Some(schema_data)) => analyze_schema_for_type(type_name, &schema_data),
        BrpResult::Success(None) => Ok(SerializationCheck {
            diagnostic_message: format!("No schema data returned for type `{type_name}`"),
        }),
        BrpResult::Error(err) => Ok(SerializationCheck {
            diagnostic_message: format!(
                "Failed to query schema for type `{type_name}`: {}",
                err.message
            ),
        }),
    }
}

/// Extract crate name from a fully-qualified type name
fn extract_crate_name(type_name: &str) -> &str {
    // Extract the first part before :: for crate name
    // e.g., "bevy_transform::components::transform::Transform" -> "bevy_transform"
    type_name.split("::").next().unwrap_or(type_name)
}

/// Analyze schema data to determine serialization support for a type
fn analyze_schema_for_type(
    type_name: &str,
    schema_data: &Value,
) -> Result<SerializationCheck, McpError> {
    // Schema response can be either an array (old format) or an object (new format)
    // Try object format first (new format where keys are type names)
    if let Some(schema_obj) = schema_data.as_object() {
        // Direct lookup by type name
        if let Some(schema) = schema_obj.get(type_name) {
            return Ok(analyze_single_type_schema(type_name, schema));
        }
    } else if let Some(schemas) = schema_data.as_array() {
        // Fall back to array format (old format)
        for schema in schemas {
            if let Some(type_path) = schema.get("typePath").and_then(Value::as_str) {
                if type_path == type_name {
                    return Ok(analyze_single_type_schema(type_name, schema));
                }
            }
        }
    } else {
        return Err(McpError::from(rmcp::model::ErrorData::internal_error(
            "Schema response is neither an array nor an object".to_string(),
            None,
        )));
    }

    // Type not found in schema
    Ok(SerializationCheck {
        diagnostic_message: format!(
            "Type `{type_name}` not found in registry schema. \
            This type may not be registered with BRP or may not exist."
        ),
    })
}

/// Analyze a single type's schema to check serialization support
fn analyze_single_type_schema(type_name: &str, schema: &Value) -> SerializationCheck {
    // Check its reflect types
    let reflect_types = schema
        .get("reflectTypes")
        .and_then(Value::as_array)
        .map(|arr| {
            arr.iter()
                .filter_map(Value::as_str)
                .map(String::from)
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();

    let has_serialize = reflect_types.contains(&"Serialize".to_string());
    let has_deserialize = reflect_types.contains(&"Deserialize".to_string());

    let diagnostic_message = if !has_serialize || !has_deserialize {
        let missing = if !has_serialize && !has_deserialize {
            "Serialize and Deserialize"
        } else if !has_serialize {
            "Serialize"
        } else {
            "Deserialize"
        };

        format!(
            "Type `{type_name}` cannot be used with BRP because it lacks {missing} trait(s). \
            Available traits: {}. \
            To fix this, the type definition needs both #[derive(Serialize, Deserialize)] \
            AND #[reflect(Serialize, Deserialize)] attributes.",
            reflect_types.join(", ")
        )
    } else {
        format!("Type `{type_name}` has proper serialization support")
    };

    SerializationCheck { diagnostic_message }
}

/// Helper functions to extract context from errors
pub fn extract_path_from_error_context(error_message: &str) -> Option<String> {
    // Look for patterns like "at path .foo.bar" or "path '.foo.bar'"
    error_message.find("at path ").map_or_else(
        || {
            error_message
                .find("path '")
                .or_else(|| error_message.find("path \""))
                .and_then(|pos| extract_path_from_position(error_message, pos + 6))
        },
        |pos| extract_path_from_position(error_message, pos + 8),
    )
}

fn extract_path_from_position(error_message: &str, start_pos: usize) -> Option<String> {
    let path_start = &error_message[start_pos..];

    // Find the end of the path (stop at quotes, spaces, or end of string)
    let end_chars = [' ', '\'', '"', '\n'];
    let path_end = path_start
        .find(|c| end_chars.contains(&c))
        .unwrap_or(path_start.len());

    let path = &path_start[..path_end];

    // Validate that it looks like a path (starts with . or contains .)
    if path.starts_with('.') || path.contains('.') {
        Some(path.to_string())
    } else {
        None
    }
}
