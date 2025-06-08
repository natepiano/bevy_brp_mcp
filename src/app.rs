//! App resolution and management

use std::time::Duration;

use anyhow::Result;
use tokio::time::sleep;

use crate::brp::BrpClient;
use crate::constants::{APP_LAUNCH_DELAY_SECS, DEFAULT_BRP_PORT, DEFAULT_BUILD_PROFILE};
use crate::{cargo, detached};

/// Information about a Bevy app
#[derive(Debug, Clone)]
pub enum AppInfo {
    /// App is running on the specified port
    Running { port: u16 },
    /// App is not running
    NotRunning,
}

/// Centralized app management
pub struct AppManager;

impl AppManager {
    /// Resolve an app name to either a running instance or binary path
    pub async fn resolve(app_name: &str) -> Result<AppInfo> {
        // First check if we have session info for this app
        if let Some(session_info) = detached::get_session_info(app_name, DEFAULT_BRP_PORT).await? {
            // We have session info, check if the app is still responding
            let client = BrpClient::new(session_info.port);
            if client.check_connection().await.unwrap_or(false) {
                return Ok(AppInfo::Running {
                    port: session_info.port,
                });
            }
        }

        // Also check if app is running on default port without session info
        // (could have been started manually)
        let client = BrpClient::new(DEFAULT_BRP_PORT);
        if client.check_connection().await.unwrap_or(false) {
            return Ok(AppInfo::Running {
                port: DEFAULT_BRP_PORT,
            });
        }

        // Not running
        Ok(AppInfo::NotRunning)
    }

    /// Find a Bevy app binary with profile handling
    pub fn find_binary(
        app_name: &str,
        profile: Option<&str>,
        roots: &[std::path::PathBuf],
    ) -> Result<std::path::PathBuf> {
        let profile_str = profile.unwrap_or(DEFAULT_BUILD_PROFILE);

        // Validate profile name to prevent command injection
        cargo::validate_profile_name(profile_str)?;

        cargo::find_bevy_binary(app_name, Some(profile_str), roots)
    }

    /// Launch an app in detached mode and wait for it to start
    pub async fn launch_app(app_name: &str, binary_path: &std::path::Path) -> Result<()> {
        // Start the app in detached mode
        let session = detached::start_detached(app_name, binary_path, DEFAULT_BRP_PORT).await?;

        eprintln!(
            "Started app '{}' with PID {} on port {}",
            app_name, session.pid, session.port
        );
        eprintln!("Log file: {:?}", session.log_file);

        // Wait for app to start responding
        sleep(Duration::from_secs(APP_LAUNCH_DELAY_SECS)).await;

        // Check if BRP is responding
        let client = BrpClient::new(DEFAULT_BRP_PORT);
        if !client.check_connection().await.unwrap_or(false) {
            // Try to clean up the process
            let _ = detached::kill_process(session.pid);
            anyhow::bail!(
                "App started but BRP is not responding on port {}",
                DEFAULT_BRP_PORT
            );
        }

        Ok(())
    }
}
