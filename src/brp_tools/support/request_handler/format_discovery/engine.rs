//! Orchestration and retry logic for format discovery

use rmcp::Error as McpError;
use serde_json::{Map, Value};

use super::constants::{
    COMPONENT_FORMAT_ERROR_CODE, FORMAT_DISCOVERY_METHODS, RESOURCE_FORMAT_ERROR_CODE,
    TIER_DETERMINISTIC, TIER_DIRECT_DISCOVERY, TIER_GENERIC_FALLBACK, TIER_SERIALIZATION,
};
use super::detection::{TierInfo, TierManager, analyze_error_pattern, check_type_serialization};
use super::transformations::{apply_pattern_fix, try_component_format_alternatives_legacy};
use crate::brp_tools::support::brp_client::{BrpError, BrpResult, execute_brp_method};
use crate::error::BrpMcpError;
use crate::tools::{
    BRP_METHOD_DESTROY, BRP_METHOD_EXTRAS_DISCOVER_FORMAT, BRP_METHOD_INSERT,
    BRP_METHOD_INSERT_RESOURCE, BRP_METHOD_MUTATE_COMPONENT, BRP_METHOD_MUTATE_RESOURCE,
    BRP_METHOD_SPAWN,
};

/// Location of type items in method parameters
#[derive(Debug, Clone, Copy)]
pub enum ParameterLocation {
    /// Type items are in a "components" object (spawn, insert)
    Components,
    /// Single type value in "value" field (`mutate_component`)
    ComponentValue,
    /// Single type value in "value" field (`insert_resource`, `mutate_resource`)
    ResourceValue,
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
        tiered_type_format_discovery(type_name, type_value, method, original_error, port).await;

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

/// Try direct discovery using `bevy_brp_extras/discover_format`
async fn try_direct_discovery(
    type_name: &str,
    port: Option<u16>,
    tier_manager: &mut TierManager,
) -> Option<(Value, String)> {
    tier_manager.start_tier(
        TIER_DIRECT_DISCOVERY,
        "Direct Discovery",
        format!("Calling brp_extras/discover_format for type: {type_name}"),
    );

    let params = serde_json::json!({
        "types": [type_name]
    });

    if let Ok(BrpResult::Success(Some(data))) =
        execute_brp_method(BRP_METHOD_EXTRAS_DISCOVER_FORMAT, Some(params), port).await
    {
        if let Some(formats) = data.get("formats").and_then(|f| f.as_object()) {
            if let Some(format_info) = formats.get(type_name) {
                // Extract spawn_format and convert to corrected value
                if let Some(spawn_format) = format_info
                    .get("spawn_format")
                    .and_then(|sf| sf.get("example"))
                {
                    tier_manager.complete_tier(
                        true,
                        format!("Direct discovery successful: found format for {type_name}"),
                    );
                    let hint = "Direct discovery from bevy_brp_extras".to_string();
                    return Some((spawn_format.clone(), hint));
                }
            }
        }
    }
    tier_manager.complete_tier(false, "Direct discovery unavailable or failed".to_string());
    None
}

/// Tiered format discovery dispatcher - replaces `try_component_format_alternatives`
/// Uses intelligent pattern matching with fallback to generic approaches
async fn tiered_type_format_discovery(
    type_name: &str,
    original_value: &Value,
    method: &str,
    error: &BrpError,
    port: Option<u16>,
) -> (Option<(Value, String)>, Vec<TierInfo>) {
    let mut tier_manager = TierManager::new();

    // ========== TIER 1: Serialization Diagnostics ==========
    // For ANY error on spawn/insert operations, queries BRP to check if types
    // support required reflection traits (Serialize/Deserialize)
    // Only check for spawn/insert operations as mutations don't require Serialize/Deserialize
    let error_analysis = analyze_error_pattern(error);
    if method == BRP_METHOD_INSERT || method == BRP_METHOD_SPAWN
    // COMMENTED OUT: Old pattern-specific logic - testing if ALL spawn/insert errors should check
    // traits && matches!(
    //     &error_analysis.pattern,
    //     Some(ErrorPattern::UnknownComponentType { .. } | ErrorPattern::UnknownComponent { .. })
    // )
    {
        tier_manager.start_tier(
            TIER_SERIALIZATION,
            "Serialization Diagnostics",
            format!("Checking serialization support for type: {type_name}"),
        );

        // Use the actual type_name from the request context instead of the extracted error type
        // This fixes the issue where we'd get "`bevy_reflect::DynamicEnum`" instead of the actual
        // component
        match check_type_serialization(type_name, port).await {
            Ok(serialization_check) => {
                tier_manager.complete_tier(true, serialization_check.diagnostic_message.clone());

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
                        tier_manager.into_vec(),
                    );
                }

                // Otherwise, return as before
                return (None, tier_manager.into_vec());
            }
            Err(e) => {
                tier_manager.complete_tier(
                    false,
                    format!("Failed to query serialization info for {type_name}: {e}"),
                );
            }
        }
    }

    // ========== TIER 2: Direct Discovery ==========
    // Calls bevy_brp_extras/discover_format to get correct format directly from the Bevy app
    if let Some(result) = try_direct_discovery(type_name, port, &mut tier_manager).await {
        return (Some(result), tier_manager.into_vec());
    }

    // ========== TIER 3: Deterministic Pattern Matching ==========
    // Uses error message patterns to determine exact format mismatches
    // and applies targeted fixes with high confidence
    if let Some(pattern) = &error_analysis.pattern {
        tier_manager.start_tier(
            TIER_DETERMINISTIC,
            "Deterministic Pattern Matching",
            format!("Matched pattern: {pattern:?}"),
        );

        if let Some((corrected_value, hint)) = apply_pattern_fix(pattern, type_name, original_value)
        {
            tier_manager.complete_tier(true, format!("Applied pattern fix: {hint}"));
            return (Some((corrected_value, hint)), tier_manager.into_vec());
        }
    }

    // ========== TIER 4: Generic Fallback ==========
    // Falls back to legacy transformation logic trying various
    // format conversions (object->array, array->string, etc.)
    tier_manager.start_tier(
        TIER_GENERIC_FALLBACK,
        "Generic Fallback",
        "Trying generic format alternatives".to_string(),
    );

    let fallback_result =
        try_component_format_alternatives_legacy(type_name, original_value, error);
    if fallback_result.is_some() {
        tier_manager.complete_tier(true, "Found generic format alternative".to_string());
    } else {
        tier_manager.complete_tier(false, "No generic alternative found".to_string());
    }

    (fallback_result, tier_manager.into_vec())
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
        _ => Err(BrpMcpError::failed_to("test component format", "validation failed").into()),
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
