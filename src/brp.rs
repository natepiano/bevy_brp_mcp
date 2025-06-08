//! BRP protocol communication

use std::time::Duration;

use anyhow::Result;
use reqwest::Client;
use serde_json::json;

use crate::constants::BRP_CLIENT_TIMEOUT_SECS;

/// Execute BRP commands against a running Bevy app
pub struct BrpClient {
    port:   u16,
    client: Client,
}

impl BrpClient {
    pub fn new(port: u16) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(BRP_CLIENT_TIMEOUT_SECS))
            .build()
            .unwrap_or_default();

        Self { port, client }
    }

    /// Check if a Bevy app is responding on the port
    pub async fn check_connection(&self) -> Result<bool> {
        // Try to call the list method which should be available on any BRP server
        let url = format!("http://localhost:{}", self.port);
        let request = json!({
            "jsonrpc": "2.0",
            "method": "bevy/list",
            "id": 1
        });

        match self.client.post(&url).json(&request).send().await {
            Ok(response) => Ok(response.status().is_success()),
            Err(_) => Ok(false),
        }
    }
}
