//! MCP prompt implementations

use rmcp::model::{
    GetPromptRequestParam, GetPromptResult, ListPromptsResult, PaginatedRequestParam,
};
use rmcp::service::RequestContext;
use rmcp::{Error as McpError, RoleServer};

/// Handle prompt listing
pub async fn list_prompts(
    _request: Option<PaginatedRequestParam>,
    _context: RequestContext<RoleServer>,
) -> Result<ListPromptsResult, McpError> {
    // These parameters are required by the MCP protocol but not needed for listing prompts
    // We don't define any prompts - the client handles prompting based on error messages
    Ok(ListPromptsResult {
        next_cursor: None,
        prompts:     vec![],
    })
}

/// Handle prompt retrieval
pub async fn get_prompt(
    GetPromptRequestParam { name, .. }: GetPromptRequestParam,
    _context: RequestContext<RoleServer>,
) -> Result<GetPromptResult, McpError> {
    // We don't define any prompts - the client handles prompting based on error messages
    Err(McpError::invalid_params(
        format!("Unknown prompt: {}", name),
        None,
    ))
}
