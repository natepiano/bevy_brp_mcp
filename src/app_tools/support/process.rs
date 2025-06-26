use std::fs::File;
use std::path::Path;
use std::process::Stdio;

use rmcp::Error as McpError;
use tokio::process::Command;

use crate::error::BrpMcpError;

/// Launch a detached process with proper setup
pub fn launch_detached_process(
    cmd: &std::process::Command,
    working_dir: &Path,
    log_file: File,
    process_name: &str,
    operation: &str,
) -> Result<u32, McpError> {
    // Clone the log file handle for stderr
    let log_file_for_stderr = log_file
        .try_clone()
        .map_err(|e| BrpMcpError::failed_to("clone log file handle", e))?;

    // Use tokio to spawn the process in a detached manner
    // We run this in a blocking context since the caller is sync
    tokio::task::block_in_place(|| {
        tokio::runtime::Handle::current().block_on(async {
            // Convert std::process::Command to tokio::process::Command
            let mut tokio_cmd = Command::new(cmd.get_program());

            // Copy args
            for arg in cmd.get_args() {
                tokio_cmd.arg(arg);
            }

            // Copy current dir and env
            tokio_cmd
                .current_dir(working_dir)
                .env("CARGO_MANIFEST_DIR", working_dir);

            // Copy other environment variables
            for (key, value) in cmd.get_envs() {
                if let Some(value) = value {
                    tokio_cmd.env(key, value);
                }
            }

            // Set stdio
            tokio_cmd
                .stdin(Stdio::null()) // Important: detach stdin so the child doesn't inherit it
                .stdout(Stdio::from(log_file))
                .stderr(Stdio::from(log_file_for_stderr))
                .kill_on_drop(false); // Don't kill when dropping the handle

            // Spawn the process
            match tokio_cmd.spawn() {
                Ok(child) => {
                    let pid = child.id().ok_or_else(|| {
                        BrpMcpError::process_failed("get PID", process_name, "no PID available")
                    })?;

                    // Don't wait for the child - let it run detached
                    // The child will continue running independently
                    Ok(pid)
                }
                Err(e) => Err(BrpMcpError::process_failed(operation, process_name, e).into()),
            }
        })
    })
}
