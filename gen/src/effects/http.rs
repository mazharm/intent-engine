use thiserror::Error;
#[derive(Debug, Error)]
pub enum HttpError {
    #[error("HTTP request failed: {0}")]
    Request(#[from] reqwest::Error),
    #[error("HTTP status error: {0}")]
    StatusError(u16),
}
pub async fn call(
    service: &str,
    operation: &str,
    request: &impl serde::Serialize,
) -> Result<serde_json::Value, HttpError> {
    match service {
        _ => Err(HttpError::StatusError(404)),
    }
}
