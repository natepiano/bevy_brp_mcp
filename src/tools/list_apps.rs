use rmcp::model::{CallToolResult, Content, Tool};
use rmcp::service::RequestContext;
use rmcp::{Error as McpError, RoleServer};

use crate::BrpMcpService;
use crate::constants::LIST_BEVY_APPS_DESC;

use super::support;

pub fn register_tool() -> Tool {
    let mut schema = serde_json::Map::new();
    schema.insert("type".to_string(), "object".into());
    schema.insert("properties".to_string(), serde_json::Map::new().into());

    Tool {
        name: "list_bevy_apps".into(),
        description: LIST_BEVY_APPS_DESC.into(),
        input_schema: std::sync::Arc::new(schema),
    }
}

pub async fn handle(
    service: &BrpMcpService,
    context: RequestContext<RoleServer>,
) -> Result<CallToolResult, McpError> {
    // Fetch current roots from client
    eprintln!("Fetching current roots from client...");
    if let Err(e) = service.fetch_roots_from_client(context.peer.clone()).await {
        eprintln!("Failed to fetch roots: {}", e);
    }
    
    let search_paths = service.roots.lock().unwrap().clone();
    let output = support::list_apps_for_paths(&search_paths);
    
    Ok(CallToolResult::success(vec![Content::text(output)]))
}