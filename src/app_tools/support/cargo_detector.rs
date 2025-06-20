//! Simple cargo detector based on `bevy_brp_tool`

use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use cargo_metadata::{Metadata, MetadataCommand, Package};

/// Information about a binary target
#[derive(Debug, Clone)]
pub struct BinaryInfo {
    /// Name of the binary
    pub name:           String,
    /// Workspace root
    pub workspace_root: PathBuf,
    /// Path to the package's Cargo.toml
    pub manifest_path:  PathBuf,
}

impl BinaryInfo {
    /// Get the path to the binary for a given profile
    pub fn get_binary_path(&self, profile: &str) -> PathBuf {
        self.workspace_root
            .join("target")
            .join(profile)
            .join(&self.name)
    }
}

/// Information about an example
#[derive(Debug, Clone)]
pub struct ExampleInfo {
    /// Name of the example
    pub name:          String,
    /// Package name
    pub package_name:  String,
    /// Path to the package's Cargo.toml
    pub manifest_path: PathBuf,
}

/// Detects binary targets in a project or workspace
pub struct CargoDetector {
    metadata: Metadata,
}

impl CargoDetector {
    /// Create a detector for a specific path
    pub fn from_path(path: impl AsRef<Path>) -> Result<Self> {
        let metadata = MetadataCommand::new()
            .current_dir(path.as_ref())
            .exec()
            .context("Failed to execute cargo metadata")?;

        Ok(Self { metadata })
    }

    /// Find all Bevy applications (binaries) in the workspace/project
    pub fn find_bevy_apps(&self) -> Vec<BinaryInfo> {
        let mut apps = Vec::new();

        for package in &self.metadata.packages {
            // Only process workspace members
            if !self.metadata.workspace_members.contains(&package.id) {
                continue;
            }

            // Check if this package depends on bevy
            if !Self::package_depends_on_bevy(package) {
                continue;
            }

            // Find all binary targets
            for target in &package.targets {
                if target.is_bin() {
                    apps.push(BinaryInfo {
                        name:           target.name.clone(),
                        workspace_root: self.metadata.workspace_root.clone().into(),
                        manifest_path:  package.manifest_path.clone().into(),
                    });
                }
            }
        }

        apps
    }

    /// Find all Bevy examples in the workspace/project
    pub fn find_bevy_examples(&self) -> Vec<ExampleInfo> {
        let mut examples = Vec::new();

        for package in &self.metadata.packages {
            // Only process workspace members
            if !self.metadata.workspace_members.contains(&package.id) {
                continue;
            }

            // Check if this package depends on bevy
            if !Self::package_depends_on_bevy(package) {
                continue;
            }

            // Find all example targets
            for target in &package.targets {
                if target.is_example() {
                    examples.push(ExampleInfo {
                        name:          target.name.clone(),
                        package_name:  package.name.to_string(),
                        manifest_path: package.manifest_path.clone().into(),
                    });
                }
            }
        }

        examples
    }

    /// Find all BRP-enabled Bevy applications (binaries) in the workspace/project
    pub fn find_brp_enabled_apps(&self) -> Vec<BinaryInfo> {
        let mut apps = Vec::new();

        for package in &self.metadata.packages {
            // Only process workspace members
            if !self.metadata.workspace_members.contains(&package.id) {
                continue;
            }

            // Explicitly ignore bevy_brp_mcp itself (it's the BRP tool, not a BRP-enabled app)
            if package.name.as_str() == "bevy_brp_mcp" {
                continue;
            }

            // Check if this package has BRP support
            if !Self::package_has_brp_support(package) {
                continue;
            }

            // Find all binary targets
            for target in &package.targets {
                if target.is_bin() {
                    apps.push(BinaryInfo {
                        name:           target.name.clone(),
                        workspace_root: self.metadata.workspace_root.clone().into(),
                        manifest_path:  package.manifest_path.clone().into(),
                    });
                }
            }
        }

        apps
    }

    fn package_depends_on_bevy(package: &Package) -> bool {
        // Check direct dependencies (including workspace dependencies)
        package.dependencies.iter().any(|dep| dep.name == "bevy")
    }

    /// Check if a package has BRP (Bevy Remote Protocol) support enabled
    fn package_has_brp_support(package: &Package) -> bool {
        // First check: Must have bevy dependency with bevy_remote feature available
        if !Self::package_has_bevy_remote_feature(package) {
            return false;
        }

        // Second check: Must actually use BRP plugins in source code
        Self::package_uses_brp_plugins(package)
    }

    /// Check if a package has `bevy_remote` feature available (either explicit or workspace
    /// inherited)
    fn package_has_bevy_remote_feature(package: &Package) -> bool {
        // Check if bevy dependency includes bevy_remote feature or uses workspace inheritance
        package.dependencies.iter().any(|dep| {
            if dep.name == "bevy" {
                // If it has explicit features, check for bevy_remote
                if dep.features.is_empty() {
                    // If no explicit features, assume workspace inheritance
                    // (we'll verify actual usage in the code scanning step)
                    true
                } else {
                    dep.features.iter().any(|feature| feature == "bevy_remote")
                }
            } else {
                false
            }
        })
    }

    /// Check if a package uses `RemotePlugin` or `BrpExtrasPlugin` in its source code
    fn package_uses_brp_plugins(package: &Package) -> bool {
        // Get the package directory
        let package_dir = package.manifest_path.parent().unwrap();

        // Check all .rs files in src/ directory
        let src_dir = package_dir.join("src");
        if !src_dir.exists() {
            return false;
        }

        Self::check_directory_for_brp_plugins(src_dir.as_std_path())
    }

    /// Recursively check directory for BRP plugin usage
    fn check_directory_for_brp_plugins(dir: &std::path::Path) -> bool {
        use std::fs;

        if let Ok(entries) = fs::read_dir(dir) {
            for entry in entries.flatten() {
                let path = entry.path();

                if path.is_dir() {
                    if Self::check_directory_for_brp_plugins(&path) {
                        return true;
                    }
                } else if path.extension().is_some_and(|ext| ext == "rs")
                    && Self::file_uses_brp_plugins(&path)
                {
                    return true;
                }
            }
        }

        false
    }

    /// Check if a specific file uses `RemotePlugin` or `BrpExtrasPlugin`
    fn file_uses_brp_plugins(file_path: &std::path::Path) -> bool {
        use std::fs;

        fs::read_to_string(file_path).is_ok_and(|content| {
            // Look for actual use statements that import BRP plugins
            // Use more specific patterns to avoid matching our own detection code
            let has_remote_plugin_import = content.contains("use bevy::remote::RemotePlugin")
                || (content.contains("use bevy::remote::{") && content.contains("RemotePlugin"));

            let has_brp_extras_plugin_import = content
                .contains("use bevy_brp_extras::BrpExtrasPlugin")
                || (content.contains("use bevy_brp_extras::{")
                    && content.contains("BrpExtrasPlugin"));

            has_remote_plugin_import || has_brp_extras_plugin_import
        })
    }
}
