use axum::response::IntoResponse;
#[derive(Debug, thiserror::Error)]
pub enum CreateRefundError {
    #[error("invalid input")]
    InvalidInput,
    #[error("payment failed")]
    PaymentFailed,
    #[error("internal error: {0}")]
    Internal(#[from] anyhow::Error),
}
impl axum::response::IntoResponse for CreateRefundError {
    fn into_response(self) -> axum::response::Response {
        let status = match &self {
            Self::InvalidInput => {
                axum::http::StatusCode::from_u16(400u16)
                    .unwrap_or(axum::http::StatusCode::INTERNAL_SERVER_ERROR)
            }
            Self::PaymentFailed => {
                axum::http::StatusCode::from_u16(502u16)
                    .unwrap_or(axum::http::StatusCode::INTERNAL_SERVER_ERROR)
            }
            Self::Internal(_) => axum::http::StatusCode::INTERNAL_SERVER_ERROR,
        };
        let body = serde_json::json!({ "error" : self.to_string(), });
        (status, axum::Json(body)).into_response()
    }
}
