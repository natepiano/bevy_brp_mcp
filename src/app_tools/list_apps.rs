use rmcp::model::{CallToolResult, Tool};
use rmcp::service::RequestContext;
use rmcp::{Error as McpError, RoleServer};
use serde_json::json;

use super::support::scanning;
use crate::BrpMcpService;
use crate::constants::{PROFILE_DEBUG, PROFILE_RELEASE};
use crate::support::{response, schema, service};
use crate::tools::{DESC_LIST_BEVY_APPS, TOOL_LIST_BEVY_APPS};

pub fn register_tool() -> Tool {
    Tool {
        name:         TOOL_LIST_BEVY_APPS.into(),
        description:  DESC_LIST_BEVY_APPS.into(),
        input_schema: schema::empty_object_schema(),
    }
}

pub async fn handle(
    service: &BrpMcpService,
    context: RequestContext<RoleServer>,
) -> Result<CallToolResult, McpError> {
    service::handle_with_paths(service, context, |search_paths| async move {
        let apps = collect_all_apps(&search_paths);

        Ok(response::success_json_response(
            format!("Found {} Bevy apps", apps.len()),
            json!({
                "apps": apps
            }),
        ))
    })
    .await
}

fn collect_all_apps(search_paths: &[std::path::PathBuf]) -> Vec<serde_json::Value> {
    let mut all_apps = Vec::new();
    let profiles = vec![PROFILE_DEBUG, PROFILE_RELEASE];

    // Use the iterator to find all cargo projects
    for path in scanning::iter_cargo_project_paths(search_paths) {
        if let Ok(detector) =
            crate::app_tools::support::cargo_detector::CargoDetector::from_path(&path)
        {
            let apps = detector.find_bevy_apps();
            for app in apps {
                let mut builds = json!({});
                for profile in &profiles {
                    let binary_path = app.get_binary_path(profile);
                    builds[profile] = json!({
                        "path": binary_path.display().to_string(),
                        "built": binary_path.exists()
                    });
                }

                all_apps.push(json!({
                    "name": app.name,
                    "workspace_root": app.workspace_root.display().to_string(),
                    "manifest_path": app.manifest_path.display().to_string(),
                    "builds": builds
                }));
            }
        }
    }

    all_apps
}
