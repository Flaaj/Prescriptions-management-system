use std::sync::RwLock;

use chrono::Utc;
use rocket::async_trait;
use uuid::Uuid;

use super::entities::{NewSession, Session};

#[derive(thiserror::Error, Debug, PartialEq)]
pub enum CreateSessionRepositoryError {
    #[error("Database error: {0}")]
    DatabaseError(String),
}

#[derive(thiserror::Error, Debug, PartialEq)]
pub enum GetSessionRepositoryError {
    #[error("Session with this id not found ({0})")]
    NotFound(Uuid),
    #[error("Database error: {0}")]
    DatabaseError(String),
}

#[derive(thiserror::Error, Debug, PartialEq)]
pub enum UpdateSessionRepositoryError {
    #[error("Session with this id not found ({0})")]
    NotFound(Uuid),
    #[error("Database error: {0}")]
    DatabaseError(String),
}

#[async_trait]
pub trait SessionsRepository: Send + Sync + 'static {
    async fn create_session(
        &self,
        new_session: NewSession,
    ) -> Result<Session, CreateSessionRepositoryError>;
    async fn get_session_by_id(&self, id: Uuid) -> Result<Session, GetSessionRepositoryError>;
    async fn update_session(
        &self,
        session: Session,
    ) -> Result<Session, UpdateSessionRepositoryError>;
}
