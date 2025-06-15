use rmcp::model::ListToolsResult;

use super::{brp_execute, check_brp, cleanup_logs, launch_app, launch_example, list_apps, list_examples, list_logs, read_log};

pub async fn register_tools() -> ListToolsResult {
    let tools = vec![
        brp_execute::register_tool(),
        check_brp::register_tool(),
        list_apps::register_tool(),
        list_examples::register_tool(),
        launch_app::register_tool(),
        launch_example::register_tool(),
        list_logs::register_tool(),
        read_log::register_tool(),
        cleanup_logs::register_tool(),
    ];

    ListToolsResult {
        next_cursor: None,
        tools,
    }
}
