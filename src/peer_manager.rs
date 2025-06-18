//! Global peer management for sending notifications

use std::sync::Arc;
use tokio::sync::Mutex;
use rmcp::{RoleServer, service::Peer};

/// Global storage for the current peer connection
static PEER_INSTANCE: once_cell::sync::Lazy<Arc<Mutex<Option<Peer<RoleServer>>>>> =
    once_cell::sync::Lazy::new(|| Arc::new(Mutex::new(None)));

/// Set the global peer instance
pub async fn set_peer(peer: Peer<RoleServer>) {
    let mut peer_guard = PEER_INSTANCE.lock().await;
    *peer_guard = Some(peer);
}

/// Get a clone of the global peer instance
pub async fn get_peer() -> Option<Peer<RoleServer>> {
    let peer_guard = PEER_INSTANCE.lock().await;
    peer_guard.clone()
}

/// Clear the global peer instance
pub async fn clear_peer() {
    let mut peer_guard = PEER_INSTANCE.lock().await;
    *peer_guard = None;
}