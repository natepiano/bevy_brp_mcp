//! Declarative tool definitions for BRP (Bevy Remote Protocol) tools.
//!
//! This module provides a declarative approach to defining BRP tools, eliminating
//! code duplication and making it easy to add new tools. Instead of writing separate
//! handler files for each tool, tools are defined as data structures that describe
//! their parameters, extractors, and response formatting.
//!
//! # Architecture
//!
//! The declarative system consists of three main components:
//! - **Tool Definitions**: Data structures describing each tool's behavior
//! - **Parameter Extractors**: Logic for extracting and validating parameters
//! - **Response Formatters**: Logic for formatting responses consistently
//!
//! # Tool Categories
//!
//! Tools are categorized into three types:
//!
//! ## Standard Tools
//! Simple CRUD operations that follow predictable patterns:
//! - Entity operations: `brp_destroy`, `brp_get`, `brp_insert`, `brp_remove`
//! - Resource operations: `brp_get_resource`, `brp_insert_resource`, `brp_remove_resource`
//! - Query operations: `brp_list`, `brp_list_resources`
//! - Utility operations: `brp_rpc_discover`
//!
//! ## Special Tools
//! Tools with minor variations requiring custom extractors or response handling:
//! - `brp_query`: Custom component count extraction
//! - `brp_spawn`: Dynamic entity extraction from response
//! - `brp_execute`: Dynamic method selection from parameters
//! - `brp_registry_schema`: Complex parameter transformation
//! - `brp_reparent`: Array parameter handling
//!
//! ## Custom Tools (Not in this module)
//! Complex tools that remain as individual implementations:
//! - `brp_status`: System process detection
//! - Watch operations: `brp_get_watch`, `brp_list_watch`, etc.
//! - App management: `launch_bevy_app`, `list_bevy_apps`, etc.
//!
//! # Adding New Tools
//!
//! To add a new standard BRP tool:
//!
//! 1. **Define constants** in `constants.rs` for the tool name, description, and BRP method
//! 2. **Add tool definition** to the appropriate function with the required parameters and
//!    formatters
//! 3. **Register in generator** (automatic if added to `get_standard_tools()`)

use crate::brp_tools::constants::{
    DESC_PORT, JSON_FIELD_COMPONENT, JSON_FIELD_COMPONENTS, JSON_FIELD_COUNT, JSON_FIELD_DATA,
    JSON_FIELD_DESTROYED_ENTITY, JSON_FIELD_ENTITY, JSON_FIELD_METADATA, JSON_FIELD_PATH,
    JSON_FIELD_PORT, JSON_FIELD_RESOURCE, JSON_FIELD_RESOURCES, JSON_FIELD_VALUE,
    PARAM_COMPONENT_COUNT, PARAM_DATA, PARAM_ENTITIES, PARAM_ENTITY_COUNT, PARAM_FILTER,
    PARAM_FORMATS, PARAM_METHOD, PARAM_PARAMS, PARAM_PARENT, PARAM_QUERY_PARAMS, PARAM_RESULT,
    PARAM_SPAWNED_ENTITY, PARAM_STRICT, PARAM_TYPES, PARAM_WITH_CRATES, PARAM_WITH_TYPES,
    PARAM_WITHOUT_CRATES, PARAM_WITHOUT_TYPES,
};
use crate::tools::{
    BRP_METHOD_DESTROY, BRP_METHOD_EXTRAS_DISCOVER_FORMAT, BRP_METHOD_EXTRAS_SCREENSHOT,
    BRP_METHOD_GET, BRP_METHOD_GET_RESOURCE, BRP_METHOD_INSERT, BRP_METHOD_INSERT_RESOURCE,
    BRP_METHOD_LIST, BRP_METHOD_LIST_RESOURCES, BRP_METHOD_MUTATE_COMPONENT,
    BRP_METHOD_MUTATE_RESOURCE, BRP_METHOD_REMOVE, BRP_METHOD_REMOVE_RESOURCE,
    BRP_METHOD_RPC_DISCOVER, DESC_BEVY_SCREENSHOT, DESC_BRP_DESTROY,
    DESC_BRP_EXTRAS_DISCOVER_FORMAT, DESC_BRP_GET, DESC_BRP_GET_RESOURCE, DESC_BRP_INSERT,
    DESC_BRP_INSERT_RESOURCE, DESC_BRP_LIST, DESC_BRP_LIST_RESOURCES, DESC_BRP_MUTATE_COMPONENT,
    DESC_BRP_MUTATE_RESOURCE, DESC_BRP_REMOVE, DESC_BRP_REMOVE_RESOURCE, DESC_BRP_RPC_DISCOVER,
    TOOL_BRP_DESTROY, TOOL_BRP_EXTRAS_DISCOVER_FORMAT, TOOL_BRP_EXTRAS_SCREENSHOT, TOOL_BRP_GET,
    TOOL_BRP_GET_RESOURCE, TOOL_BRP_INSERT, TOOL_BRP_INSERT_RESOURCE, TOOL_BRP_LIST,
    TOOL_BRP_LIST_RESOURCES, TOOL_BRP_MUTATE_COMPONENT, TOOL_BRP_MUTATE_RESOURCE, TOOL_BRP_REMOVE,
    TOOL_BRP_REMOVE_RESOURCE, TOOL_BRP_RPC_DISCOVER,
};

/// Represents a parameter definition for a BRP tool
#[derive(Clone)]
pub struct ParamDef {
    /// Parameter name as it appears in the API
    pub name:        &'static str,
    /// Description of the parameter
    pub description: &'static str,
    /// Whether this parameter is required
    pub required:    bool,
    /// Type of the parameter
    pub param_type:  ParamType,
}

/// Types of parameters that can be defined
#[derive(Clone)]
pub enum ParamType {
    /// A numeric parameter (typically entity IDs or ports)
    Number,
    /// A string parameter
    String,
    /// A boolean parameter
    Boolean,
    /// An array of strings
    StringArray,
    /// Any JSON value (object, array, etc.)
    Any,
}

/// Defines how to format the response for a tool
#[derive(Clone)]
pub struct FormatterDef {
    /// Type of formatter to use
    pub formatter_type:  FormatterType,
    /// Template for success messages
    pub template:        &'static str,
    /// Fields to include in the response
    pub response_fields: Vec<ResponseField>,
}

/// Types of formatters available
#[derive(Clone)]
pub enum FormatterType {
    /// Entity operation formatter
    EntityOperation(&'static str),
    /// Resource operation formatter
    ResourceOperation,
    /// Simple formatter (no special formatting)
    Simple,
}

/// Defines a field to include in the response
#[derive(Clone)]
pub struct ResponseField {
    /// Name of the field in the response
    pub name:      &'static str,
    /// Type of extractor to use
    pub extractor: ExtractorType,
}

/// Types of extractors for response fields
#[derive(Clone)]
pub enum ExtractorType {
    /// Extract entity from params
    EntityFromParams,
    /// Extract resource from params
    ResourceFromParams,
    /// Pass through data from BRP response
    PassThroughData,
    /// Pass through entire result
    PassThroughResult,
    /// Extract entity count from data
    EntityCountFromData,
    /// Extract component count from data
    ComponentCountFromData,
    /// Extract entity from response data (for spawn operation)
    EntityFromResponse,
    /// Extract total component count from nested query results
    QueryComponentCount,
    /// Extract query parameters from request context
    QueryParamsFromContext,
    /// Extract specific parameter from request context
    ParamFromContext(&'static str),
}

/// Type of parameter extractor to use
#[derive(Clone)]
pub enum ParamExtractorType {
    /// Pass through all parameters
    Passthrough,
    /// Extract entity parameter
    Entity { required: bool },
    /// Extract resource parameter
    Resource,
    /// Extract empty params
    EmptyParams,
    /// Custom extractor for BRP execute (dynamic method)
    BrpExecute,
    /// Custom extractor for registry schema (parameter transformation)
    RegistrySchema,
}

/// Complete definition of a BRP tool
#[derive(Clone)]
pub struct BrpToolDef {
    /// Tool name (e.g., "`bevy_destroy`")
    pub name:            &'static str,
    /// Tool description
    pub description:     &'static str,
    /// BRP method to call (e.g., "bevy/destroy")
    pub method:          &'static str,
    /// Parameters for the tool
    pub params:          Vec<ParamDef>,
    /// Parameter extractor type
    pub param_extractor: ParamExtractorType,
    /// Response formatter definition
    pub formatter:       FormatterDef,
}

/// Get all standard tool definitions
#[allow(clippy::too_many_lines)]
pub fn get_standard_tools() -> Vec<BrpToolDef> {
    vec![
        // bevy_destroy
        BrpToolDef {
            name:            TOOL_BRP_DESTROY,
            description:     DESC_BRP_DESTROY,
            method:          BRP_METHOD_DESTROY,
            params:          vec![
                ParamDef {
                    name:        JSON_FIELD_ENTITY,
                    description: "The entity ID to destroy",
                    required:    true,
                    param_type:  ParamType::Number,
                },
                ParamDef {
                    name:        JSON_FIELD_PORT,
                    description: DESC_PORT,
                    required:    false,
                    param_type:  ParamType::Number,
                },
            ],
            param_extractor: ParamExtractorType::Entity { required: true },
            formatter:       FormatterDef {
                formatter_type:  FormatterType::EntityOperation(JSON_FIELD_DESTROYED_ENTITY),
                template:        "Successfully destroyed entity {entity}",
                response_fields: vec![ResponseField {
                    name:      JSON_FIELD_DESTROYED_ENTITY,
                    extractor: ExtractorType::EntityFromParams,
                }],
            },
        },
        // bevy_get
        BrpToolDef {
            name:            TOOL_BRP_GET,
            description:     DESC_BRP_GET,
            method:          BRP_METHOD_GET,
            params:          vec![
                ParamDef {
                    name:        JSON_FIELD_ENTITY,
                    description: "The entity ID to get component data from",
                    required:    true,
                    param_type:  ParamType::Number,
                },
                ParamDef {
                    name:        JSON_FIELD_COMPONENTS,
                    description: "Array of component types to retrieve. Each component must be a fully-qualified type name",
                    required:    true,
                    param_type:  ParamType::Any,
                },
                ParamDef {
                    name:        JSON_FIELD_PORT,
                    description: DESC_PORT,
                    required:    false,
                    param_type:  ParamType::Number,
                },
            ],
            param_extractor: ParamExtractorType::Passthrough,
            formatter:       FormatterDef {
                formatter_type:  FormatterType::EntityOperation(JSON_FIELD_ENTITY),
                template:        "Retrieved component data from entity {entity}",
                response_fields: vec![
                    ResponseField {
                        name:      JSON_FIELD_ENTITY,
                        extractor: ExtractorType::EntityFromParams,
                    },
                    ResponseField {
                        name:      JSON_FIELD_COMPONENTS,
                        extractor: ExtractorType::PassThroughData,
                    },
                ],
            },
        },
        // bevy_list
        BrpToolDef {
            name:            TOOL_BRP_LIST,
            description:     DESC_BRP_LIST,
            method:          BRP_METHOD_LIST,
            params:          vec![
                ParamDef {
                    name:        JSON_FIELD_ENTITY,
                    description: "Optional entity ID to list components for",
                    required:    false,
                    param_type:  ParamType::Number,
                },
                ParamDef {
                    name:        JSON_FIELD_PORT,
                    description: DESC_PORT,
                    required:    false,
                    param_type:  ParamType::Number,
                },
            ],
            param_extractor: ParamExtractorType::Entity { required: false },
            formatter:       FormatterDef {
                formatter_type:  FormatterType::Simple,
                template:        "Listed {count} components",
                response_fields: vec![
                    ResponseField {
                        name:      JSON_FIELD_COMPONENTS,
                        extractor: ExtractorType::PassThroughData,
                    },
                    ResponseField {
                        name:      JSON_FIELD_COUNT,
                        extractor: ExtractorType::ComponentCountFromData,
                    },
                ],
            },
        },
        // bevy_remove
        BrpToolDef {
            name:            TOOL_BRP_REMOVE,
            description:     DESC_BRP_REMOVE,
            method:          BRP_METHOD_REMOVE,
            params:          vec![
                ParamDef {
                    name:        JSON_FIELD_ENTITY,
                    description: "The entity ID to remove components from",
                    required:    true,
                    param_type:  ParamType::Number,
                },
                ParamDef {
                    name:        JSON_FIELD_COMPONENTS,
                    description: "Array of component type names to remove",
                    required:    true,
                    param_type:  ParamType::Any,
                },
                ParamDef {
                    name:        JSON_FIELD_PORT,
                    description: DESC_PORT,
                    required:    false,
                    param_type:  ParamType::Number,
                },
            ],
            param_extractor: ParamExtractorType::Passthrough,
            formatter:       FormatterDef {
                formatter_type:  FormatterType::EntityOperation(JSON_FIELD_ENTITY),
                template:        "Successfully removed components from entity {entity}",
                response_fields: vec![ResponseField {
                    name:      JSON_FIELD_ENTITY,
                    extractor: ExtractorType::EntityFromParams,
                }],
            },
        },
        // bevy_insert
        BrpToolDef {
            name:            TOOL_BRP_INSERT,
            description:     DESC_BRP_INSERT,
            method:          BRP_METHOD_INSERT,
            params:          vec![
                ParamDef {
                    name:        JSON_FIELD_ENTITY,
                    description: "The entity ID to insert components into",
                    required:    true,
                    param_type:  ParamType::Number,
                },
                ParamDef {
                    name:        JSON_FIELD_COMPONENTS,
                    description: "Object containing component data to insert. Keys are component types, values are component data. Note: Math types use array format - Vec2: [x,y], Vec3: [x,y,z], Vec4/Quat: [x,y,z,w], not objects with named fields.",
                    required:    true,
                    param_type:  ParamType::Any,
                },
                ParamDef {
                    name:        JSON_FIELD_PORT,
                    description: DESC_PORT,
                    required:    false,
                    param_type:  ParamType::Number,
                },
            ],
            param_extractor: ParamExtractorType::Passthrough,
            formatter:       FormatterDef {
                formatter_type:  FormatterType::EntityOperation(JSON_FIELD_ENTITY),
                template:        "Successfully inserted components into entity {entity}",
                response_fields: vec![ResponseField {
                    name:      JSON_FIELD_ENTITY,
                    extractor: ExtractorType::EntityFromParams,
                }],
            },
        },
        // bevy_get_resource
        BrpToolDef {
            name:            TOOL_BRP_GET_RESOURCE,
            description:     DESC_BRP_GET_RESOURCE,
            method:          BRP_METHOD_GET_RESOURCE,
            params:          vec![
                ParamDef {
                    name:        JSON_FIELD_RESOURCE,
                    description: "The fully-qualified type name of the resource to get",
                    required:    true,
                    param_type:  ParamType::String,
                },
                ParamDef {
                    name:        JSON_FIELD_PORT,
                    description: DESC_PORT,
                    required:    false,
                    param_type:  ParamType::Number,
                },
            ],
            param_extractor: ParamExtractorType::Resource,
            formatter:       FormatterDef {
                formatter_type:  FormatterType::ResourceOperation,
                template:        "Retrieved resource: {resource}",
                response_fields: vec![
                    ResponseField {
                        name:      JSON_FIELD_RESOURCE,
                        extractor: ExtractorType::ResourceFromParams,
                    },
                    ResponseField {
                        name:      JSON_FIELD_DATA,
                        extractor: ExtractorType::PassThroughData,
                    },
                ],
            },
        },
        // bevy_insert_resource
        BrpToolDef {
            name:            TOOL_BRP_INSERT_RESOURCE,
            description:     DESC_BRP_INSERT_RESOURCE,
            method:          BRP_METHOD_INSERT_RESOURCE,
            params:          vec![
                ParamDef {
                    name:        JSON_FIELD_RESOURCE,
                    description: "The fully-qualified type name of the resource to insert or update",
                    required:    true,
                    param_type:  ParamType::String,
                },
                ParamDef {
                    name:        JSON_FIELD_VALUE,
                    description: "The resource value to insert. Note: Math types use array format - Vec2: [x,y], Vec3: [x,y,z], Vec4/Quat: [x,y,z,w], not objects with named fields.",
                    required:    true,
                    param_type:  ParamType::Any,
                },
                ParamDef {
                    name:        JSON_FIELD_PORT,
                    description: DESC_PORT,
                    required:    false,
                    param_type:  ParamType::Number,
                },
            ],
            param_extractor: ParamExtractorType::Passthrough,
            formatter:       FormatterDef {
                formatter_type:  FormatterType::ResourceOperation,
                template:        "Successfully inserted/updated resource: {resource}",
                response_fields: vec![ResponseField {
                    name:      JSON_FIELD_RESOURCE,
                    extractor: ExtractorType::ResourceFromParams,
                }],
            },
        },
        // bevy_remove_resource
        BrpToolDef {
            name:            TOOL_BRP_REMOVE_RESOURCE,
            description:     DESC_BRP_REMOVE_RESOURCE,
            method:          BRP_METHOD_REMOVE_RESOURCE,
            params:          vec![
                ParamDef {
                    name:        JSON_FIELD_RESOURCE,
                    description: "The fully-qualified type name of the resource to remove",
                    required:    true,
                    param_type:  ParamType::String,
                },
                ParamDef {
                    name:        JSON_FIELD_PORT,
                    description: DESC_PORT,
                    required:    false,
                    param_type:  ParamType::Number,
                },
            ],
            param_extractor: ParamExtractorType::Resource,
            formatter:       FormatterDef {
                formatter_type:  FormatterType::ResourceOperation,
                template:        "Successfully removed resource: {resource}",
                response_fields: vec![ResponseField {
                    name:      JSON_FIELD_RESOURCE,
                    extractor: ExtractorType::ResourceFromParams,
                }],
            },
        },
        // bevy_mutate_component
        BrpToolDef {
            name:            TOOL_BRP_MUTATE_COMPONENT,
            description:     DESC_BRP_MUTATE_COMPONENT,
            method:          BRP_METHOD_MUTATE_COMPONENT,
            params:          vec![
                ParamDef {
                    name:        JSON_FIELD_ENTITY,
                    description: "The entity ID containing the component to mutate",
                    required:    true,
                    param_type:  ParamType::Number,
                },
                ParamDef {
                    name:        JSON_FIELD_COMPONENT,
                    description: "The fully-qualified type name of the component to mutate",
                    required:    true,
                    param_type:  ParamType::String,
                },
                ParamDef {
                    name:        JSON_FIELD_PATH,
                    description: "The path to the field within the component (e.g., 'translation.x')",
                    required:    true,
                    param_type:  ParamType::String,
                },
                ParamDef {
                    name:        JSON_FIELD_VALUE,
                    description: "The new value for the field. Note: Math types use array format - Vec2: [x,y], Vec3: [x,y,z], Vec4/Quat: [x,y,z,w], not objects with named fields.",
                    required:    true,
                    param_type:  ParamType::Any,
                },
                ParamDef {
                    name:        JSON_FIELD_PORT,
                    description: DESC_PORT,
                    required:    false,
                    param_type:  ParamType::Number,
                },
            ],
            param_extractor: ParamExtractorType::Passthrough,
            formatter:       FormatterDef {
                formatter_type:  FormatterType::EntityOperation(JSON_FIELD_ENTITY),
                template:        "Successfully mutated component on entity {entity}",
                response_fields: vec![ResponseField {
                    name:      JSON_FIELD_ENTITY,
                    extractor: ExtractorType::EntityFromParams,
                }],
            },
        },
        // bevy_mutate_resource
        BrpToolDef {
            name:            TOOL_BRP_MUTATE_RESOURCE,
            description:     DESC_BRP_MUTATE_RESOURCE,
            method:          BRP_METHOD_MUTATE_RESOURCE,
            params:          vec![
                ParamDef {
                    name:        JSON_FIELD_RESOURCE,
                    description: "The fully-qualified type name of the resource to mutate",
                    required:    true,
                    param_type:  ParamType::String,
                },
                ParamDef {
                    name:        JSON_FIELD_PATH,
                    description: "The path to the field within the resource (e.g., 'settings.volume')",
                    required:    true,
                    param_type:  ParamType::String,
                },
                ParamDef {
                    name:        JSON_FIELD_VALUE,
                    description: "The new value for the field. Note: Math types use array format - Vec2: [x,y], Vec3: [x,y,z], Vec4/Quat: [x,y,z,w], not objects with named fields.",
                    required:    true,
                    param_type:  ParamType::Any,
                },
                ParamDef {
                    name:        JSON_FIELD_PORT,
                    description: DESC_PORT,
                    required:    false,
                    param_type:  ParamType::Number,
                },
            ],
            param_extractor: ParamExtractorType::Passthrough,
            formatter:       FormatterDef {
                formatter_type:  FormatterType::ResourceOperation,
                template:        "Successfully mutated resource: {resource}",
                response_fields: vec![ResponseField {
                    name:      JSON_FIELD_RESOURCE,
                    extractor: ExtractorType::ResourceFromParams,
                }],
            },
        },
        // bevy_list_resources
        BrpToolDef {
            name:            TOOL_BRP_LIST_RESOURCES,
            description:     DESC_BRP_LIST_RESOURCES,
            method:          BRP_METHOD_LIST_RESOURCES,
            params:          vec![ParamDef {
                name:        JSON_FIELD_PORT,
                description: DESC_PORT,
                required:    false,
                param_type:  ParamType::Number,
            }],
            param_extractor: ParamExtractorType::EmptyParams,
            formatter:       FormatterDef {
                formatter_type:  FormatterType::Simple,
                template:        "Listed {count} resources",
                response_fields: vec![
                    ResponseField {
                        name:      JSON_FIELD_RESOURCES,
                        extractor: ExtractorType::PassThroughData,
                    },
                    ResponseField {
                        name:      JSON_FIELD_COUNT,
                        extractor: ExtractorType::ComponentCountFromData,
                    },
                ],
            },
        },
        // bevy_rpc_discover
        BrpToolDef {
            name:            TOOL_BRP_RPC_DISCOVER,
            description:     DESC_BRP_RPC_DISCOVER,
            method:          BRP_METHOD_RPC_DISCOVER,
            params:          vec![ParamDef {
                name:        JSON_FIELD_PORT,
                description: DESC_PORT,
                required:    false,
                param_type:  ParamType::Number,
            }],
            param_extractor: ParamExtractorType::EmptyParams,
            formatter:       FormatterDef {
                formatter_type:  FormatterType::Simple,
                template:        "Retrieved BRP method discovery information",
                response_fields: vec![ResponseField {
                    name:      JSON_FIELD_METADATA,
                    extractor: ExtractorType::PassThroughResult,
                }],
            },
        },
        // bevy_brp_extras/discover_format
        BrpToolDef {
            name:            TOOL_BRP_EXTRAS_DISCOVER_FORMAT,
            description:     DESC_BRP_EXTRAS_DISCOVER_FORMAT,
            method:          BRP_METHOD_EXTRAS_DISCOVER_FORMAT,
            params:          vec![
                ParamDef {
                    name:        PARAM_TYPES,
                    description: "Array of fully-qualified component type names to discover formats for",
                    required:    true,
                    param_type:  ParamType::StringArray,
                },
                ParamDef {
                    name:        JSON_FIELD_PORT,
                    description: DESC_PORT,
                    required:    false,
                    param_type:  ParamType::Number,
                },
            ],
            param_extractor: ParamExtractorType::Passthrough,
            formatter:       FormatterDef {
                formatter_type:  FormatterType::Simple,
                template:        "Format discovery completed",
                response_fields: vec![ResponseField {
                    name:      PARAM_FORMATS,
                    extractor: ExtractorType::PassThroughData,
                }],
            },
        },
        // bevy_screenshot
        BrpToolDef {
            name:            TOOL_BRP_EXTRAS_SCREENSHOT,
            description:     DESC_BEVY_SCREENSHOT,
            method:          BRP_METHOD_EXTRAS_SCREENSHOT,
            params:          vec![
                ParamDef {
                    name:        JSON_FIELD_PATH,
                    description: "File path where the screenshot should be saved",
                    required:    true,
                    param_type:  ParamType::String,
                },
                ParamDef {
                    name:        JSON_FIELD_PORT,
                    description: DESC_PORT,
                    required:    false,
                    param_type:  ParamType::Number,
                },
            ],
            param_extractor: ParamExtractorType::Passthrough,
            formatter:       FormatterDef {
                formatter_type:  FormatterType::Simple,
                template:        "Successfully captured screenshot and saved to {path}",
                response_fields: vec![
                    ResponseField {
                        name:      JSON_FIELD_PATH,
                        extractor: ExtractorType::ParamFromContext(JSON_FIELD_PATH),
                    },
                    ResponseField {
                        name:      JSON_FIELD_PORT,
                        extractor: ExtractorType::ParamFromContext(JSON_FIELD_PORT),
                    },
                ],
            },
        },
    ]
}

/// Get tool definitions for tools with special variations
#[allow(clippy::too_many_lines)]
pub fn get_special_tools() -> Vec<BrpToolDef> {
    vec![
        // bevy_query - has custom extractors for component counts
        BrpToolDef {
            name:            crate::tools::TOOL_BEVY_QUERY,
            description:     "Query entities using the bevy/query BRP method. This powerful tool allows you to search for entities based on their components, applying filters and returning component data. This tool wraps the bevy/query method for easier use.",
            method:          crate::tools::BRP_METHOD_QUERY,
            params:          vec![
                ParamDef {
                    name:        PARAM_DATA,
                    description: "Object specifying what component data to retrieve. Properties: components (array), option (array), has (array)",
                    required:    true,
                    param_type:  ParamType::Any,
                },
                ParamDef {
                    name:        PARAM_FILTER,
                    description: "Object specifying which entities to query. Properties: with (array), without (array)",
                    required:    true,
                    param_type:  ParamType::Any,
                },
                ParamDef {
                    name:        PARAM_STRICT,
                    description: "If true, returns error on unknown component types (default: false)",
                    required:    false,
                    param_type:  ParamType::Boolean,
                },
                ParamDef {
                    name:        JSON_FIELD_PORT,
                    description: DESC_PORT,
                    required:    false,
                    param_type:  ParamType::Number,
                },
            ],
            param_extractor: ParamExtractorType::Passthrough,
            formatter:       FormatterDef {
                formatter_type:  FormatterType::Simple,
                template:        "Query completed successfully",
                response_fields: vec![
                    ResponseField {
                        name:      JSON_FIELD_DATA,
                        extractor: ExtractorType::PassThroughData,
                    },
                    ResponseField {
                        name:      PARAM_ENTITY_COUNT,
                        extractor: ExtractorType::EntityCountFromData,
                    },
                    ResponseField {
                        name:      PARAM_COMPONENT_COUNT,
                        extractor: ExtractorType::QueryComponentCount,
                    },
                    ResponseField {
                        name:      PARAM_QUERY_PARAMS,
                        extractor: ExtractorType::QueryParamsFromContext,
                    },
                ],
            },
        },
        // bevy_spawn - has dynamic entity extraction from response
        BrpToolDef {
            name:            crate::tools::TOOL_BEVY_SPAWN,
            description:     "Spawn a new entity with components using the bevy/spawn BRP method. Creates a new entity in the Bevy world with the specified components.",
            method:          crate::tools::BRP_METHOD_SPAWN,
            params:          vec![
                ParamDef {
                    name:        JSON_FIELD_COMPONENTS,
                    description: "Object containing component data to spawn with. Keys are component types, values are component data. Note: Math types use array format - Vec2: [x,y], Vec3: [x,y,z], Vec4/Quat: [x,y,z,w], not objects with named fields.",
                    required:    false,
                    param_type:  ParamType::Any,
                },
                ParamDef {
                    name:        JSON_FIELD_PORT,
                    description: DESC_PORT,
                    required:    false,
                    param_type:  ParamType::Number,
                },
            ],
            param_extractor: ParamExtractorType::Passthrough,
            formatter:       FormatterDef {
                formatter_type:  FormatterType::EntityOperation(PARAM_SPAWNED_ENTITY),
                template:        "Successfully spawned entity",
                response_fields: vec![
                    ResponseField {
                        name:      PARAM_SPAWNED_ENTITY,
                        extractor: ExtractorType::EntityFromResponse,
                    },
                    ResponseField {
                        name:      JSON_FIELD_COMPONENTS,
                        extractor: ExtractorType::ParamFromContext(JSON_FIELD_COMPONENTS),
                    },
                ],
            },
        },
        // brp_execute - has dynamic method selection
        BrpToolDef {
            name:            crate::tools::TOOL_BRP_EXECUTE,
            description:     "Execute any Bevy Remote Protocol (BRP) method on a running Bevy app. This tool allows you to send arbitrary BRP commands and receive responses.",
            method:          "", // Dynamic method
            params:          vec![
                ParamDef {
                    name:        PARAM_METHOD,
                    description: "The BRP method to execute (e.g., 'rpc.discover', 'bevy/get', 'bevy/query')",
                    required:    true,
                    param_type:  ParamType::String,
                },
                ParamDef {
                    name:        PARAM_PARAMS,
                    description: "Optional parameters for the method, as a JSON object or array",
                    required:    false,
                    param_type:  ParamType::Any,
                },
                ParamDef {
                    name:        JSON_FIELD_PORT,
                    description: DESC_PORT,
                    required:    false,
                    param_type:  ParamType::Number,
                },
            ],
            param_extractor: ParamExtractorType::BrpExecute,
            formatter:       FormatterDef {
                formatter_type:  FormatterType::Simple,
                template:        "Method executed successfully",
                response_fields: vec![ResponseField {
                    name:      PARAM_RESULT,
                    extractor: ExtractorType::PassThroughResult,
                }],
            },
        },
        // bevy_registry_schema - has complex parameter transformation
        BrpToolDef {
            name:            crate::tools::TOOL_BEVY_REGISTRY_SCHEMA,
            description:     "Get JSON schema information for registered types using the bevy/registry/schema BRP method. Retrieves type schema definitions from the Bevy app's reflection registry.",
            method:          crate::tools::BRP_METHOD_REGISTRY_SCHEMA,
            params:          vec![
                ParamDef {
                    name:        PARAM_WITH_CRATES,
                    description: "Include only types from these crates (e.g., [\"bevy_transform\", \"my_game\"])",
                    required:    false,
                    param_type:  ParamType::StringArray,
                },
                ParamDef {
                    name:        PARAM_WITHOUT_CRATES,
                    description: "Exclude types from these crates (e.g., [\"bevy_render\", \"bevy_pbr\"])",
                    required:    false,
                    param_type:  ParamType::StringArray,
                },
                ParamDef {
                    name:        PARAM_WITH_TYPES,
                    description: "Include only types with these reflect traits (e.g., [\"Component\", \"Resource\"])",
                    required:    false,
                    param_type:  ParamType::StringArray,
                },
                ParamDef {
                    name:        PARAM_WITHOUT_TYPES,
                    description: "Exclude types with these reflect traits (e.g., [\"RenderResource\"])",
                    required:    false,
                    param_type:  ParamType::StringArray,
                },
                ParamDef {
                    name:        JSON_FIELD_PORT,
                    description: DESC_PORT,
                    required:    false,
                    param_type:  ParamType::Number,
                },
            ],
            param_extractor: ParamExtractorType::RegistrySchema,
            formatter:       FormatterDef {
                formatter_type:  FormatterType::Simple,
                template:        "Retrieved schema information",
                response_fields: vec![ResponseField {
                    name:      JSON_FIELD_DATA,
                    extractor: ExtractorType::PassThroughData,
                }],
            },
        },
        // bevy_reparent - has array parameter handling
        BrpToolDef {
            name:            crate::tools::TOOL_BEVY_REPARENT,
            description:     "Change the parent of an entity using the bevy/reparent BRP method. Modifies the hierarchical relationship between entities by setting or removing parent-child relationships.",
            method:          crate::tools::BRP_METHOD_REPARENT,
            params:          vec![
                ParamDef {
                    name:        PARAM_ENTITIES,
                    description: "Array of entity IDs to reparent",
                    required:    true,
                    param_type:  ParamType::Any,
                },
                ParamDef {
                    name:        PARAM_PARENT,
                    description: "The new parent entity ID (omit to remove parent)",
                    required:    false,
                    param_type:  ParamType::Number,
                },
                ParamDef {
                    name:        JSON_FIELD_PORT,
                    description: DESC_PORT,
                    required:    false,
                    param_type:  ParamType::Number,
                },
            ],
            param_extractor: ParamExtractorType::Passthrough,
            formatter:       FormatterDef {
                formatter_type:  FormatterType::Simple,
                template:        "Successfully reparented entities",
                response_fields: vec![
                    ResponseField {
                        name:      PARAM_ENTITIES,
                        extractor: ExtractorType::ParamFromContext(PARAM_ENTITIES),
                    },
                    ResponseField {
                        name:      PARAM_PARENT,
                        extractor: ExtractorType::ParamFromContext(PARAM_PARENT),
                    },
                ],
            },
        },
    ]
}
