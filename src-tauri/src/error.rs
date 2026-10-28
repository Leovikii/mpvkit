use serde::Serialize;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AppError {
    pub code: &'static str,
    pub message: String,
    pub phase: &'static str,
    pub retryable: bool,
}

impl AppError {
    pub fn new(code: &'static str, message: impl Into<String>, phase: &'static str) -> Self {
        Self {
            code,
            message: message.into(),
            phase,
            retryable: true,
        }
    }
}

pub type Result<T> = std::result::Result<T, AppError>;
