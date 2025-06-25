use std::path::PathBuf;

use rmcp::Error as McpError;

use super::cargo_detector::{BinaryInfo, CargoDetector, ExampleInfo};
use crate::error::BrpMcpError;

/// Iterator over all valid Cargo project paths found in the given search paths
/// Yields paths to directories containing Cargo.toml files
pub fn iter_cargo_project_paths(search_paths: &[PathBuf]) -> impl Iterator<Item = PathBuf> + '_ {
    search_paths.iter().flat_map(|root| {
        let mut paths = Vec::new();

        // Check the root itself
        if root.join("Cargo.toml").exists() {
            paths.push(root.clone());
        }

        // Check immediate subdirectories
        if let Ok(entries) = std::fs::read_dir(root) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() && path.join("Cargo.toml").exists() {
                    // Skip hidden directories and target
                    if let Some(name) = path.file_name() {
                        let name_str = name.to_string_lossy();
                        if name_str.starts_with('.') || name_str == "target" {
                            continue;
                        }
                    }
                    paths.push(path);
                }
            }
        }

        paths.into_iter()
    })
}

/// Find a specific app by name across search paths
pub fn find_app_by_name(
    app_name: &str,
    search_paths: &[PathBuf],
) -> Option<super::cargo_detector::BinaryInfo> {
    // Use the generic iterator to find all cargo projects
    for path in iter_cargo_project_paths(search_paths) {
        if let Ok(detector) = CargoDetector::from_path(&path) {
            let apps = detector.find_bevy_apps();
            if let Some(app) = apps.into_iter().find(|a| a.name == app_name) {
                return Some(app);
            }
        }
    }
    None
}

/// Find a specific example by name across search paths
pub fn find_example_by_name(
    example_name: &str,
    search_paths: &[PathBuf],
) -> Option<super::cargo_detector::ExampleInfo> {
    // Use the generic iterator to find all cargo projects
    for path in iter_cargo_project_paths(search_paths) {
        if let Ok(detector) = CargoDetector::from_path(&path) {
            let examples = detector.find_bevy_examples();
            if let Some(example) = examples.into_iter().find(|e| e.name == example_name) {
                return Some(example);
            }
        }
    }
    None
}

/// Find a required app by name, returning an error if not found
/// This eliminates the duplicated pattern of finding an app with error handling
pub fn find_required_app(app_name: &str, search_paths: &[PathBuf]) -> Result<BinaryInfo, McpError> {
    find_app_by_name(app_name, search_paths).ok_or_else(|| {
        BrpMcpError::missing(&format!("Bevy app '{app_name}' in search paths")).into()
    })
}

/// Find a required example by name, returning an error if not found
/// This eliminates the duplicated pattern of finding an example with error handling
pub fn find_required_example(
    example_name: &str,
    search_paths: &[PathBuf],
) -> Result<ExampleInfo, McpError> {
    find_example_by_name(example_name, search_paths).ok_or_else(|| {
        BrpMcpError::missing(&format!("Bevy example '{example_name}' in search paths")).into()
    })
}
