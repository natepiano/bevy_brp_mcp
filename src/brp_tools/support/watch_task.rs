//! Background task management for watch connections

use std::path::PathBuf;

use futures::StreamExt;
use serde_json::Value;
use tracing::{debug, error, info};

use crate::brp_tools::support::json_rpc_builder::BrpJsonRpcBuilder;
use crate::brp_tools::support::watch_logger::{self, BufferedWatchLogger};
use super::watch_manager::{WATCH_MANAGER, WatchInfo};

/// Process the watch stream from the BRP server
async fn process_watch_stream(
    response: reqwest::Response,
    entity_id: u64,
    logger: &BufferedWatchLogger,
) -> Result<(), String> {
    if !response.status().is_success() {
        let error_msg = format!(
            "BRP server returned error {}: {}",
            response.status(),
            response.status().canonical_reason().unwrap_or("Unknown")
        );
        error!("{}", error_msg);
        return Err(error_msg);
    }

    // Read the streaming response
    let mut stream = response.bytes_stream();

    while let Some(chunk) = stream.next().await {
        match chunk {
            Ok(bytes) => {
                // Convert bytes to string and try to parse as SSE
                if let Ok(text) = std::str::from_utf8(&bytes) {
                    // Split by newlines to process SSE events
                    for line in text.lines() {
                        if line.trim().is_empty() {
                            continue;
                        }

                        // Handle SSE format: "data: {json}"
                        if let Some(json_str) = line.strip_prefix("data: ") {
                            if let Ok(data) = serde_json::from_str::<Value>(json_str) {
                                debug!(
                                    "Received watch update for entity {}: {:?}",
                                    entity_id, data
                                );

                                // Extract the result from JSON-RPC response
                                if let Some(result) = data.get("result") {
                                    // Write to log file
                                    if let Err(e) = logger
                                        .write_update("COMPONENT_UPDATE", result.clone())
                                        .await
                                    {
                                        error!("Failed to write watch update to log: {}", e);
                                    }
                                } else {
                                    debug!("No result in JSON-RPC response: {:?}", data);
                                }
                            } else {
                                debug!("Failed to parse SSE data as JSON: {}", json_str);
                            }
                        } else {
                            debug!("Received non-SSE line: {}", line);
                        }
                    }
                }
            }
            Err(e) => {
                error!("Error reading stream chunk: {}", e);
                break;
            }
        }
    }

    info!("Watch stream ended for entity {}", entity_id);
    Ok(())
}

/// Run the watch connection in a spawned task
async fn run_watch_connection(
    watch_id: u32,
    entity_id: u64,
    watch_type: String,
    brp_method: String,
    params: Value,
    port: u16,
    logger: BufferedWatchLogger,
) {
    info!(
        "Starting {} watch task for entity {} on port {}",
        watch_type, entity_id, port
    );

    // Create HTTP client for streaming
    let url = format!("http://localhost:{port}/jsonrpc");
    let client = reqwest::Client::new();

    // Build JSON-RPC request for watching
    let request_body = BrpJsonRpcBuilder::new(&brp_method)
        .params(params)
        .build()
        .to_string();

    match client
        .post(&url)
        .header("Content-Type", "application/json")
        .body(request_body)
        // Don't set timeout for streaming connections
        .send()
        .await
    {
        Ok(response) => {
            if let Err(e) = process_watch_stream(response, entity_id, &logger).await {
                error!("Watch stream processing failed: {}", e);
            }
        }
        Err(e) => {
            error!("Failed to connect to BRP server: {}", e);
            let _ = logger
                .write_update(
                    "CONNECTION_ERROR",
                    serde_json::json!({
                        "error": e.to_string(),
                        "timestamp": chrono::Local::now().to_rfc3339()
                    }),
                )
                .await;
        }
    }

    // Write final log entry
    let _ = logger
        .write_update(
            "WATCH_ENDED",
            serde_json::json!({
                "entity": entity_id,
                "timestamp": chrono::Local::now().to_rfc3339()
            }),
        )
        .await;

    // Remove this watch from the active watches
    {
        let mut manager = WATCH_MANAGER.lock().await;
        if manager.active_watches.remove(&watch_id).is_some() {
            info!(
                "Watch {} for entity {} automatically cleaned up after connection ended",
                watch_id, entity_id
            );
        }
    }
}

/// Generic function to start a watch task
async fn start_watch_task(
    entity_id: u64,
    watch_type: &str,
    brp_method: &str,
    params: Value,
    port: u16,
) -> Result<(u32, PathBuf), String> {
    // Get watch_id first from manager and release lock immediately
    let watch_id = {
        let manager = WATCH_MANAGER.lock().await;
        manager.next_id()
    };

    // Now create log path with proper watch_id
    let log_path = watch_logger::get_watch_log_path(watch_id, entity_id, watch_type);

    // Create buffered logger
    let logger = BufferedWatchLogger::new(log_path.clone());

    // Create initial log entry
    let log_data = match params.clone() {
        Value::Object(mut map) => {
            map.insert("port".to_string(), serde_json::json!(port));
            map.insert(
                "timestamp".to_string(),
                serde_json::json!(chrono::Local::now().to_rfc3339()),
            );
            Value::Object(map)
        }
        _ => serde_json::json!({
            "entity": entity_id,
            "port": port,
            "timestamp": chrono::Local::now().to_rfc3339()
        }),
    };

    logger
        .write_update("WATCH_STARTED", log_data)
        .await
        .map_err(|e| format!("Failed to write initial log: {e}"))?;

    let watch_type_owned = watch_type.to_string();
    let brp_method_owned = brp_method.to_string();

    let handle = tokio::spawn(run_watch_connection(
        watch_id,
        entity_id,
        watch_type_owned,
        brp_method_owned,
        params,
        port,
        logger,
    ));

    // Register with watch manager (with actual registration this time)
    {
        let mut manager = WATCH_MANAGER.lock().await;
        manager.active_watches.insert(
            watch_id,
            (
                WatchInfo {
                    watch_id,
                    entity_id,
                    watch_type: watch_type.to_string(),
                    log_path: log_path.clone(),
                    port,
                },
                handle,
            ),
        );
    }

    Ok((watch_id, log_path))
}

/// Start a background task for entity component watching
pub async fn start_entity_watch_task(
    entity_id: u64,
    components: Option<Vec<String>>,
    port: u16,
) -> Result<(u32, PathBuf), String> {
    // Validate components parameter
    let components = components.ok_or_else(|| {
        "Components parameter is required for entity watch. Specify which components to monitor.".to_string()
    })?;

    if components.is_empty() {
        return Err(
            "Components array cannot be empty. Specify at least one component to watch."
                .to_string(),
        );
    }

    // Build the watch parameters
    let params = serde_json::json!({
        "entity": entity_id,
        "components": components
    });

    start_watch_task(entity_id, "get", "bevy/get+watch", params, port).await
}

/// Start a background task for entity list watching
pub async fn start_list_watch_task(entity_id: u64, port: u16) -> Result<(u32, PathBuf), String> {
    let params = serde_json::json!({
        "entity": entity_id
    });

    start_watch_task(entity_id, "list", "bevy/list+watch", params, port).await
}
