use rmcp::model::{CallToolResult, Tool};
use rmcp::service::RequestContext;
use rmcp::{Error as McpError, RoleServer};
use serde_json::json;

use super::support::scanning;
use crate::BrpMcpService;
use crate::constants::{DESC_LIST_BEVY_EXAMPLES, TOOL_LIST_BEVY_EXAMPLES};
use crate::support::{response, schema, service};

pub fn register_tool() -> Tool {
    Tool {
        name:         TOOL_LIST_BEVY_EXAMPLES.into(),
        description:  DESC_LIST_BEVY_EXAMPLES.into(),
        input_schema: schema::empty_object_schema(),
    }
}

pub async fn handle(
    service: &BrpMcpService,
    context: RequestContext<RoleServer>,
) -> Result<CallToolResult, McpError> {
    service::handle_with_paths(service, context, |search_paths| async move {
        let examples = collect_all_examples(&search_paths);

        Ok(response::success_json_response(
            format!("Found {} Bevy examples", examples.len()),
            json!({
                "examples": examples
            }),
        ))
    })
    .await
}

fn collect_all_examples(search_paths: &[std::path::PathBuf]) -> Vec<serde_json::Value> {
    let mut all_examples = Vec::new();

    // Use the iterator to find all cargo projects
    for path in scanning::iter_cargo_project_paths(search_paths) {
        if let Ok(detector) = crate::cargo_detector::CargoDetector::from_path(&path) {
            let examples = detector.find_bevy_examples();
            for example in examples {
                all_examples.push(json!({
                    "name": example.name,
                    "package_name": example.package_name,
                    "manifest_path": example.manifest_path.display().to_string()
                }));
            }
        }
    }

    all_examples
}
