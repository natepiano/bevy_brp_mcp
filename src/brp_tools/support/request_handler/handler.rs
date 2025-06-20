use rmcp::model::CallToolResult;
use rmcp::service::RequestContext;
use rmcp::{Error as McpError, RoleServer};
use serde_json::{Value, json};

use super::config::{BrpHandlerConfig, FormatterContext};
use super::format_discovery::execute_brp_method_with_format_discovery;
use crate::BrpMcpService;
use crate::brp_tools::support::brp_client::BrpResult;
use crate::brp_tools::support::response_formatter::BrpMetadata;
use crate::support::debug_tools;

/// Unified handler for all BRP methods (both static and dynamic)
#[allow(clippy::too_many_lines)]
pub async fn handle_brp_request(
    _service: &BrpMcpService,
    request: rmcp::model::CallToolRequestParam,
    _context: RequestContext<RoleServer>,
    config: &BrpHandlerConfig,
) -> Result<CallToolResult, McpError> {
    // Extract parameters using the configured extractor
    let extracted = config.param_extractor.extract(&request)?;

    // Determine the actual method to call
    let method_name = extracted
        .method
        .as_deref()
        .or(config.method)
        .ok_or_else(|| {
            McpError::invalid_params("No method specified for BRP call".to_string(), None)
        })?;

    // Call BRP using format discovery
    let enhanced_result = execute_brp_method_with_format_discovery(
        method_name,
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
        method_name
    };
    let metadata = BrpMetadata::new(metadata_method, extracted.port);

    // Process response using ResponseFormatter, including format corrections if present
    match enhanced_result.result {
        BrpResult::Success(data) => {
            let mut response_data = data.unwrap_or(Value::Null);

            // Add format corrections and debug info if present
            if !enhanced_result.format_corrections.is_empty()
                || !enhanced_result.debug_info.is_empty()
            {
                let mut additions = json!({});

                if !enhanced_result.format_corrections.is_empty() {
                    let corrections_value = json!(
                        enhanced_result
                            .format_corrections
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
                    additions["format_corrections"] = corrections_value;
                }

                if !enhanced_result.debug_info.is_empty() && debug_tools::is_debug_enabled() {
                    additions["debug_info"] = json!(enhanced_result.debug_info);
                }

                // If response_data is an object, add fields
                if let Value::Object(ref mut map) = response_data {
                    if let Value::Object(add_map) = additions {
                        map.extend(add_map);
                    }
                } else {
                    // If not an object, wrap it
                    let mut wrapped = json!({
                        "data": response_data,
                        "format_corrections": additions.get("format_corrections").cloned().unwrap_or(json!([]))
                    });

                    // Only add debug_info if debug mode is enabled
                    if debug_tools::is_debug_enabled() {
                        wrapped["debug_info"] =
                            additions.get("debug_info").cloned().unwrap_or(json!([]));
                    }

                    response_data = wrapped;
                }
            }

            Ok(formatter.format_success(&response_data, metadata))
        }
        BrpResult::Error(mut error_info) => {
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

                if let Value::Object(ref mut map) = data_obj {
                    // Store original error message if we replaced it with enhanced message
                    if has_enhanced {
                        map.insert("original_error".to_string(), json!(original_error_message));
                    }

                    // Add debug info only if debug mode is enabled
                    if !enhanced_result.debug_info.is_empty() && debug_tools::is_debug_enabled() {
                        map.insert("debug_info".to_string(), json!(enhanced_result.debug_info));
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
                        map.insert("format_corrections".to_string(), json!(corrections));
                    }
                }

                error_info.data = Some(data_obj);
            }

            Ok(formatter.format_error(error_info, &metadata))
        }
    }
}
