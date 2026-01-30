use thiserror::Error;
#[derive(Debug, Error)]
pub enum EventError {
    #[error("Event publish failed: {0}")]
    Publish(String),
}
pub async fn emit(
    topic: &str,
    payload: &impl serde::Serialize,
) -> Result<(), EventError> {
    tracing::info!("Emitting event to topic: {}", topic);
    Ok(())
}
