//! Shared constants across the application

/// Default port for Bevy Remote Protocol
pub const DEFAULT_BRP_PORT: u16 = 15702;

/// Default build profile when not specified
pub const DEFAULT_BUILD_PROFILE: &str = "debug";

/// Default timeout for BRP client requests
pub const BRP_CLIENT_TIMEOUT_SECS: u64 = 2;

/// Delay after launching app before checking status
pub const APP_LAUNCH_DELAY_SECS: u64 = 2;
