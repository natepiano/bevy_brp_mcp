use rmcp::model::ListToolsResult;

use super::{list_apps, list_examples, launch};

pub async fn register_tools() -> ListToolsResult {
    let tools = vec![
        list_apps::register_tool(),
        list_examples::register_tool(),
        launch::register_tool(),
    ];

    ListToolsResult {
        next_cursor: None,
        tools,
    }
}