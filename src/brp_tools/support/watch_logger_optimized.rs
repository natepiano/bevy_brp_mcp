//! Optimized watch logging with buffering and batching

use std::io::Write;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::mpsc;
use tokio::sync::Mutex;
use tokio::fs::OpenOptions;
use tokio::io::AsyncWriteExt;
use tracing::{error, debug};

/// Log entry to be written
#[derive(Debug)]
pub struct LogEntry {
    pub update_type: String,
    pub data: serde_json::Value,
    pub timestamp: chrono::DateTime<chrono::Local>,
}

/// Buffered logger for watch updates
pub struct BufferedWatchLogger {
    tx: mpsc::Sender<LogEntry>,
}

impl BufferedWatchLogger {
    /// Create a new buffered logger and spawn the writer task
    pub async fn new(log_path: PathBuf) -> std::io::Result<Self> {
        let (tx, rx) = mpsc::channel(1000); // Buffer up to 1000 messages
        
        // Spawn the writer task
        tokio::spawn(async move {
            if let Err(e) = write_task(log_path, rx).await {
                error!("Watch logger write task failed: {}", e);
            }
        });
        
        Ok(Self { tx })
    }
    
    /// Queue a log entry for writing (non-blocking)
    pub async fn write_update(
        &self,
        update_type: &str,
        data: serde_json::Value,
    ) -> Result<(), String> {
        let entry = LogEntry {
            update_type: update_type.to_string(),
            data,
            timestamp: chrono::Local::now(),
        };
        
        self.tx.send(entry).await
            .map_err(|_| "Logger channel closed".to_string())
    }
}

/// Background task that batches and writes log entries
async fn write_task(
    log_path: PathBuf,
    mut rx: mpsc::Receiver<LogEntry>,
) -> std::io::Result<()> {
    // Open file once and keep it open
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&log_path)
        .await?;
    
    // Buffer for batching writes
    let mut buffer = String::with_capacity(8192); // 8KB buffer
    let mut last_flush = tokio::time::Instant::now();
    let flush_interval = std::time::Duration::from_millis(100); // Flush every 100ms
    
    loop {
        // Try to receive with timeout
        let timeout = tokio::time::timeout(
            flush_interval,
            rx.recv()
        ).await;
        
        match timeout {
            Ok(Some(entry)) => {
                // Format entry into buffer
                let timestamp = entry.timestamp.format("%Y-%m-%d %H:%M:%S%.3f");
                if let Ok(json) = serde_json::to_string(&entry.data) {
                    let _ = writeln!(
                        &mut buffer,
                        "[{}] {}: {}",
                        timestamp,
                        entry.update_type,
                        json
                    );
                }
                
                // Check if we should flush (buffer size or time)
                if buffer.len() > 4096 || last_flush.elapsed() > flush_interval {
                    file.write_all(buffer.as_bytes()).await?;
                    file.flush().await?;
                    buffer.clear();
                    last_flush = tokio::time::Instant::now();
                    debug!("Flushed watch log buffer");
                }
            }
            Ok(None) => {
                // Channel closed, flush remaining buffer and exit
                if !buffer.is_empty() {
                    file.write_all(buffer.as_bytes()).await?;
                    file.flush().await?;
                }
                break;
            }
            Err(_) => {
                // Timeout - flush if buffer has content
                if !buffer.is_empty() {
                    file.write_all(buffer.as_bytes()).await?;
                    file.flush().await?;
                    buffer.clear();
                    last_flush = tokio::time::Instant::now();
                }
            }
        }
    }
    
    Ok(())
}

/// Get the log file path for a watch (same as before)
pub fn get_watch_log_path(watch_id: u32, entity_id: u64, watch_type: &str) -> PathBuf {
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();

    let filename = format!(
        "bevy_brp_mcp_watch_{}_{}_{}_{}.log",
        watch_id, watch_type, entity_id, timestamp
    );

    std::env::temp_dir().join(filename)
}