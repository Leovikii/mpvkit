use serde::Serialize;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AppError {
    pub code: &'static str,
    pub message: &'static str,
    pub phase: &'static str,
    pub retryable: bool,
}

impl AppError {
    pub fn new(code: &'static str, message: &'static str, phase: &'static str) -> Self {
        Self {
            code,
            message,
            phase,
            retryable: true,
        }
    }
}

pub type Result<T> = std::result::Result<T, AppError>;
