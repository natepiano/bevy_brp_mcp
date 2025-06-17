use std::sync::Arc;

use serde_json::{Map, Value};

use crate::constants::{DEFAULT_PROFILE, PARAM_PROFILE, PROFILE_DEBUG, PROFILE_RELEASE};

/// Builder for creating JSON schemas for tool registration
pub struct SchemaBuilder {
    properties: Map<String, Value>,
    required:   Vec<String>,
}

impl SchemaBuilder {
    pub fn new() -> Self {
        Self {
            properties: Map::new(),
            required:   Vec::new(),
        }
    }

    /// Add a string property to the schema
    pub fn add_string_property(mut self, name: &str, description: &str, required: bool) -> Self {
        let mut prop = Map::new();
        prop.insert("type".to_string(), "string".into());
        prop.insert("description".to_string(), description.into());
        self.properties.insert(name.to_string(), prop.into());

        if required {
            self.required.push(name.to_string());
        }

        self
    }

    /// Add a string array property to the schema
    pub fn add_string_array_property(
        mut self,
        name: &str,
        description: &str,
        required: bool,
    ) -> Self {
        let mut prop = Map::new();
        prop.insert("type".to_string(), "array".into());

        let mut items = Map::new();
        items.insert("type".to_string(), "string".into());
        prop.insert("items".to_string(), items.into());

        prop.insert("description".to_string(), description.into());
        self.properties.insert(name.to_string(), prop.into());

        if required {
            self.required.push(name.to_string());
        }

        self
    }

    /// Add a number property to the schema
    pub fn add_number_property(mut self, name: &str, description: &str, required: bool) -> Self {
        let mut prop = Map::new();
        prop.insert("type".to_string(), "number".into());
        prop.insert("description".to_string(), description.into());
        self.properties.insert(name.to_string(), prop.into());

        if required {
            self.required.push(name.to_string());
        }

        self
    }

    /// Add a boolean property to the schema
    pub fn add_boolean_property(mut self, name: &str, description: &str, required: bool) -> Self {
        let mut prop = Map::new();
        prop.insert("type".to_string(), "boolean".into());
        prop.insert("description".to_string(), description.into());
        self.properties.insert(name.to_string(), prop.into());

        if required {
            self.required.push(name.to_string());
        }

        self
    }

    /// Add a property that can be any type (object, array, null, etc.)
    pub fn add_any_property(mut self, name: &str, description: &str, required: bool) -> Self {
        let mut prop = Map::new();
        prop.insert("type".to_string(), vec!["object", "array", "null"].into());
        prop.insert("description".to_string(), description.into());
        self.properties.insert(name.to_string(), prop.into());

        if required {
            self.required.push(name.to_string());
        }

        self
    }

    /// Add an enum property to the schema
    pub fn add_enum_property(
        mut self,
        name: &str,
        description: &str,
        values: Vec<&str>,
        default: Option<&str>,
        required: bool,
    ) -> Self {
        let mut prop = Map::new();
        prop.insert("type".to_string(), "string".into());
        prop.insert("enum".to_string(), values.into());
        prop.insert("description".to_string(), description.into());

        if let Some(default_value) = default {
            prop.insert("default".to_string(), default_value.into());
        }

        self.properties.insert(name.to_string(), prop.into());

        if required {
            self.required.push(name.to_string());
        }

        self
    }

    /// Add a standard profile property (debug/release) to the schema
    pub fn add_profile_property(self) -> Self {
        self.add_enum_property(
            PARAM_PROFILE,
            "Build profile to use (debug or release)",
            vec![PROFILE_DEBUG, PROFILE_RELEASE],
            Some(DEFAULT_PROFILE),
            false,
        )
    }

    /// Build the final schema
    pub fn build(self) -> Arc<Map<String, Value>> {
        let mut schema = Map::new();
        schema.insert("type".to_string(), "object".into());
        schema.insert("properties".to_string(), self.properties.into());

        if !self.required.is_empty() {
            schema.insert("required".to_string(), self.required.into());
        }

        Arc::new(schema)
    }
}

/// Create a simple object schema with no properties (for tools with no parameters)
pub fn empty_object_schema() -> Arc<Map<String, Value>> {
    let mut schema = Map::new();
    schema.insert("type".to_string(), "object".into());
    schema.insert("properties".to_string(), Map::new().into());
    Arc::new(schema)
}
