// Local support modules for brp_tools

mod brp_client;
mod json_rpc_builder;
mod request_handler;
mod response_formatter;
mod watch_logger;
mod watch_response;
mod watch_task;

pub use json_rpc_builder::BrpJsonRpcBuilder;
pub use request_handler::{
    BrpExecuteExtractor, BrpHandlerConfig, EntityParamExtractor, ExtractedParams, ParamExtractor,
    PassthroughExtractor, SimplePortExtractor, handle_brp_request,
};
pub use response_formatter::{FieldExtractor, ResponseFormatterFactory, extractors};
pub use watch_response::{format_watch_start_response, format_watch_stop_response};
pub use watch_task::{start_entity_watch_task, start_list_watch_task};
