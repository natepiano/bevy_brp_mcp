//! Constants and static regex patterns for format discovery

use once_cell::sync::Lazy;
use regex::Regex;

use crate::tools::{
    BRP_METHOD_INSERT, BRP_METHOD_INSERT_RESOURCE, BRP_METHOD_MUTATE_COMPONENT,
    BRP_METHOD_MUTATE_RESOURCE, BRP_METHOD_SPAWN,
};

/// Error code for component type format errors from BRP
pub const COMPONENT_FORMAT_ERROR_CODE: i32 = -23402;

/// Error code for resource type format errors from BRP
pub const RESOURCE_FORMAT_ERROR_CODE: i32 = -23501;

/// Tier constants for format discovery
pub const TIER_SERIALIZATION: u8 = 1;
pub const TIER_DIRECT_DISCOVERY: u8 = 2;
pub const TIER_DETERMINISTIC: u8 = 3;
pub const TIER_GENERIC_FALLBACK: u8 = 4;

/// Methods that support format discovery (components and resources)
pub const FORMAT_DISCOVERY_METHODS: &[&str] = &[
    BRP_METHOD_SPAWN,
    BRP_METHOD_INSERT,
    BRP_METHOD_MUTATE_COMPONENT,
    BRP_METHOD_INSERT_RESOURCE,
    BRP_METHOD_MUTATE_RESOURCE,
];

/// Static regex patterns for error analysis - Based on exact Bevy error strings
pub static TRANSFORM_SEQUENCE_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"expected a sequence of (\d+) f32 values").expect("Invalid regex"));
pub static EXPECTED_TYPE_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"expected `([a-zA-Z_:]+(?::[a-zA-Z_:]+)*)`").expect("Invalid regex"));
pub static ACCESS_ERROR_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"Error accessing element with `([^`]+)` access(?:\s*\(offset \d+\))?: (.+)")
        .expect("Invalid regex")
});
pub static TYPE_MISMATCH_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"Expected ([a-zA-Z0-9_\[\]]+) access to access a ([a-zA-Z0-9_]+), found a ([a-zA-Z0-9_]+) instead\.")
        .expect("Invalid regex")
});
pub static VARIANT_TYPE_MISMATCH_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"Expected variant ([a-zA-Z0-9_\[\]]+) access to access a ([a-zA-Z0-9_]+) variant, found a ([a-zA-Z0-9_]+) variant instead\.")
        .expect("Invalid regex")
});
pub static MISSING_FIELD_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r#"The ([a-zA-Z0-9_]+) accessed doesn't have (?:an? )?[`"]([^`"]+)[`"] field"#)
        .expect("Invalid regex")
});
pub static UNKNOWN_COMPONENT_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"Unknown component type: `([^`]+)`").expect("Invalid regex"));
pub static TUPLE_STRUCT_PATH_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r#"(?:at path|path)\s+[`"]?([^`"\s]+)[`"]?"#).expect("Invalid regex"));
pub static MATH_TYPE_ARRAY_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(Vec2|Vec3|Vec4|Quat)\s+(?:expects?|requires?|needs?)\s+array")
        .expect("Invalid regex")
});
pub static UNKNOWN_COMPONENT_TYPE_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"Unknown component type(?::\s*)?[`']?([^`'\s]+)[`']?").expect("Invalid regex")
});
