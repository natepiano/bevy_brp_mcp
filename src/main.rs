use std::error::Error;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use rmcp::model::{
    CallToolRequestParam, CallToolResult, ListToolsResult, PaginatedRequestParam,
    ServerCapabilities, Tool,
};
use rmcp::service::RequestContext;
use rmcp::transport::stdio;
use rmcp::{Error as McpError, RoleServer, ServerHandler, ServiceExt};

mod cargo_detector;

const SERVER_INSTRUCTIONS: &str = include_str!("../help_text_files/server_instructions.txt");

#[derive(Clone)]
struct BrpMcpService {
    roots: Arc<Mutex<Vec<PathBuf>>>,
}

impl BrpMcpService {
    fn new() -> Self {
        Self {
            roots: Arc::new(Mutex::new(Vec::new())),
        }
    }
    
    fn add_app_to_table(&self, output: &mut String, app: &crate::cargo_detector::BinaryInfo, profiles: &[&str]) {
        output.push_str(&format!("{}\n", app.name));
        for profile in profiles {
            let target_dir = app.workspace_root.join("target").join(profile);
            let binary_path = target_dir.join(&app.name);
            let exists = binary_path.exists();
            
            output.push_str(&format!(
                "  {} - {} {}\n",
                profile,
                binary_path.display(),
                if exists { "[built]" } else { "[not built]" }
            ));
        }
        output.push('\n');
    }
}

impl ServerHandler for BrpMcpService {
    fn get_info(&self) -> rmcp::model::ServerInfo {
        rmcp::model::ServerInfo {
            instructions: Some(SERVER_INSTRUCTIONS.to_string()),
            capabilities: ServerCapabilities::builder()
                .enable_tools()
                .build(),
            ..Default::default()
        }
    }

    async fn list_tools(
        &self,
        _request: PaginatedRequestParam,
        _context: RequestContext<RoleServer>,
    ) -> Result<ListToolsResult, McpError> {
        // The ServerHandler trait requires these parameters for all implementations,
        // but they're not needed for listing tools (no pagination or context required)
        let mut tools = vec![];

        // List bevy apps tool (no parameters)
        {
            let mut schema = serde_json::Map::new();
            schema.insert("type".to_string(), "object".into());
            schema.insert("properties".to_string(), serde_json::Map::new().into());

            tools.push(Tool {
                name: "list_bevy_apps".into(),
                description: "List all Bevy apps in client roots and their immediate subdirectories with profile information and build status.".into(),
                input_schema: std::sync::Arc::new(schema),
            });
        }

        // List bevy apps in path tool (required path parameter)
        {
            let mut schema = serde_json::Map::new();
            schema.insert("type".to_string(), "object".into());
            
            let mut properties = serde_json::Map::new();
            let mut path_schema = serde_json::Map::new();
            path_schema.insert("type".to_string(), "string".into());
            path_schema.insert("description".to_string(), "Path to search for Bevy apps".into());
            properties.insert("path".to_string(), path_schema.into());
            
            schema.insert("properties".to_string(), properties.into());
            schema.insert("required".to_string(), vec!["path"].into());

            tools.push(Tool {
                name: "list_bevy_apps_in_path".into(),
                description: "List all Bevy apps in a specific directory with profile information and build status.".into(),
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
                description: "List all Bevy examples found in client roots and their immediate subdirectories.".into(),
                input_schema: std::sync::Arc::new(schema),
            });
        }

        // List bevy examples in path tool (required path parameter)
        {
            let mut schema = serde_json::Map::new();
            schema.insert("type".to_string(), "object".into());
            
            let mut properties = serde_json::Map::new();
            let mut path_schema = serde_json::Map::new();
            path_schema.insert("type".to_string(), "string".into());
            path_schema.insert("description".to_string(), "Path to search for Bevy examples".into());
            properties.insert("path".to_string(), path_schema.into());
            
            schema.insert("properties".to_string(), properties.into());
            schema.insert("required".to_string(), vec!["path"].into());

            tools.push(Tool {
                name: "list_bevy_examples_in_path".into(),
                description: "List all Bevy examples found in a specific directory.".into(),
                input_schema: std::sync::Arc::new(schema),
            });
        }

        Ok(ListToolsResult {
            next_cursor: None,
            tools,
        })
    }

    async fn call_tool(
        &self,
        request: CallToolRequestParam,
        context: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, McpError> {
        match request.name.as_ref() {
            "list_bevy_apps" => {
                // No parameters - always use client roots
                eprintln!("Fetching current roots from client...");
                if let Err(e) = self.fetch_roots_from_client(context.peer.clone()).await {
                    eprintln!("Failed to fetch roots: {}", e);
                }
                let search_paths = self.roots.lock().unwrap().clone();
                let output = self.list_apps_for_paths(&search_paths);
                Ok(CallToolResult::success(vec![rmcp::model::Content::text(output)]))
            }
            "list_bevy_apps_in_path" => {
                // Required path parameter
                let path = request
                    .arguments
                    .as_ref()
                    .and_then(|args| args.get("path"))
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| McpError::invalid_params("Missing required parameter: path", None))?;
                
                let search_paths = vec![PathBuf::from(path)];
                let output = self.list_apps_for_paths(&search_paths);
                Ok(CallToolResult::success(vec![rmcp::model::Content::text(output)]))
            }
            "list_bevy_examples" => {
                // No parameters - always use client roots
                eprintln!("Fetching current roots from client...");
                if let Err(e) = self.fetch_roots_from_client(context.peer.clone()).await {
                    eprintln!("Failed to fetch roots: {}", e);
                }
                let search_paths = self.roots.lock().unwrap().clone();
                let output = self.list_examples_for_paths(&search_paths);
                Ok(CallToolResult::success(vec![rmcp::model::Content::text(output)]))
            }
            "list_bevy_examples_in_path" => {
                // Required path parameter
                let path = request
                    .arguments
                    .as_ref()
                    .and_then(|args| args.get("path"))
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| McpError::invalid_params("Missing required parameter: path", None))?;
                
                let search_paths = vec![PathBuf::from(path)];
                let output = self.list_examples_for_paths(&search_paths);
                Ok(CallToolResult::success(vec![rmcp::model::Content::text(output)]))
            }
            _ => Err(McpError::invalid_params(
                format!("Unknown tool: {}", request.name),
                None,
            )),
        }
    }
}

impl BrpMcpService {
    async fn fetch_roots_from_client(
        &self,
        peer: rmcp::service::Peer<RoleServer>,
    ) -> Result<(), Box<dyn Error>> {
        // Use the peer extension method to list roots
        match peer.list_roots().await {
            Ok(result) => {
                eprintln!("Received {} roots from client", result.roots.len());
                for (i, root) in result.roots.iter().enumerate() {
                    eprintln!(
                        "  Root {}: {} ({})",
                        i + 1,
                        root.uri,
                        root.name.as_deref().unwrap_or("unnamed")
                    );
                }

                let paths: Vec<PathBuf> = result
                    .roots
                    .iter()
                    .filter_map(|root| {
                        // Parse the file:// URI
                        if let Some(path) = root.uri.strip_prefix("file://") {
                            Some(PathBuf::from(path))
                        } else {
                            eprintln!("Warning: Ignoring non-file URI: {}", root.uri);
                            None
                        }
                    })
                    .collect();

                // Update our roots
                let mut roots = self.roots.lock().unwrap();
                *roots = paths;
                eprintln!("Processed roots: {:?}", *roots);
            }
            Err(e) => {
                eprintln!("Failed to send roots/list request: {}", e);
            }
        }

        Ok(())
    }

    fn list_apps_for_paths(&self, search_paths: &[PathBuf]) -> String {
        let mut output = String::new();
        output.push_str("Bevy Apps\n");
        output.push_str("---------\n\n");

        // Common profiles to check
        let profiles = vec!["debug", "release"];

        // Check each search path and its immediate subdirectories
        for root in search_paths {
            // Check the root itself
            if root.join("Cargo.toml").exists() {
                if let Ok(detector) = crate::cargo_detector::CargoDetector::from_path(root) {
                    let apps = detector.find_bevy_apps();
                    for app in apps {
                        self.add_app_to_table(&mut output, &app, &profiles);
                    }
                }
            }
            
            // Check immediate subdirectories
            if let Ok(entries) = std::fs::read_dir(root) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.is_dir() && path.join("Cargo.toml").exists() {
                        // Skip hidden directories and target
                        if let Some(name) = path.file_name() {
                            let name_str = name.to_string_lossy();
                            if name_str.starts_with('.') || name_str == "target" {
                                continue;
                            }
                        }
                        
                        if let Ok(detector) = crate::cargo_detector::CargoDetector::from_path(&path) {
                            let apps = detector.find_bevy_apps();
                            for app in apps {
                                self.add_app_to_table(&mut output, &app, &profiles);
                            }
                        }
                    }
                }
            }
        }
        
        output.push('\n');
        output
    }

    fn list_examples_for_paths(&self, search_paths: &[PathBuf]) -> String {
        let mut output = String::new();
        output.push_str("Bevy Examples\n");
        output.push_str("-------------\n\n");

        let mut all_examples = Vec::new();

        // Check each search path and its immediate subdirectories
        for root in search_paths {
            // Check the root itself
            if root.join("Cargo.toml").exists() {
                if let Ok(detector) = crate::cargo_detector::CargoDetector::from_path(root) {
                    let examples = detector.find_bevy_examples();
                    for example in examples {
                        all_examples.push(format!("{} ({})", example.name, example.package_name));
                    }
                }
            }
            
            // Check immediate subdirectories
            if let Ok(entries) = std::fs::read_dir(root) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.is_dir() && path.join("Cargo.toml").exists() {
                        // Skip hidden directories and target
                        if let Some(name) = path.file_name() {
                            let name_str = name.to_string_lossy();
                            if name_str.starts_with('.') || name_str == "target" {
                                continue;
                            }
                        }
                        
                        if let Ok(detector) = crate::cargo_detector::CargoDetector::from_path(&path) {
                            let examples = detector.find_bevy_examples();
                            for example in examples {
                                all_examples.push(format!("{} ({})", example.name, example.package_name));
                            }
                        }
                    }
                }
            }
        }

        if all_examples.is_empty() {
            output.push_str("No Bevy examples found.");
        } else {
            for example in all_examples {
                output.push_str(&format!("- {}\n", example));
            }
        }
        
        output.push('\n');
        output
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let service = BrpMcpService::new();
    let server = service.serve(stdio()).await?;
    server.waiting().await?;
    Ok(())
}
