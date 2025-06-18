use std::fmt::Write;

use rmcp::model::CallToolResult;
use rmcp::service::RequestContext;
use rmcp::{Error as McpError, RoleServer};
use serde_json::{Value, json};

use super::config::{BrpHandlerConfig, FormatterContext};
use super::format_discovery::execute_brp_method_with_format_discovery;
use crate::BrpMcpService;
use crate::brp_tools::support::brp_client::BrpResult;
use crate::brp_tools::support::response_formatter::BrpMetadata;

/// Unified handler for all BRP methods (both static and dynamic)
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

                if !enhanced_result.debug_info.is_empty() {
                    additions["debug_info"] = json!(enhanced_result.debug_info);
                }

                // If response_data is an object, add fields
                if let Value::Object(ref mut map) = response_data {
                    if let Value::Object(add_map) = additions {
                        map.extend(add_map);
                    }
                } else {
                    // If not an object, wrap it
                    response_data = json!({
                        "data": response_data,
                        "format_corrections": additions.get("format_corrections").cloned().unwrap_or(json!([])),
                        "debug_info": additions.get("debug_info").cloned().unwrap_or(json!([]))
                    });
                }
            }

            Ok(formatter.format_success(&response_data, metadata))
        }
        BrpResult::Error(mut error_info) => {
            // Add debug info to error message if present
            if !enhanced_result.debug_info.is_empty() {
                write!(
                    error_info.message,
                    "\n\nDEBUG INFO:\n{}",
                    enhanced_result.debug_info.join("\n")
                )
                .unwrap();
            }
            Ok(formatter.format_error(error_info, &metadata))
        }
    }
}
