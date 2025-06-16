# BRP Type Registration Guide

This guide explains how to make your Bevy Components and Resources visible to the Bevy Remote Protocol (BRP).

## The Three Requirements

For any type to be queryable via BRP, it must meet **ALL** of these requirements:

### 1. Required Derives
Your type must derive all four traits:
- `Component` or `Resource` (depending on type)
- `Reflect`
- `Serialize`
- `Deserialize`

### 2. Reflect Attribute
You must add the `#[reflect(...)]` attribute listing the traits:
- For components: `#[reflect(Component, Serialize, Deserialize)]`
- For resources: `#[reflect(Resource, Serialize, Deserialize)]`

### 3. Type Registration
You must register the type with your Bevy app:
```rust
app.register_type::<YourType>();
```

## Complete Examples

### Component Example
```rust
use bevy::prelude::*;
use serde::{Serialize, Deserialize};

#[derive(Component, Reflect, Serialize, Deserialize)]
#[reflect(Component, Serialize, Deserialize)]
struct Health {
    pub current: f32,
    pub max: f32,
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(RemotePlugin::default())
        .register_type::<Health>()  // <-- Don't forget this!
        .run();
}
```

### Resource Example
```rust
use bevy::prelude::*;
use serde::{Serialize, Deserialize};

#[derive(Resource, Reflect, Serialize, Deserialize)]
#[reflect(Resource, Serialize, Deserialize)]
struct GameSettings {
    pub difficulty: String,
    pub sound_volume: f32,
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(RemotePlugin::default())
        .register_type::<GameSettings>()  // <-- Don't forget this!
        .run();
}
```

## Common Issues

### Issue: Type doesn't appear in bevy/list
**Cause**: Missing one of the three requirements
**Solution**: Verify all three requirements are met

### Issue: Compilation errors with derives
**Cause**: Missing serde dependency
**Solution**: Add to Cargo.toml:
```toml
[dependencies]
serde = { version = "1.0", features = ["derive"] }
```

### Issue: "the trait `Serialize` is not implemented"
**Cause**: Complex types in your struct that don't implement Serialize
**Solution**: Either:
- Use `#[reflect(skip_serializing)]` on problematic fields
- Implement custom serialization
- Use simpler types that support serialization

## Built-in Types

Many Bevy built-in components are already registered and will appear automatically:
- `Transform`
- `GlobalTransform`
- `Name`
- `Parent`
- `Children`
- And many more...

## Checking Registration

Use the `brp_list` tool to verify your types are registered:
```bash
# List all registered components
brp_list

# Check a specific entity
brp_list --entity_id 0
```

If your type doesn't appear, double-check all three requirements!

## Type Names in BRP

Once registered, your types will appear with their full module path:
- `my_game::components::health::Health`
- `my_game::resources::settings::GameSettings`

Use these fully-qualified names when querying with other BRP methods.
