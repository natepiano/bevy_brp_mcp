// Module organization
mod config;
mod extractors;
mod handler;
mod traits;

// Public exports
pub use config::{BrpHandlerConfig, FormatterContext};
pub use extractors::{
    BrpExecuteExtractor, EntityParamExtractor, PassthroughExtractor, SimplePortExtractor,
};
pub use handler::handle_brp_request;
pub use traits::{ParamExtractor, ExtractedParams};
