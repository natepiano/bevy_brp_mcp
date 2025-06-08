//! Cargo metadata and binary detection
//!
//! Adapted from bevy_brp_tool for finding Bevy application binaries

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, OnceLock};
use std::time::{Duration, Instant};

use anyhow::{Context, Result};
use cargo_metadata::{Metadata, MetadataCommand, Package};

/// Cache entry with TTL
#[derive(Debug, Clone)]
struct CacheEntry {
    metadata:  Option<Metadata>,
    timestamp: Instant,
}

impl CacheEntry {
    fn new(metadata: Option<Metadata>) -> Self {
        Self {
            metadata,
            timestamp: Instant::now(),
        }
    }

    fn is_expired(&self, ttl: Duration) -> bool {
        self.timestamp.elapsed() > ttl
    }
}

/// Cached cargo metadata to avoid repeated expensive calls, keyed by directory
/// Cache entries expire after 5 minutes
static METADATA_CACHE: OnceLock<Mutex<HashMap<PathBuf, CacheEntry>>> = OnceLock::new();

/// Cache TTL - 5 minutes
const CACHE_TTL: Duration = Duration::from_secs(300);

/// Validate that a profile name is safe to use in cargo commands
/// Profile names should only contain alphanumeric characters, hyphens, and underscores
pub fn validate_profile_name(profile: &str) -> Result<()> {
    if profile.is_empty() {
        anyhow::bail!("Profile name cannot be empty");
    }

    // Check for invalid characters
    for ch in profile.chars() {
        if !ch.is_alphanumeric() && ch != '-' && ch != '_' {
            anyhow::bail!(
                "Profile name '{}' contains invalid character '{}'. Only alphanumeric characters, hyphens, and underscores are allowed.",
                profile,
                ch
            );
        }
    }

    // Check length (reasonable limit)
    if profile.len() > 64 {
        anyhow::bail!(
            "Profile name '{}' is too long. Maximum length is 64 characters.",
            profile
        );
    }

    Ok(())
}

/// Information about a binary target
#[derive(Debug, Clone)]
pub struct BinaryInfo {
    /// Name of the binary
    pub name:        String,
    /// Whether this binary depends on Bevy
    pub is_bevy_app: bool,
}

/// Find a Bevy app binary by name and profile
pub fn find_bevy_binary(
    app_name: &str,
    profile: Option<&str>,
    roots: &[PathBuf],
) -> Result<PathBuf> {
    eprintln!("Looking for binary '{}' in {} roots", app_name, roots.len());

    let profile_dir = profile.unwrap_or(crate::constants::DEFAULT_BUILD_PROFILE);

    // Try each root directory to find the binary
    for (idx, root) in roots.iter().enumerate() {
        eprintln!("\nSearching root {}: {}", idx + 1, root.display());

        // First check the root itself
        if let Ok(detector) = CargoDetector::from_path(root) {
            if let Some(binary_info) = detector.find_binary_by_name(app_name) {
                let binary_path = detector
                    .target_directory()
                    .join(profile_dir)
                    .join(&binary_info.name);
                eprintln!("Found binary '{}' in root: {}", app_name, root.display());
                eprintln!("Binary path: {}", binary_path.display());
                eprintln!("Is Bevy app: {}", binary_info.is_bevy_app);

                // Check if the binary actually exists
                if binary_path.exists() {
                    if binary_info.is_bevy_app {
                        eprintln!("Binary exists at: {}", binary_path.display());
                        return Ok(binary_path);
                    } else {
                        // For non-Bevy apps, return an error
                        return Err(anyhow::anyhow!(
                            "Binary '{}' found but does not depend on Bevy",
                            app_name
                        ));
                    }
                } else {
                    eprintln!("Binary not built yet at: {}", binary_path.display());
                    if binary_info.is_bevy_app {
                        // Return an error with a specific message that we can handle
                        return Err(anyhow::anyhow!(
                            "Binary '{}' found in project at {} but not built. Run 'cargo build --profile {}' to build it.",
                            app_name,
                            root.display(),
                            profile_dir
                        ));
                    } else {
                        return Err(anyhow::anyhow!(
                            "Binary '{}' found but does not depend on Bevy",
                            app_name
                        ));
                    }
                }
            }
        }

        // Now search all subdirectories of the root
        if let Ok(binary_path) = search_subdirectories_for_binary(root, app_name, profile_dir) {
            return Ok(binary_path);
        }
    }

    // If not found in any root, try current directory as fallback
    if roots.is_empty() {
        eprintln!("No roots provided, checking current directory");
        let detector = CargoDetector::new()?;

        let binary_info = detector
            .find_binary_by_name(app_name)
            .ok_or_else(|| anyhow::anyhow!("Binary '{}' not found", app_name))?;

        if !binary_info.is_bevy_app {
            anyhow::bail!("Binary '{}' does not depend on Bevy", app_name);
        }

        let binary_path = detector
            .target_directory()
            .join(profile_dir)
            .join(&binary_info.name);

        // Check if the binary actually exists
        if binary_path.exists() {
            Ok(binary_path)
        } else {
            Err(anyhow::anyhow!(
                "Binary '{}' found but not built. Run 'cargo build --profile {}' to build it.",
                app_name,
                profile_dir
            ))
        }
    } else {
        anyhow::bail!(
            "Binary '{}' not found in any of the provided roots or their subdirectories",
            app_name
        )
    }
}

/// Recursively search subdirectories for a binary
fn search_subdirectories_for_binary(
    dir: &Path,
    app_name: &str,
    profile_dir: &str,
) -> Result<PathBuf> {
    use std::fs;

    let mut found_but_not_built = None;

    // Read directory entries
    let entries = fs::read_dir(dir)?;

    for entry in entries {
        let entry = entry?;
        let path = entry.path();

        // Skip hidden directories and target directories
        if let Some(name) = path.file_name() {
            let name_str = name.to_string_lossy();
            if name_str.starts_with('.') || name_str == "target" {
                continue;
            }
        }

        if path.is_dir() {
            // Check if this directory is a Cargo project
            if path.join("Cargo.toml").exists() {
                eprintln!("Checking: {}", path.display());

                if let Ok(detector) = CargoDetector::from_path(&path) {
                    if let Some(binary_info) = detector.find_binary_by_name(app_name) {
                        let binary_path = detector
                            .target_directory()
                            .join(profile_dir)
                            .join(&binary_info.name);
                        eprintln!("Found binary '{}' in: {}", app_name, path.display());
                        eprintln!("Binary path: {}", binary_path.display());
                        eprintln!("Is Bevy app: {}", binary_info.is_bevy_app);

                        // Check if the binary actually exists
                        if binary_path.exists() {
                            if binary_info.is_bevy_app {
                                eprintln!("Binary exists at: {}", binary_path.display());
                                return Ok(binary_path);
                            } else {
                                // For non-Bevy apps, return an error
                                return Err(anyhow::anyhow!(
                                    "Binary '{}' found but does not depend on Bevy",
                                    app_name
                                ));
                            }
                        } else {
                            eprintln!("Binary not built yet at: {}", binary_path.display());
                            // Remember this for later, but continue searching
                            if found_but_not_built.is_none() && binary_info.is_bevy_app {
                                found_but_not_built = Some((binary_path, path.clone()));
                            }
                        }
                    }
                }
            }

            // Recursively search subdirectories
            if let Ok(binary_path) = search_subdirectories_for_binary(&path, app_name, profile_dir)
            {
                return Ok(binary_path);
            }
        }
    }

    // If we found the binary but it wasn't built, report that specifically
    if let Some((_, project_path)) = found_but_not_built {
        return Err(anyhow::anyhow!(
            "Binary '{}' found in project at {} but not built. Run 'cargo build --profile {}' in that directory to build it.",
            app_name,
            project_path.display(),
            profile_dir
        ));
    }

    anyhow::bail!("Binary '{}' not found in {}", app_name, dir.display())
}

/// Detects binary targets in the current project or workspace
struct CargoDetector {
    metadata: Metadata,
}

impl CargoDetector {
    /// Create a new detector for the current directory
    fn new() -> Result<Self> {
        Self::from_path(std::env::current_dir()?)
    }

    /// Create a new detector for a specific path
    fn from_path(path: impl AsRef<Path>) -> Result<Self> {
        let current_dir = path
            .as_ref()
            .canonicalize()
            .unwrap_or_else(|_| path.as_ref().to_path_buf());

        // Get or initialize the cache
        let cache = METADATA_CACHE.get_or_init(|| Mutex::new(HashMap::new()));

        // Try to get cached metadata first
        let metadata = {
            let mut cache_guard = cache.lock().unwrap();

            // Check if we have a valid cache entry
            let use_cached = cache_guard
                .get(&current_dir)
                .map(|entry| !entry.is_expired(CACHE_TTL))
                .unwrap_or(false);

            if use_cached {
                cache_guard.get(&current_dir).unwrap().metadata.clone()
            } else {
                // Not in cache or expired, execute cargo metadata
                let result = MetadataCommand::new().current_dir(&current_dir).exec().ok();

                // Cache the result (even if None)
                cache_guard.insert(current_dir.clone(), CacheEntry::new(result.clone()));
                result
            }
        };

        match metadata {
            Some(metadata) => Ok(Self { metadata }),
            None => {
                // If cache failed and no cached result, try one more direct execution
                let metadata = MetadataCommand::new()
                    .current_dir(&current_dir)
                    .exec()
                    .context("Failed to execute cargo metadata")?;

                Ok(Self { metadata })
            }
        }
    }

    /// Find a binary by name
    fn find_binary_by_name(&self, name: &str) -> Option<BinaryInfo> {
        // First look for the binary in all packages
        let mut found_binary = None;
        
        for package in &self.metadata.packages {
            // Only process workspace members
            if !self.metadata.workspace_members.contains(&package.id) {
                continue;
            }

            for target in &package.targets {
                if target.is_bin() && target.name == name {
                    found_binary = Some(BinaryInfo {
                        name:        target.name.clone(),
                        is_bevy_app: self.package_depends_on_bevy(package),
                    });
                    // If it's a Bevy app, prefer it and return immediately
                    if found_binary.as_ref().unwrap().is_bevy_app {
                        return found_binary;
                    }
                }
            }
        }
        
        // Return any found binary (even if not a Bevy app)
        found_binary
    }

    fn package_depends_on_bevy(&self, package: &Package) -> bool {
        // Check direct dependencies
        for dep in &package.dependencies {
            if dep.name == "bevy" {
                return true;
            }
        }

        // For workspace members, if the workspace has bevy, assume the package might use it
        // This is a more permissive heuristic since cargo_metadata doesn't resolve workspace deps
        if self.metadata.workspace_members.contains(&package.id) && self.workspace_has_bevy() {
            // The package is in a workspace that uses bevy, so it's likely a bevy app
            // We'll let the user know if it's not actually a bevy app when they try to run it
            return true;
        }

        false
    }

    fn workspace_has_bevy(&self) -> bool {
        // Check if any workspace member depends on bevy
        self.metadata
            .packages
            .iter()
            .filter(|pkg| self.metadata.workspace_members.contains(&pkg.id))
            .any(|pkg| pkg.dependencies.iter().any(|dep| dep.name == "bevy"))
    }

    /// Get the target directory where binaries are built
    fn target_directory(&self) -> &Path {
        self.metadata.target_directory.as_ref()
    }
}
