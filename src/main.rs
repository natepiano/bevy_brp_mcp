use rmcp::{
    model::{CallToolResult, Content, ServerCapabilities},
    transport::stdio,
    Error as McpError,
    ServerHandler,
    ServiceExt,
    tool,
};
use std::error::Error;

#[derive(Clone)]
struct HelloWorldService;

#[tool(tool_box)]
impl HelloWorldService {
    fn new() -> Self {
        Self
    }

    #[tool(description = "Says hello to the world")]
    fn hello(&self) -> Result<CallToolResult, McpError> {
        Ok(CallToolResult::success(vec![Content::text(
            "Hello, World!",
        )]))
    }

    #[tool(description = "Gets a greeting with a custom name")]
    fn greet(
        &self,
        #[tool(param)] name: String,
    ) -> Result<CallToolResult, McpError> {
        Ok(CallToolResult::success(vec![Content::text(format!(
            "Hello, {}!",
            name
        ))]))
    }
}

#[tool(tool_box)]
impl ServerHandler for HelloWorldService {
    fn get_info(&self) -> rmcp::model::ServerInfo {
        rmcp::model::ServerInfo {
            instructions: Some("A minimal hello world MCP server".to_string()),
            capabilities: ServerCapabilities::builder().enable_tools().build(),
            ..Default::default()
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let service = HelloWorldService::new();
    let server = service.serve(stdio()).await?;
    server.waiting().await?;
    Ok(())
}