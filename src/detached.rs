//! Detached process management for Bevy apps

use std::env;
use std::fs::{self, File};
use std::io::Write;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::time::SystemTime;

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use sysinfo::{Pid, System};

/// Session information for a detached app
#[derive(Debug)]
pub struct DetachedSession {
    pub pid:      u32,
    pub port:     u16,
    pub log_file: PathBuf,
}

/// Persistent session information stored in temp directory
#[derive(Debug, Serialize, Deserialize)]
pub struct SessionInfo {
    pub pid:        u32,
    pub port:       u16,
    pub log_file:   PathBuf,
    pub start_time: SystemTime,
    pub app_name:   String,
}

/// Get the session file prefix used for all session-related files
pub fn get_session_prefix() -> &'static str {
    "bevy_brp_mcp_session"
}

/// Get the path to the session info file for a given app name and port
pub fn get_session_info_path(app_name: &str, port: u16) -> PathBuf {
    env::temp_dir().join(format!(
        "{}_{}_port_{}.json",
        get_session_prefix(),
        app_name,
        port
    ))
}

/// Get the path for a session log file with a given timestamp
pub fn get_session_log_path(app_name: &str, timestamp: u128) -> PathBuf {
    env::temp_dir().join(format!(
        "{}_{}_{}.log",
        get_session_prefix(),
        app_name,
        timestamp
    ))
}

/// Start app in detached mode with output redirected to log file
pub async fn start_detached(
    app_name: &str,
    binary_path: &std::path::Path,
    port: u16,
) -> Result<DetachedSession> {
    // Generate unique log file name in temp directory using timestamp
    let timestamp = SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)?
        .as_millis();
    let log_file = get_session_log_path(app_name, timestamp);

    // Create log file
    let mut file = File::create(&log_file)
        .with_context(|| format!("Failed to create log file: {:?}", log_file))?;
    writeln!(file, "=== Bevy BRP MCP Detached Session ===")?;
    writeln!(file, "Started at: {:?}", SystemTime::now())?;
    writeln!(file, "Port: {}", port)?;
    writeln!(file, "App name: {}", app_name)?;
    writeln!(file, "Binary path: {:?}", binary_path)?;
    writeln!(file, "============================================\n")?;
    file.sync_all()?;

    // Get the directory containing the binary as the working directory
    let working_dir = binary_path
        .parent()
        .ok_or_else(|| anyhow::anyhow!("Binary path has no parent directory"))?;

    // Start the app in background with output redirected to log file
    let log_file_for_redirect = File::options().append(true).open(&log_file)?;

    let child = Command::new(binary_path)
        .current_dir(working_dir)
        .stdout(Stdio::from(log_file_for_redirect.try_clone()?))
        .stderr(Stdio::from(log_file_for_redirect))
        .spawn()
        .with_context(|| format!("Failed to start app: {:?}", binary_path))?;

    let pid = child.id();

    // Save session info to temp directory
    let session_info = SessionInfo {
        pid,
        port,
        log_file: log_file.clone(),
        start_time: SystemTime::now(),
        app_name: app_name.to_string(),
    };

    let session_info_path = get_session_info_path(app_name, port);
    let session_json = serde_json::to_string_pretty(&session_info)?;
    fs::write(&session_info_path, session_json)
        .with_context(|| format!("Failed to save session info to {:?}", session_info_path))?;

    Ok(DetachedSession {
        pid,
        port,
        log_file,
    })
}

/// Get information about a running detached session
pub async fn get_session_info(app_name: &str, port: u16) -> Result<Option<SessionInfo>> {
    let session_info_path = get_session_info_path(app_name, port);

    if !session_info_path.exists() {
        return Ok(None);
    }

    // Read session info
    let contents = fs::read_to_string(&session_info_path)?;
    let session_info: SessionInfo = serde_json::from_str(&contents)?;

    // Check if process is still alive
    if !is_process_alive(session_info.pid) {
        // Clean up stale session info
        let _ = fs::remove_file(&session_info_path);
        return Ok(None);
    }

    Ok(Some(session_info))
}

/// Check if a process is still alive
pub fn is_process_alive(pid: u32) -> bool {
    let mut system = System::new();
    system.refresh_processes(
        sysinfo::ProcessesToUpdate::Some(&[Pid::from_u32(pid)]),
        false,
    );
    system.process(Pid::from_u32(pid)).is_some()
}

/// Kill a process by PID
pub fn kill_process(pid: u32) -> Result<()> {
    let mut system = System::new();
    system.refresh_processes(
        sysinfo::ProcessesToUpdate::Some(&[Pid::from_u32(pid)]),
        false,
    );

    if let Some(process) = system.process(Pid::from_u32(pid)) {
        process.kill();
        Ok(())
    } else {
        // Process not found - this is not an error, it might have already exited
        Ok(())
    }
}
