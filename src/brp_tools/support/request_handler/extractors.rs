use rmcp::Error as McpError;
use serde_json::json;

use super::traits::{ExtractedParams, ParamExtractor};
use crate::brp_tools::constants::{DEFAULT_BRP_PORT, JSON_FIELD_ENTITY, JSON_FIELD_PORT};
use crate::support::params::{extract_any_value, extract_optional_number, extract_required_number};
use crate::types::BrpExecuteParams;

/// Helper function to extract and validate port from request
fn extract_port(request: &rmcp::model::CallToolRequestParam) -> Result<u16, McpError> {
    u16::try_from(extract_optional_number(
        request,
        JSON_FIELD_PORT,
        u64::from(DEFAULT_BRP_PORT),
    )?)
    .map_err(|_| {
        McpError::invalid_params("Port number must be a valid u16".to_string(), None)
    })
}

/// Simple parameter extractor that just extracts port
pub struct SimplePortExtractor;

impl ParamExtractor for SimplePortExtractor {
    fn extract(
        &self,
        request: &rmcp::model::CallToolRequestParam,
    ) -> Result<ExtractedParams, McpError> {
        let port = extract_port(request)?;
        Ok(ExtractedParams {
            method: None,
            params: None,
            port,
        })
    }
}

/// Parameter extractor that extracts port and passes through all arguments as params
pub struct PassthroughExtractor;

impl ParamExtractor for PassthroughExtractor {
    fn extract(
        &self,
        request: &rmcp::model::CallToolRequestParam,
    ) -> Result<ExtractedParams, McpError> {
        let port = extract_port(request)?;
        let params = request.arguments.clone().map(serde_json::Value::Object);
        Ok(ExtractedParams {
            method: None,
            params,
            port,
        })
    }
}

/// Parameter extractor for entity-based operations (destroy, list)
pub struct EntityParamExtractor {
    /// Whether entity is required
    pub required: bool,
}

impl ParamExtractor for EntityParamExtractor {
    fn extract(
        &self,
        request: &rmcp::model::CallToolRequestParam,
    ) -> Result<ExtractedParams, McpError> {
        let port = extract_port(request)?;

        let params = if self.required {
            let entity = extract_required_number(request, JSON_FIELD_ENTITY)?;
            Some(json!({ JSON_FIELD_ENTITY: entity }))
        } else {
            // For optional entity (like in list)
            extract_any_value(request, JSON_FIELD_ENTITY)
                .and_then(serde_json::Value::as_u64)
                .map(|id| json!({ JSON_FIELD_ENTITY: id }))
        };

        Ok(ExtractedParams {
            method: None,
            params,
            port,
        })
    }
}

/// Parameter extractor for `brp_execute` operations
pub struct BrpExecuteExtractor;

impl ParamExtractor for BrpExecuteExtractor {
    fn extract(
        &self,
        request: &rmcp::model::CallToolRequestParam,
    ) -> Result<ExtractedParams, McpError> {
        let params: BrpExecuteParams = serde_json::from_value(serde_json::Value::Object(
            request.arguments.clone().unwrap_or_default(),
        ))
        .map_err(|e| {
            McpError::from(rmcp::model::ErrorData::invalid_params(
                format!("Invalid parameters: {e}"),
                None,
            ))
        })?;

        Ok(ExtractedParams {
            method: Some(params.method),
            params: params.params,
            port: params.port,
        })
    }
}
