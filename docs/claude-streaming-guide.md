# Claude Code Streaming Integration Guide

This guide explains how to use the new SSE streaming features in the bevy_brp_mcp server with Claude Code.

## Overview

The bevy_brp_mcp server now supports real-time streaming of Bevy entity and component changes through four new tools:

- `bevy_start_entity_watch` - Watch specific entities for component changes
- `bevy_start_list_watch` - Watch for component additions/removals across all entities  
- `bevy_stop_watch` - Stop active watch subscriptions
- `bevy_list_active_watches` - List all active streaming subscriptions

These tools use Server-Sent Events (SSE) to deliver real-time updates while maintaining the standard MCP request-response pattern for tool calls.

## How Streaming Works with Claude Code

### Dual Transport Architecture

The MCP server uses a **dual transport architecture**:

1. **STDIO Transport**: Used for normal MCP tool calls and responses
2. **SSE Streaming**: Used for delivering real-time updates in the background

When you start a watch, Claude Code:
1. Sends the tool call via STDIO (normal MCP)
2. Receives the initial response with a `watch_id`
3. Begins receiving streaming updates in the background via SSE
4. Processes streaming notifications as they arrive

### Non-Blocking Operation

**Important**: Starting a streaming watch does NOT block Claude Code. The tool call returns immediately with a watch ID, and subsequent updates are delivered asynchronously.

This means you can:
- Start multiple watches simultaneously
- Continue using other MCP tools while watches are active
- Receive updates in real-time without interrupting your workflow

## Basic Usage Examples

### Watching a Specific Entity

```javascript
// Start watching entity 123 for Transform component changes
const watch = await tools.bevy_start_entity_watch({
  entity: 123,
  components: ["bevy_transform::components::transform::Transform"],
  port: 15702
});

console.log(`Started watch: ${watch.watch_id}`);
// Immediately returns, streaming begins in background

// Continue with other work - updates arrive automatically
// You'll see notifications when the Transform component changes
```

### Monitoring Component Lifecycle

```javascript
// Watch for new entities with Sprite components
const spriteWatch = await tools.bevy_start_list_watch({
  components: ["bevy_sprite::sprite::Sprite"]
});

// The watch is now active - you'll get notifications when:
// - New entities are spawned with Sprite components
// - Sprite components are added to existing entities  
// - Entities with Sprite components are destroyed
// - Sprite components are removed from entities
```

## Practical Workflows

### Debugging Entity Behavior

When debugging why an entity isn't behaving correctly:

```javascript
// 1. Start watching the problematic entity
const entityWatch = await tools.bevy_start_entity_watch({
  entity: 456, // Entity that's misbehaving
  components: [
    "bevy_transform::components::transform::Transform",
    "bevy_sprite::sprite::Sprite",
    "my_game::components::Player"
  ]
});

// 2. Reproduce the issue - you'll see real-time updates
// 3. Observe what components change and when
// 4. Stop the watch when done debugging
await tools.bevy_stop_watch({ watch_id: entityWatch.watch_id });
```

### Performance Analysis

Monitor entity creation patterns:

```javascript
// Watch for high-frequency entity spawning
const performanceWatch = await tools.bevy_start_list_watch({
  components: [
    "bevy_transform::components::transform::Transform"
  ]
});

// Let it run during gameplay to see spawning patterns
// You'll get notifications for every entity creation/destruction

// Check active watches periodically
const active = await tools.bevy_list_active_watches();
console.log(`Monitoring ${active.count} streams`);

// Stop when analysis is complete
await tools.bevy_stop_watch({ watch_id: performanceWatch.watch_id });
```

### Multi-Entity Coordination

Watch multiple related entities:

```javascript
// Start watches for a group of related entities
const playerWatch = await tools.bevy_start_entity_watch({
  entity: 100, // Player entity
  components: ["my_game::components::Player", "bevy_transform::components::transform::Transform"]
});

const enemyWatch = await tools.bevy_start_entity_watch({
  entity: 200, // Enemy entity  
  components: ["my_game::components::Enemy", "bevy_transform::components::transform::Transform"]
});

// Both watches run simultaneously
// You'll see updates from both entities in real-time

// Clean up both watches
await tools.bevy_stop_watch({ watch_id: playerWatch.watch_id });
await tools.bevy_stop_watch({ watch_id: enemyWatch.watch_id });
```

## Resource Management

### Watch Limits and Performance

- **Be mindful of resource usage**: Each active watch consumes memory and network resources
- **Stop watches when done**: Always clean up streaming subscriptions
- **Limit concurrent watches**: Avoid running too many watches simultaneously
- **Use specific filters**: Watch only the components you need

### Best Practices

```javascript
// ✅ Good: Specific component filtering
const goodWatch = await tools.bevy_start_entity_watch({
  entity: 123,
  components: ["bevy_transform::components::transform::Transform"] // Only watch Transform
});

// ❌ Avoid: Watching all components (high traffic)
const noisyWatch = await tools.bevy_start_entity_watch({
  entity: 123
  // No components filter = watches everything
});

// ✅ Good: Clean up resources
await tools.bevy_stop_watch({ watch_id: goodWatch.watch_id });
```

### Auditing Active Watches

Regularly check what's running:

```javascript
// See all active watches
const active = await tools.bevy_list_active_watches();

console.log(`Active watches: ${active.count}`);
active.watches.forEach(watch => {
  console.log(`- ${watch.watch_id}: ${watch.method}`);
  console.log(`  Params: ${JSON.stringify(watch.params)}`);
});

// Stop specific watches if needed
for (const watch of active.watches) {
  if (watch.params.entity === 123) {
    await tools.bevy_stop_watch({ watch_id: watch.watch_id });
    console.log(`Stopped watch for entity 123`);
  }
}
```

## Notification Format

Streaming updates arrive as MCP notifications with this structure:

```json
{
  "method": "streaming_update",
  "params": {
    "watch_id": "entity-watch-550e8400-e29b-41d4-a716-446655440000",
    "data": {
      // Component data that changed
      "bevy_transform::components::transform::Transform": {
        "translation": {"x": 10.0, "y": 20.0, "z": 0.0},
        "rotation": {"x": 0.0, "y": 0.0, "z": 0.0, "w": 1.0},
        "scale": {"x": 1.0, "y": 1.0, "z": 1.0}
      }
    },
    "event_type": "component_changed",
    "timestamp": "2024-01-01T12:00:00Z"
  }
}
```

## Troubleshooting

### Common Issues

**"Watch manager not initialized"**
- The MCP server is starting up
- Wait a moment and try again

**"BRP connection failed"**  
- Check that your Bevy app is running
- Verify the port number (default: 15702)
- Ensure the Bevy app has BRP enabled

**"No streaming updates received"**
- Verify the entity/components exist using regular `bevy_get` or `bevy_query` tools
- Check that the watched components are registered with BRP
- Ensure the entity actually has the components you're watching

**"Too many active watches"**
- Use `bevy_list_active_watches` to see what's running
- Stop unnecessary watches with `bevy_stop_watch`
- Consider using more specific component filters

### Debug Workflow

1. **Verify basic connectivity**: Use `brp_status` to check BRP connection
2. **Test regular tools first**: Use `bevy_get` or `bevy_query` to verify entities exist
3. **Start simple watches**: Begin with single-entity, single-component watches
4. **Monitor resource usage**: Use `bevy_list_active_watches` to track active streams
5. **Clean up systematically**: Stop watches in reverse order of creation

## Integration with Analysis Workflows

### Game Development

```javascript
// Monitor player movement during gameplay testing
const playerMovement = await tools.bevy_start_entity_watch({
  entity: playerId,
  components: ["bevy_transform::components::transform::Transform"]
});

// Run gameplay test scenarios
// Observe position changes in real-time
// Identify movement anomalies or performance issues

await tools.bevy_stop_watch({ watch_id: playerMovement.watch_id });
```

### System Debugging

```javascript
// Monitor system interactions
const systemDebug = await tools.bevy_start_list_watch({
  components: ["my_game::components::Health", "my_game::components::Damage"]
});

// Trigger game events
// Watch how Health and Damage components are managed
// Identify system timing issues or state corruption

await tools.bevy_stop_watch({ watch_id: systemDebug.watch_id });
```

This streaming integration enables powerful real-time analysis capabilities while maintaining the familiar MCP tool interaction model that Claude Code users expect.