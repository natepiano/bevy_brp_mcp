//! Format transformations and fix logic

use serde_json::{Map, Value};

use super::detection::{ErrorPattern, extract_path_from_error_context};
use super::field_mapper::map_field_to_tuple_index;
use super::path_parser::{parse_generic_enum_field_access, parse_path_to_field_access};
use crate::brp_tools::request_handler::constants::{
    FIELD_LABEL, FIELD_NAME, FIELD_TEXT, FIELD_VALUE,
};
use crate::brp_tools::support::brp_client::BrpError;

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
            is_variant,
        } => {
            // Handle type mismatch errors (including variant mismatches)
            let mismatch_info = if *is_variant {
                StructureMismatchInfo::VariantTypeMismatch {
                    expected: expected.clone(),
                    actual:   actual.clone(),
                    access:   access.clone(),
                }
            } else {
                StructureMismatchInfo::TypeMismatch {
                    expected: expected.clone(),
                    actual:   actual.clone(),
                    access:   access.clone(),
                }
            };
            fix_structure_mismatch(type_name, original_value, mismatch_info)
        }
        ErrorPattern::MissingField {
            field_name,
            type_name: _,
        } => {
            // Handle missing field errors - convert to tuple access
            let mismatch_info = StructureMismatchInfo::MissingField {
                field_name: field_name.clone(),
            };
            fix_structure_mismatch(type_name, original_value, mismatch_info)
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

/// Hints about what kind of transformation might be needed based on error analysis
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransformationHint {
    /// Try converting to string (e.g., Name component expects string)
    NeedsString,
    /// Try converting to array (e.g., Vec3, Transform expects arrays)
    NeedsArray,
    /// Try converting to object (less common)
    NeedsObject,
    /// Try tuple struct conversions (field access issues)
    NeedsTupleAccess,
    /// No clear hint - try all possibilities
    Unknown,
}

/// Analyze error message to determine what transformation might help
pub fn analyze_error_for_transformation_hint(error: &BrpError) -> TransformationHint {
    let message = &error.message;

    // Check for string expectations
    if message.contains("expected string")
        || message.contains("String")
        || message.contains("Name")
        || message.contains("expected `bevy_ecs::name::Name`")
    {
        return TransformationHint::NeedsString;
    }

    // Check for array expectations
    if message.contains("expected array")
        || message.contains("sequence")
        || message.contains("Vec2")
        || message.contains("Vec3")
        || message.contains("Vec4")
        || message.contains("Quat")
        || message.contains("Transform")
    {
        return TransformationHint::NeedsArray;
    }

    // Check for tuple struct access issues
    if message.contains("tuple struct")
        || message.contains("tuple_struct")
        || message.contains("TupleIndex")
        || message.contains("found a tuple struct instead")
    {
        return TransformationHint::NeedsTupleAccess;
    }

    // Check for object expectations (rare)
    if message.contains("expected object") || message.contains("struct") {
        return TransformationHint::NeedsObject;
    }

    // No clear hint
    TransformationHint::Unknown
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

/// Get transformations based on transformation hint and value type
pub fn get_transformations_for_hint(
    hint: TransformationHint,
    value: &Value,
) -> Vec<TransformationType> {
    match (hint, value) {
        // If we need a string
        (TransformationHint::NeedsString, Value::Object(_)) => {
            vec![TransformationType::ObjectToString]
        }
        (
            TransformationHint::NeedsString | TransformationHint::NeedsTupleAccess,
            Value::Array(_),
        ) => {
            vec![TransformationType::ArrayToString]
        }

        // If we need an array
        (TransformationHint::NeedsArray, Value::Object(_)) => {
            vec![TransformationType::ObjectToArray]
        }

        // If we need an object
        (TransformationHint::NeedsObject, Value::Array(_)) => {
            vec![TransformationType::ArrayToObject]
        }

        // Tuple access issues often need array->string or object->string conversions
        (TransformationHint::NeedsTupleAccess, Value::Object(_)) => vec![
            TransformationType::ObjectToString,
            TransformationType::ObjectToArray,
        ],

        // Unknown hint - try all possibilities
        (TransformationHint::Unknown, _) => get_possible_transformations(value),

        // If value is already the right type, no transformation needed
        _ => vec![],
    }
}

/// Legacy format discovery function (renamed from `try_component_format_alternatives`)
/// Now uses guided transformation based on error analysis
pub fn try_component_format_alternatives_legacy(
    type_name: &str,
    original_value: &Value,
    error: &BrpError,
) -> Option<(Value, String)> {
    // First, analyze the error to get a transformation hint
    let hint = analyze_error_for_transformation_hint(error);

    // Get guided transformations based on the hint
    let transformations = get_transformations_for_hint(hint, original_value);

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
pub fn fix_access_error(
    type_name: &str,
    original_value: &Value,
    access: &str,
    error_type: &str,
) -> Option<(Value, String)> {
    // STEP 1: Try path suggestions first (before value format fixes)
    let field_path = extract_path_from_error_context(error_type);

    if let Some(path) = &field_path {
        // Try to convert the path using generic enum field access parsing
        if let Some(suggested_path) = parse_generic_enum_field_access(path) {
            let hint = format!(
                "`{type_name}` AccessError: try using path `{suggested_path}` instead of `{path}`"
            );
            // Return the original value unchanged, but with a path suggestion
            return Some((original_value.clone(), hint));
        }

        // Try the type-safe path conversion system
        if let Some(field_access) = parse_path_to_field_access(path) {
            let suggested_path = map_field_to_tuple_index(&field_access);
            if suggested_path != *path {
                let hint = format!(
                    "`{type_name}` AccessError: try using path `{suggested_path}` instead of `{path}`"
                );
                // Return the original value unchanged, but with a path suggestion
                return Some((original_value.clone(), hint));
            }
        }

        // Try the simple path fix function
        let fixed_path = fix_tuple_struct_path(path);
        if fixed_path != *path {
            let hint = format!(
                "`{type_name}` AccessError: try using path `{fixed_path}` instead of `{path}`"
            );
            // Return the original value unchanged, but with a path suggestion
            return Some((original_value.clone(), hint));
        }
    }

    // STEP 2: If path suggestions didn't work, try value format fixes (existing logic)
    if let Some(path) = field_path {
        if let Some(result) = fix_tuple_struct_format(type_name, original_value, &path) {
            return Some(result);
        }
    }

    // STEP 3: Fallback to generic access pattern fixes based on the access type
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

/// Extract single value from single-field object
fn extract_single_field_value(obj: &serde_json::Map<String, Value>) -> Option<(&str, &Value)> {
    if obj.len() == 1 {
        obj.iter().next().map(|(k, v)| (k.as_str(), v))
    } else {
        None
    }
}

/// Convert single-field object to value for tuple struct access
fn convert_object_to_tuple_access(
    type_name: &str,
    obj: &serde_json::Map<String, Value>,
    context: &str,
) -> Option<(Value, String)> {
    extract_single_field_value(obj).map(|(field_name, value)| {
        let hint =
            format!("`{type_name}` {context}: converted field '{field_name}' to tuple access");
        (value.clone(), hint)
    })
}

/// Convert array to single element for struct access
fn convert_array_to_struct_access(
    type_name: &str,
    arr: &[Value],
    context: &str,
) -> Option<(Value, String)> {
    arr.first().map(|element| {
        let hint = format!("`{type_name}` {context}: using first array element");
        (element.clone(), hint)
    })
}

/// Try to convert field name to tuple index and extract element from array
fn try_tuple_struct_field_access(
    type_name: &str,
    field_name: &str,
    original_value: &Value,
) -> Option<(Value, String)> {
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
            Value::Object(obj) => {
                let context =
                    format!("MissingField '{field_name}': converted object to tuple struct access");
                return convert_object_to_tuple_access(type_name, obj, &context);
            }
            _ => {}
        }
    }
    None
}

/// Try to extract enum variant value from object
fn try_enum_variant_extraction(
    type_name: &str,
    field_name: &str,
    obj: &serde_json::Map<String, Value>,
) -> Option<(Value, String)> {
    // Try to find the variant field
    obj.get(field_name).map_or_else(
        || {
            // Fallback: try single field extraction
            extract_single_field_value(obj).map(|(actual_field, value)| {
                let hint = format!(
                    "`{type_name}` MissingField '{field_name}': used field '{actual_field}' instead"
                );
                (value.clone(), hint)
            })
        },
        |variant_value| {
            let hint =
                format!("`{type_name}` MissingField '{field_name}': extracted enum variant value");
            Some((variant_value.clone(), hint))
        },
    )
}

/// Handle type mismatch scenarios
fn handle_type_mismatch(
    type_name: &str,
    original_value: &Value,
    expected: &str,
    actual: &str,
    access: &str,
) -> Option<(Value, String)> {
    // Common type mismatches and their fixes
    match (expected, actual) {
        // Trying to access a struct field on a tuple struct
        ("struct", "tuple_struct") => {
            if let Value::Object(obj) = original_value {
                let context =
                    format!("TypeMismatch: Expected {expected} access to access a {actual}");
                return convert_object_to_tuple_access(type_name, obj, &context);
            }
        }
        // Trying to access a tuple index on a struct
        ("tuple_struct", "struct") => {
            if let Value::Array(arr) = original_value {
                let context =
                    format!("TypeMismatch: Expected {expected} access to access a {actual}");
                return convert_array_to_struct_access(type_name, arr, &context);
            }
        }
        // Enum variant mismatches
        ("variant", "tuple_struct") | ("tuple_struct", "variant") => {
            // Try to convert between variant and tuple struct formats
            match original_value {
                Value::Object(obj) => {
                    let context = format!(
                        "TypeMismatch: Expected {expected}, found {actual}, extracting inner value"
                    );
                    return convert_object_to_tuple_access(type_name, obj, &context);
                }
                Value::Array(arr) => {
                    let context = format!("TypeMismatch: Expected {expected}, found {actual}");
                    return convert_array_to_struct_access(type_name, arr, &context);
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
                let context = format!("TypeMismatch with {access} access");
                if let Some((field_name, value)) = extract_single_field_value(obj) {
                    let hint = format!("`{type_name}` {context}: using field '{field_name}'");
                    return Some((value.clone(), hint));
                }
            }
        }
        "TupleIndex" => {
            // Tuple index access mismatch
            if let Value::Array(arr) = original_value {
                let context = format!("TypeMismatch with {access} access");
                return convert_array_to_struct_access(type_name, arr, &context);
            }
        }
        _ => {}
    }
    None
}

/// Handle variant type mismatch scenarios
fn handle_variant_type_mismatch(
    type_name: &str,
    original_value: &Value,
    expected: &str,
    actual: &str,
    access: &str,
) -> Option<(Value, String)> {
    // Common enum variant mismatches
    match (expected, actual) {
        // Tuple variant vs struct variant
        ("tuple", "struct") => {
            if let Value::Object(obj) = original_value {
                if let Some((variant_name, value)) = extract_single_field_value(obj) {
                    let hint = format!(
                        "`{type_name}` VariantTypeMismatch: Expected {expected} variant access to access a {actual} variant, \
                                    converted '{variant_name}' to tuple variant format"
                    );
                    return Some((value.clone(), hint));
                }
            }
        }
        // Struct variant vs tuple variant
        ("struct", "tuple") => {
            if let Value::Array(arr) = original_value {
                let context = format!(
                    "VariantTypeMismatch: Expected {expected} variant access to access a {actual} variant, converted array to struct variant format"
                );
                return convert_array_to_struct_access(type_name, arr, &context);
            }
        }
        _ => {}
    }

    // Use access type to determine conversion
    match access {
        "Field" | "FieldMut" => {
            // Field access on enum variant, likely needs tuple conversion
            if let Value::Object(obj) = original_value {
                let context = format!(
                    "VariantTypeMismatch with {access} access: converted to variant element"
                );
                return convert_object_to_tuple_access(type_name, obj, &context);
            }
        }
        "TupleIndex" => {
            // Tuple index access on enum variant
            if let Value::Array(arr) = original_value {
                let context =
                    format!("VariantTypeMismatch with {access} access: using variant element");
                return convert_array_to_struct_access(type_name, arr, &context);
            }
        }
        _ => {}
    }
    None
}

/// Handle missing field scenarios
fn handle_missing_field(
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
        if let Some(result) = try_tuple_struct_field_access(type_name, field_name, original_value) {
            return Some(result);
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
            if let Some(result) = try_enum_variant_extraction(type_name, field_name, obj) {
                return Some(result);
            }
        }
    }

    // Generic fallback: try to extract any reasonable value
    match original_value {
        Value::Object(obj) => {
            if let Some((actual_field, value)) = extract_single_field_value(obj) {
                let hint = format!(
                    "`{type_name}` MissingField '{field_name}': used available field '{actual_field}'"
                );
                return Some((value.clone(), hint));
            }
        }
        Value::Array(arr) => {
            let context = format!("MissingField '{field_name}'");
            return convert_array_to_struct_access(type_name, arr, &context);
        }
        _ => {}
    }
    None
}

/// Fix structure mismatch errors (consolidates type mismatch, variant type mismatch, and missing
/// field fixes)
fn fix_structure_mismatch(
    type_name: &str,
    original_value: &Value,
    mismatch_info: StructureMismatchInfo,
) -> Option<(Value, String)> {
    // Handle different types of structure mismatches
    match mismatch_info {
        StructureMismatchInfo::TypeMismatch {
            expected,
            actual,
            access,
        } => handle_type_mismatch(type_name, original_value, &expected, &actual, &access),
        StructureMismatchInfo::VariantTypeMismatch {
            expected,
            actual,
            access,
        } => handle_variant_type_mismatch(type_name, original_value, &expected, &actual, &access),
        StructureMismatchInfo::MissingField { field_name } => {
            handle_missing_field(type_name, original_value, &field_name)
        }
    }
}

/// Enum to represent different structure mismatch scenarios
#[derive(Debug, Clone)]
enum StructureMismatchInfo {
    TypeMismatch {
        expected: String,
        actual:   String,
        access:   String,
    },
    VariantTypeMismatch {
        expected: String,
        actual:   String,
        access:   String,
    },
    MissingField {
        field_name: String,
    },
}

/// Smart format discovery that consolidates Tiers 3 and 4
/// Combines deterministic pattern matching with generic fallback transformations
pub fn apply_smart_format_discovery(
    type_name: &str,
    original_value: &Value,
    error: &BrpError,
    error_pattern: Option<&ErrorPattern>,
) -> Option<(Value, String)> {
    // First try deterministic pattern matching (Tier 3)
    if let Some(pattern) = error_pattern {
        if let Some(result) = apply_pattern_fix(pattern, type_name, original_value) {
            return Some(result);
        }
    }

    // If pattern matching didn't work, fall back to generic transformations (Tier 4)
    try_component_format_alternatives_legacy(type_name, original_value, error)
}
