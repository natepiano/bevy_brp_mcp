use std::error::Error;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use rmcp::model::{
    CallToolRequestParam, CallToolResult, GetPromptRequestParam, GetPromptResult,
    ListPromptsResult, ListToolsResult, PaginatedRequestParam, ServerCapabilities,
};
use rmcp::service::RequestContext;
use rmcp::transport::stdio;
use rmcp::{Error as McpError, RoleServer, ServerHandler, ServiceExt};

mod app_tools;
mod brp_tools;
mod constants;
mod log_tools;
mod prompts;
mod registry;
mod support;
mod types;
mod watch_manager;

use constants::BEVY_BRP_MCP_INFO;

#[derive(Clone)]
pub struct BrpMcpService {
    pub roots:       Arc<Mutex<Vec<PathBuf>>>,
    prompt_registry: Arc<prompts::PromptRegistry>,
}

impl BrpMcpService {
    fn new() -> Self {
        Self {
            roots:           Arc::new(Mutex::new(Vec::new())),
            prompt_registry: Arc::new(prompts::PromptRegistry::new()),
        }
    }
}

impl ServerHandler for BrpMcpService {
    fn get_info(&self) -> rmcp::model::ServerInfo {
        rmcp::model::ServerInfo {
            instructions: Some(BEVY_BRP_MCP_INFO.to_string()),
            capabilities: ServerCapabilities::builder()
                .enable_tools()
                .enable_prompts()
                .build(),
            ..Default::default()
        }
    }

    async fn list_tools(
        &self,
        _request: PaginatedRequestParam,
        _context: RequestContext<RoleServer>,
    ) -> Result<ListToolsResult, McpError> {
        Ok(registry::register_tools())
    }

    async fn call_tool(
        &self,
        request: CallToolRequestParam,
        context: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, McpError> {
        registry::handle_tool_call(self, request, context).await
    }

    async fn list_prompts(
        &self,
        request: PaginatedRequestParam,
        context: RequestContext<RoleServer>,
    ) -> Result<ListPromptsResult, McpError> {
        self.prompt_registry.list_prompts(request, context)
    }

    async fn get_prompt(
        &self,
        request: GetPromptRequestParam,
        context: RequestContext<RoleServer>,
    ) -> Result<GetPromptResult, McpError> {
        self.prompt_registry.get_prompt(&request.name, context)
    }
}

impl BrpMcpService {
    /// Fetches search roots from the connected MCP client.
    ///
    /// # Errors
    ///
    /// Returns an error if the MCP client cannot be contacted or if the `list_roots` call fails.
    ///
    /// # Panics
    ///
    /// Panics if the mutex lock on roots is poisoned.
    pub async fn fetch_roots_from_client(
        &self,
        peer: rmcp::service::Peer<RoleServer>,
    ) -> Result<(), Box<dyn Error>> {
        // Use the peer extension method to list roots
        match peer.list_roots().await {
            Ok(result) => {
                tracing::debug!("Received {} roots from client", result.roots.len());
                for (i, root) in result.roots.iter().enumerate() {
                    tracing::debug!(
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
                        root.uri.strip_prefix("file://").map_or_else(
                            || {
                                tracing::warn!("Ignoring non-file URI: {}", root.uri);
                                None
                            },
                            |path| Some(PathBuf::from(path)),
                        )
                    })
                    .collect();

                // Update our roots
                let mut roots = self.roots.lock().unwrap();
                *roots = paths;
                tracing::debug!("Processed roots: {:?}", *roots);
            }
            Err(e) => {
                tracing::error!("Failed to send roots/list request: {}", e);
            }
        }

        Ok(())
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Initialize logging to both stderr and a file
    use tracing_subscriber::layer::SubscriberExt;
    use tracing_subscriber::util::SubscriberInitExt;

    let log_file_name = "mcp_server_debug.log";

    // Create file appender
    let file_appender = tracing_appender::rolling::never("/tmp", log_file_name);
    let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);

    // Create layers
    let file_layer = tracing_subscriber::fmt::layer()
        .with_writer(non_blocking)
        .with_ansi(false);

    let stderr_layer = tracing_subscriber::fmt::layer().with_writer(std::io::stderr);

    // Use RUST_LOG if set, otherwise default to debug level for bevy_brp_mcp
    let env_filter = if std::env::var("RUST_LOG").is_ok() {
        tracing_subscriber::EnvFilter::from_default_env()
    } else {
        tracing_subscriber::EnvFilter::new("bevy_brp_mcp=debug,info")
    };

    // Combine layers
    tracing_subscriber::registry()
        .with(env_filter)
        .with(file_layer)
        .with(stderr_layer)
        .init();

    tracing::debug!("MCP Server starting with logging enabled");

    // Initialize the watch manager
    watch_manager::initialize_watch_manager().await;

    let service = BrpMcpService::new();

    tracing::info!("Starting stdio server");
    let server = service.serve(stdio()).await?;
    server.waiting().await?;

    Ok(())
}
