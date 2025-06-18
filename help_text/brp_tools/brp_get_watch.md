# bevy_start_entity_watch

Start watching an entity for component changes with file-based logging.

## Usage

This tool starts a background watch on a specific entity to monitor changes to its components. When components are added, removed, or modified on the watched entity, updates are written to a log file that can be read using `read_log`.

## Parameters

- **entity** (required): The entity ID to watch for component changes
- **components** (required): Array of specific component types to watch. Must contain at least one component. Without this parameter, the watch would not detect any changes
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
2. **Component Updates**: Log COMPONENT_UPDATE entries whenever:
   - Watched components are added to the entity
   - Watched components are removed from the entity  
   - Watched component values are modified (via `bevy_mutate_component` or other means)
   Note: Only the components specified in the `components` array are monitored
3. **Entity Destruction**: Log when the watched entity is destroyed
4. **Errors**: Log any WATCH_ERROR entries if connection issues occur

## Log File Format

Updates are written to a timestamped log file with entries like:
```
[2024-01-15 10:30:45.123] WATCH_STARTED: {"entity":123,"components":["Transform"],"port":15702,"timestamp":"2024-01-15T10:30:45.123Z"}
[2024-01-15 10:30:46.456] COMPONENT_UPDATE: {"entity":123,"components":{"Transform":{"translation":{"x":10.0,"y":20.0,"z":0.0},...}}}
```

**Important**: The watch continues running in the background until you explicitly stop it with `bevy_stop_watch` or the Bevy app shuts down. Active watches consume resources, so remember to stop them when no longer needed.

## Example Usage

```json
{
  "entity": 123,
  "components": [
    "bevy_transform::components::transform::Transform",
    "bevy_sprite::sprite::Sprite"
  ],
  "port": 15702
}
```

This will watch entity 123 for changes to its Transform and Sprite components.

**Important**: The watch only monitors the components you explicitly specify. To watch all components on an entity:
1. First use `bevy_list` to get all component types on the entity
2. Then pass all those component types in the `components` array

## Managing Watches

- Use `bevy_list_active_watches` to see all active streaming subscriptions
- Use `bevy_stop_watch` with the returned `watch_id` to stop the subscription
- Watches automatically stop if the BRP connection is lost

## Error Handling

Common errors:
- **Entity not found**: The specified entity doesn't exist
- **BRP connection failed**: Unable to connect to the Bevy app
- **Watch manager not initialized**: The MCP server's streaming system isn't ready
- **Invalid component types**: Specified components aren't registered with BRP
- **Missing components parameter**: The components array is required and must contain at least one component
- **Empty components array**: At least one component must be specified to watch