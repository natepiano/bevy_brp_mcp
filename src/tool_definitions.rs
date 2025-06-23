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
//! # Tool Types and Handler Support
//!
//! The system supports two handler types:
//!
//! ## BRP Handlers (`HandlerType::Brp`)
//! Execute remote BRP method calls over network. Used for most core Bevy operations.
//! Example: `bevy/destroy`, `bevy/get`, `bevy/insert`
//!
//! ## Local Handlers (`HandlerType::Local`)
//! Execute local functions within the MCP server. Used for log management, app lifecycle, etc.
//! Example: `list_logs`, `launch_bevy_app`, `cleanup_logs`
//!
//! # Response Formatting
//!
//! ## `FormatterDef::default()`
//! For local tools that don't need special response formatting. Returns empty formatter
//! with `FormatterType::Simple`, empty template, and no response fields.
//!
//! ## Custom Formatters
//! For BRP tools that need structured responses with field extraction and templating.
//!
//! # Helper Functions
//!
//! Common parameter patterns are available as helper functions:
//! - `add_port_param()`: Standard port parameter (optional, numeric)
//! - `add_entity_param()`: Entity ID parameter (required, numeric)
//! - `add_components_param()`: Component types array parameter (required, any)
//!
//! # Adding New Tools
//!
//! ## Standard BRP Tools
//! 1. **Define constants** in `constants.rs` for the tool name, description, and BRP method
//! 2. **Add tool definition** to `get_standard_tools()` with:
//!    - `HandlerType::Brp { method: "bevy/method_name" }`
//!    - Appropriate parameter extractors
//!    - Response formatters with field extraction
//! 3. **Registration is automatic** via `get_all_tools()`
//!
//! ## Local Tools
//! 1. **Add tool definition** to `get_log_tools()` or `get_app_tools()` with:
//!    - `HandlerType::Local { handler: "function_name" }`
//!    - `FormatterDef::default()` for simple responses
//! 2. **Implement handler** in `tool_generator.rs` `generate_local_handler()` match
//! 3. **Create handler function** in appropriate module (e.g., `log_tools::function_name`)
//!
//! ## Special BRP Tools
//! For tools with custom parameter extraction or response formatting:
//! 1. **Add to `get_special_tools()`** instead of standard tools
//! 2. **Use custom `ParamExtractorType`** (e.g., `BrpExecute`, `RegistrySchema`)
//! 3. **Implement custom extractors** in `tool_generator.rs` if needed
//!
//! # Best Practices
//!
//! - **Use helper functions** for common parameters (`add_port_param()`, etc.)
//! - **Use `FormatterDef::default()`** for local tools
//! - **Group related tools** in appropriate getter functions
//! - **Prefer declarative definitions** over custom handlers when possible
//! - **Add unit tests** for new parameter extractors and formatters
//!
//! # Example: Adding a New BRP Tool
//!
//! ```rust
//! // 1. Add to get_standard_tools()
//! BrpToolDef {
//!     name:            "bevy_new_operation",
//!     description:     "Performs a new operation",
//!     handler:         HandlerType::Brp {
//!         method: "bevy/new_operation",
//!     },
//!     params:          vec![
//!         add_entity_param(), // Use helper for common params
//!         add_port_param(),
//!     ],
//!     param_extractor: ParamExtractorType::Entity { required: true },
//!     formatter:       FormatterDef {
//!         formatter_type:  FormatterType::EntityOperation("entity"),
//!         template:        "Operation completed on entity {entity}",
//!         response_fields: vec![ResponseField {
//!             name:      "entity",
//!             extractor: ExtractorType::EntityFromParams,
//!         }],
//!     },
//! }
//! ```
//!
//! # Example: Adding a New Local Tool
//!
//! ```rust
//! // 1. Add to get_log_tools() or get_app_tools()
//! BrpToolDef {
//!     name: "my_local_tool",
//!     description: "Does something locally",
//!     handler: HandlerType::Local { handler: "my_function" },
//!     params: vec![
//!         ParamDef {
//!             name: "input",
//!             description: "Input parameter",
//!             required: true,
//!             param_type: ParamType::String,
//!         }
//!     ],
//!     param_extractor: ParamExtractorType::Passthrough,
//!     formatter: FormatterDef::default(), // Simple local tool
//! }
//!
//! // 2. Add handler to tool_generator.rs generate_local_handler()
//! "my_function" => my_module::my_function::handle(service, request, context),
//!
//! // 3. Implement in my_module::my_function
//! pub fn handle(
//!     service: &BrpMcpService,
//!     request: &CallToolRequestParam,
//!     context: RequestContext<RoleServer>
//! ) -> Result<CallToolResult, McpError> {
//!     // Implementation
//! }
//! ```

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
    BRP_METHOD_RPC_DISCOVER, DESC_BEVY_DESTROY, DESC_BEVY_GET, DESC_BEVY_GET_RESOURCE,
    DESC_BEVY_INSERT, DESC_BEVY_INSERT_RESOURCE, DESC_BEVY_LIST, DESC_BEVY_LIST_RESOURCES,
    DESC_BEVY_MUTATE_COMPONENT, DESC_BEVY_MUTATE_RESOURCE, DESC_BEVY_REMOVE,
    DESC_BEVY_REMOVE_RESOURCE, DESC_BEVY_RPC_DISCOVER, DESC_BRP_EXTRAS_DISCOVER_FORMAT,
    DESC_BRP_EXTRAS_SCREENSHOT, TOOL_BEVY_DESTROY, TOOL_BEVY_GET, TOOL_BEVY_GET_RESOURCE,
    TOOL_BEVY_INSERT, TOOL_BEVY_INSERT_RESOURCE, TOOL_BEVY_LIST, TOOL_BEVY_LIST_RESOURCES,
    TOOL_BEVY_MUTATE_COMPONENT, TOOL_BEVY_MUTATE_RESOURCE, TOOL_BEVY_REMOVE,
    TOOL_BEVY_REMOVE_RESOURCE, TOOL_BEVY_RPC_DISCOVER, TOOL_BRP_EXTRAS_DISCOVER_FORMAT,
    TOOL_BRP_EXTRAS_SCREENSHOT,
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

impl FormatterDef {
    /// Creates a default formatter for local tools that don't need special formatting
    pub const fn default() -> Self {
        Self {
            formatter_type:  FormatterType::Simple,
            template:        "",
            response_fields: vec![],
        }
    }
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

/// Type of handler for the tool
#[derive(Clone)]
pub enum HandlerType {
    /// BRP handler - calls a BRP method
    Brp {
        /// BRP method to call (e.g., "bevy/destroy")
        method: &'static str,
    },
    /// Local handler - executes local logic  
    Local {
        /// Handler function name (e.g., "`list_logs`", "`read_log`")
        handler: &'static str,
    },
}

/// Complete definition of a BRP tool
#[derive(Clone)]
pub struct BrpToolDef {
    /// Tool name (e.g., "`bevy_destroy`")
    pub name:            &'static str,
    /// Tool description
    pub description:     &'static str,
    /// Handler type (BRP or Local)
    pub handler:         HandlerType,
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
            name:            TOOL_BEVY_DESTROY,
            description:     DESC_BEVY_DESTROY,
            handler:         HandlerType::Brp {
                method: BRP_METHOD_DESTROY,
            },
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
            name:            TOOL_BEVY_GET,
            description:     DESC_BEVY_GET,
            handler:         HandlerType::Brp {
                method: BRP_METHOD_GET,
            },
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
            name:            TOOL_BEVY_LIST,
            description:     DESC_BEVY_LIST,
            handler:         HandlerType::Brp {
                method: BRP_METHOD_LIST,
            },
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
            name:            TOOL_BEVY_REMOVE,
            description:     DESC_BEVY_REMOVE,
            handler:         HandlerType::Brp {
                method: BRP_METHOD_REMOVE,
            },
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
            name:            TOOL_BEVY_INSERT,
            description:     DESC_BEVY_INSERT,
            handler:         HandlerType::Brp {
                method: BRP_METHOD_INSERT,
            },
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
            name:            TOOL_BEVY_GET_RESOURCE,
            description:     DESC_BEVY_GET_RESOURCE,
            handler:         HandlerType::Brp {
                method: BRP_METHOD_GET_RESOURCE,
            },
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
            name:            TOOL_BEVY_INSERT_RESOURCE,
            description:     DESC_BEVY_INSERT_RESOURCE,
            handler:         HandlerType::Brp {
                method: BRP_METHOD_INSERT_RESOURCE,
            },
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
            name:            TOOL_BEVY_REMOVE_RESOURCE,
            description:     DESC_BEVY_REMOVE_RESOURCE,
            handler:         HandlerType::Brp {
                method: BRP_METHOD_REMOVE_RESOURCE,
            },
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
            name:            TOOL_BEVY_MUTATE_COMPONENT,
            description:     DESC_BEVY_MUTATE_COMPONENT,
            handler:         HandlerType::Brp {
                method: BRP_METHOD_MUTATE_COMPONENT,
            },
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
            name:            TOOL_BEVY_MUTATE_RESOURCE,
            description:     DESC_BEVY_MUTATE_RESOURCE,
            handler:         HandlerType::Brp {
                method: BRP_METHOD_MUTATE_RESOURCE,
            },
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
            name:            TOOL_BEVY_LIST_RESOURCES,
            description:     DESC_BEVY_LIST_RESOURCES,
            handler:         HandlerType::Brp {
                method: BRP_METHOD_LIST_RESOURCES,
            },
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
            name:            TOOL_BEVY_RPC_DISCOVER,
            description:     DESC_BEVY_RPC_DISCOVER,
            handler:         HandlerType::Brp {
                method: BRP_METHOD_RPC_DISCOVER,
            },
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
            handler:         HandlerType::Brp {
                method: BRP_METHOD_EXTRAS_DISCOVER_FORMAT,
            },
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
            description:     DESC_BRP_EXTRAS_SCREENSHOT,
            handler:         HandlerType::Brp {
                method: BRP_METHOD_EXTRAS_SCREENSHOT,
            },
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
            description:     crate::tools::DESC_BEVY_QUERY,
            handler:         HandlerType::Brp {
                method: crate::tools::BRP_METHOD_QUERY,
            },
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
            description:     crate::tools::DESC_BEVY_SPAWN,
            handler:         HandlerType::Brp {
                method: crate::tools::BRP_METHOD_SPAWN,
            },
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
            description:     crate::tools::DESC_BRP_EXECUTE,
            handler:         HandlerType::Brp { method: "" }, // Dynamic method
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
            description:     crate::tools::DESC_BEVY_REGISTRY_SCHEMA,
            handler:         HandlerType::Brp {
                method: crate::tools::BRP_METHOD_REGISTRY_SCHEMA,
            },
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
            description:     crate::tools::DESC_BEVY_REPARENT,
            handler:         HandlerType::Brp {
                method: crate::tools::BRP_METHOD_REPARENT,
            },
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

/// Get log tool definitions
pub fn get_log_tools() -> Vec<BrpToolDef> {
    vec![
        // list_logs
        BrpToolDef {
            name:            crate::tools::TOOL_LIST_LOGS,
            description:     crate::tools::DESC_LIST_LOGS,
            handler:         HandlerType::Local {
                handler: "list_logs",
            },
            params:          vec![ParamDef {
                name:        "app_name",
                description: "Optional filter to list logs for a specific app only",
                required:    false,
                param_type:  ParamType::String,
            }],
            param_extractor: ParamExtractorType::Passthrough,
            formatter:       FormatterDef::default(),
        },
        // read_log
        BrpToolDef {
            name:            crate::tools::TOOL_READ_LOG,
            description:     crate::tools::DESC_READ_LOG,
            handler:         HandlerType::Local {
                handler: "read_log",
            },
            params:          vec![
                ParamDef {
                    name:        "filename",
                    description: "The log filename (e.g., bevy_brp_mcp_myapp_1234567890.log)",
                    required:    true,
                    param_type:  ParamType::String,
                },
                ParamDef {
                    name:        "keyword",
                    description: "Optional keyword to filter lines (case-insensitive)",
                    required:    false,
                    param_type:  ParamType::String,
                },
                ParamDef {
                    name:        "tail_lines",
                    description: "Optional number of lines to read from the end of file",
                    required:    false,
                    param_type:  ParamType::Number,
                },
            ],
            param_extractor: ParamExtractorType::Passthrough,
            formatter:       FormatterDef::default(),
        },
        // cleanup_logs
        BrpToolDef {
            name:            crate::tools::TOOL_CLEANUP_LOGS,
            description:     crate::tools::DESC_CLEANUP_LOGS,
            handler:         HandlerType::Local {
                handler: "cleanup_logs",
            },
            params:          vec![
                ParamDef {
                    name:        "app_name",
                    description: "Optional filter to delete logs for a specific app only",
                    required:    false,
                    param_type:  ParamType::String,
                },
                ParamDef {
                    name:        "older_than_seconds",
                    description: "Optional filter to delete logs older than N seconds",
                    required:    false,
                    param_type:  ParamType::Number,
                },
            ],
            param_extractor: ParamExtractorType::Passthrough,
            formatter:       FormatterDef::default(),
        },
    ]
}

/// Get app tool definitions
pub fn get_app_tools() -> Vec<BrpToolDef> {
    vec![
        // list_bevy_apps
        BrpToolDef {
            name:            crate::tools::TOOL_LIST_BEVY_APPS,
            description:     crate::tools::DESC_LIST_BEVY_APPS,
            handler:         HandlerType::Local {
                handler: "list_bevy_apps",
            },
            params:          vec![],
            param_extractor: ParamExtractorType::EmptyParams,
            formatter:       FormatterDef::default(),
        },
        // list_brp_apps
        BrpToolDef {
            name:            crate::tools::TOOL_LIST_BRP_APPS,
            description:     crate::tools::DESC_LIST_BRP_APPS,
            handler:         HandlerType::Local {
                handler: "list_brp_apps",
            },
            params:          vec![],
            param_extractor: ParamExtractorType::EmptyParams,
            formatter:       FormatterDef::default(),
        },
        // list_bevy_examples
        BrpToolDef {
            name:            crate::tools::TOOL_LIST_BEVY_EXAMPLES,
            description:     crate::tools::DESC_LIST_BEVY_EXAMPLES,
            handler:         HandlerType::Local {
                handler: "list_bevy_examples",
            },
            params:          vec![],
            param_extractor: ParamExtractorType::EmptyParams,
            formatter:       FormatterDef::default(),
        },
        // launch_bevy_app
        BrpToolDef {
            name:            crate::tools::TOOL_LAUNCH_BEVY_APP,
            description:     crate::tools::DESC_LAUNCH_BEVY_APP,
            handler:         HandlerType::Local {
                handler: "launch_bevy_app",
            },
            params:          vec![
                ParamDef {
                    name:        "app_name",
                    description: "Name of the Bevy app to launch",
                    required:    true,
                    param_type:  ParamType::String,
                },
                ParamDef {
                    name:        "profile",
                    description: "Build profile to use (debug or release)",
                    required:    false,
                    param_type:  ParamType::String,
                },
            ],
            param_extractor: ParamExtractorType::Passthrough,
            formatter:       FormatterDef::default(),
        },
        // launch_bevy_example
        BrpToolDef {
            name:            crate::tools::TOOL_LAUNCH_BEVY_EXAMPLE,
            description:     crate::tools::DESC_LAUNCH_BEVY_EXAMPLE,
            handler:         HandlerType::Local {
                handler: "launch_bevy_example",
            },
            params:          vec![
                ParamDef {
                    name:        "example_name",
                    description: "Name of the Bevy example to launch",
                    required:    true,
                    param_type:  ParamType::String,
                },
                ParamDef {
                    name:        "profile",
                    description: "Build profile to use (debug or release)",
                    required:    false,
                    param_type:  ParamType::String,
                },
            ],
            param_extractor: ParamExtractorType::Passthrough,
            formatter:       FormatterDef::default(),
        },
    ]
}

/// Get all tool definitions - combines standard, special, log, and app tools
pub fn get_all_tools() -> Vec<BrpToolDef> {
    let mut tools = Vec::new();

    // Add standard tools
    tools.extend(get_standard_tools());

    // Add special tools
    tools.extend(get_special_tools());

    // Add log tools
    tools.extend(get_log_tools());

    // Add app tools
    tools.extend(get_app_tools());

    tools
}

