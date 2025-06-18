# bevy_stop_watch

Stop an active watch subscription by its watch ID.

## Usage

This tool stops a previously started watch subscription and its background task. The watch log file remains available for reading after the watch is stopped.

## Parameters

- **watch_id** (required): The numeric watch ID returned from `bevy_start_entity_watch` or `bevy_start_list_watch`

## Returns

Returns a JSON object containing:
- **status**: "success" if the watch was stopped successfully, "error" if failed
- **message**: Confirmation or error message

## When to Stop Watches

**Always stop watches when**:
- Analysis or debugging is complete
- Switching to monitor different entities/components
- Before starting many new watches (to avoid resource exhaustion)
- When the watched entity is no longer relevant

**Automatic stopping occurs when**:
- The Bevy app shuts down
- The BRP connection is lost
- The MCP server restarts

## Log File Persistence

When a watch is stopped:
- The background streaming task is terminated
- The log file remains on disk for later analysis
- Use `read_log` to examine the captured data
- Use `cleanup_logs` to remove old watch logs

This allows you to:
- Stop resource-intensive watches while keeping their data
- Analyze watch data after the fact
- Compare data from multiple watch sessions

## Example Usage

Stop a specific watch:
```json
{
  "watch_id": 1
}
```

## Best Practices

**Resource Management**:
- Always pair `bevy_start_*_watch` calls with `bevy_stop_watch`
- Use `bevy_list_active_watches` to audit running subscriptions
- Stop watches before starting new ones if you're at resource limits

**Log Management**:
- Watch logs persist after stopping the watch
- Use `cleanup_logs` to remove old watch logs
- Filter logs by app name when cleaning up

## Managing Multiple Watches

When managing many watches:

1. **Keep track of watch IDs**: They're simple integers starting from 1
2. **Use log file names**: They include entity ID and watch type for identification
3. **Batch operations**: Stop related watches together
4. **Audit regularly**: Use `bevy_list_active_watches` to see what's running

## Example Workflow

```javascript
// Start watching an entity
const entityWatch = await tools.bevy_start_entity_watch({
  entity: 123,
  components: ["Transform"]
});
// Returns: { watch_id: 1, log_path: "/tmp/bevy_brp_mcp_watch_1_get_123_1234567890.log" }

// Start watching component list changes
const listWatch = await tools.bevy_start_list_watch({
  entity: 456
});
// Returns: { watch_id: 2, log_path: "/tmp/bevy_brp_mcp_watch_2_list_456_1234567891.log" }

// Check active watches
const active = await tools.bevy_list_active_watches();
// Shows both watches with their details

// Stop watches when done
await tools.bevy_stop_watch({ watch_id: 1 });
await tools.bevy_stop_watch({ watch_id: 2 });

// Read captured data later
await tools.read_log({ 
  filename: "bevy_brp_mcp_watch_1_get_123_1234567890.log" 
});
```

## Error Handling

**Successful stop**:
```json
{
  "status": "success",
  "message": "Successfully stopped watch 1"
}
```

**Watch not found**:
```json
{
  "status": "error", 
  "message": "Watch 99 not found"
}
```