//! Format transformations and fix logic

use serde_json::{Map, Value};

use super::detection::{ErrorPattern, extract_path_from_error_context};
use super::field_mapper::map_field_to_tuple_index;
use super::path_parser::{parse_generic_enum_field_access, parse_path_to_field_access};
use crate::brp_tools::support::brp_client::BrpError;
use crate::brp_tools::support::request_handler::constants::{
    FIELD_LABEL, FIELD_NAME, FIELD_TEXT, FIELD_VALUE,
};

/// Helper function to format type mismatch error messages
pub fn type_format_error(type_name: &str, expected: &str, found: &str) -> String {
    format!("`{type_name}` expects {expected} format, not {found}")
}

/// Helper function to format array expectation messages
pub fn type_expects_array(type_name: &str, array_type: &str) -> String {
    format!("`{type_name}` {array_type} expects array format")
}

/// Helper function to fix tuple struct paths for all enum tuple variants
/// Uses the new type-safe system for better maintainability and correctness
pub fn fix_tuple_struct_path(path: &str) -> String {
    // First, try the type-safe approach using our new parsing system
    if let Some(field_access) = parse_path_to_field_access(path) {
        return map_field_to_tuple_index(&field_access);
    }

    // Fallback: handle simple field access patterns
    match path {
        // Simple tuple struct field access (not nested) - these remain direct indices
        ".x" => ".0".to_string(),
        ".y" => ".1".to_string(),
        ".z" => ".2".to_string(),

        // Generic patterns for unknown enum variants
        _ => {
            // Try generic enum field access parsing as fallback
            if let Some(fixed_path) = parse_generic_enum_field_access(path) {
                return fixed_path;
            }

            // Ultimate fallback: return original path
            path.to_string()
        }
    }
}

/// Generic function to convert object values to array format
/// Handles Vec2 [x, y], Vec3 [x, y, z], Vec4/Quat [x, y, z, w]
pub fn convert_to_array_format(value: &Value, field_names: &[&str]) -> Option<Value> {
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
pub fn convert_to_math_type_array(value: &Value, math_type: &str) -> Option<Value> {
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
pub fn extract_string_value(value: &Value) -> Option<(String, String)> {
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

/// Transformation types for generic format conversion
#[derive(Debug, Clone, Copy)]
pub enum TransformationType {
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
pub fn apply_transformation(value: &Value, transformation: TransformationType) -> Option<Value> {
    match transformation {
        TransformationType::ObjectToString => transform_object_to_string(value),
        TransformationType::ObjectToArray => transform_object_to_array(value),
        TransformationType::ArrayToString => transform_array_to_string(value),
        TransformationType::ArrayToObject => transform_array_to_object(value),
    }
}

/// Get possible transformations based on the source value type
pub fn get_possible_transformations(value: &Value) -> Vec<TransformationType> {
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
pub fn try_component_format_alternatives_legacy(
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
        // Enum variant mismatches
        ("variant", "tuple_struct") | ("tuple_struct", "variant") => {
            // Try to convert between variant and tuple struct formats
            match original_value {
                Value::Object(obj) if obj.len() == 1 => {
                    if let Some((_, value)) = obj.iter().next() {
                        let hint = format!(
                            "`{type_name}` TypeMismatch: Expected {expected}, found {actual}, \
                            extracting inner value"
                        );
                        return Some((value.clone(), hint));
                    }
                }
                Value::Array(arr) if !arr.is_empty() => {
                    let hint = format!(
                        "`{type_name}` TypeMismatch: Expected {expected}, found {actual}, \
                        using first element"
                    );
                    return Some((arr[0].clone(), hint));
                }
                _ => {}
            }
        }
        _ => {}
    }

    // Use access type as additional context
    match access {
        "Field" | "FieldMut" => {
            // Field access mismatch, try extracting single field
            if let Value::Object(obj) = original_value {
                if obj.len() == 1 {
                    if let Some((field_name, value)) = obj.iter().next() {
                        let hint = format!(
                            "`{type_name}` TypeMismatch with {access} access: using field '{field_name}'"
                        );
                        return Some((value.clone(), hint));
                    }
                }
            }
        }
        "TupleIndex" => {
            // Tuple index access mismatch
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
