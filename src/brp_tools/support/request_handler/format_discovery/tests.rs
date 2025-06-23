//! Tests for format discovery functionality

#![allow(clippy::expect_used, clippy::unwrap_used, clippy::panic)]

use serde_json::json;

use super::constants::*;
use super::detection::{ErrorPattern, analyze_error_pattern};
use super::engine::is_type_format_error;
use super::transformations::{
    apply_pattern_fix, convert_to_math_type_array, extract_string_value, fix_tuple_struct_path,
};
use crate::brp_tools::support::brp_client::BrpError;
use crate::brp_tools::support::request_handler::format_discovery::transformations::fix_access_error;

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
        message: "Error accessing element with Field access at path .LinearRgba.red".to_string(),
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

#[test]
fn test_fix_access_error_path_suggestions() {
    // Test path suggestion for color field access
    let original_value = json!({"red": 1.0, "green": 0.5, "blue": 0.2, "alpha": 1.0});

    let result = fix_access_error(
        "bevy_color::linear_rgba::LinearRgba",
        &original_value,
        "Field",
        "Error accessing element with Field access at path .LinearRgba.red",
    );

    assert!(result.is_some());
    let (returned_value, hint) = result.unwrap();

    // Value should be returned unchanged for path suggestions
    assert_eq!(returned_value, original_value);

    // Hint should suggest the correct path
    assert!(hint.contains("try using path `.0.0` instead of `.LinearRgba.red`"));
}

#[test]
fn test_fix_access_error_math_field_suggestions() {
    // Test path suggestion for math vector field access
    let original_value = json!([1.0, 2.0, 3.0]);

    let result = fix_access_error(
        "bevy_math::vec3::Vec3",
        &original_value,
        "Field",
        "Error accessing element with Field access at path .Vec3.x",
    );

    assert!(result.is_some());
    let (returned_value, hint) = result.unwrap();

    // Value should be returned unchanged for path suggestions
    assert_eq!(returned_value, original_value);

    // Hint should suggest the correct path
    assert!(hint.contains("try using path `.0.0` instead of `.Vec3.x`"));
}

#[test]
fn test_fix_access_error_generic_enum_suggestions() {
    // Test path suggestion for generic enum variants
    let original_value =
        json!({"SomeColor": {"red": 1.0, "green": 0.5, "blue": 0.0, "alpha": 1.0}});

    let result = fix_access_error(
        "some_crate::SomeEnum",
        &original_value,
        "Field",
        "Error accessing element with Field access at path .SomeColor.green",
    );

    assert!(result.is_some());
    let (returned_value, hint) = result.unwrap();

    // Value should be returned unchanged for path suggestions
    assert_eq!(returned_value, original_value);

    // Hint should suggest the correct path
    assert!(hint.contains("try using path `.0.1` instead of `.SomeColor.green`"));
}

#[test]
fn test_fix_access_error_fallback_to_value_fixes() {
    // Test that when path suggestions don't work, it falls back to value format fixes
    let original_value = json!({"field_name": "some_value"});

    let result = fix_access_error(
        "some_type::SomeType",
        &original_value,
        "Field",
        "Error accessing element with Field access at path .unknown_path",
    );

    // This should still return a result with the existing fallback logic
    // The exact behavior depends on the existing logic, but it should not crash
    // and should provide some kind of transformation or hint
    if let Some((_, hint)) = result {
        // Should provide some form of assistance, either path suggestion or value transformation
        assert!(!hint.is_empty());
    }
}

#[test]
fn test_fix_access_error_simple_field_path() {
    // Test simple field path conversion (like .x -> .0)
    let original_value = json!([1.0, 2.0, 3.0]);

    let result = fix_access_error(
        "some_type::TupleStruct",
        &original_value,
        "Field",
        "Error accessing element with Field access at path .x",
    );

    assert!(result.is_some());
    let (returned_value, hint) = result.unwrap();

    // Value should be returned unchanged for path suggestions
    assert_eq!(returned_value, original_value);

    // Hint should suggest the correct path
    assert!(hint.contains("try using path `.0` instead of `.x`"));
}

#[test]
fn test_fix_access_error_integration_with_pattern_matching() {
    // Test integration with the actual pattern matching system
    use super::detection::analyze_error_pattern;
    use super::transformations::apply_pattern_fix;

    let error = BrpError {
        code:    COMPONENT_FORMAT_ERROR_CODE,
        message: "Error accessing element with `Field` access: failed at path .LinearRgba.red"
            .to_string(),
        data:    None,
    };

    // First, ensure pattern detection works
    let analysis = analyze_error_pattern(&error);
    assert!(analysis.pattern.is_some());

    // Then test the fix application
    let original_value = json!({"red": 1.0, "green": 0.5, "blue": 0.2, "alpha": 1.0});
    let pattern = analysis.pattern.as_ref().unwrap();
    let result = apply_pattern_fix(
        pattern,
        "bevy_color::linear_rgba::LinearRgba",
        &original_value,
    );

    assert!(result.is_some());
    let (returned_value, hint) = result.unwrap();

    // Should return original value with path suggestion
    assert_eq!(returned_value, original_value);
    assert!(hint.contains("try using path `.0.0` instead of `.LinearRgba.red`"));
}
