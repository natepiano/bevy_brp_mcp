use rmcp::Error as McpError;
use rmcp::model::CallToolResult;
use serde_json::Value;

use super::utils::parse_brp_response;

/// Metadata about a BRP request for response formatting
#[derive(Debug, Clone)]
pub struct BrpMetadata {
    pub method: String,
    pub port:   u16,
}

impl BrpMetadata {
    pub fn new(method: &str, port: u16) -> Self {
        Self {
            method: method.to_string(),
            port,
        }
    }
}

/// Structured error information for BRP responses
#[derive(Debug, Clone)]
pub struct BrpError {
    pub code:    Option<i64>,
    pub message: String,
    pub data:    Option<Value>,
}

impl BrpError {
    pub fn new(message: String) -> Self {
        Self {
            code: None,
            message,
            data: None,
        }
    }
}

/// Trait for formatting BRP responses in method-specific ways
pub trait BrpResponseFormatter {
    /// Format a successful BRP response
    fn format_success(&self, data: Value, metadata: BrpMetadata) -> CallToolResult;

    /// Format an error BRP response
    fn format_error(&self, error: BrpError, metadata: BrpMetadata) -> CallToolResult;
}


/// Generic function to process BRP responses using a formatter
pub fn process_brp_response<F: BrpResponseFormatter>(
    brp_result: CallToolResult,
    formatter: F,
    metadata: BrpMetadata,
) -> Result<CallToolResult, McpError> {
    // Extract and format the response
    if let Some(content) = brp_result.content.first() {
        if let Some(text) = content.as_text() {
            // Parse the response from brp_execute
            let response = parse_brp_response(&text.text)?;

            // Extract the data from the response
            let data = response.get("data").ok_or_else(|| {
                McpError::internal_error("Invalid response format from BRP method", None)
            })?;

            // Use the formatter to format the success response
            Ok(formatter.format_success(data.clone(), metadata))
        } else {
            let error = BrpError::new("No text content in BRP response".to_string());
            Ok(formatter.format_error(error, metadata))
        }
    } else {
        let error = BrpError::new("No content in BRP response".to_string());
        Ok(formatter.format_error(error, metadata))
    }
}
