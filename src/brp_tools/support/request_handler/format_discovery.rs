//! Auto-format discovery for BRP component serialization
//!
//! This module provides error-driven component format auto-discovery that intercepts
//! BRP responses and automatically detects and corrects component serialization format
//! errors with zero boilerplate in individual tools.

use std::sync::OnceLock;

use regex::Regex;
use rmcp::Error as McpError;
use serde_json::{Map, Value};

use super::super::brp_client::{BrpError, BrpResult, execute_brp_method};
use crate::brp_tools::constants::{
    BRP_METHOD_DESTROY, BRP_METHOD_INSERT, BRP_METHOD_MUTATE_COMPONENT, BRP_METHOD_REGISTRY_SCHEMA,
    BRP_METHOD_SPAWN,
};

/// Error code for component format errors from BRP
const COMPONENT_ERROR_CODE: i32 = -23402;

/// Tier constants for format discovery
const TIER_DETERMINISTIC: u8 = 1;
const TIER_SERIALIZATION: u8 = 2;
const TIER_GENERIC_FALLBACK: u8 = 3;

/// Static regex patterns for error analysis
static TRANSFORM_SEQUENCE_REGEX: OnceLock<Regex> = OnceLock::new();
static EXPECTED_TYPE_REGEX: OnceLock<Regex> = OnceLock::new();

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
}

/// Result of error pattern analysis
#[derive(Debug, Clone)]
pub struct ErrorAnalysis {
    pub pattern:    Option<ErrorPattern>,
    pub confidence: f32, // 0.0 to 1.0
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

/// Methods that support component format discovery
const COMPONENT_METHODS: &[&str] = &[
    BRP_METHOD_SPAWN,
    BRP_METHOD_INSERT,
    BRP_METHOD_MUTATE_COMPONENT,
];

/// Format correction information for a component
#[derive(Debug, Clone)]
pub struct FormatCorrection {
    pub component:        String,
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
            serde_json::to_string_pretty(data).unwrap_or_else(|_| "<serialization error>".to_string())
        ));
    }

    // Check if this is a component format error that we can fix
    if let BrpResult::Error(ref error) = initial_result {
        debug_info.push(format!(
            "Format Discovery: Got error code {}, checking if component method",
            error.code
        ));

        if COMPONENT_METHODS.contains(&method) {
            debug_info.push(format!(
                "Format Discovery: Method '{method}' is in COMPONENT_METHODS"
            ));

            if is_component_format_error(error) {
                debug_info.push(
                    "Format Discovery: Error is component format error, attempting discovery".to_string()
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
                    "Format Discovery: Error is NOT a component format error (code: {})",
                    error.code
                ));
            }
        } else {
            debug_info.push(format!(
                "Format Discovery: Method '{method}' is NOT in COMPONENT_METHODS"
            ));
        }
    } else {
        debug_info.push(
            "Format Discovery: Initial request succeeded, no discovery needed".to_string()
        );
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

/// Detect if an error is a component format error that can be fixed
pub const fn is_component_format_error(error: &BrpError) -> bool {
    error.code == COMPONENT_ERROR_CODE
}

/// Analyze error message to identify known patterns
pub fn analyze_error_pattern(error: &BrpError) -> ErrorAnalysis {
    let message = &error.message;

    // Pattern 1: Transform sequence errors
    let transform_regex = TRANSFORM_SEQUENCE_REGEX
        .get_or_init(|| Regex::new(r"expected a sequence of (\d+) f32 values").unwrap());

    if let Some(captures) = transform_regex.captures(message) {
        if let Ok(count) = captures[1].parse::<usize>() {
            return ErrorAnalysis {
                pattern:    Some(ErrorPattern::TransformSequence {
                    expected_count: count,
                }),
                confidence: 0.95,
            };
        }
    }

    // Pattern 2: Expected specific type
    let expected_type_regex = EXPECTED_TYPE_REGEX
        .get_or_init(|| Regex::new(r"expected ([a-zA-Z_:]+(?::[a-zA-Z_:]+)*)").unwrap());

    if let Some(captures) = expected_type_regex.captures(message) {
        let expected_type = captures[1].to_string();
        return ErrorAnalysis {
            pattern:    Some(ErrorPattern::ExpectedType { expected_type }),
            confidence: 0.90,
        };
    }

    // Pattern 3: Math type array format
    if message.contains("Vec3")
        || message.contains("Quat")
        || message.contains("Vec2")
        || message.contains("Vec4")
    {
        let math_type = if message.contains("Vec3") {
            "Vec3".to_string()
        } else if message.contains("Quat") {
            "Quat".to_string()
        } else if message.contains("Vec2") {
            "Vec2".to_string()
        } else {
            "Vec4".to_string()
        };

        return ErrorAnalysis {
            pattern:    Some(ErrorPattern::MathTypeArray { math_type }),
            confidence: 0.85,
        };
    }

    // Pattern 4: Unknown component type (DynamicEnum issue)
    if message.contains("Unknown component type") && message.contains("DynamicEnum") {
        let component_type = message
            .split("Unknown component type: ")
            .nth(1)
            .unwrap_or("unknown")
            .trim()
            .to_string();

        return ErrorAnalysis {
            pattern:    Some(ErrorPattern::UnknownComponentType { component_type }),
            confidence: 0.95,
        };
    }

    // No pattern matched
    ErrorAnalysis {
        pattern:    None,
        confidence: 0.0,
    }
}

/// Check if a component supports serialization by querying the registry schema
pub async fn check_component_serialization(
    component_type: &str,
    port: Option<u16>,
) -> Result<SerializationCheck, McpError> {
    // Query the registry schema for this specific component type
    let schema_params = serde_json::json!({
        "with_types": ["Component"],
        "with_crates": [extract_crate_name(component_type)]
    });

    let schema_result =
        execute_brp_method(BRP_METHOD_REGISTRY_SCHEMA, Some(schema_params), port).await?;

    match schema_result {
        BrpResult::Success(Some(schema_data)) => {
            analyze_schema_for_component(component_type, &schema_data)
        }
        BrpResult::Success(None) => Ok(SerializationCheck {
            diagnostic_message: format!("No schema data returned for component `{component_type}`"),
        }),
        BrpResult::Error(err) => Ok(SerializationCheck {
            diagnostic_message: format!(
                "Failed to query schema for component `{component_type}`: {}",
                err.message
            ),
        }),
    }
}

/// Extract crate name from a fully-qualified component type
fn extract_crate_name(component_type: &str) -> &str {
    // Extract the first part before :: for crate name
    // e.g., "bevy_transform::components::transform::Transform" -> "bevy_transform"
    component_type.split("::").next().unwrap_or(component_type)
}

/// Analyze schema data to determine serialization support for a component
fn analyze_schema_for_component(
    component_type: &str,
    schema_data: &Value,
) -> Result<SerializationCheck, McpError> {
    // Schema response should be an array of type definitions
    let schemas = schema_data.as_array().ok_or_else(|| {
        McpError::from(rmcp::model::ErrorData::internal_error(
            "Schema response is not an array".to_string(),
            None,
        ))
    })?;

    // Look for our specific component type
    for schema in schemas {
        if let Some(type_path) = schema.get("typePath").and_then(Value::as_str) {
            if type_path == component_type {
                // Found our component, check its reflect types
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
                        "Component `{component_type}` is missing {missing} trait(s). Available traits: {}. \
                        To fix this, add #[derive(Serialize, Deserialize)] or use #[reflect(Serialize, Deserialize)] \
                        in your component definition.",
                        reflect_types.join(", ")
                    )
                } else {
                    format!("Component `{component_type}` has proper serialization support")
                };

                return Ok(SerializationCheck { diagnostic_message });
            }
        }
    }

    // Component not found in schema
    Ok(SerializationCheck {
        diagnostic_message: format!(
            "Component `{component_type}` not found in registry schema. \
            This component may not be registered with BRP or may not exist."
        ),
    })
}

/// Generic function to convert object values to array format
/// Handles Vec2 [x, y], Vec3 [x, y, z], Vec4/Quat [x, y, z, w]
fn convert_to_array_format(value: &Value, field_names: &[&str]) -> Option<Value> {
    match value {
        Value::Object(obj) => {
            // Extract fields in order and convert to f32
            let mut values = Vec::new();
            for field_name in field_names {
                let field_value = obj.get(*field_name)?.as_f64()? as f32;
                values.push(serde_json::json!(field_value));
            }
            Some(Value::Array(values))
        }
        Value::Array(arr) if arr.len() == field_names.len() => {
            // Already in array format, validate all are numbers
            if arr.iter().all(|v| v.is_number()) {
                Some(value.clone())
            } else {
                None
            }
        }
        _ => None,
    }
}

/// Apply specific format correction based on error pattern
pub fn apply_pattern_fix(
    pattern: &ErrorPattern,
    component_name: &str,
    original_value: &Value,
) -> Option<(Value, String)> {
    match pattern {
        ErrorPattern::TransformSequence { expected_count } => {
            apply_transform_sequence_fix(component_name, original_value, *expected_count)
        }
        ErrorPattern::ExpectedType { expected_type } => {
            apply_expected_type_fix(component_name, original_value, expected_type)
        }
        ErrorPattern::MathTypeArray { math_type } => {
            apply_math_type_array_fix(component_name, original_value, math_type)
        }
        ErrorPattern::UnknownComponentType { .. } => {
            // This pattern is handled by Tier 2 (registry checking), not direct conversion
            None
        }
    }
}

/// Fix Transform component expecting sequence of f32 values
fn apply_transform_sequence_fix(
    component_name: &str,
    original_value: &Value,
    expected_count: usize,
) -> Option<(Value, String)> {
    // Transform typically expects Vec3 arrays for translation/scale and Quat array for rotation
    if let Value::Object(obj) = original_value {
        let mut corrected = Map::new();
        let mut hint_parts = Vec::new();

        // Convert Vec3 fields (translation, scale)
        for field in ["translation", "scale"] {
            if let Some(field_value) = obj.get(field) {
                if let Some(vec3_array) = convert_to_vec3_array(field_value) {
                    corrected.insert(field.to_string(), vec3_array);
                    hint_parts.push(format!("`{}` converted to Vec3 array format", field));
                } else {
                    corrected.insert(field.to_string(), field_value.clone());
                }
            }
        }

        // Convert Quat field (rotation)
        if let Some(rotation_value) = obj.get("rotation") {
            if let Some(quat_array) = convert_to_quat_array(rotation_value) {
                corrected.insert("rotation".to_string(), quat_array);
                hint_parts.push("`rotation` converted to Quat array format".to_string());
            } else {
                corrected.insert("rotation".to_string(), rotation_value.clone());
            }
        }

        if !corrected.is_empty() {
            let hint = format!(
                "`{}` Transform expected {} f32 values in sequence - {}",
                component_name,
                expected_count,
                hint_parts.join(", ")
            );
            return Some((Value::Object(corrected), hint));
        }
    }

    None
}

/// Fix component expecting a specific type (e.g., Name expects string)
fn apply_expected_type_fix(
    component_name: &str,
    original_value: &Value,
    expected_type: &str,
) -> Option<(Value, String)> {
    // Handle Name component specifically
    if expected_type.contains("::Name") || expected_type.contains("::name::Name") {
        return apply_name_component_fix(component_name, original_value);
    }

    // Handle other known type patterns
    if expected_type.contains("String") {
        return convert_to_string_format(component_name, original_value);
    }

    None
}

/// Fix Name component format
fn apply_name_component_fix(
    component_name: &str,
    original_value: &Value,
) -> Option<(Value, String)> {
    match original_value {
        Value::Object(obj) => {
            // If it's an object, try to extract a string value
            if let Some(Value::String(name)) = obj.get("name").or_else(|| obj.get("value")) {
                return Some((
                    Value::String(name.clone()),
                    format!(
                        "`{}` Name component expects string format, not object",
                        component_name
                    ),
                ));
            }
        }
        Value::Array(arr) => {
            // If it's an array with one string, extract it
            if arr.len() == 1 {
                if let Value::String(name) = &arr[0] {
                    return Some((
                        Value::String(name.clone()),
                        format!(
                            "`{}` Name component expects string format, not array",
                            component_name
                        ),
                    ));
                }
            }
        }
        _ => {}
    }

    None
}

/// Fix math type array format (Vec3, Quat, etc.)
fn apply_math_type_array_fix(
    component_name: &str,
    original_value: &Value,
    math_type: &str,
) -> Option<(Value, String)> {
    match math_type {
        "Vec3" => convert_to_vec3_array(original_value).map(|arr| {
            (
                arr,
                format!("`{}` Vec3 expects array format [x, y, z]", component_name),
            )
        }),
        "Vec2" => convert_to_vec2_array(original_value).map(|arr| {
            (
                arr,
                format!("`{}` Vec2 expects array format [x, y]", component_name),
            )
        }),
        "Vec4" => convert_to_vec4_array(original_value).map(|arr| {
            (
                arr,
                format!(
                    "`{}` Vec4 expects array format [x, y, z, w]",
                    component_name
                ),
            )
        }),
        "Quat" => convert_to_quat_array(original_value).map(|arr| {
            (
                arr,
                format!(
                    "`{}` Quat expects array format [x, y, z, w]",
                    component_name
                ),
            )
        }),
        _ => None,
    }
}

/// Convert value to Vec3 array format [x, y, z]
fn convert_to_vec3_array(value: &Value) -> Option<Value> {
    convert_to_array_format(value, &["x", "y", "z"])
}

/// Convert value to Vec2 array format [x, y]
fn convert_to_vec2_array(value: &Value) -> Option<Value> {
    convert_to_array_format(value, &["x", "y"])
}

/// Convert value to Vec4 array format [x, y, z, w]
fn convert_to_vec4_array(value: &Value) -> Option<Value> {
    convert_to_array_format(value, &["x", "y", "z", "w"])
}

/// Convert value to Quat array format [x, y, z, w]
fn convert_to_quat_array(value: &Value) -> Option<Value> {
    // Quat has the same format as Vec4
    convert_to_vec4_array(value)
}

/// Convert value to string format
fn convert_to_string_format(
    component_name: &str,
    original_value: &Value,
) -> Option<(Value, String)> {
    match original_value {
        Value::Object(obj) => {
            // Try common field names that might contain the string value
            for field in ["name", "value", "text", "label"] {
                if let Some(Value::String(s)) = obj.get(field) {
                    return Some((
                        Value::String(s.clone()),
                        format!(
                            "`{}` expects string format, extracted from `{}` field",
                            component_name, field
                        ),
                    ));
                }
            }
        }
        Value::Array(arr) => {
            if arr.len() == 1 {
                if let Value::String(s) = &arr[0] {
                    return Some((
                        Value::String(s.clone()),
                        format!(
                            "`{}` expects string format, extracted from single-element array",
                            component_name
                        ),
                    ));
                }
            }
        }
        _ => {}
    }

    None
}

/// Unified format discovery for all component methods
async fn attempt_format_discovery(
    method: &str,
    params: &Value,
    port: Option<u16>,
    original_error: &BrpError,
) -> Result<EnhancedBrpResult, McpError> {
    match method {
        BRP_METHOD_SPAWN | BRP_METHOD_INSERT => {
            attempt_components_format_discovery(method, params, port, original_error).await
        }
        BRP_METHOD_MUTATE_COMPONENT => {
            attempt_value_format_discovery(method, params, port, original_error).await
        }
        _ => Ok(EnhancedBrpResult {
            result:             BrpResult::Error(original_error.clone()),
            format_corrections: Vec::new(),
            debug_info:         vec![format!(
                "Format Discovery: Unsupported method for discovery: {method}"
            )],
        }),
    }
}

/// Extract and validate components from parameters
fn extract_and_validate_components<'a>(
    params: &'a Value,
    _original_error: &BrpError,
) -> Option<&'a Map<String, Value>> {
    match params.get("components") {
        Some(Value::Object(components)) => Some(components),
        _ => None,
    }
}

/// Process a single component for format discovery
async fn process_single_component(
    component_name: &str,
    component_value: &Value,
    method: &str,
    port: Option<u16>,
    original_error: &BrpError,
    debug_info: &mut Vec<String>,
) -> Result<(Option<(Value, String)>, Vec<TierInfo>), McpError> {
    debug_info.push(format!(
        "Format Discovery: Checking component '{}' with value: {:?}",
        component_name, component_value
    ));

    let (discovery_result, mut tier_info) =
        tiered_component_format_discovery(component_name, component_value, original_error, port)
            .await;

    // Add component context to tier info
    for info in &mut tier_info {
        info.action = format!("[{}] {}", component_name, info.action);
    }

    match discovery_result {
        Some((corrected_value, hint)) => {
            debug_info.push(format!(
                "Format Discovery: Found alternative for '{}': {:?}",
                component_name, corrected_value
            ));

            // For spawn, validate the format by testing; for insert, just trust it
            let final_format = if method == BRP_METHOD_SPAWN {
                match test_component_format_with_spawn(component_name, &corrected_value, port).await
                {
                    Ok(validated_format) => validated_format,
                    Err(_) => return Ok((None, tier_info)), /* Skip this component if validation
                                                             * fails */
                }
            } else {
                corrected_value
            };

            Ok((Some((final_format, hint)), tier_info))
        }
        None => {
            debug_info.push(format!(
                "Format Discovery: No alternative found for '{}'",
                component_name
            ));
            Ok((None, tier_info))
        }
    }
}

/// Apply corrections and retry the BRP request
async fn apply_corrections_and_retry(
    method: &str,
    params: &Value,
    port: Option<u16>,
    corrected_components: Map<String, Value>,
    format_corrections: Vec<FormatCorrection>,
    mut debug_info: Vec<String>,
) -> Result<EnhancedBrpResult, McpError> {
    debug_info.push(format!(
        "Format Discovery: Found {} corrections, retrying request",
        format_corrections.len()
    ));

    // Try the request with corrected components
    let mut corrected_params = params.clone();
    corrected_params["components"] = Value::Object(corrected_components);

    let result = execute_brp_method(method, Some(corrected_params), port).await?;
    debug_info.push(format!("Format Discovery: Retry result: {:?}", result));

    Ok(EnhancedBrpResult {
        result,
        format_corrections,
        debug_info,
    })
}

/// Unified format discovery for spawn/insert operations (work with "components" parameter)
async fn attempt_components_format_discovery(
    method: &str,
    params: &Value,
    port: Option<u16>,
    original_error: &BrpError,
) -> Result<EnhancedBrpResult, McpError> {
    // Extract components from parameters
    let components = match extract_and_validate_components(params, original_error) {
        Some(comps) => comps,
        None => {
            return Ok(EnhancedBrpResult {
                result:             BrpResult::Error(original_error.clone()),
                format_corrections: Vec::new(),
                debug_info:         vec![
                    "Format Discovery: No components object found in params".to_string(),
                ],
            });
        }
    };

    let mut format_corrections = Vec::new();
    let mut corrected_components = Map::new();
    let mut debug_info = vec![format!(
        "Format Discovery: Found {} components to check",
        components.len()
    )];
    let mut all_tier_info = Vec::new();

    // Process each component
    for (component_name, component_value) in components {
        let (discovery_result, tier_info) = process_single_component(
            component_name,
            component_value,
            method,
            port,
            original_error,
            &mut debug_info,
        )
        .await?;

        all_tier_info.extend(tier_info);

        match discovery_result {
            Some((final_format, hint)) => {
                format_corrections.push(FormatCorrection {
                    component: component_name.to_string(),
                    original_format: component_value.clone(),
                    corrected_format: final_format.clone(),
                    hint,
                });
                corrected_components.insert(component_name.to_string(), final_format);
            }
            None => {
                // Keep original format if no alternative found
                corrected_components.insert(component_name.to_string(), component_value.clone());
            }
        }
    }

    // Add tier information to debug_info
    debug_info.extend(tier_info_to_debug_strings(&all_tier_info));

    if format_corrections.is_empty() {
        debug_info.push("Format Discovery: No corrections were possible".to_string());
        // No corrections were possible
        Ok(EnhancedBrpResult {
            result: BrpResult::Error(original_error.clone()),
            format_corrections: Vec::new(),
            debug_info,
            })
    } else {
        // Apply corrections and retry
        apply_corrections_and_retry(
            method,
            params,
            port,
            corrected_components,
            format_corrections,
            debug_info,
        )
        .await
    }
}

/// Format discovery for `mutate_component` operations (work with "value" parameter)
async fn attempt_value_format_discovery(
    method: &str,
    params: &Value,
    port: Option<u16>,
    original_error: &BrpError,
) -> Result<EnhancedBrpResult, McpError> {
    let Some(component_name) = params.get("component").and_then(|v| v.as_str()) else {
        return Ok(EnhancedBrpResult {
            result:             BrpResult::Error(original_error.clone()),
            format_corrections: Vec::new(),
            debug_info:         vec![format!(
                "Format Discovery: No component name found in mutate params"
            )],
        });
    };

    let Some(original_value) = params.get("value") else {
        return Ok(EnhancedBrpResult {
            result:             BrpResult::Error(original_error.clone()),
            format_corrections: Vec::new(),
            debug_info:         vec![format!("Format Discovery: No value found in mutate params")],
        });
    };

    let (discovery_result, tier_info) =
        tiered_component_format_discovery(component_name, original_value, original_error, port)
            .await;

    let mut debug_info = vec![format!(
        "Format Discovery: Processing mutate component '{}'",
        component_name
    )];
    debug_info.extend(tier_info_to_debug_strings(&tier_info));

    match discovery_result {
        Some((corrected_value, hint)) => {
            let mut corrected_params = params.clone();
            corrected_params["value"] = corrected_value.clone();

            let result = execute_brp_method(method, Some(corrected_params), port).await?;
            let format_corrections = vec![FormatCorrection {
                component: component_name.to_string(),
                original_format: original_value.clone(),
                corrected_format: corrected_value,
                hint,
            }];

            debug_info.push(format!("Format Discovery: Found alternative for mutate component '{}', retried successfully", component_name));
            Ok(EnhancedBrpResult {
                result,
                format_corrections,
                debug_info,
            })
        }
        None => {
            debug_info.push(format!(
                "Format Discovery: No alternative found for mutate component '{}'",
                component_name
            ));
            Ok(EnhancedBrpResult {
                result: BrpResult::Error(original_error.clone()),
                format_corrections: Vec::new(),
                debug_info,
            })
        }
    }
}

/// Tiered format discovery dispatcher - replaces try_component_format_alternatives
/// Uses intelligent pattern matching with fallback to generic approaches
async fn tiered_component_format_discovery(
    component_name: &str,
    original_value: &Value,
    error: &BrpError,
    port: Option<u16>,
) -> (Option<(Value, String)>, Vec<TierInfo>) {
    let mut tier_info = Vec::new();

    // Tier 1: Deterministic Pattern Matching
    let error_analysis = analyze_error_pattern(error);
    if let Some(pattern) = &error_analysis.pattern {
        if error_analysis.confidence >= 0.8 {
            tier_info.push(TierInfo {
                tier:      TIER_DETERMINISTIC,
                tier_name: "Deterministic Pattern Matching".to_string(),
                action:    format!("Matched pattern: {:?}", pattern),
                success:   false, // Will be updated if successful
            });

            if let Some((corrected_value, hint)) =
                apply_pattern_fix(pattern, component_name, original_value)
            {
                tier_info.last_mut().unwrap().success = true;
                tier_info.last_mut().unwrap().action = format!("Applied pattern fix: {}", hint);
                return (Some((corrected_value, hint)), tier_info);
            }
        }
    }

    // Tier 2: Serialization Diagnostics (for UnknownComponentType pattern)
    if let Some(ErrorPattern::UnknownComponentType { component_type }) = &error_analysis.pattern {
        tier_info.push(TierInfo {
            tier:      TIER_SERIALIZATION,
            tier_name: "Serialization Diagnostics".to_string(),
            action:    format!(
                "Checking serialization support for component: {}",
                component_type
            ),
            success:   false,
        });

        match check_component_serialization(component_type, port).await {
            Ok(serialization_check) => {
                tier_info.last_mut().unwrap().success = true;
                tier_info.last_mut().unwrap().action =
                    serialization_check.diagnostic_message.clone();

                // Return diagnostic information instead of a fix
                return (None, tier_info); // No fix, just diagnostic
            }
            Err(_) => {
                tier_info.last_mut().unwrap().action =
                    "Failed to query serialization info".to_string();
            }
        }
    }

    // Tier 3: Generic Fallback (existing logic)
    tier_info.push(TierInfo {
        tier:      TIER_GENERIC_FALLBACK,
        tier_name: "Generic Fallback".to_string(),
        action:    "Trying generic format alternatives".to_string(),
        success:   false,
    });

    let fallback_result =
        try_component_format_alternatives_legacy(component_name, original_value, error);
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
    StringToObject,
    ArrayToString,
    ArrayToObject,
}

/// Apply a transformation to convert between formats
fn apply_transformation(value: &Value, transformation: TransformationType) -> Option<Value> {
    match transformation {
        TransformationType::ObjectToString => {
            if let Value::Object(map) = value {
                // Try to extract string from common field names
                for field in ["value", "name", "text", "label"] {
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
        TransformationType::ObjectToArray => {
            if let Value::Object(map) = value {
                let values: Vec<Value> = map.values().cloned().collect();
                if !values.is_empty() {
                    return Some(Value::Array(values));
                }
            }
            None
        }
        TransformationType::StringToObject => {
            if let Value::String(s) = value {
                let mut map = Map::new();
                map.insert("value".to_string(), Value::String(s.clone()));
                return Some(Value::Object(map));
            }
            None
        }
        TransformationType::ArrayToString => {
            if let Value::Array(arr) = value {
                if arr.len() == 1 {
                    if let Value::String(s) = &arr[0] {
                        return Some(Value::String(s.clone()));
                    }
                }
            }
            None
        }
        TransformationType::ArrayToObject => {
            if let Value::Array(arr) = value {
                let mut map = Map::new();
                map.insert("items".to_string(), Value::Array(arr.clone()));
                return Some(Value::Object(map));
            }
            None
        }
    }
}

/// Get possible transformations based on the source value type
fn get_possible_transformations(value: &Value) -> Vec<TransformationType> {
    match value {
        Value::Object(_) => vec![
            TransformationType::ObjectToString,
            TransformationType::ObjectToArray,
        ],
        Value::String(_) => vec![TransformationType::StringToObject],
        Value::Array(_) => vec![
            TransformationType::ArrayToString,
            TransformationType::ArrayToObject,
        ],
        _ => vec![],
    }
}

/// Legacy format discovery function (renamed from try_component_format_alternatives)
/// Since we can't reliably parse error messages, we try all reasonable alternatives
fn try_component_format_alternatives_legacy(
    component_name: &str,
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
                    format!("`{component_name}` expects string format, not object")
                }
                TransformationType::ObjectToArray => {
                    format!("`{component_name}` expects array format, not object")
                }
                TransformationType::StringToObject => {
                    format!("`{component_name}` expects object format, not string")
                }
                TransformationType::ArrayToString => {
                    format!("`{component_name}` expects string format, not array")
                }
                TransformationType::ArrayToObject => {
                    format!("`{component_name}` expects object format, not array")
                }
            };
            return Some((transformed_value, hint));
        }
    }

    None
}

/// Test a component format by spawning a test entity
async fn test_component_format_with_spawn(
    component_name: &str,
    component_value: &Value,
    port: Option<u16>,
) -> Result<Value, McpError> {
    let mut test_components = Map::new();
    test_components.insert(component_name.to_string(), component_value.clone());

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
