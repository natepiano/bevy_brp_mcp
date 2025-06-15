# Bevy Remote Protocol (BRP) Methods Reference

This document provides a complete inventory of all available BRP methods for the bevy_brp_mcp MCP server.

## Overview

The Bevy Remote Protocol provides 18 built-in methods for interacting with a Bevy application's ECS (Entity Component System). All methods use JSON-RPC 2.0 format and are accessed through the `brp_execute` tool.

## Method Categories

### Entity Component Methods (9 methods)

#### 1. `bevy/get`
Retrieve the values of one or more components from an entity.

**Parameters:**
- `entity` (number): The entity ID
- `components` (string[]): Array of fully-qualified component type names
- `strict` (boolean, optional): Fail if any component is missing (default: false)

**Returns:**
- `components`: Map of component names to values
- `errors`: Map of component names to errors (if strict=false)

**Example:**
```json
{
  "method": "bevy/get",
  "params": {
    "entity": 4294967298,
    "components": ["bevy_transform::components::transform::Transform"]
  }
}
```

#### 2. `bevy/query`
Query entities by components with optional filters.

**Parameters:**
- `data` (object):
  - `components` (string[], optional): Components to fetch
  - `option` (string[], optional): Components to fetch if present
  - `has` (string[], optional): Components to check presence
- `filter` (object, optional):
  - `with` (string[], optional): Components that must be present
  - `without` (string[], optional): Components that must NOT be present
- `strict` (boolean, optional): Fail if components can't be reflected

**Returns:**
Array of objects containing:
- `entity`: The entity ID
- `components`: Map of component values
- `has`: Map of boolean presence values

**Example:**
```json
{
  "method": "bevy/query",
  "params": {
    "data": {
      "components": ["bevy_transform::components::transform::Transform"]
    },
    "filter": {
      "with": ["bevy_core::name::Name"]
    }
  }
}
```

#### 3. `bevy/spawn`
Create a new entity with components.

**Parameters:**
- `components` (object): Map of component type names to values

**Returns:**
- `entity`: The ID of the newly created entity

**Example:**
```json
{
  "method": "bevy/spawn",
  "params": {
    "components": {
      "bevy_core::name::Name": {
        "name": "My Entity"
      }
    }
  }
}
```

#### 4. `bevy/destroy`
Despawn an entity.

**Parameters:**
- `entity` (number): The entity ID to despawn

**Returns:** null

**Example:**
```json
{
  "method": "bevy/destroy",
  "params": {
    "entity": 4294967298
  }
}
```

#### 5. `bevy/insert`
Insert components into an existing entity.

**Parameters:**
- `entity` (number): The entity ID
- `components` (object): Map of component type names to values

**Returns:** null

#### 6. `bevy/remove`
Remove components from an entity.

**Parameters:**
- `entity` (number): The entity ID
- `components` (string[]): Array of component type names to remove

**Returns:** null

#### 7. `bevy/mutate_component`
Mutate a field in a component.

**Parameters:**
- `entity` (number): The entity ID
- `component` (string): The component type name
- `path` (string): Path to the field (e.g., "translation.x")
- `value` (any): The new value

**Returns:** null

#### 8. `bevy/reparent`
Change entity parent-child relationships.

**Parameters:**
- `entities` (number[]): Array of entity IDs to reparent
- `parent` (number, optional): New parent entity ID (omit to remove parent)

**Returns:** null

#### 9. `bevy/list`
List all registered components or components on a specific entity.

**Parameters:**
- `entity` (number, optional): Entity ID to list components for

**Returns:**
- Array of fully-qualified component type names

**Example:**
```json
{
  "method": "bevy/list",
  "params": {
    "entity": 4294967298
  }
}
```

### Watch Methods (2 methods)

#### 10. `bevy/get+watch`
Watch component changes on an entity (returns changes since last call).

**Parameters:**
- `entity` (number): The entity ID
- `components` (string[]): Components to watch
- `strict` (boolean, optional): Fail if components missing

**Returns:**
- `components`: Map of changed component values
- `removed`: Array of removed component names
- `errors`: Map of errors (if strict=false)

#### 11. `bevy/list+watch`
Watch component additions/removals on an entity.

**Parameters:**
- `entity` (number): The entity ID

**Returns:**
- `added`: Array of added component type names
- `removed`: Array of removed component type names

### Resource Methods (5 methods)

#### 12. `bevy/get_resource`
Get a resource value from the world.

**Parameters:**
- `resource` (string): The resource type name

**Returns:**
- `value`: The resource value

#### 13. `bevy/insert_resource`
Insert or update a resource.

**Parameters:**
- `resource` (string): The resource type name
- `value` (any): The resource value

**Returns:** null

#### 14. `bevy/remove_resource`
Remove a resource from the world.

**Parameters:**
- `resource` (string): The resource type name

**Returns:** null

#### 15. `bevy/mutate_resource`
Mutate a field in a resource.

**Parameters:**
- `resource` (string): The resource type name
- `path` (string): Path to the field
- `value` (any): The new value

**Returns:** null

#### 16. `bevy/list_resources`
List all registered resource types.

**Parameters:** none

**Returns:**
- Array of resource type names

### Discovery/Schema Methods (2 methods)

#### 17. `bevy/registry/schema`
Get JSON schema information for registered types.

**Parameters:**
- Query filter (optional)

**Returns:**
- JSON schemas for types

#### 18. `rpc.discover`
Discover all available methods with their schemas.

**Parameters:** none

**Returns:**
- OpenRPC document describing all methods

## Implementation Priority

### High Priority (Simple & Essential)
1. **`bevy/list`** - Simple listing functionality
2. **`bevy/destroy`** - Basic entity management
3. **`bevy/get`** - Essential for inspection
4. **`bevy/query`** - Core search functionality

### Medium Priority (Useful)
5. **`bevy/spawn`** - Entity creation
6. **`bevy/insert`** - Component addition
7. **`bevy/remove`** - Component removal
8. **`bevy/list_resources`** - Resource discovery

### Low Priority (Advanced)
- Watch methods (require persistent connections)
- Mutation methods (complex path handling)
- Resource manipulation (less common use cases)

## Usage with brp_execute

All methods are called using the `brp_execute` tool:

```json
{
  "tool": "brp_execute",
  "arguments": {
    "method": "bevy/list",
    "params": {},
    "port": 15702
  }
}
```

## Common Type Names

When working with BRP, you'll often need fully-qualified type names:

- `bevy_transform::components::transform::Transform`
- `bevy_core::name::Name`
- `bevy_transform::components::global_transform::GlobalTransform`
- `bevy_hierarchy::components::parent::Parent`
- `bevy_hierarchy::components::children::Children`

Use `bevy/list` without parameters to discover all available component types in your Bevy app.