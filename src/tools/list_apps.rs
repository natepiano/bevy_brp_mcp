use rmcp::model::{CallToolResult, Tool};
use rmcp::service::RequestContext;
use rmcp::{Error as McpError, RoleServer};
use serde_json::json;

use crate::BrpMcpService;
use crate::constants::{LIST_BEVY_APPS_DESC, PROFILE_DEBUG, PROFILE_RELEASE};

use super::support;

pub fn register_tool() -> Tool {
    Tool {
        name: "list_bevy_apps".into(),
        description: LIST_BEVY_APPS_DESC.into(),
        input_schema: support::schema::empty_object_schema(),
    }
}

pub async fn handle(
    service: &BrpMcpService,
    context: RequestContext<RoleServer>,
) -> Result<CallToolResult, McpError> {
    support::service::handle_with_paths(service, context, |search_paths| async move {
        let apps = collect_all_apps(&search_paths);
        
        Ok(support::response::success_json_response(
            format!("Found {} Bevy apps", apps.len()),
            json!({
                "apps": apps
            })
        ))
    }).await
}

fn collect_all_apps(search_paths: &[std::path::PathBuf]) -> Vec<serde_json::Value> {
    let mut all_apps = Vec::new();
    let profiles = vec![PROFILE_DEBUG, PROFILE_RELEASE];
    
    // Use the iterator to find all cargo projects
    for path in support::scanning::iter_cargo_project_paths(search_paths) {
        if let Ok(detector) = crate::cargo_detector::CargoDetector::from_path(&path) {
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