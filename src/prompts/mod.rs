use rmcp::model::{GetPromptResult, ListPromptsResult, PaginatedRequestParam};
use rmcp::service::RequestContext;
use rmcp::{Error as McpError, RoleServer};

use crate::types::{Prompt, PromptResponse};

mod discover_methods;
mod support;

pub use discover_methods::DiscoverMethodsPrompt;

/// Registry for all available prompts
pub struct PromptRegistry {
    prompts: Vec<Box<dyn PromptHandler>>,
}

impl PromptRegistry {
    pub fn new() -> Self {
        let mut registry = Self {
            prompts: Vec::new(),
        };

        // Register all prompts
        registry.register(Box::new(DiscoverMethodsPrompt));

        registry
    }

    fn register(&mut self, prompt: Box<dyn PromptHandler>) {
        self.prompts.push(prompt);
    }

    /// List all available prompts
    #[allow(clippy::unnecessary_wraps)]
    pub fn list_prompts(
        &self,
        _request: PaginatedRequestParam,
        _context: RequestContext<RoleServer>,
    ) -> Result<ListPromptsResult, McpError> {
        Ok(ListPromptsResult {
            prompts:     self
                .prompts
                .iter()
                .map(|handler| handler.get_prompt_info())
                .map(|p| rmcp::model::Prompt {
                    name:        p.name,
                    description: Some(p.description),
                    arguments:   p.arguments.map(|args| {
                        args.into_iter()
                            .map(|arg| rmcp::model::PromptArgument {
                                name:        arg.name,
                                description: Some(arg.description),
                                required:    arg.required,
                            })
                            .collect()
                    }),
                })
                .collect(),
            next_cursor: None,
        })
    }

    /// Get a specific prompt by name
    pub fn get_prompt(
        &self,
        name: &str,
        _context: RequestContext<RoleServer>,
    ) -> Result<GetPromptResult, McpError> {
        let handler = self
            .prompts
            .iter()
            .find(|h| h.get_prompt_info().name == name)
            .ok_or_else(|| {
                McpError::from(rmcp::model::ErrorData::invalid_request(
                    "Prompt not found",
                    None,
                ))
            })?;

        let response = handler.execute()?;

        Ok(GetPromptResult {
            description: Some(handler.get_prompt_info().description),
            messages:    response
                .messages
                .into_iter()
                .map(|msg| rmcp::model::PromptMessage {
                    role:    rmcp::model::PromptMessageRole::Assistant,
                    content: rmcp::model::PromptMessageContent::Text {
                        text: match msg.content {
                            crate::types::PromptContent::Text { text } => text,
                        },
                    },
                })
                .collect(),
        })
    }
}

/// Trait for prompt handlers
pub trait PromptHandler: Send + Sync {
    /// Get the prompt information
    fn get_prompt_info(&self) -> Prompt;

    /// Execute the prompt and return a response
    fn execute(&self) -> Result<PromptResponse, McpError>;
}
