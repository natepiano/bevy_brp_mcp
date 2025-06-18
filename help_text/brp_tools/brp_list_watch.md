# bevy_start_list_watch

Start watching an entity for component list changes (additions/removals) with file-based logging.

## Usage

This tool starts a background watch on a specific entity to monitor when components are added to or removed from it. Unlike `bevy_start_entity_watch` which monitors component value changes, this watches for structural changes to the entity's component list.

## Parameters

- **entity** (required): The entity ID to watch for component list changes
- **port** (optional): BRP port to connect to (default: 15702)

## Returns

Returns a JSON object containing:
- **status**: "success" if the watch was started successfully  
- **watch_id**: Unique identifier for this watch subscription (use with `bevy_stop_watch`)
- **message**: Confirmation message
- **log_path**: Path to the log file where watch updates are written

## Watch Behavior

Once started, this watch will:

1. **Initial Entry**: Log a WATCH_STARTED entry with timestamp and parameters
2. **List Updates**: Log LIST_UPDATE entries when:
   - Components are added to the entity
   - Components are removed from the entity
3. **Entity Destruction**: Log when the watched entity is destroyed
4. **Errors**: Log any WATCH_ERROR entries if connection issues occur

## Log File Format

Updates are written to a timestamped log file with entries like:
```
[2024-01-15 10:30:45.123] WATCH_STARTED: {"entity":123,"port":15702,"timestamp":"2024-01-15T10:30:45.123Z"}
[2024-01-15 10:30:46.456] LIST_UPDATE: {"entity":123,"added":["Sprite"],"removed":[],"current":["Transform","Sprite"]}
[2024-01-15 10:30:47.789] LIST_UPDATE: {"entity":123,"added":[],"removed":["Sprite"],"current":["Transform"]}
```

**Important**: The watch continues running in the background until you explicitly stop it with `bevy_stop_watch` or the Bevy app shuts down. Active watches consume resources, so remember to stop them when no longer needed.

## Example Usage

Watch entity 123 for component additions/removals:
```json
{
  "entity": 123,
  "port": 15702  
}
```

## Use Cases

**Game Development**:
- Debug component insertion/removal issues
- Monitor entity state transitions
- Track component-based state machines

**System Analysis**:
- Understand how systems modify entities
- Debug component lifecycle
- Verify component dependencies

## Managing Watches

- Use `bevy_list_active_watches` to see all active watch subscriptions  
- Use `bevy_stop_watch` with the returned `watch_id` to stop the subscription
- Use `read_log` with the `log_path` to view watch updates
- Use `cleanup_logs` to remove old watch logs

## Reading Watch Logs

After starting a watch, you can read its updates:
```bash
# List all logs to find your watch log
list_logs

# Read specific watch log
read_log --filename bevy_brp_mcp_watch_1_list_123_1234567890.log

# Read only recent updates
read_log --filename bevy_brp_mcp_watch_1_list_123_1234567890.log --tail_lines 20
```

## Error Handling

Common errors:
- **Entity not found**: The specified entity doesn't exist
- **BRP connection failed**: Unable to connect to the Bevy app
- **Watch manager busy**: Too many concurrent watch operations