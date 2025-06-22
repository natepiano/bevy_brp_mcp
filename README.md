# bevy_brp_mcp

[![Crates.io](https://img.shields.io/crates/v/bevy_brp_mcp.svg)](https://crates.io/crates/bevy_brp_mcp)
[![Documentation](https://docs.rs/bevy_brp_mcp/badge.svg)](https://docs.rs/bevy_brp_mcp/)
[![MIT/Apache 2.0](https://img.shields.io/badge/license-MIT%2FApache-blue.svg)](https://github.com/yourusername/bevy_brp_mcp#license)
[![Crates.io](https://img.shields.io/crates/d/bevy_brp_mcp.svg)](https://crates.io/crates/bevy_brp_mcp)

A Model Context Protocol (MCP) server that enables AI coding assistants to control launch, inspect and mutateBevy applications via the Bevy Remote Protocol (BRP). This tool bridges the gap between coding agents and Bevy by providing comprehensive BRP integration as an MCP server.

## Bevy Compatibility

| bevy | bevy_brp_mcp |
|------|--------------|
| 0.16 | 0.1          |

The bevy_brp_mcp crate follows Bevy's version numbering and releases new versions for each Bevy release.

## Features

### Core BRP Operations
- **Entity Management**: Create, destroy, query, and modify entities
- **Component Operations**: Get, insert, remove, and mutate components on entities
- **Resource Management**: Access and modify global resources
- **Query System**: Advanced entity querying with filters
- **Hierarchy Operations**: Parent-child entity relationships

### Application Discovery & Management
- **App Discovery**: Find and list Bevy applications in your workspace
- **Build Status**: Check which apps are built and ready to run
- **Launch Management**: Start apps with proper asset loading and logging
- **Example Support**: Discover and run Bevy examples from your projects

### Real-time Monitoring
- **Component Watching**: Monitor component changes on specific entities
- **Log Management**: Centralized logging for all launched applications
- **Process Status**: Check if apps are running with BRP enabled

### Enhanced BRP Integration
- **Format Discovery**: Get correct JSON formats for BRP operations (via bevy_brp_extras)
- **Screenshot Capture**: Take screenshots of running Bevy applications
- **Graceful Shutdown**: Clean application termination

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
bevy_brp_mcp = "0.1"
```

## Usage

### With AI Coding Assistants

bevy_brp_mcp is designed to be used with AI coding assistants that support MCP (like Claude). The MCP server provides tools that allow the AI to:

1. Discover and launch your Bevy applications
2. Inspect and modify entity components in real-time
3. Monitor application state and debug issues
4. Take screenshots and manage application lifecycle

### Standalone Usage

You can also run the MCP server directly:

```bash
cargo run --bin bevy_brp_mcp
```

The server will start and listen for MCP connections on stdio.

### Setting Up Your Bevy App

For full functionality, your Bevy app should include BRP support:

```rust
use bevy::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(bevy::remote::RemotePlugin::default()) // Enable BRP
        .run();
}
```

For enhanced features like screenshots and format discovery, also add [bevy_brp_extras](https://github.com/natepiano/bevy_brp_extras):

```rust
use bevy::prelude::*;
use bevy_brp_extras::BrpExtrasPlugin;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(BrpExtrasPlugin) // Enhanced BRP features
        .run();
}
```

## Tool Categories

### BRP Core Tools
- `bevy_get` - Get component data from entities
- `bevy_insert` - Insert components into entities
- `bevy_remove` - Remove components from entities
- `bevy_mutate_component` - Modify specific component fields
- `bevy_spawn` - Create new entities with components
- `bevy_destroy` - Remove entities from the world
- `bevy_query` - Query entities by component filters
- `bevy_list` - List available components and entities
- `bevy_get_resource` - Access global resources
- `bevy_insert_resource` - Set global resources
- `bevy_remove_resource` - Remove global resources
- `bevy_mutate_resource` - Modify resource fields
- `bevy_list_resources` - List available resources
- `bevy_reparent` - Change entity hierarchy
- `bevy_registry_schema` - Get type schemas
- `bevy_rpc_discover` - Discover available BRP methods

### Application Management Tools
- `list_bevy_apps` - Find Bevy applications in workspace
- `list_brp_apps` - Find BRP-enabled applications
- `list_bevy_examples` - Find Bevy examples
- `launch_bevy_app` - Launch applications with proper setup
- `launch_bevy_example` - Run examples with cargo
- `brp_status` - Check if apps are running with BRP

### Enhanced Features (via bevy_brp_extras)
- `brp_extras_screenshot` - Capture application screenshots
- `brp_extras_shutdown` - Gracefully shutdown applications
- `brp_extras_discover_format` - Get correct JSON formats for BRP operations

### Monitoring & Debugging Tools
- `brp_get_watch` - Monitor component changes on entities
- `brp_list_watch` - Monitor component additions/removals
- `bevy_stop_watch` - Stop monitoring subscriptions
- `bevy_list_active_watches` - List active monitoring sessions

### Log Management Tools
- `list_logs` - List application log files
- `read_log` - Read log file contents with filtering
- `cleanup_logs` - Remove old log files

## Integration with bevy_brp_extras

This crate is designed to work seamlessly with [bevy_brp_extras](https://github.com/natepiano/bevy_brp_extras). When both are used together:

1. Add `BrpExtrasPlugin` to your Bevy app for enhanced BRP features
2. Use `bevy_brp_mcp` with your AI coding assistant
3. Additional methods like screenshot, shutdown, and format discovery will be automatically available
4. Get proper JSON formats for complex BRP operations

## Example Workflow

1. **Discovery**: Use `list_bevy_apps` to find available applications
2. **Launch**: Use `launch_bevy_app` to start your game with proper logging
3. **Inspect**: Use `bevy_query` to find entities of interest
4. **Monitor**: Use `brp_get_watch` to observe entity changes in real-time
5. **Modify**: Use `bevy_mutate_component` to adjust entity properties
6. **Debug**: Use `read_log` to examine application output
7. **Capture**: Use `brp_extras_screenshot` to document current state

## Logging

All launched applications create detailed log files in `/tmp/` with names like:
- `bevy_brp_mcp_myapp_1234567890.log` (application logs)
- `bevy_brp_mcp_watch_123_get_456_1234567890.log` (monitoring logs)

Use the log management tools to view and clean up these files.

## License

Dual-licensed under either:
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)
- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)

at your option.
