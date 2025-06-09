use std::error::Error;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use rmcp::model::{
    CallToolRequestParam, CallToolResult, ListToolsResult, PaginatedRequestParam,
    ServerCapabilities,
};
use rmcp::service::RequestContext;
use rmcp::transport::stdio;
use rmcp::{Error as McpError, RoleServer, ServerHandler, ServiceExt};

mod cargo_detector;
mod constants;
mod tools;


#[derive(Clone)]
pub struct BrpMcpService {
    pub roots: Arc<Mutex<Vec<PathBuf>>>,
}

impl BrpMcpService {
    fn new() -> Self {
        Self {
            roots: Arc::new(Mutex::new(Vec::new())),
        }
    }
}

impl ServerHandler for BrpMcpService {
    fn get_info(&self) -> rmcp::model::ServerInfo {
        rmcp::model::ServerInfo {
            instructions: None,
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
        Ok(tools::register_tools().await)
    }

    async fn call_tool(
        &self,
        request: CallToolRequestParam,
        context: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, McpError> {
        tools::handle_tool_call(self, request, context).await
    }
}

impl BrpMcpService {
    pub async fn fetch_roots_from_client(
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
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let service = BrpMcpService::new();
    let server = service.serve(stdio()).await?;
    server.waiting().await?;
    Ok(())
}
