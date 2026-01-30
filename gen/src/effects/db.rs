use thiserror::Error;
#[derive(Debug, Error)]
pub enum DbError {
    #[error("Database error: {0}")]
    Database(String),
    #[error("Not found")]
    NotFound,
}
pub async fn read<T>(table: &str, query: &impl serde::Serialize) -> Result<T, DbError>
where
    T: serde::de::DeserializeOwned,
{
    todo!("Implement database read")
}
pub async fn write(table: &str, data: &impl serde::Serialize) -> Result<(), DbError> {
    todo!("Implement database write")
}
pub async fn delete(table: &str, query: &impl serde::Serialize) -> Result<(), DbError> {
    todo!("Implement database delete")
}
