use rmcp::model::{CallToolRequestParam, CallToolResult, ListToolsResult, Tool};
use rmcp::service::RequestContext;
use rmcp::{Error as McpError, RoleServer};

use crate::BrpMcpService;
use crate::constants::{
    PROFILE_DEBUG, PROFILE_RELEASE, DEFAULT_PROFILE,
    LIST_BEVY_APPS_DESC, LIST_BEVY_EXAMPLES_DESC, LAUNCH_BEVY_APP_DESC
};

mod support;
mod launch;

pub async fn register_tools() -> ListToolsResult {
    let mut tools = vec![];

    // List bevy apps tool (no parameters)
    {
        let mut schema = serde_json::Map::new();
        schema.insert("type".to_string(), "object".into());
        schema.insert("properties".to_string(), serde_json::Map::new().into());

        tools.push(Tool {
            name: "list_bevy_apps".into(),
            description: LIST_BEVY_APPS_DESC.into(),
            input_schema: std::sync::Arc::new(schema),
        });
    }
    
    // List bevy examples tool (no parameters)
    {
        let mut schema = serde_json::Map::new();
        schema.insert("type".to_string(), "object".into());
        schema.insert("properties".to_string(), serde_json::Map::new().into());

        tools.push(Tool {
            name: "list_bevy_examples".into(),
            description: LIST_BEVY_EXAMPLES_DESC.into(),
            input_schema: std::sync::Arc::new(schema),
        });
    }

    // Launch bevy app tool
    {
        let mut schema = serde_json::Map::new();
        schema.insert("type".to_string(), "object".into());
        
        let mut properties = serde_json::Map::new();
        
        let mut app_name_schema = serde_json::Map::new();
        app_name_schema.insert("type".to_string(), "string".into());
        app_name_schema.insert("description".to_string(), "Name of the Bevy app to launch".into());
        properties.insert("app_name".to_string(), app_name_schema.into());
        
        let mut profile_schema = serde_json::Map::new();
        profile_schema.insert("type".to_string(), "string".into());
        profile_schema.insert("enum".to_string(), vec![PROFILE_DEBUG, PROFILE_RELEASE].into());
        profile_schema.insert("default".to_string(), DEFAULT_PROFILE.into());
        profile_schema.insert("description".to_string(), "Build profile to use (debug or release)".into());
        properties.insert("profile".to_string(), profile_schema.into());
        
        schema.insert("properties".to_string(), properties.into());
        schema.insert("required".to_string(), vec!["app_name"].into());

        tools.push(Tool {
            name: "launch_bevy_app".into(),
            description: LAUNCH_BEVY_APP_DESC.into(),
            input_schema: std::sync::Arc::new(schema),
        });
    }

    ListToolsResult {
        next_cursor: None,
        tools,
    }
}

pub async fn handle_tool_call(
    service: &BrpMcpService,
    request: CallToolRequestParam,
    context: RequestContext<RoleServer>,
) -> Result<CallToolResult, McpError> {
    match request.name.as_ref() {
        "list_bevy_apps" => {
            // No parameters - always use client roots
            eprintln!("Fetching current roots from client...");
            if let Err(e) = service.fetch_roots_from_client(context.peer.clone()).await {
                eprintln!("Failed to fetch roots: {}", e);
            }
            let search_paths = service.roots.lock().unwrap().clone();
            let output = support::list_apps_for_paths(&search_paths);
            Ok(CallToolResult::success(vec![rmcp::model::Content::text(output)]))
        }
        "list_bevy_examples" => {
            // No parameters - always use client roots
            eprintln!("Fetching current roots from client...");
            if let Err(e) = service.fetch_roots_from_client(context.peer.clone()).await {
                eprintln!("Failed to fetch roots: {}", e);
            }
            let search_paths = service.roots.lock().unwrap().clone();
            let output = support::list_examples_for_paths(&search_paths);
            Ok(CallToolResult::success(vec![rmcp::model::Content::text(output)]))
        }
        "launch_bevy_app" => {
            // Get parameters
            let app_name = request
                .arguments
                .as_ref()
                .and_then(|args| args.get("app_name"))
                .and_then(|v| v.as_str())
                .ok_or_else(|| McpError::invalid_params("Missing required parameter: app_name", None))?;
            
            let profile = request
                .arguments
                .as_ref()
                .and_then(|args| args.get("profile"))
                .and_then(|v| v.as_str())
                .unwrap_or(DEFAULT_PROFILE);
            
            // Fetch current roots
            eprintln!("Fetching current roots from client...");
            if let Err(e) = service.fetch_roots_from_client(context.peer.clone()).await {
                eprintln!("Failed to fetch roots: {}", e);
            }
            let search_paths = service.roots.lock().unwrap().clone();
            
            // Launch the app
            launch::launch_bevy_app(app_name, profile, &search_paths).await
        }
        _ => Err(McpError::invalid_params(
            format!("Unknown tool: {}", request.name),
            None,
        )),
    }
}