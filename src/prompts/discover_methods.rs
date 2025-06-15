use rmcp::Error as McpError;
use crate::types::Prompt;
use crate::prompts::{PromptHandler, support::text_response};
use crate::types::PromptResponse;

pub struct DiscoverMethodsPrompt;

impl PromptHandler for DiscoverMethodsPrompt {
    fn get_prompt_info(&self) -> Prompt {
        Prompt {
            name: "discover-brp-methods".to_string(),
            description: "List all available Bevy Remote Protocol methods and their capabilities".to_string(),
            arguments: None,
        }
    }
    
    fn execute(&self) -> Result<PromptResponse, McpError> {
        Ok(text_response(
            "To discover all available BRP methods, use the brp_execute tool with method=\"rpc.discover\". \
             This will return a comprehensive list of all available Bevy Remote Protocol methods and their capabilities."
        ))
    }
}