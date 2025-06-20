//! Constants and static regex patterns for format discovery

use std::sync::OnceLock;

use regex::Regex;

use crate::brp_tools::constants::{
    BRP_METHOD_INSERT, BRP_METHOD_INSERT_RESOURCE, BRP_METHOD_MUTATE_COMPONENT,
    BRP_METHOD_MUTATE_RESOURCE, BRP_METHOD_SPAWN,
};

/// Error code for component type format errors from BRP
pub const COMPONENT_FORMAT_ERROR_CODE: i32 = -23402;

/// Error code for resource type format errors from BRP
pub const RESOURCE_FORMAT_ERROR_CODE: i32 = -23501;

/// Tier constants for format discovery
pub const TIER_DETERMINISTIC: u8 = 1;
pub const TIER_SERIALIZATION: u8 = 2;
pub const TIER_GENERIC_FALLBACK: u8 = 3;

/// Methods that support format discovery (components and resources)
pub const FORMAT_DISCOVERY_METHODS: &[&str] = &[
    BRP_METHOD_SPAWN,
    BRP_METHOD_INSERT,
    BRP_METHOD_MUTATE_COMPONENT,
    BRP_METHOD_INSERT_RESOURCE,
    BRP_METHOD_MUTATE_RESOURCE,
];

/// Static regex patterns for error analysis - Based on exact Bevy error strings
pub static TRANSFORM_SEQUENCE_REGEX: OnceLock<Regex> = OnceLock::new();
pub static EXPECTED_TYPE_REGEX: OnceLock<Regex> = OnceLock::new();
pub static ACCESS_ERROR_REGEX: OnceLock<Regex> = OnceLock::new();
pub static TYPE_MISMATCH_REGEX: OnceLock<Regex> = OnceLock::new();
pub static VARIANT_TYPE_MISMATCH_REGEX: OnceLock<Regex> = OnceLock::new();
pub static MISSING_FIELD_REGEX: OnceLock<Regex> = OnceLock::new();
pub static UNKNOWN_COMPONENT_REGEX: OnceLock<Regex> = OnceLock::new();
pub static TUPLE_STRUCT_PATH_REGEX: OnceLock<Regex> = OnceLock::new();
pub static MATH_TYPE_ARRAY_REGEX: OnceLock<Regex> = OnceLock::new();
pub static UNKNOWN_COMPONENT_TYPE_REGEX: OnceLock<Regex> = OnceLock::new();
