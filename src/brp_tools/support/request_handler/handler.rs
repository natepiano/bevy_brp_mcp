use std::fs;
use std::time::{SystemTime, UNIX_EPOCH};

use rmcp::model::CallToolResult;
use rmcp::service::RequestContext;
use rmcp::{Error as McpError, RoleServer};
use serde_json::{Value, json};

use super::config::{BrpHandlerConfig, FormatterContext};
use super::format_discovery::{
    EnhancedBrpResult, FormatCorrection, execute_brp_method_with_format_discovery,
};
use super::traits::ExtractedParams;
use crate::BrpMcpService;
use crate::brp_tools::constants::{
    JSON_FIELD_DATA, JSON_FIELD_DEBUG_INFO, JSON_FIELD_FORMAT_CORRECTIONS,
    JSON_FIELD_ORIGINAL_ERROR,
};
use crate::brp_tools::support::brp_client::{BrpError, BrpResult};
use crate::brp_tools::support::response_formatter::{BrpMetadata, ResponseFormatter};
use crate::error::BrpMcpError;
use crate::support::debug_tools;
use crate::tools::MAX_RESPONSE_TOKENS;

const CHARS_PER_TOKEN: usize = 4;

/// Result of parameter extraction from a request
pub struct RequestParams {
    /// Extracted parameters from the configured extractor
    pub extracted: ExtractedParams,
}

/// Extract and validate all parameters from a BRP request
fn extract_request_params(
    request: &rmcp::model::CallToolRequestParam,
    config: &BrpHandlerConfig,
) -> Result<RequestParams, McpError> {
    // Extract parameters using the configured extractor
    let extracted = config.param_extractor.extract(request)?;

    Ok(RequestParams { extracted })
}

/// Resolve the actual BRP method name to call
fn resolve_brp_method(
    extracted: &ExtractedParams,
    config: &BrpHandlerConfig,
) -> Result<String, McpError> {
    extracted
        .method
        .as_deref()
        .or(config.method)
        .map(String::from)
        .ok_or_else(|| BrpMcpError::missing("BRP method specification").into())
}

/// Check if response exceeds token limit and save to file if needed
fn handle_large_response(
    response_data: &Value,
    method_name: &str,
) -> Result<Option<Value>, McpError> {
    let response_json = serde_json::to_string(response_data)
        .map_err(|e| -> McpError { BrpMcpError::failed_to("serialize response", e).into() })?;

    let estimated_tokens = response_json.len() / CHARS_PER_TOKEN;

    if estimated_tokens > MAX_RESPONSE_TOKENS {
        // Generate timestamp for unique filename
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|e| -> McpError { BrpMcpError::failed_to("get timestamp", e).into() })?
            .as_secs();

        let sanitized_method = method_name.replace('/', "_");
        let filename = format!("brp_response_{sanitized_method}_{timestamp}.json");
        let filepath = std::env::temp_dir().join(&filename);

        // Save response to file
        fs::write(&filepath, &response_json).map_err(|e| -> McpError {
            BrpMcpError::io_failed("write response", &filepath, e).into()
        })?;

        // Return fallback response with file information
        let fallback_response = json!({
            "status": "success",
            "message": format!("Response too large ({} tokens). Saved to {}", estimated_tokens, filepath.display()),
            "filepath": filepath.to_string_lossy(),
            "instructions": "Use Read tool to examine, Grep to search, or jq commands to filter the data."
        });

        Ok(Some(fallback_response))
    } else {
        Ok(None)
    }
}

/// Add format corrections and debug info to response data
fn add_format_corrections(
    response_data: &mut Value,
    format_corrections: &[FormatCorrection],
    debug_info: &[String],
) {
    if format_corrections.is_empty() && debug_info.is_empty() {
        return;
    }

    let mut additions = json!({});

    if !format_corrections.is_empty() {
        let corrections_value = json!(
            format_corrections
                .iter()
                .map(|correction| {
                    json!({
                        "component": correction.component,
                        "original_format": correction.original_format,
                        "corrected_format": correction.corrected_format,
                        "hint": correction.hint
                    })
                })
                .collect::<Vec<_>>()
        );
        additions[JSON_FIELD_FORMAT_CORRECTIONS] = corrections_value;
    }

    if !debug_info.is_empty() && debug_tools::is_debug_enabled() {
        additions[JSON_FIELD_DEBUG_INFO] = json!(debug_info);
    }

    // If response_data is an object, add fields
    if let Value::Object(map) = response_data {
        if let Value::Object(add_map) = additions {
            map.extend(add_map);
        }
    } else {
        // If not an object, wrap it
        let mut wrapped = json!({
            JSON_FIELD_DATA: response_data.clone(),
            JSON_FIELD_FORMAT_CORRECTIONS: additions.get(JSON_FIELD_FORMAT_CORRECTIONS).cloned().unwrap_or(json!([]))
        });

        // Only add debug_info if debug mode is enabled
        if debug_tools::is_debug_enabled() {
            wrapped[JSON_FIELD_DEBUG_INFO] = additions
                .get(JSON_FIELD_DEBUG_INFO)
                .cloned()
                .unwrap_or(json!([]));
        }

        *response_data = wrapped;
    }
}

/// Context for processing responses
struct ResponseContext<'a> {
    formatter: &'a ResponseFormatter,
    metadata:  BrpMetadata,
}

/// Process a successful BRP response
fn process_success_response(
    data: Option<Value>,
    enhanced_result: &EnhancedBrpResult,
    method_name: &str,
    context: ResponseContext<'_>,
) -> Result<CallToolResult, McpError> {
    let mut response_data = data.unwrap_or(Value::Null);

    // Add format corrections and debug info if present
    add_format_corrections(
        &mut response_data,
        &enhanced_result.format_corrections,
        &enhanced_result.debug_info,
    );

    // Check if response is too large and use file fallback if needed
    let final_data = handle_large_response(&response_data, method_name)?
        .map_or(response_data, |fallback_response| fallback_response);

    Ok(context
        .formatter
        .format_success(&final_data, context.metadata))
}

/// Process an error BRP response
fn process_error_response(
    mut error_info: BrpError,
    enhanced_result: &EnhancedBrpResult,
    formatter: &ResponseFormatter,
    metadata: &BrpMetadata,
) -> CallToolResult {
    let original_error_message = error_info.message.clone();

    // Check if we have an enhanced diagnostic message from format discovery
    let enhanced_message = enhanced_result
        .format_corrections
        .iter()
        .find(|correction| correction.hint.contains("cannot be used with BRP"))
        .map(|correction| correction.hint.clone());

    // Use enhanced message if available, otherwise keep original
    let has_enhanced = enhanced_message.is_some();
    if let Some(enhanced_msg) = enhanced_message {
        error_info.message = enhanced_msg;
    }

    // Add debug info and format corrections to error data if present
    if !enhanced_result.debug_info.is_empty()
        || !enhanced_result.format_corrections.is_empty()
        || has_enhanced
    {
        let mut data_obj = error_info.data.unwrap_or_else(|| json!({}));

        if let Value::Object(map) = &mut data_obj {
            // Store original error message if we replaced it with enhanced message
            if has_enhanced {
                map.insert(
                    JSON_FIELD_ORIGINAL_ERROR.to_string(),
                    json!(original_error_message),
                );
            }

            // Add debug info only if debug mode is enabled
            if !enhanced_result.debug_info.is_empty() && debug_tools::is_debug_enabled() {
                map.insert(
                    JSON_FIELD_DEBUG_INFO.to_string(),
                    json!(enhanced_result.debug_info),
                );
            }

            // Add format corrections
            if !enhanced_result.format_corrections.is_empty() {
                let corrections = enhanced_result
                    .format_corrections
                    .iter()
                    .map(|c| {
                        json!({
                            "component": c.component,
                            "hint": c.hint,
                            "original_format": c.original_format,
                            "corrected_format": c.corrected_format
                        })
                    })
                    .collect::<Vec<_>>();
                map.insert(
                    JSON_FIELD_FORMAT_CORRECTIONS.to_string(),
                    json!(corrections),
                );
            }
        }

        error_info.data = Some(data_obj);
    }

    formatter.format_error(error_info, metadata)
}

/// Unified handler for all BRP methods (both static and dynamic)
pub async fn handle_brp_request(
    _service: &BrpMcpService,
    request: rmcp::model::CallToolRequestParam,
    _context: RequestContext<RoleServer>,
    config: &BrpHandlerConfig,
) -> Result<CallToolResult, McpError> {
    // Extract all parameters from the request
    let params = extract_request_params(&request, config)?;
    let extracted = params.extracted;

    // Determine the actual method to call
    let method_name = resolve_brp_method(&extracted, config)?;

    // Call BRP using format discovery
    let enhanced_result = execute_brp_method_with_format_discovery(
        &method_name,
        extracted.params.clone(),
        Some(extracted.port),
    )
    .await?;

    // Create formatter and metadata
    let formatter_context = FormatterContext {
        params: extracted.params.clone(),
    };
    let formatter = config.formatter_factory.create(formatter_context);

    // Use "brp_execute" for dynamic methods for special error formatting
    let metadata_method = if extracted.method.is_some() {
        "brp_execute"
    } else {
        &method_name
    };
    let metadata = BrpMetadata::new(metadata_method, extracted.port);

    // Process response using ResponseFormatter, including format corrections if present
    match &enhanced_result.result {
        BrpResult::Success(data) => {
            let context = ResponseContext {
                formatter: &formatter,
                metadata,
            };
            process_success_response(data.clone(), &enhanced_result, &method_name, context)
        }
        BrpResult::Error(error_info) => Ok(process_error_response(
            error_info.clone(),
            &enhanced_result,
            &formatter,
            &metadata,
        )),
    }
}
