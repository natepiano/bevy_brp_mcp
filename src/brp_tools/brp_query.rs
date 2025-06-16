use rmcp::model::{CallToolResult, Tool};
use rmcp::service::RequestContext;
use rmcp::{Error as McpError, RoleServer};
use serde_json::{Value, json};

use super::constants::{
    DEFAULT_BRP_PORT, DEFAULT_ENTITY_COUNT, ERROR_CODE_THRESHOLD, JSON_FIELD_CODE, JSON_FIELD_DATA,
    JSON_FIELD_DATA_LOWERCASE, JSON_FIELD_ENTITY_COUNT, JSON_FIELD_ERROR_CODE, JSON_FIELD_MESSAGE,
    JSON_FIELD_METADATA, JSON_FIELD_METHOD, JSON_FIELD_PORT, JSON_FIELD_QUERY_PARAMS,
    JSON_FIELD_STATUS, QUERY_FIELD_COMPONENTS, QUERY_FIELD_HAS, QUERY_FIELD_OPTION,
    QUERY_FIELD_WITH, QUERY_FIELD_WITHOUT, RESPONSE_STATUS_ERROR, RESPONSE_STATUS_SUCCESS,
};
use super::support::builder::BrpRequestBuilder;
use super::support::response_processor::{BrpMetadata, BrpResponseFormatter, process_brp_response};
use super::support::serialization::json_tool_result;
use super::support::utils::to_execute_request;
use crate::BrpMcpService;
use crate::constants::{BRP_METHOD_QUERY, DESC_BRP_QUERY, TOOL_BRP_QUERY};
use crate::support::schema;

/// Data specification for bevy/query
#[derive(Debug, Clone)]
struct QueryData {
    components: Option<Vec<String>>, // Components to fetch
    option:     Option<Vec<String>>, // Components to fetch if present
    has:        Option<Vec<String>>, // Components to check presence
}

/// Filter specification for bevy/query
#[derive(Debug, Clone)]
struct QueryFilter {
    with:    Option<Vec<String>>, // Components that must be present
    without: Option<Vec<String>>, // Components that must NOT be present
}

pub fn register_tool() -> Tool {
    Tool {
        name: TOOL_BRP_QUERY.into(),
        description: DESC_BRP_QUERY.into(),
        input_schema: schema::SchemaBuilder::new()
            .add_any_property(
                "data",
                "Object specifying what component data to retrieve. Properties: components (array), option (array), has (array)",
                false
            )
            .add_any_property(
                "filter",
                "Object specifying which entities to query. Properties: with (array), without (array)",
                false
            )
            .add_boolean_property(
                "strict",
                "If true, returns error on unknown component types (default: false)",
                false
            )
            .add_number_property("port", &format!("The BRP port (default: {})", DEFAULT_BRP_PORT), false)
            .build(),
    }
}

pub async fn handle(
    _service: &BrpMcpService,
    request: rmcp::model::CallToolRequestParam,
    context: RequestContext<RoleServer>,
) -> Result<CallToolResult, McpError> {
    // Extract parameters
    let args = request.arguments.as_ref();

    let port = args
        .and_then(|args| args.get("port"))
        .and_then(|v| v.as_u64())
        .map(|v| v as u16)
        .unwrap_or(DEFAULT_BRP_PORT);

    // Build BRP request using the builder
    let mut builder = BrpRequestBuilder::new(BRP_METHOD_QUERY).port(port);

    // Handle data parameter
    if let Some(data) = args.and_then(|args| args.get("data")) {
        builder = builder.data(data.clone());
    }

    // Handle filter parameter
    if let Some(filter) = args.and_then(|args| args.get("filter")) {
        builder = builder.filter(filter.clone());
    }

    // Handle strict parameter
    if let Some(strict) = args
        .and_then(|args| args.get("strict"))
        .and_then(|v| v.as_bool())
    {
        builder = builder.strict(strict);
    }

    let brp_params = builder.build();

    // Convert to request format expected by brp_execute
    let execute_request = to_execute_request(brp_params)?;

    // Call brp_execute
    let result = super::brp_execute::handle_brp_execute(execute_request, context).await?;

    // Extract and parse the actual query parameters
    let data = args.and_then(|a| a.get("data")).and_then(parse_query_data);
    let filter = args
        .and_then(|a| a.get("filter"))
        .and_then(parse_query_filter);
    let strict = args.and_then(|a| a.get("strict")).and_then(|v| v.as_bool());

    // Create formatter and metadata
    let formatter = QueryFormatter::new(data, filter, strict);
    let metadata = BrpMetadata::new(BRP_METHOD_QUERY, port);

    // Use the response processor to handle the result
    process_brp_response(result, formatter, metadata)
}

/// Formatter for bevy/query responses
struct QueryFormatter {
    data:   Option<QueryData>,
    filter: Option<QueryFilter>,
    strict: Option<bool>,
}

impl QueryFormatter {
    fn new(data: Option<QueryData>, filter: Option<QueryFilter>, strict: Option<bool>) -> Self {
        Self {
            data,
            filter,
            strict,
        }
    }
}

impl BrpResponseFormatter for QueryFormatter {
    fn format_success(&self, data: Value, _metadata: BrpMetadata) -> CallToolResult {
        // Check if the response contains an embedded error (happens with strict mode)
        if let Some(obj) = data.as_object() {
            if let Some(code) = obj.get(JSON_FIELD_CODE).and_then(|c| c.as_i64()) {
                if code < ERROR_CODE_THRESHOLD {
                    // This is an error response embedded in the data
                    let error_message = if let Some(error_data) = obj.get(JSON_FIELD_DATA_LOWERCASE)
                    {
                        format!(
                            "BRP query failed with error code {}: {:?}",
                            code, error_data
                        )
                    } else {
                        format!("BRP query failed with error code {}", code)
                    };

                    let formatted_error = json!({
                        JSON_FIELD_STATUS: RESPONSE_STATUS_ERROR,
                        JSON_FIELD_MESSAGE: error_message,
                        JSON_FIELD_ERROR_CODE: code,
                        JSON_FIELD_METADATA: {
                            JSON_FIELD_QUERY_PARAMS: {
                                "data": self.data.as_ref().map(|d| json!({
                                    "components": d.components,
                                    "option": d.option,
                                    "has": d.has
                                })),
                                "filter": self.filter.as_ref().map(|f| json!({
                                    "with": f.with,
                                    "without": f.without
                                })),
                                "strict": self.strict
                            }
                        }
                    });

                    return json_tool_result(&formatted_error);
                }
            }
        }

        // Count entities in results
        let entity_count = if let Some(arr) = data.as_array() {
            arr.len()
        } else {
            DEFAULT_ENTITY_COUNT
        };

        // Format the response
        let formatted_data = json!({
            JSON_FIELD_STATUS: RESPONSE_STATUS_SUCCESS,
            JSON_FIELD_MESSAGE: format!("Query returned {} entities", entity_count),
            JSON_FIELD_DATA: data,
            JSON_FIELD_METADATA: {
                JSON_FIELD_ENTITY_COUNT: entity_count,
                JSON_FIELD_QUERY_PARAMS: {
                    "data": self.data.as_ref().map(|d| json!({
                        "components": d.components,
                        "option": d.option,
                        "has": d.has
                    })),
                    "filter": self.filter.as_ref().map(|f| json!({
                        "with": f.with,
                        "without": f.without
                    })),
                    "strict": self.strict
                }
            }
        });

        json_tool_result(&formatted_data)
    }

    fn format_error(
        &self,
        error: super::support::response_processor::BrpError,
        metadata: BrpMetadata,
    ) -> CallToolResult {
        let formatted_error = json!({
            JSON_FIELD_STATUS: RESPONSE_STATUS_ERROR,
            JSON_FIELD_MESSAGE: error.message,
            JSON_FIELD_ERROR_CODE: error.code,
            JSON_FIELD_DATA: error.data,
            JSON_FIELD_METADATA: {
                JSON_FIELD_METHOD: metadata.method,
                JSON_FIELD_PORT: metadata.port,
                JSON_FIELD_QUERY_PARAMS: {
                    "data": self.data.as_ref().map(|d| json!({
                        "components": d.components,
                        "option": d.option,
                        "has": d.has
                    })),
                    "filter": self.filter.as_ref().map(|f| json!({
                        "with": f.with,
                        "without": f.without
                    })),
                    "strict": self.strict
                }
            }
        });

        json_tool_result(&formatted_error)
    }
}

/// Parse the data parameter for bevy/query
fn parse_query_data(data: &Value) -> Option<QueryData> {
    let obj = data.as_object()?;

    Some(QueryData {
        components: obj
            .get(QUERY_FIELD_COMPONENTS)
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str().map(String::from))
                    .collect()
            }),
        option:     obj
            .get(QUERY_FIELD_OPTION)
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str().map(String::from))
                    .collect()
            }),
        has:        obj
            .get(QUERY_FIELD_HAS)
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str().map(String::from))
                    .collect()
            }),
    })
}

/// Parse the filter parameter for bevy/query
fn parse_query_filter(filter: &Value) -> Option<QueryFilter> {
    let obj = filter.as_object()?;

    Some(QueryFilter {
        with:    obj
            .get(QUERY_FIELD_WITH)
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str().map(String::from))
                    .collect()
            }),
        without: obj
            .get(QUERY_FIELD_WITHOUT)
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str().map(String::from))
                    .collect()
            }),
    })
}
