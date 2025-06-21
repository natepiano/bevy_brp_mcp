use std::collections::HashMap;

use serde_json::Value;

use crate::tools::MAX_RESPONSE_TOKENS;

const CHARS_PER_TOKEN: usize = 3; // Simple heuristic: ~3 chars per token

#[derive(Debug, Clone)]
pub struct PaginatedResponse {
    pub data:        Value,
    pub has_more:    bool,
    pub page:        usize,
    pub total_pages: usize,
}

impl PaginatedResponse {
    pub const fn complete(data: Value) -> Self {
        Self {
            data,
            has_more: false,
            page: 0,
            total_pages: 1,
        }
    }

    pub const fn paginated(data: Value, page: usize, total_pages: usize) -> Self {
        Self {
            data,
            has_more: page < total_pages - 1,
            page,
            total_pages,
        }
    }
}

/// Simple pagination - just split data if it's too big
pub fn paginate_if_needed(data: Value, page: usize) -> Result<PaginatedResponse, String> {
    // Quick size check
    let json_str =
        serde_json::to_string(&data).map_err(|e| format!("JSON serialization failed: {e}"))?;
    let estimated_tokens = json_str.len() / CHARS_PER_TOKEN;

    // If small enough, check if user requested a valid page
    if estimated_tokens <= MAX_RESPONSE_TOKENS {
        if page == 0 {
            return Ok(PaginatedResponse::complete(data));
        }
        // User requested page 1+ but data only has 1 page
        return Err(format!("Page {page} not found. Total pages: 1"));
    }

    // Need to paginate - split the data
    match data {
        Value::Array(arr) => paginate_array(&arr, page),
        Value::Object(obj) => paginate_object(obj, page),
        _ => Ok(PaginatedResponse::complete(data)), // Can't paginate primitives
    }
}

fn paginate_array(arr: &[Value], page: usize) -> Result<PaginatedResponse, String> {
    if arr.is_empty() {
        return Ok(PaginatedResponse::complete(Value::Array(vec![])));
    }

    // Find optimal page size by binary search
    let mut page_size = 1;
    let mut max_size = arr.len();

    while page_size < max_size {
        let test_size = (page_size + max_size).div_ceil(2);
        let test_chunk: Vec<Value> = arr.iter().take(test_size).cloned().collect();
        let test_json = serde_json::to_string(&test_chunk).unwrap_or_default();

        if test_json.len() / CHARS_PER_TOKEN <= MAX_RESPONSE_TOKENS {
            page_size = test_size;
        } else {
            max_size = test_size - 1;
        }
    }

    // Ensure at least 1 item per page
    page_size = page_size.max(1);

    let total_pages = arr.len().div_ceil(page_size);

    if page >= total_pages {
        return Err(format!("Page {page} not found. Total pages: {total_pages}"));
    }

    let start = page * page_size;
    let end = (start + page_size).min(arr.len());
    let chunk: Vec<Value> = arr[start..end].to_vec();

    Ok(PaginatedResponse::paginated(
        Value::Array(chunk),
        page,
        total_pages,
    ))
}

fn paginate_object(
    obj: serde_json::Map<String, Value>,
    page: usize,
) -> Result<PaginatedResponse, String> {
    if obj.is_empty() {
        return Ok(PaginatedResponse::complete(Value::Object(
            serde_json::Map::new(),
        )));
    }

    // Convert to sorted vector for consistent pagination
    let mut entries: Vec<(String, Value)> = obj.into_iter().collect();
    entries.sort_by(|a, b| a.0.cmp(&b.0));

    // Find optimal page size
    let mut page_size = 1;
    let mut max_size = entries.len();

    while page_size < max_size {
        let test_size = (page_size + max_size).div_ceil(2);
        let test_map: HashMap<String, Value> = entries.iter().take(test_size).cloned().collect();
        let test_json = serde_json::to_string(&test_map).unwrap_or_default();

        if test_json.len() / CHARS_PER_TOKEN <= MAX_RESPONSE_TOKENS {
            page_size = test_size;
        } else {
            max_size = test_size - 1;
        }
    }

    page_size = page_size.max(1);

    let total_pages = entries.len().div_ceil(page_size);

    if page >= total_pages {
        return Err(format!("Page {page} not found. Total pages: {total_pages}"));
    }

    let start = page * page_size;
    let end = (start + page_size).min(entries.len());
    let chunk: serde_json::Map<String, Value> = entries[start..end].iter().cloned().collect();

    Ok(PaginatedResponse::paginated(
        Value::Object(chunk),
        page,
        total_pages,
    ))
}

/// Create pagination parameters for tool definitions - simplified to just a page parameter
pub fn create_pagination_params() -> Vec<crate::brp_tools::tool_definitions::ParamDef> {
    vec![crate::brp_tools::tool_definitions::ParamDef {
        name:        "page",
        description: "Page number for pagination (0-based, default: 0)",
        required:    false,
        param_type:  crate::brp_tools::tool_definitions::ParamType::Number,
    }]
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    #[test]
    fn test_small_response_not_paginated() {
        let data = json!({"small": "data"});
        let result = paginate_if_needed(data.clone(), 0).unwrap();

        assert_eq!(result.data, data);
        assert!(!result.has_more);
        assert_eq!(result.page, 0);
        assert_eq!(result.total_pages, 1);
    }

    #[test]
    fn test_array_pagination() {
        // Create a large array that will need pagination
        let large_string = "x".repeat(1000);
        let large_array: Vec<Value> = (0..100)
            .map(|i| {
                json!({
                    "id": i,
                    "data": large_string
                })
            })
            .collect();

        let result = paginate_if_needed(Value::Array(large_array), 0).unwrap();

        assert!(result.has_more);
        assert_eq!(result.page, 0);
        assert!(result.total_pages > 1);

        if let Value::Array(page_data) = result.data {
            assert!(!page_data.is_empty());
        } else {
            panic!("Expected array data");
        }
    }

    #[test]
    fn test_object_pagination() {
        let mut large_obj = serde_json::Map::new();
        let large_string = "x".repeat(1000);

        for i in 0..100 {
            large_obj.insert(
                format!("key_{i:03}"),
                json!({
                    "id": i,
                    "data": large_string
                }),
            );
        }

        let result = paginate_if_needed(Value::Object(large_obj), 0).unwrap();

        assert!(result.has_more);
        assert_eq!(result.page, 0);
        assert!(result.total_pages > 1);

        if let Value::Object(page_data) = result.data {
            assert!(!page_data.is_empty());
        } else {
            panic!("Expected object data");
        }
    }

    #[test]
    fn test_page_bounds() {
        let data = json!([1, 2, 3]);

        // Page 0 should work for small data
        let result = paginate_if_needed(data.clone(), 0);
        assert!(result.is_ok());

        // Page 999 should fail for small data (only 1 page exists)
        let result = paginate_if_needed(data, 999);
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .contains("Page 999 not found. Total pages: 1")
        );
    }

    #[test]
    fn test_empty_array() {
        let data = json!([]);
        let result = paginate_if_needed(data.clone(), 0).unwrap();

        assert_eq!(result.data, data);
        assert!(!result.has_more);
    }
}
