use serde_json::{Value, json};

use crate::brp_tools::constants::*;

pub struct BrpResponseBuilder {
    status:     &'static str,
    message:    String,
    data:       Option<Value>,
    error_code: Option<i64>,
    metadata:   Option<Value>,
}

impl BrpResponseBuilder {
    pub fn success(message: impl Into<String>) -> Self {
        Self {
            status:     RESPONSE_STATUS_SUCCESS,
            message:    message.into(),
            data:       None,
            error_code: None,
            metadata:   None,
        }
    }

    pub fn error(message: impl Into<String>) -> Self {
        Self {
            status:     RESPONSE_STATUS_ERROR,
            message:    message.into(),
            data:       None,
            error_code: None,
            metadata:   None,
        }
    }

    pub fn data(mut self, data: Value) -> Self {
        self.data = Some(data);
        self
    }

    pub fn error_code(mut self, code: i64) -> Self {
        self.error_code = Some(code);
        self
    }

    pub fn metadata(mut self, metadata: Value) -> Self {
        self.metadata = Some(metadata);
        self
    }

    pub fn build(self) -> Value {
        let mut response = json!({
            JSON_FIELD_STATUS: self.status,
            JSON_FIELD_MESSAGE: self.message,
        });

        if let Some(data) = self.data {
            response[JSON_FIELD_DATA] = data;
        }

        if let Some(code) = self.error_code {
            response[JSON_FIELD_ERROR_CODE] = json!(code);
        }

        if let Some(metadata) = self.metadata {
            response[JSON_FIELD_METADATA] = metadata;
        }

        response
    }
}
