//! Simple cargo detector based on bevy_brp_tool

use std::path::{Path, PathBuf};
use anyhow::{Context, Result};
use cargo_metadata::{Metadata, MetadataCommand, Package};

/// Information about a binary target
#[derive(Debug, Clone)]
pub struct BinaryInfo {
    /// Name of the binary
    pub name: String,
    /// Workspace root
    pub workspace_root: PathBuf,
    /// Path to the package's Cargo.toml
    pub manifest_path: PathBuf,
}

/// Information about an example
#[derive(Debug, Clone)]
pub struct ExampleInfo {
    /// Name of the example
    pub name: String,
    /// Package name
    pub package_name: String,
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
            if !self.package_depends_on_bevy(package) {
                continue;
            }
            
            // Find all binary targets
            for target in &package.targets {
                if target.is_bin() {
                    apps.push(BinaryInfo {
                        name: target.name.clone(),
                        workspace_root: self.metadata.workspace_root.clone().into(),
                        manifest_path: package.manifest_path.clone().into(),
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
            if !self.package_depends_on_bevy(package) {
                continue;
            }
            
            // Find all example targets
            for target in &package.targets {
                if target.is_example() {
                    examples.push(ExampleInfo {
                        name: target.name.clone(),
                        package_name: package.name.to_string(),
                    });
                }
            }
        }
        
        examples
    }
    
    fn package_depends_on_bevy(&self, package: &Package) -> bool {
        // Check direct dependencies (including workspace dependencies)
        package.dependencies.iter().any(|dep| dep.name == "bevy")
    }
}