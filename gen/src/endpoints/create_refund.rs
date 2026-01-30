use axum::{extract::State, Json};
use crate::types::{RefundRequest, RefundResponse};
use crate::workflows::refund_workflow;
use crate::errors::CreateRefundError;
pub async fn create_refund(
    Json(input): Json<RefundRequest>,
) -> Result<Json<RefundResponse>, CreateRefundError> {
    let _timeout = std::time::Duration::from_millis(1500u32 as u64);
    let result = refund_workflow::refund_workflow(input).await?;
    Ok(Json(result))
}
