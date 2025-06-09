use std::path::PathBuf;

use crate::cargo_detector::{BinaryInfo, CargoDetector};
use crate::constants::{PROFILE_DEBUG, PROFILE_RELEASE};

pub fn add_app_to_table(output: &mut String, app: &BinaryInfo, profiles: &[&str]) {
    output.push_str(&format!("{}\n", app.name));
    for profile in profiles {
        let target_dir = app.workspace_root.join("target").join(profile);
        let binary_path = target_dir.join(&app.name);
        let exists = binary_path.exists();
        
        output.push_str(&format!(
            "  {} - {} {}\n",
            profile,
            binary_path.display(),
            if exists { "[built]" } else { "[not built]" }
        ));
    }
    output.push('\n');
}

pub fn list_apps_for_paths(search_paths: &[PathBuf]) -> String {
    let mut output = String::new();
    output.push_str("Bevy Apps\n");
    output.push_str("---------\n\n");

    // Common profiles to check
    let profiles = vec![PROFILE_DEBUG, PROFILE_RELEASE];

    // Check each search path and its immediate subdirectories
    for root in search_paths {
        // Check the root itself
        if root.join("Cargo.toml").exists() {
            if let Ok(detector) = CargoDetector::from_path(root) {
                let apps = detector.find_bevy_apps();
                for app in apps {
                    add_app_to_table(&mut output, &app, &profiles);
                }
            }
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
                    
                    if let Ok(detector) = CargoDetector::from_path(&path) {
                        let apps = detector.find_bevy_apps();
                        for app in apps {
                            add_app_to_table(&mut output, &app, &profiles);
                        }
                    }
                }
            }
        }
    }
    
    output.push('\n');
    output
}

pub fn list_examples_for_paths(search_paths: &[PathBuf]) -> String {
    let mut output = String::new();
    output.push_str("Bevy Examples\n");
    output.push_str("-------------\n\n");

    let mut all_examples = Vec::new();

    // Check each search path and its immediate subdirectories
    for root in search_paths {
        // Check the root itself
        if root.join("Cargo.toml").exists() {
            if let Ok(detector) = CargoDetector::from_path(root) {
                let examples = detector.find_bevy_examples();
                for example in examples {
                    all_examples.push(format!("{} ({})", example.name, example.package_name));
                }
            }
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
                    
                    if let Ok(detector) = CargoDetector::from_path(&path) {
                        let examples = detector.find_bevy_examples();
                        for example in examples {
                            all_examples.push(format!("{} ({})", example.name, example.package_name));
                        }
                    }
                }
            }
        }
    }

    if all_examples.is_empty() {
        output.push_str("No Bevy examples found.");
    } else {
        for example in all_examples {
            output.push_str(&format!("- {}\n", example));
        }
    }
    
    output.push('\n');
    output
}