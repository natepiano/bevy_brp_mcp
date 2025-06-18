# bevy_list_active_watches

List all currently active watch subscriptions.

## Usage

This tool returns information about all currently running background watch tasks. Use this to audit active watches, manage resources, and find log file paths.

## Parameters

This tool takes no parameters.

## Returns

Returns a JSON object containing:
- **status**: "success" 
- **count**: Number of active watch subscriptions
- **watches**: Array of active watch objects
- **message**: Summary message

Each watch object contains:
- **watch_id**: Numeric identifier for the watch (monotonically increasing from 1)
- **entity_id**: The entity being watched
- **watch_type**: Type of watch ("get" or "list")
- **log_path**: Path to the log file where updates are written
- **port**: BRP port the watch is connected to

## Use Cases

**Resource Management**:
- See how many background watch tasks are running
- Identify watches that might no longer be needed
- Find log paths for active watches

**Debugging**:
- Verify that watches you started are actually running
- Locate log files for reading watch data
- Check which entities are being monitored

## Example Response

```json
{
  "status": "success",
  "count": 3,
  "watches": [
    {
      "watch_id": 1,
      "entity_id": 123,
      "watch_type": "get",
      "log_path": "/tmp/bevy_brp_mcp_watch_1_get_123_1705315845.log",
      "port": 15702
    },
    {
      "watch_id": 2,
      "entity_id": 456,
      "watch_type": "list",
      "log_path": "/tmp/bevy_brp_mcp_watch_2_list_456_1705315850.log",
      "port": 15702  
    },
    {
      "watch_id": 3,
      "entity_id": 789,
      "watch_type": "get",
      "log_path": "/tmp/bevy_brp_mcp_watch_3_get_789_1705315855.log",
      "port": 15703
    }
  ],
  "message": "Found 3 active watch(es)"
}
```

## Log File Naming

Log files follow a predictable naming pattern:
`bevy_brp_mcp_watch_{watch_id}_{watch_type}_{entity_id}_{timestamp}.log`

This makes it easy to:
- Identify which watch created the log
- Know which entity was being watched
- See when the watch was started
- Filter logs by watch type

## Empty Response

When no watches are active:

```json
{
  "status": "success",
  "count": 0,
  "watches": [],
  "message": "Found 0 active watch(es)"
}
```

## Practical Workflow

**Check current watches and read their logs**:
```javascript
// Get all active watches
const active = await tools.bevy_list_active_watches();
console.log(`Currently ${active.count} watches running`);

// Read the log from the first watch
if (active.count > 0) {
  const firstWatch = active.watches[0];
  const logContent = await tools.read_log({
    filename: firstWatch.log_path.split('/').pop()
  });
}
```

**Monitor specific entities**:
```javascript
// Find all watches for entity 123
const active = await tools.bevy_list_active_watches();
const entity123Watches = active.watches.filter(w => w.entity_id === 123);

console.log(`Found ${entity123Watches.length} watches for entity 123`);
entity123Watches.forEach(w => {
  console.log(`Watch ${w.watch_id} (${w.watch_type}): ${w.log_path}`);
});
```

**Resource management**:
```javascript
// Check watch count before starting new ones
const active = await tools.bevy_list_active_watches();

if (active.count > 10) {
  console.log("High watch count - consider stopping some:");
  active.watches.forEach(w => {
    console.log(`  Watch ${w.watch_id}: entity ${w.entity_id} (${w.watch_type})`);
  });
}

// Stop oldest watches (lowest IDs)
const oldestWatch = active.watches.reduce((min, w) => 
  w.watch_id < min.watch_id ? w : min
);
await tools.bevy_stop_watch({ watch_id: oldestWatch.watch_id });
```