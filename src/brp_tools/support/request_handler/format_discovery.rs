//! Auto-format discovery for BRP type serialization
//!
//! This module provides error-driven type format auto-discovery that intercepts
//! BRP responses and automatically detects and corrects type serialization format
//! errors with zero boilerplate in individual tools. Works with both components and resources.

use std::sync::OnceLock;

use regex::Regex;
use rmcp::Error as McpError;
use serde_json::{Map, Value};

use super::super::brp_client::{BrpError, BrpResult, execute_brp_method};
use super::constants::{FIELD_LABEL, FIELD_NAME, FIELD_TEXT, FIELD_VALUE};
use crate::brp_tools::constants::{
    BRP_METHOD_DESTROY, BRP_METHOD_INSERT, BRP_METHOD_INSERT_RESOURCE, BRP_METHOD_MUTATE_COMPONENT,
    BRP_METHOD_MUTATE_RESOURCE, BRP_METHOD_REGISTRY_SCHEMA, BRP_METHOD_SPAWN,
};

/// Error code for component type format errors from BRP
const COMPONENT_FORMAT_ERROR_CODE: i32 = -23402;

/// Error code for resource type format errors from BRP
const RESOURCE_FORMAT_ERROR_CODE: i32 = -23501;

/// Tier constants for format discovery
const TIER_DETERMINISTIC: u8 = 1;
const TIER_SERIALIZATION: u8 = 2;
const TIER_GENERIC_FALLBACK: u8 = 3;

/// Static regex patterns for error analysis - Based on exact Bevy error strings
static TRANSFORM_SEQUENCE_REGEX: OnceLock<Regex> = OnceLock::new();
static EXPECTED_TYPE_REGEX: OnceLock<Regex> = OnceLock::new();
static ACCESS_ERROR_REGEX: OnceLock<Regex> = OnceLock::new();
static TYPE_MISMATCH_REGEX: OnceLock<Regex> = OnceLock::new();
static VARIANT_TYPE_MISMATCH_REGEX: OnceLock<Regex> = OnceLock::new();
static MISSING_FIELD_REGEX: OnceLock<Regex> = OnceLock::new();
static UNKNOWN_COMPONENT_REGEX: OnceLock<Regex> = OnceLock::new();
static TUPLE_STRUCT_PATH_REGEX: OnceLock<Regex> = OnceLock::new();
static MATH_TYPE_ARRAY_REGEX: OnceLock<Regex> = OnceLock::new();
static UNKNOWN_COMPONENT_TYPE_REGEX: OnceLock<Regex> = OnceLock::new();

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

/// Location of type items in method parameters
#[derive(Debug, Clone, Copy)]
enum ParameterLocation {
    /// Type items are in a "components" object (spawn, insert)
    Components,
    /// Single type value in "value" field (`mutate_component`)
    ComponentValue,
    /// Single type value in "value" field (`insert_resource`, `mutate_resource`)
    ResourceValue,
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

/// Methods that support format discovery (components and resources)
const FORMAT_DISCOVERY_METHODS: &[&str] = &[
    BRP_METHOD_SPAWN,
    BRP_METHOD_INSERT,
    BRP_METHOD_MUTATE_COMPONENT,
    BRP_METHOD_INSERT_RESOURCE,
    BRP_METHOD_MUTATE_RESOURCE,
];

/// Helper function to format type mismatch error messages
fn type_format_error(type_name: &str, expected: &str, found: &str) -> String {
    format!("`{type_name}` expects {expected} format, not {found}")
}

/// Helper function to format array expectation messages
fn type_expects_array(type_name: &str, array_type: &str) -> String {
    format!("`{type_name}` {array_type} expects array format")
}

/// Helper function to fix tuple struct paths for all enum tuple variants
fn fix_tuple_struct_path(path: &str) -> String {
    match path {
        // ===== All Index 0 Mappings (.0.0) =====
        // Color: red, hue, lightness, x | Math: x components
        ".LinearRgba.red" | ".LinearRgba.r" | ".Srgba.red" | ".Srgba.r" | ".Hsla.hue"
        | ".Hsla.h" | ".Hsva.hue" | ".Hsva.h" | ".Hwba.hue" | ".Hwba.h" | ".Laba.lightness"
        | ".Laba.l" | ".Lcha.lightness" | ".Lcha.l" | ".Oklaba.lightness" | ".Oklaba.l"
        | ".Oklcha.lightness" | ".Oklcha.l" | ".Xyza.x" | ".Vec2.x" | ".Vec3.x" | ".Vec4.x"
        | ".Quat.x" | ".IVec2.x" | ".IVec3.x" | ".IVec4.x" | ".UVec2.x" | ".UVec3.x"
        | ".UVec4.x" | ".DVec2.x" | ".DVec3.x" | ".DVec4.x" => ".0.0".to_string(),

        // ===== All Index 1 Mappings (.0.1) =====
        // Color: green, saturation, whiteness, Lab 'a', chroma, y | Math: y components
        ".LinearRgba.green" | ".LinearRgba.g" | ".Srgba.green" | ".Srgba.g"
        | ".Hsla.saturation" | ".Hsla.s" | ".Hsva.saturation" | ".Hsva.s" | ".Hwba.whiteness"
        | ".Hwba.w" | ".Laba.a" | ".Oklaba.a" | ".Lcha.chroma" | ".Lcha.c" | ".Oklcha.chroma"
        | ".Oklcha.c" | ".Xyza.y" | ".Vec2.y" | ".Vec3.y" | ".Vec4.y" | ".Quat.y" | ".IVec2.y"
        | ".IVec3.y" | ".IVec4.y" | ".UVec2.y" | ".UVec3.y" | ".UVec4.y" | ".DVec2.y"
        | ".DVec3.y" | ".DVec4.y" => ".0.1".to_string(),

        // ===== All Index 2 Mappings (.0.2) =====
        // Color: blue, lightness, value, blackness, Lab 'b', hue for Lab variants, z | Math: z
        // components
        ".LinearRgba.blue" | ".LinearRgba.b" | ".Srgba.blue" | ".Srgba.b" | ".Hsla.lightness"
        | ".Hsla.l" | ".Hsva.value" | ".Hsva.v" | ".Hwba.blackness" | ".Hwba.b" | ".Laba.b"
        | ".Oklaba.b" | ".Lcha.hue" | ".Lcha.h" | ".Oklcha.hue" | ".Oklcha.h" | ".Xyza.z"
        | ".Vec3.z" | ".Vec4.z" | ".Quat.z" | ".IVec3.z" | ".IVec4.z" | ".UVec3.z" | ".UVec4.z"
        | ".DVec3.z" | ".DVec4.z" => ".0.2".to_string(),

        // ===== All Index 3 Mappings (.0.3) =====
        // Color: alpha | Math: w components
        ".LinearRgba.alpha" | ".LinearRgba.a" | ".Srgba.alpha" | ".Srgba.a" | ".Hsla.alpha"
        | ".Hsla.a" | ".Hsva.alpha" | ".Hsva.a" | ".Hwba.alpha" | ".Hwba.a" | ".Laba.alpha"
        | ".Lcha.alpha" | ".Lcha.a" | ".Oklaba.alpha" | ".Oklcha.alpha" | ".Oklcha.a"
        | ".Xyza.alpha" | ".Xyza.a" | ".Vec4.w" | ".Quat.w" | ".IVec4.w" | ".UVec4.w"
        | ".DVec4.w" => ".0.3".to_string(),

        // ===== Simple Tuple Struct Field Access =====
        // Direct field access on tuple structs (not nested)
        ".x" => ".0".to_string(),
        ".y" => ".1".to_string(),
        ".z" => ".2".to_string(),

        // ===== Generic Patterns =====
        // Generic field access patterns for tuple structs and enum variants
        p if p.starts_with('.') && p.contains('.') => {
            // Try to convert nested field access to tuple access
            // e.g., ".SomeEnum.field" -> ".0.field" or ".SomeColor.red" -> ".0.0"
            let parts: Vec<&str> = p.split('.').collect();
            if parts.len() >= 3 && !parts[1].is_empty() && !parts[2].is_empty() {
                let variant_name = parts[1];
                let field_name = parts[2];

                // Check if the second part looks like an enum variant (starts with uppercase)
                if variant_name
                    .chars()
                    .next()
                    .is_some_and(|c| c.is_ascii_uppercase())
                {
                    // For color enum variants, try to map common field names to indices
                    match field_name {
                        // Index 0: First position fields
                        "red" | "r" | "hue" | "h" | "lightness" | "l" | "x" => ".0.0".to_string(),
                        // Index 1: Second position fields (including special cases)
                        "green" | "g" | "saturation" | "s" | "y" | "whiteness" | "chroma" | "c" => {
                            ".0.1".to_string()
                        }
                        // Index 2: Third position fields
                        "blue" | "b" | "value" | "v" | "z" | "blackness" => ".0.2".to_string(),
                        // Index 3: Fourth position fields
                        "alpha" | "w" => ".0.3".to_string(),
                        // Special case for 'a' - could be alpha or Lab 'a' component
                        "a" => {
                            if variant_name.contains("Lab") {
                                ".0.1".to_string() // Lab 'a' component
                            } else {
                                ".0.3".to_string() // Alpha component
                            }
                        }
                        _ => {
                            // Generic enum variant field access -> use tuple index 0 and preserve
                            // field path
                            if parts.len() > 3 {
                                format!(".0.{}", parts[2..].join("."))
                            } else {
                                format!(".0.{field_name}")
                            }
                        }
                    }
                } else {
                    // Not an enum variant, keep original path
                    path.to_string()
                }
            } else {
                // Not enough parts, keep original path
                path.to_string()
            }
        }
        _ => path.to_string(),
    }
}

/// Get the parameter location for a given method
fn get_parameter_location(method: &str) -> ParameterLocation {
    match method {
        BRP_METHOD_MUTATE_COMPONENT => ParameterLocation::ComponentValue,
        BRP_METHOD_INSERT_RESOURCE | BRP_METHOD_MUTATE_RESOURCE => ParameterLocation::ResourceValue,
        _ => ParameterLocation::Components, // Default: spawn, insert, and others
    }
}

/// Extract type items based on parameter location
fn extract_type_items(params: &Value, location: ParameterLocation) -> Vec<(String, Value)> {
    match location {
        ParameterLocation::Components => {
            // Extract from "components" object
            if let Some(Value::Object(components)) = params.get("components") {
                components
                    .iter()
                    .map(|(name, value)| (name.clone(), value.clone()))
                    .collect()
            } else {
                Vec::new()
            }
        }
        ParameterLocation::ComponentValue => {
            // Extract single component from "component" and "value" fields
            if let (Some(type_name), Some(value)) = (
                params.get("component").and_then(Value::as_str),
                params.get("value"),
            ) {
                vec![(type_name.to_string(), value.clone())]
            } else {
                Vec::new()
            }
        }
        ParameterLocation::ResourceValue => {
            // Extract single resource from "resource" and "value" fields
            if let (Some(resource_name), Some(value)) = (
                params.get("resource").and_then(Value::as_str),
                params.get("value"),
            ) {
                vec![(resource_name.to_string(), value.clone())]
            } else {
                Vec::new()
            }
        }
    }
}

/// Apply corrections to reconstruct params based on parameter location
fn apply_corrections(
    params: &Value,
    location: ParameterLocation,
    corrected_items: &[(String, Value)],
) -> Value {
    let mut corrected_params = params.clone();

    match location {
        ParameterLocation::Components => {
            // Rebuild "components" object
            let mut components_map = Map::new();
            for (name, value) in corrected_items {
                components_map.insert(name.clone(), value.clone());
            }
            corrected_params["components"] = Value::Object(components_map);
        }
        ParameterLocation::ComponentValue => {
            // Update "value" field for component mutations
            if let Some((_, value)) = corrected_items.first() {
                corrected_params["value"] = value.clone();
            }
        }
        ParameterLocation::ResourceValue => {
            // Update "value" field for resource operations
            if let Some((_, value)) = corrected_items.first() {
                corrected_params["value"] = value.clone();
            }
        }
    }

    corrected_params
}

/// Format correction information for a type (component or resource)
#[derive(Debug, Clone)]
pub struct FormatCorrection {
    pub component:        String, // Keep field name for API compatibility
    pub original_format:  Value,
    pub corrected_format: Value,
    pub hint:             String,
}

/// Enhanced response with format corrections
#[derive(Debug, Clone)]
pub struct EnhancedBrpResult {
    pub result:             BrpResult,
    pub format_corrections: Vec<FormatCorrection>,
    pub debug_info:         Vec<String>,
}

/// Execute a BRP method with automatic format discovery
pub async fn execute_brp_method_with_format_discovery(
    method: &str,
    params: Option<Value>,
    port: Option<u16>,
) -> Result<EnhancedBrpResult, McpError> {
    let mut debug_info = vec![format!(
        "Format Discovery: FUNCTION CALLED! Executing method '{method}' with params: {params:?}"
    )];

    // Log the exact parameters being sent
    if let Some(ref p) = params {
        debug_info.push(format!(
            "Format Discovery: RAW PARAMS SENT: {}",
            serde_json::to_string_pretty(p).unwrap_or_else(|_| "<serialization error>".to_string())
        ));
    }

    // First attempt - try the original request
    let initial_result = execute_brp_method(method, params.clone(), port).await?;
    debug_info.push(format!(
        "Format Discovery: Initial result: {initial_result:?}"
    ));

    // Log the successful response details
    if let BrpResult::Success(ref data) = initial_result {
        debug_info.push(format!(
            "Format Discovery: SUCCESS RESPONSE DATA: {}",
            serde_json::to_string_pretty(data)
                .unwrap_or_else(|_| "<serialization error>".to_string())
        ));
    }

    // Check if this is a type format error that we can fix
    if let BrpResult::Error(ref error) = initial_result {
        debug_info.push(format!(
            "Format Discovery: Got error code {}, checking if method '{}' supports format discovery",
            error.code, method
        ));

        if FORMAT_DISCOVERY_METHODS.contains(&method) {
            debug_info.push(format!(
                "Format Discovery: Method '{method}' is in FORMAT_DISCOVERY_METHODS"
            ));

            if is_type_format_error(error) {
                debug_info.push(
                    "Format Discovery: Error is type format error, attempting discovery"
                        .to_string(),
                );

                if let Some(params) = params.as_ref() {
                    let mut discovery_result =
                        attempt_format_discovery(method, params, port, error).await?;
                    discovery_result.debug_info.extend(debug_info);
                    return Ok(discovery_result);
                }
                debug_info.push("Format Discovery: No params available for discovery".to_string());
            } else {
                debug_info.push(format!(
                    "Format Discovery: Error is NOT a type format error (code: {})",
                    error.code
                ));
            }
        } else {
            debug_info.push(format!(
                "Format Discovery: Method '{method}' is NOT in FORMAT_DISCOVERY_METHODS"
            ));
        }
    } else {
        debug_info
            .push("Format Discovery: Initial request succeeded, no discovery needed".to_string());
    }

    // Return original result if no format discovery needed/possible
    debug_info.push(format!(
        "Format Discovery: Returning original result with {} debug messages",
        debug_info.len()
    ));
    Ok(EnhancedBrpResult {
        result: initial_result,
        format_corrections: Vec::new(),
        debug_info,
    })
}

/// Detect if an error is a type format error that can be fixed (component or resource)
pub const fn is_type_format_error(error: &BrpError) -> bool {
    error.code == COMPONENT_FORMAT_ERROR_CODE || error.code == RESOURCE_FORMAT_ERROR_CODE
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

/// Generic function to convert object values to array format
/// Handles Vec2 [x, y], Vec3 [x, y, z], Vec4/Quat [x, y, z, w]
fn convert_to_array_format(value: &Value, field_names: &[&str]) -> Option<Value> {
    match value {
        Value::Object(obj) => {
            // Extract fields in order and convert to f32
            let mut values = Vec::new();
            for field_name in field_names {
                #[allow(clippy::cast_possible_truncation)]
                let field_value = obj.get(*field_name)?.as_f64()? as f32;
                values.push(serde_json::json!(field_value));
            }
            Some(Value::Array(values))
        }
        Value::Array(arr) if arr.len() == field_names.len() => {
            // Already in array format, validate all are numbers
            if arr.iter().all(Value::is_number) {
                Some(value.clone())
            } else {
                None
            }
        }
        _ => None,
    }
}

/// Generic function to convert math types to array format
/// Supports Vec2 [x, y], Vec3 [x, y, z], Vec4/Quat [x, y, z, w]
fn convert_to_math_type_array(value: &Value, math_type: &str) -> Option<Value> {
    let field_names = match math_type {
        "Vec2" => &["x", "y"][..],
        "Vec3" => &["x", "y", "z"][..],
        "Vec4" | "Quat" => &["x", "y", "z", "w"][..],
        _ => return None,
    };
    convert_to_array_format(value, field_names)
}

/// Extract string value from various input formats
/// Returns (`extracted_string`, `source_description`)
fn extract_string_value(value: &Value) -> Option<(String, String)> {
    match value {
        Value::Object(obj) => {
            // Try common field names that might contain the string value
            for field in [FIELD_NAME, FIELD_VALUE, FIELD_TEXT, FIELD_LABEL] {
                if let Some(Value::String(s)) = obj.get(field) {
                    return Some((s.clone(), format!("from `{field}` field")));
                }
            }
            // For single-field objects, use the value
            if obj.len() == 1 {
                if let Some((field_name, Value::String(s))) = obj.iter().next() {
                    return Some((s.clone(), format!("from `{field_name}` field")));
                }
            }
        }
        Value::Array(arr) => {
            // If it's an array with one string, extract it
            if arr.len() == 1 {
                if let Value::String(s) = &arr[0] {
                    return Some((s.clone(), "from single-element array".to_string()));
                }
            }
        }
        Value::String(s) => {
            // Already a string
            return Some((s.clone(), "already string format".to_string()));
        }
        _ => {}
    }
    None
}

/// Apply specific format correction based on error pattern
pub fn apply_pattern_fix(
    pattern: &ErrorPattern,
    type_name: &str,
    original_value: &Value,
) -> Option<(Value, String)> {
    match pattern {
        ErrorPattern::TransformSequence { expected_count } => {
            apply_transform_sequence_fix(type_name, original_value, *expected_count)
        }
        ErrorPattern::ExpectedType { expected_type } => {
            apply_expected_type_fix(type_name, original_value, expected_type)
        }
        ErrorPattern::MathTypeArray { math_type } => {
            apply_math_type_array_fix(type_name, original_value, math_type)
        }
        ErrorPattern::UnknownComponentType { .. } | ErrorPattern::UnknownComponent { .. } => {
            // These patterns are handled by Tier 2 (registry checking), not direct conversion
            None
        }
        ErrorPattern::TupleStructAccess { field_path } => {
            fix_tuple_struct_format(type_name, original_value, field_path)
        }
        ErrorPattern::AccessError { access, error_type } => {
            // Handle Bevy AccessError patterns - convert field access to tuple access
            fix_access_error(type_name, original_value, access, error_type)
        }
        ErrorPattern::TypeMismatch {
            expected,
            actual,
            access,
        } => {
            // Handle type mismatch errors
            fix_type_mismatch(type_name, original_value, expected, actual, access)
        }
        ErrorPattern::VariantTypeMismatch {
            expected,
            actual,
            access,
        } => {
            // Handle variant type mismatch for enums
            fix_variant_type_mismatch(type_name, original_value, expected, actual, access)
        }
        ErrorPattern::MissingField {
            field_name,
            type_name: _,
        } => {
            // Handle missing field errors - convert to tuple access
            fix_missing_field(type_name, original_value, field_name)
        }
    }
}

/// Fix Transform component expecting sequence of f32 values
fn apply_transform_sequence_fix(
    type_name: &str,
    original_value: &Value,
    expected_count: usize,
) -> Option<(Value, String)> {
    // Early return for non-object values
    let Value::Object(obj) = original_value else {
        return None;
    };

    // Transform typically expects Vec3 arrays for translation/scale and Quat array for rotation
    let mut corrected = Map::new();
    let mut hint_parts = Vec::new();

    // Convert Vec3 fields (translation, scale)
    for field in ["translation", "scale"] {
        if let Some(field_value) = obj.get(field) {
            if let Some(vec3_array) = convert_to_math_type_array(field_value, "Vec3") {
                corrected.insert(field.to_string(), vec3_array);
                hint_parts.push(format!("`{field}` converted to Vec3 array format"));
            } else {
                corrected.insert(field.to_string(), field_value.clone());
            }
        }
    }

    // Convert Quat field (rotation)
    if let Some(rotation_value) = obj.get("rotation") {
        if let Some(quat_array) = convert_to_math_type_array(rotation_value, "Quat") {
            corrected.insert("rotation".to_string(), quat_array);
            hint_parts.push("`rotation` converted to Quat array format".to_string());
        } else {
            corrected.insert("rotation".to_string(), rotation_value.clone());
        }
    }

    if corrected.is_empty() {
        None
    } else {
        let hint = format!(
            "`{type_name}` Transform expected {expected_count} f32 values in sequence - {}",
            hint_parts.join(", ")
        );
        Some((Value::Object(corrected), hint))
    }
}

/// Fix component expecting a specific type (e.g., Name expects string)
fn apply_expected_type_fix(
    type_name: &str,
    original_value: &Value,
    expected_type: &str,
) -> Option<(Value, String)> {
    // Handle Name component specifically
    if expected_type.contains("::Name") || expected_type.contains("::name::Name") {
        return apply_name_component_fix(type_name, original_value);
    }

    // Handle other known type patterns
    if expected_type.contains("String") {
        return convert_to_string_format(type_name, original_value);
    }

    None
}

/// Fix Name component format
fn apply_name_component_fix(type_name: &str, original_value: &Value) -> Option<(Value, String)> {
    if let Some((extracted_string, source_description)) = extract_string_value(original_value) {
        let format_type = match original_value {
            Value::Object(_) => "object",
            Value::Array(_) => "array",
            _ => "other",
        };

        Some((
            Value::String(extracted_string),
            format!(
                "`{type_name} Name component` expects string format, extracted {source_description} (was {format_type})"
            ),
        ))
    } else {
        None
    }
}

/// Fix math type array format (Vec3, Quat, etc.)
fn apply_math_type_array_fix(
    type_name: &str,
    original_value: &Value,
    math_type: &str,
) -> Option<(Value, String)> {
    match math_type {
        "Vec3" => convert_to_math_type_array(original_value, "Vec3")
            .map(|arr| (arr, type_expects_array(type_name, "Vec3") + " [x, y, z]")),
        "Vec2" => convert_to_math_type_array(original_value, "Vec2")
            .map(|arr| (arr, type_expects_array(type_name, "Vec2") + " [x, y]")),
        "Vec4" => convert_to_math_type_array(original_value, "Vec4")
            .map(|arr| (arr, type_expects_array(type_name, "Vec4") + " [x, y, z, w]")),
        "Quat" => convert_to_math_type_array(original_value, "Quat")
            .map(|arr| (arr, type_expects_array(type_name, "Quat") + " [x, y, z, w]")),
        _ => None,
    }
}

/// Convert value to string format
fn convert_to_string_format(type_name: &str, original_value: &Value) -> Option<(Value, String)> {
    if let Some((extracted_string, source_description)) = extract_string_value(original_value) {
        Some((
            Value::String(extracted_string),
            format!("`{type_name}` expects string format, extracted {source_description}"),
        ))
    } else {
        None
    }
}

/// Fix tuple struct path access errors
fn fix_tuple_struct_format(
    type_name: &str,
    original_value: &Value,
    field_path: &str,
) -> Option<(Value, String)> {
    // Tuple structs use numeric indices like .0, .1, etc.
    // If the error mentions a field path, it might be trying to access
    // a field using the wrong syntax

    // Common patterns:
    // - Trying to access .value on a tuple struct that should be .0
    // - Trying to use named fields on a tuple struct
    // - Enum tuple variants like LinearRgba with color field names

    // Apply enum-specific path fixes
    let fixed_path = fix_tuple_struct_path(field_path);

    match original_value {
        Value::Object(obj) => {
            // If we have an object with a single field, try converting to tuple access
            if obj.len() == 1 {
                if let Some((_, value)) = obj.iter().next() {
                    return Some((
                        value.clone(),
                        format!(
                            "`{type_name}` is a tuple struct, use numeric indices like .0 instead of named fields"
                        ),
                    ));
                }
            }
        }
        Value::Array(arr) => {
            // If we have an array and the path suggests index access, extract the element
            // Use the fixed path which may have been transformed from enum variant field names
            if let Ok(index) = fixed_path.trim_start_matches('.').parse::<usize>() {
                if let Some(element) = arr.get(index) {
                    let hint = if fixed_path == field_path {
                        format!("`{type_name}` tuple struct element at index {index} extracted")
                    } else {
                        format!(
                            "`{type_name}` tuple struct: converted '{field_path}' to '{fixed_path}' for element access"
                        )
                    };
                    return Some((element.clone(), hint));
                }
            }
        }
        _ => {}
    }

    None
}

/// Unified format discovery for all type methods (components and resources)
/// Extraction phase: Get parameter location and extract type items
fn extract_discovery_context(
    method: &str,
    params: &Value,
    debug_info: &mut Vec<String>,
) -> Option<(ParameterLocation, Vec<(String, Value)>)> {
    debug_info.push(format!(
        "Format Discovery: Attempting discovery for method '{method}'"
    ));

    // Get parameter location based on method
    let location = get_parameter_location(method);
    debug_info.push(format!(
        "Format Discovery: Parameter location: {location:?}"
    ));

    // Extract type items based on location
    let type_items = extract_type_items(params, location);
    if type_items.is_empty() {
        debug_info.push("Format Discovery: No type items found in params".to_string());
        return None;
    }

    debug_info.push(format!(
        "Format Discovery: Found {} type items to check",
        type_items.len()
    ));

    Some((location, type_items))
}

/// Processing phase: Process type items and generate corrections
async fn process_type_items_for_corrections(
    type_items: &[(String, Value)],
    method: &str,
    port: Option<u16>,
    original_error: &BrpError,
    debug_info: &mut Vec<String>,
) -> Result<(Vec<FormatCorrection>, Vec<(String, Value)>, Vec<TierInfo>), McpError> {
    let mut format_corrections = Vec::new();
    let mut corrected_items = Vec::new();
    let mut all_tier_info = Vec::new();

    // Process each type item
    for (type_name, type_value) in type_items {
        let (discovery_result, tier_info) = process_single_type_item(
            type_name,
            type_value,
            method,
            port,
            original_error,
            debug_info,
        )
        .await?;

        all_tier_info.extend(tier_info);

        match discovery_result {
            Some((final_format, hint)) => {
                format_corrections.push(FormatCorrection {
                    component: type_name.clone(),
                    original_format: type_value.clone(),
                    corrected_format: final_format.clone(),
                    hint,
                });
                corrected_items.push((type_name.clone(), final_format));
            }
            None => {
                // Keep original format if no alternative found
                corrected_items.push((type_name.clone(), type_value.clone()));
            }
        }
    }

    Ok((format_corrections, corrected_items, all_tier_info))
}

/// Data needed for building discovery result
struct DiscoveryResultData {
    format_corrections: Vec<FormatCorrection>,
    corrected_items:    Vec<(String, Value)>,
    all_tier_info:      Vec<TierInfo>,
}

/// Result building phase: Build final result with retrying if corrections found
async fn build_discovery_result(
    method: &str,
    params: &Value,
    location: ParameterLocation,
    data: DiscoveryResultData,
    original_error: &BrpError,
    port: Option<u16>,
    debug_info: &mut Vec<String>,
) -> Result<EnhancedBrpResult, McpError> {
    let DiscoveryResultData {
        format_corrections,
        corrected_items,
        all_tier_info,
    } = data;
    // Add tier information to debug_info
    debug_info.extend(tier_info_to_debug_strings(&all_tier_info));

    if format_corrections.is_empty() {
        debug_info.push("Format Discovery: No corrections were possible".to_string());
        Ok(EnhancedBrpResult {
            result:             BrpResult::Error(original_error.clone()),
            format_corrections: Vec::new(),
            debug_info:         debug_info.clone(),
        })
    } else {
        // Apply corrections and retry
        debug_info.push(format!(
            "Format Discovery: Found {} corrections, retrying request",
            format_corrections.len()
        ));

        let corrected_params = apply_corrections(params, location, &corrected_items);
        let result = execute_brp_method(method, Some(corrected_params), port).await?;
        debug_info.push(format!("Format Discovery: Retry result: {result:?}"));

        Ok(EnhancedBrpResult {
            result,
            format_corrections,
            debug_info: debug_info.clone(),
        })
    }
}

async fn attempt_format_discovery(
    method: &str,
    params: &Value,
    port: Option<u16>,
    original_error: &BrpError,
) -> Result<EnhancedBrpResult, McpError> {
    let mut debug_info = Vec::new();

    // Phase 1: Extraction
    let Some((location, type_items)) = extract_discovery_context(method, params, &mut debug_info)
    else {
        return Ok(EnhancedBrpResult {
            result: BrpResult::Error(original_error.clone()),
            format_corrections: Vec::new(),
            debug_info,
        });
    };

    // Phase 2: Processing
    let (format_corrections, corrected_items, all_tier_info) = process_type_items_for_corrections(
        &type_items,
        method,
        port,
        original_error,
        &mut debug_info,
    )
    .await?;

    // Phase 3: Result Building
    let result_data = DiscoveryResultData {
        format_corrections,
        corrected_items,
        all_tier_info,
    };

    build_discovery_result(
        method,
        params,
        location,
        result_data,
        original_error,
        port,
        &mut debug_info,
    )
    .await
}

/// Process a single type item (component or resource) for format discovery
async fn process_single_type_item(
    type_name: &str,
    type_value: &Value,
    method: &str,
    port: Option<u16>,
    original_error: &BrpError,
    debug_info: &mut Vec<String>,
) -> Result<(Option<(Value, String)>, Vec<TierInfo>), McpError> {
    debug_info.push(format!(
        "Format Discovery: Checking type '{type_name}' with value: {type_value:?}"
    ));

    let (discovery_result, mut tier_info) =
        tiered_type_format_discovery(type_name, type_value, original_error, port).await;

    // Add type context to tier info
    for info in &mut tier_info {
        info.action = format!("[{}] {}", type_name, info.action);
    }

    if let Some((corrected_value, hint)) = discovery_result {
        debug_info.push(format!(
            "Format Discovery: Found alternative for '{type_name}': {corrected_value:?}"
        ));

        // For spawn, validate the format by testing; for insert, just trust it
        let final_format = if method == BRP_METHOD_SPAWN {
            match test_component_format_with_spawn(type_name, &corrected_value, port).await {
                Ok(validated_format) => validated_format,
                Err(_) => return Ok((None, tier_info)), // Skip this type if validation fails
            }
        } else {
            corrected_value
        };

        Ok((Some((final_format, hint)), tier_info))
    } else {
        debug_info.push(format!(
            "Format Discovery: No alternative found for '{type_name}'"
        ));
        Ok((None, tier_info))
    }
}

/// Tiered format discovery dispatcher - replaces `try_component_format_alternatives`
/// Uses intelligent pattern matching with fallback to generic approaches
async fn tiered_type_format_discovery(
    type_name: &str,
    original_value: &Value,
    error: &BrpError,
    port: Option<u16>,
) -> (Option<(Value, String)>, Vec<TierInfo>) {
    let mut tier_info = Vec::new();

    // ========== TIER 1: Deterministic Pattern Matching ==========
    // Uses error message patterns to determine exact format mismatches
    // and applies targeted fixes with high confidence
    let error_analysis = analyze_error_pattern(error);
    if let Some(pattern) = &error_analysis.pattern {
        tier_info.push(TierInfo {
            tier:      TIER_DETERMINISTIC,
            tier_name: "Deterministic Pattern Matching".to_string(),
            action:    format!("Matched pattern: {pattern:?}"),
            success:   false, // Will be updated if successful
        });

        if let Some((corrected_value, hint)) = apply_pattern_fix(pattern, type_name, original_value)
        {
            tier_info.last_mut().unwrap().success = true;
            tier_info.last_mut().unwrap().action = format!("Applied pattern fix: {hint}");
            return (Some((corrected_value, hint)), tier_info);
        }
    }

    // ========== TIER 2: Serialization Diagnostics ==========
    // For UnknownComponentType errors, queries BRP to check if types
    // support required reflection traits (Serialize/Deserialize)
    if let Some(ErrorPattern::UnknownComponentType { component_type: _ }) = &error_analysis.pattern
    {
        tier_info.push(TierInfo {
            tier:      TIER_SERIALIZATION,
            tier_name: "Serialization Diagnostics".to_string(),
            action:    format!("Checking serialization support for type: {type_name}"),
            success:   false,
        });

        // Use the actual type_name from the request context instead of the extracted error type
        // This fixes the issue where we'd get "`bevy_reflect::DynamicEnum`" instead of the actual
        // component
        match check_type_serialization(type_name, port).await {
            Ok(serialization_check) => {
                tier_info.last_mut().unwrap().success = true;
                tier_info
                    .last_mut()
                    .unwrap()
                    .action
                    .clone_from(&serialization_check.diagnostic_message);

                // If this is a missing trait error, make it prominent by returning it as a "hint"
                // This ensures the diagnostic message is clearly visible to the user
                if serialization_check
                    .diagnostic_message
                    .contains("cannot be used with BRP")
                {
                    // Return the diagnostic as a pseudo-correction with no actual value change
                    // This makes the error message prominent in the output
                    return (
                        Some((
                            original_value.clone(),
                            serialization_check.diagnostic_message,
                        )),
                        tier_info,
                    );
                }

                // Otherwise, return as before
                return (None, tier_info);
            }
            Err(e) => {
                tier_info.last_mut().unwrap().action =
                    format!("Failed to query serialization info for {type_name}: {e}");
            }
        }
    }

    // ========== TIER 3: Generic Fallback ==========
    // Falls back to legacy transformation logic trying various
    // format conversions (object->array, array->string, etc.)
    tier_info.push(TierInfo {
        tier:      TIER_GENERIC_FALLBACK,
        tier_name: "Generic Fallback".to_string(),
        action:    "Trying generic format alternatives".to_string(),
        success:   false,
    });

    let fallback_result =
        try_component_format_alternatives_legacy(type_name, original_value, error);
    if fallback_result.is_some() {
        tier_info.last_mut().unwrap().success = true;
        tier_info.last_mut().unwrap().action = "Found generic format alternative".to_string();
    } else {
        tier_info.last_mut().unwrap().action = "No generic alternative found".to_string();
    }

    (fallback_result, tier_info)
}

/// Transformation type for format conversion
#[derive(Debug, Clone, Copy)]
enum TransformationType {
    ObjectToString,
    ObjectToArray,
    ArrayToString,
    ArrayToObject,
}

/// Transform object to string by extracting from common field names
fn transform_object_to_string(value: &Value) -> Option<Value> {
    if let Value::Object(map) = value {
        // Try to extract string from common field names
        for field in [FIELD_VALUE, FIELD_NAME, FIELD_TEXT, FIELD_LABEL] {
            if let Some(Value::String(s)) = map.get(field) {
                return Some(Value::String(s.clone()));
            }
        }
        // For single-field objects, use the value
        if map.len() == 1 {
            if let Some((_, Value::String(s))) = map.iter().next() {
                return Some(Value::String(s.clone()));
            }
        }
    }
    None
}

/// Transform object to array by collecting all values
fn transform_object_to_array(value: &Value) -> Option<Value> {
    if let Value::Object(map) = value {
        let values: Vec<Value> = map.values().cloned().collect();
        if !values.is_empty() {
            return Some(Value::Array(values));
        }
    }
    None
}

/// Transform single-element array to string
fn transform_array_to_string(value: &Value) -> Option<Value> {
    if let Value::Array(arr) = value {
        if arr.len() == 1 {
            if let Value::String(s) = &arr[0] {
                return Some(Value::String(s.clone()));
            }
        }
    }
    None
}

/// Transform array to object by wrapping in "items" field
fn transform_array_to_object(value: &Value) -> Option<Value> {
    if let Value::Array(arr) = value {
        let mut map = Map::new();
        map.insert("items".to_string(), Value::Array(arr.clone()));
        return Some(Value::Object(map));
    }
    None
}

/// Apply a transformation to convert between formats
fn apply_transformation(value: &Value, transformation: TransformationType) -> Option<Value> {
    match transformation {
        TransformationType::ObjectToString => transform_object_to_string(value),
        TransformationType::ObjectToArray => transform_object_to_array(value),
        TransformationType::ArrayToString => transform_array_to_string(value),
        TransformationType::ArrayToObject => transform_array_to_object(value),
    }
}

/// Get possible transformations based on the source value type
fn get_possible_transformations(value: &Value) -> Vec<TransformationType> {
    match value {
        Value::Object(_) => vec![
            TransformationType::ObjectToString,
            TransformationType::ObjectToArray,
        ],
        Value::Array(_) => vec![
            TransformationType::ArrayToString,
            TransformationType::ArrayToObject,
        ],
        _ => vec![], // No transformations for strings and other types
    }
}

/// Legacy format discovery function (renamed from `try_component_format_alternatives`)
/// Since we can't reliably parse error messages, we try all reasonable alternatives
fn try_component_format_alternatives_legacy(
    type_name: &str,
    original_value: &Value,
    _error: &BrpError,
) -> Option<(Value, String)> {
    // Get possible transformations for this value type
    let transformations = get_possible_transformations(original_value);

    // Try each transformation
    for transformation in transformations {
        if let Some(transformed_value) = apply_transformation(original_value, transformation) {
            let hint = match transformation {
                TransformationType::ObjectToString => {
                    type_format_error(type_name, "string", "object")
                }
                TransformationType::ObjectToArray => {
                    type_format_error(type_name, "array", "object")
                }
                TransformationType::ArrayToString => {
                    type_format_error(type_name, "string", "array")
                }
                TransformationType::ArrayToObject => {
                    type_format_error(type_name, "object", "array")
                }
            };
            return Some((transformed_value, hint));
        }
    }

    None
}

/// Test a component format by spawning a test entity
async fn test_component_format_with_spawn(
    type_name: &str,
    component_value: &Value,
    port: Option<u16>,
) -> Result<Value, McpError> {
    let mut test_components = Map::new();
    test_components.insert(type_name.to_string(), component_value.clone());

    let test_params = serde_json::json!({
        "components": test_components
    });

    let result = execute_brp_method(BRP_METHOD_SPAWN, Some(test_params), port).await?;

    match result {
        BrpResult::Success(Some(response)) => {
            // If spawn succeeded, clean up the test entity
            if let Some(entity_id) = response.get("entity").and_then(Value::as_u64) {
                let destroy_params = serde_json::json!({
                    "entity": entity_id
                });
                // Attempt cleanup, but don't fail if it doesn't work
                let _ = execute_brp_method(BRP_METHOD_DESTROY, Some(destroy_params), port).await;
            }
            Ok(component_value.clone())
        }
        _ => Err(McpError::from(rmcp::model::ErrorData::internal_error(
            "Component format test failed".to_string(),
            None,
        ))),
    }
}

/// Convert tier information to debug strings
fn tier_info_to_debug_strings(tier_info: &[TierInfo]) -> Vec<String> {
    let mut debug_strings = Vec::new();

    if !tier_info.is_empty() {
        debug_strings.push("Tiered Format Discovery Results:".to_string());

        for info in tier_info {
            let status_icon = if info.success { "SUCCESS" } else { "FAILED" };
            debug_strings.push(format!(
                "  {} Tier {}: {} - {}",
                status_icon, info.tier, info.tier_name, info.action
            ));
        }

        // Summary
        let successful_tiers: Vec<_> = tier_info.iter().filter(|t| t.success).collect();
        if successful_tiers.is_empty() {
            debug_strings.push("No tiers succeeded".to_string());
        } else {
            let tier_numbers: Vec<String> = successful_tiers
                .iter()
                .map(|t| t.tier.to_string())
                .collect();
            debug_strings.push(format!("Successful tier(s): {}", tier_numbers.join(", ")));
        }
    }

    debug_strings
}

/// Extract path information from error context strings
/// Used to parse paths from error messages like "at path .LinearRgba.red"
#[allow(clippy::option_if_let_else)]
fn extract_path_from_error_context(error_message: &str) -> Option<String> {
    // Pattern 1: "at path X" or "path X"
    if let Some(start_idx) = error_message.find("at path ") {
        let path_start = start_idx + "at path ".len();
        extract_path_from_position(error_message, path_start)
    } else if let Some(start_idx) = error_message.find("path ") {
        let path_start = start_idx + "path ".len();
        extract_path_from_position(error_message, path_start)
    } else {
        None
    }
}

/// Helper function to extract path from a specific position in an error message
fn extract_path_from_position(error_message: &str, start_pos: usize) -> Option<String> {
    let remaining = &error_message[start_pos..];

    // Remove quotes if present
    let trimmed = remaining.trim_start_matches(['`', '"']);

    // Find the end of the path (space, quote, or end of string)
    let end_pos = trimmed
        .find([' ', '`', '"', '.', ',', ':', ';'])
        .unwrap_or(trimmed.len());

    if end_pos > 0 {
        Some(trimmed[..end_pos].to_string())
    } else {
        None
    }
}

/// Fix Bevy `AccessError` patterns
fn fix_access_error(
    type_name: &str,
    original_value: &Value,
    access: &str,
    error_type: &str,
) -> Option<(Value, String)> {
    // Use helper functions to extract more information from the error_type
    let field_path = extract_path_from_error_context(error_type);

    // If we found a path, try to fix tuple struct access
    if let Some(path) = field_path {
        return fix_tuple_struct_format(type_name, original_value, &path);
    }

    // Fallback: Try generic access pattern fixes based on the access type
    match access {
        "Field" | "FieldMut" => {
            // Field access errors often mean we're trying to access a field on a tuple struct
            // Try converting to tuple access
            match original_value {
                Value::Object(obj) if obj.len() == 1 => {
                    if let Some((field_name, value)) = obj.iter().next() {
                        let hint = format!(
                            "`{type_name}` AccessError with {access} access: converted field '{field_name}' to tuple access"
                        );
                        return Some((value.clone(), hint));
                    }
                }
                _ => {}
            }
        }
        "TupleIndex" => {
            // Tuple index access errors might mean incorrect array format
            if let Value::Array(arr) = original_value {
                if !arr.is_empty() {
                    let hint = format!(
                        "`{type_name}` AccessError with {access} access: using first array element"
                    );
                    return Some((arr[0].clone(), hint));
                }
            }
        }
        _ => {}
    }

    None
}

/// Fix type mismatch errors
fn fix_type_mismatch(
    type_name: &str,
    original_value: &Value,
    expected: &str,
    actual: &str,
    access: &str,
) -> Option<(Value, String)> {
    // Common type mismatches and their fixes
    match (expected, actual) {
        // Trying to access a struct field on a tuple struct
        ("struct", "tuple_struct") => match original_value {
            Value::Object(obj) if obj.len() == 1 => {
                if let Some((field_name, value)) = obj.iter().next() {
                    let hint = format!(
                        "`{type_name}` TypeMismatch: Expected {expected} access to access a {actual}, \
                            converted field '{field_name}' to tuple access"
                    );
                    return Some((value.clone(), hint));
                }
            }
            _ => {}
        },
        // Trying to access a tuple index on a struct
        ("tuple_struct", "struct") => {
            if let Value::Array(arr) = original_value {
                if !arr.is_empty() {
                    let hint = format!(
                        "`{type_name}` TypeMismatch: Expected {expected} access to access a {actual}, \
                        using first array element"
                    );
                    return Some((arr[0].clone(), hint));
                }
            }
        }
        // Array vs object mismatches
        ("array", "object") => {
            if let Some(array_value) = transform_object_to_array(original_value) {
                let hint = format!(
                    "`{type_name}` TypeMismatch: Expected {expected} access to access a {actual}, \
                    converted object to array"
                );
                return Some((array_value, hint));
            }
        }
        ("object", "array") => {
            if let Some(object_value) = transform_array_to_object(original_value) {
                let hint = format!(
                    "`{type_name}` TypeMismatch: Expected {expected} access to access a {actual}, \
                    converted array to object"
                );
                return Some((object_value, hint));
            }
        }
        // String vs other type mismatches
        ("string", _) => {
            if let Some(string_value) = extract_string_value(original_value) {
                let hint = format!(
                    "`{type_name}` TypeMismatch: Expected {expected} access to access a {actual}, \
                    extracted string value: {}",
                    string_value.1
                );
                return Some((Value::String(string_value.0), hint));
            }
        }
        _ => {}
    }

    // Try using access pattern to determine fix
    match access {
        "Field" | "FieldMut" => {
            // Field access on wrong type, try tuple conversion
            match original_value {
                Value::Object(obj) if obj.len() == 1 => {
                    if let Some((_, value)) = obj.iter().next() {
                        let hint = format!(
                            "`{type_name}` TypeMismatch with {access} access: converted to tuple element"
                        );
                        return Some((value.clone(), hint));
                    }
                }
                _ => {}
            }
        }
        "TupleIndex" => {
            // Tuple index access on wrong type, try array access
            if let Value::Array(arr) = original_value {
                if !arr.is_empty() {
                    let hint = format!(
                        "`{type_name}` TypeMismatch with {access} access: using array element"
                    );
                    return Some((arr[0].clone(), hint));
                }
            }
        }
        _ => {}
    }

    None
}

/// Fix variant type mismatch for enums
fn fix_variant_type_mismatch(
    type_name: &str,
    original_value: &Value,
    expected: &str,
    actual: &str,
    access: &str,
) -> Option<(Value, String)> {
    // Common enum variant mismatches
    match (expected, actual) {
        // Tuple variant vs struct variant
        ("tuple", "struct") => match original_value {
            Value::Object(obj) if obj.len() == 1 => {
                if let Some((variant_name, value)) = obj.iter().next() {
                    let hint = format!(
                        "`{type_name}` VariantTypeMismatch: Expected {expected} variant access to access a {actual} variant, \
                            converted '{variant_name}' to tuple variant format"
                    );
                    return Some((value.clone(), hint));
                }
            }
            _ => {}
        },
        // Struct variant vs tuple variant
        ("struct", "tuple") => {
            if let Value::Array(arr) = original_value {
                if arr.len() == 1 {
                    // Single element tuple variant, convert to struct-like format
                    let hint = format!(
                        "`{type_name}` VariantTypeMismatch: Expected {expected} variant access to access a {actual} variant, \
                        converted array to struct variant format"
                    );
                    return Some((arr[0].clone(), hint));
                }
            }
        }
        _ => {}
    }

    // Use access type to determine conversion
    match access {
        "Field" | "FieldMut" => {
            // Field access on enum variant, likely needs tuple conversion
            match original_value {
                Value::Object(obj) if obj.len() == 1 => {
                    if let Some((_, value)) = obj.iter().next() {
                        let hint = format!(
                            "`{type_name}` VariantTypeMismatch with {access} access: converted to variant element"
                        );
                        return Some((value.clone(), hint));
                    }
                }
                _ => {}
            }
        }
        "TupleIndex" => {
            // Tuple index access on enum variant
            if let Value::Array(arr) = original_value {
                if !arr.is_empty() {
                    let hint = format!(
                        "`{type_name}` VariantTypeMismatch with {access} access: using variant element"
                    );
                    return Some((arr[0].clone(), hint));
                }
            }
        }
        _ => {}
    }

    None
}

/// Fix missing field errors
fn fix_missing_field(
    type_name: &str,
    original_value: &Value,
    field_name: &str,
) -> Option<(Value, String)> {
    // Missing field errors often occur when:
    // 1. Trying to access a named field on a tuple struct
    // 2. Trying to access a field that doesn't exist
    // 3. Enum variant field access issues

    // Check if this is a tuple struct access issue
    if field_name
        .chars()
        .next()
        .is_some_and(|c| c.is_ascii_lowercase())
    {
        // Likely a field name like "red", "x", "y", etc.
        // Try to convert to tuple struct access
        let fixed_path = fix_tuple_struct_path(&format!(".{field_name}"));
        if fixed_path != format!(".{field_name}") {
            // The path was transformed, so it's likely a tuple struct
            match original_value {
                Value::Array(arr) => {
                    // Extract the correct index from the fixed path
                    if let Some(index_str) = fixed_path.strip_prefix('.') {
                        if let Ok(index) = index_str.parse::<usize>() {
                            if let Some(element) = arr.get(index) {
                                let hint = format!(
                                    "`{type_name}` MissingField '{field_name}': converted to tuple struct index {index}"
                                );
                                return Some((element.clone(), hint));
                            }
                        }
                    }
                }
                Value::Object(obj) if obj.len() == 1 => {
                    // Single field object, likely needs tuple conversion
                    if let Some((_, value)) = obj.iter().next() {
                        let hint = format!(
                            "`{type_name}` MissingField '{field_name}': converted object to tuple struct access"
                        );
                        return Some((value.clone(), hint));
                    }
                }
                _ => {}
            }
        }
    }

    // Check if this is an enum variant field access issue
    if field_name
        .chars()
        .next()
        .is_some_and(|c| c.is_ascii_uppercase())
    {
        // Likely an enum variant name like "LinearRgba"
        if let Value::Object(obj) = original_value {
            // Try to find the variant field
            if let Some(variant_value) = obj.get(field_name) {
                let hint = format!(
                    "`{type_name}` MissingField '{field_name}': extracted enum variant value"
                );
                return Some((variant_value.clone(), hint));
            } else if obj.len() == 1 {
                // Single field object, use its value
                if let Some((actual_field, value)) = obj.iter().next() {
                    let hint = format!(
                        "`{type_name}` MissingField '{field_name}': used field '{actual_field}' instead"
                    );
                    return Some((value.clone(), hint));
                }
            }
        }
    }

    // Generic fallback: try to extract any reasonable value
    match original_value {
        Value::Object(obj) if obj.len() == 1 => {
            if let Some((actual_field, value)) = obj.iter().next() {
                let hint = format!(
                    "`{type_name}` MissingField '{field_name}': used available field '{actual_field}'"
                );
                return Some((value.clone(), hint));
            }
        }
        Value::Array(arr) if !arr.is_empty() => {
            let hint =
                format!("`{type_name}` MissingField '{field_name}': used first array element");
            return Some((arr[0].clone(), hint));
        }
        _ => {}
    }

    None
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    #[test]
    fn test_fix_tuple_struct_path_linear_rgba() {
        // Test the original LinearRgba tuple struct error case
        assert_eq!(fix_tuple_struct_path(".LinearRgba.red"), ".0.0");
        assert_eq!(fix_tuple_struct_path(".LinearRgba.r"), ".0.0");
        assert_eq!(fix_tuple_struct_path(".LinearRgba.green"), ".0.1");
        assert_eq!(fix_tuple_struct_path(".LinearRgba.g"), ".0.1");
        assert_eq!(fix_tuple_struct_path(".LinearRgba.blue"), ".0.2");
        assert_eq!(fix_tuple_struct_path(".LinearRgba.b"), ".0.2");
        assert_eq!(fix_tuple_struct_path(".LinearRgba.alpha"), ".0.3");
        assert_eq!(fix_tuple_struct_path(".LinearRgba.a"), ".0.3");
    }

    #[test]
    fn test_fix_tuple_struct_path_other_color_variants() {
        // Test other Bevy color variants
        assert_eq!(fix_tuple_struct_path(".Srgba.red"), ".0.0");
        assert_eq!(fix_tuple_struct_path(".Hsla.hue"), ".0.0");
        assert_eq!(fix_tuple_struct_path(".Hsva.saturation"), ".0.1");
        assert_eq!(fix_tuple_struct_path(".Hwba.blackness"), ".0.2");
        assert_eq!(fix_tuple_struct_path(".Laba.a"), ".0.1");
        assert_eq!(fix_tuple_struct_path(".Lcha.chroma"), ".0.1");
        assert_eq!(fix_tuple_struct_path(".Xyza.z"), ".0.2");
    }

    #[test]
    fn test_fix_tuple_struct_path_math_types() {
        // Test Bevy math vector types
        assert_eq!(fix_tuple_struct_path(".Vec3.x"), ".0.0");
        assert_eq!(fix_tuple_struct_path(".Vec3.y"), ".0.1");
        assert_eq!(fix_tuple_struct_path(".Vec3.z"), ".0.2");
        assert_eq!(fix_tuple_struct_path(".Quat.w"), ".0.3");
        assert_eq!(fix_tuple_struct_path(".IVec2.x"), ".0.0");
        assert_eq!(fix_tuple_struct_path(".DVec4.w"), ".0.3");
    }

    #[test]
    fn test_fix_tuple_struct_path_simple_access() {
        // Test simple field access on tuple structs
        assert_eq!(fix_tuple_struct_path(".x"), ".0");
        assert_eq!(fix_tuple_struct_path(".y"), ".1");
        assert_eq!(fix_tuple_struct_path(".z"), ".2");
    }

    #[test]
    fn test_analyze_error_pattern_tuple_struct_access() {
        let error = BrpError {
            code:    COMPONENT_FORMAT_ERROR_CODE,
            message: "Error accessing element with Field access at path .LinearRgba.red"
                .to_string(),
            data:    None,
        };

        let analysis = analyze_error_pattern(&error);
        assert!(analysis.pattern.is_some());

        if let Some(ErrorPattern::TupleStructAccess { field_path }) = analysis.pattern {
            assert_eq!(field_path, ".LinearRgba.red");
        } else {
            panic!(
                "Expected TupleStructAccess pattern, got: {:?}",
                analysis.pattern
            );
        }
    }

    #[test]
    fn test_analyze_error_pattern_transform_sequence() {
        let error = BrpError {
            code:    COMPONENT_FORMAT_ERROR_CODE,
            message: "Transform component expected a sequence of 3 f32 values".to_string(),
            data:    None,
        };

        let analysis = analyze_error_pattern(&error);
        assert!(analysis.pattern.is_some());

        if let Some(ErrorPattern::TransformSequence { expected_count }) = analysis.pattern {
            assert_eq!(expected_count, 3);
        } else {
            panic!(
                "Expected TransformSequence pattern, got: {:?}",
                analysis.pattern
            );
        }
    }

    #[test]
    fn test_analyze_error_pattern_expected_type() {
        let error = BrpError {
            code:    COMPONENT_FORMAT_ERROR_CODE,
            message: "expected `bevy_ecs::name::Name`".to_string(),
            data:    None,
        };

        let analysis = analyze_error_pattern(&error);
        assert!(analysis.pattern.is_some());

        if let Some(ErrorPattern::ExpectedType { expected_type }) = analysis.pattern {
            assert_eq!(expected_type, "bevy_ecs::name::Name");
        } else {
            panic!("Expected ExpectedType pattern, got: {:?}", analysis.pattern);
        }
    }

    #[test]
    fn test_analyze_error_pattern_math_type_array() {
        let error = BrpError {
            code:    COMPONENT_FORMAT_ERROR_CODE,
            message: "Vec3 expects array format".to_string(),
            data:    None,
        };

        let analysis = analyze_error_pattern(&error);
        assert!(analysis.pattern.is_some());

        if let Some(ErrorPattern::MathTypeArray { math_type }) = analysis.pattern {
            assert_eq!(math_type, "Vec3");
        } else {
            panic!(
                "Expected MathTypeArray pattern, got: {:?}",
                analysis.pattern
            );
        }
    }

    #[test]
    fn test_apply_pattern_fix_linear_rgba_case() {
        // Test the original failing case: LinearRgba tuple struct access
        let pattern = ErrorPattern::TupleStructAccess {
            field_path: ".LinearRgba.red".to_string(),
        };

        let original_value = json!({
            "LinearRgba": { "red": 1.0, "green": 0.0, "blue": 0.0, "alpha": 1.0 }
        });

        let result = apply_pattern_fix(&pattern, "bevy_render::color::Color", &original_value);
        assert!(result.is_some());

        let (corrected_value, hint) = result.unwrap();
        // Should extract the nested object since we're accessing a tuple variant
        assert!(corrected_value.is_object());
        assert!(hint.contains("tuple struct"));
        assert!(hint.contains("numeric indices"));

        // Verify the extracted object has the correct color fields
        let obj = corrected_value.as_object().unwrap();
        assert!((obj.get("red").unwrap().as_f64().unwrap() - 1.0).abs() < f64::EPSILON);
        assert!((obj.get("green").unwrap().as_f64().unwrap() - 0.0).abs() < f64::EPSILON);
        assert!((obj.get("blue").unwrap().as_f64().unwrap() - 0.0).abs() < f64::EPSILON);
        assert!((obj.get("alpha").unwrap().as_f64().unwrap() - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_apply_pattern_fix_transform_sequence() {
        let pattern = ErrorPattern::TransformSequence { expected_count: 3 };

        let original_value = json!({
            "translation": { "x": 1.0, "y": 2.0, "z": 3.0 },
            "rotation": { "x": 0.0, "y": 0.0, "z": 0.0, "w": 1.0 },
            "scale": { "x": 1.0, "y": 1.0, "z": 1.0 }
        });

        let result = apply_pattern_fix(
            &pattern,
            "bevy_transform::components::transform::Transform",
            &original_value,
        );
        assert!(result.is_some());

        let (corrected_value, hint) = result.unwrap();
        assert!(corrected_value.is_object());
        assert!(hint.contains("Transform"));
        assert!(hint.contains("array format"));

        // Check that math types were converted to arrays
        let corrected_obj = corrected_value.as_object().unwrap();
        if let Some(translation) = corrected_obj.get("translation") {
            assert!(translation.is_array());
            let arr = translation.as_array().unwrap();
            assert_eq!(arr.len(), 3);
        }
    }

    #[test]
    fn test_apply_pattern_fix_expected_type_name() {
        let pattern = ErrorPattern::ExpectedType {
            expected_type: "bevy_ecs::name::Name".to_string(),
        };

        let original_value = json!({ "name": "TestEntity" });

        let result = apply_pattern_fix(&pattern, "bevy_ecs::name::Name", &original_value);
        assert!(result.is_some());

        let (corrected_value, hint) = result.unwrap();
        assert_eq!(corrected_value, json!("TestEntity"));
        assert!(hint.contains("Name component"));
        assert!(hint.contains("string format"));
    }

    #[test]
    fn test_apply_pattern_fix_math_type_array() {
        let pattern = ErrorPattern::MathTypeArray {
            math_type: "Vec3".to_string(),
        };

        let original_value = json!({ "x": 1.0, "y": 2.0, "z": 3.0 });

        let result = apply_pattern_fix(&pattern, "bevy_math::vector::Vec3", &original_value);
        assert!(result.is_some());

        let (corrected_value, hint) = result.unwrap();
        assert_eq!(corrected_value, json!([1.0, 2.0, 3.0]));
        assert!(hint.contains("Vec3"));
        assert!(hint.contains("array format"));
        assert!(hint.contains("[x, y, z]"));
    }

    #[test]
    fn test_extract_string_value() {
        // Test various input formats
        assert_eq!(
            extract_string_value(&json!("direct_string")),
            Some((
                "direct_string".to_string(),
                "already string format".to_string()
            ))
        );

        assert_eq!(
            extract_string_value(&json!({"name": "test_name"})),
            Some(("test_name".to_string(), "from `name` field".to_string()))
        );

        assert_eq!(
            extract_string_value(&json!({"value": "test_value"})),
            Some(("test_value".to_string(), "from `value` field".to_string()))
        );

        assert_eq!(
            extract_string_value(&json!(["single_element"])),
            Some((
                "single_element".to_string(),
                "from single-element array".to_string()
            ))
        );

        // Test single-field object
        assert_eq!(
            extract_string_value(&json!({"custom_field": "custom_value"})),
            Some((
                "custom_value".to_string(),
                "from `custom_field` field".to_string()
            ))
        );
    }

    #[test]
    fn test_convert_to_math_type_array() {
        // Test Vec3 conversion
        let vec3_obj = json!({ "x": 1.0, "y": 2.0, "z": 3.0 });
        let result = convert_to_math_type_array(&vec3_obj, "Vec3");
        assert_eq!(result, Some(json!([1.0, 2.0, 3.0])));

        // Test Vec2 conversion
        let vec2_obj = json!({ "x": 5.0, "y": 6.0 });
        let result = convert_to_math_type_array(&vec2_obj, "Vec2");
        assert_eq!(result, Some(json!([5.0, 6.0])));

        // Test Quat conversion
        let quat_obj = json!({ "x": 0.0, "y": 0.0, "z": 0.0, "w": 1.0 });
        let result = convert_to_math_type_array(&quat_obj, "Quat");
        assert_eq!(result, Some(json!([0.0, 0.0, 0.0, 1.0])));

        // Test already array format
        let vec3_array = json!([1.0, 2.0, 3.0]);
        let result = convert_to_math_type_array(&vec3_array, "Vec3");
        assert_eq!(result, Some(json!([1.0, 2.0, 3.0])));

        // Test invalid input
        let invalid = json!({ "x": 1.0 }); // Missing y, z for Vec3
        let result = convert_to_math_type_array(&invalid, "Vec3");
        assert_eq!(result, None);
    }

    #[test]
    fn test_is_type_format_error() {
        // Test component format error
        let component_error = BrpError {
            code:    COMPONENT_FORMAT_ERROR_CODE,
            message: "Component type format error".to_string(),
            data:    None,
        };
        assert!(is_type_format_error(&component_error));

        // Test resource format error
        let resource_error = BrpError {
            code:    RESOURCE_FORMAT_ERROR_CODE,
            message: "Resource type format error".to_string(),
            data:    None,
        };
        assert!(is_type_format_error(&resource_error));

        // Test unrelated error code
        let other_error = BrpError {
            code:    -32602, // JSON-RPC invalid params error
            message: "Invalid params".to_string(),
            data:    None,
        };
        assert!(!is_type_format_error(&other_error));
    }
}
