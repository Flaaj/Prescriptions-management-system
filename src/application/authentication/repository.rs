use rocket::async_trait;

use super::entities::{NewUser, User};

#[derive(thiserror::Error, Debug, PartialEq)]
pub enum CreateUserRepositoryError {
    #[error("Database error: {0}")]
    DatabaseError(String),
}

#[derive(thiserror::Error, Debug, PartialEq)]
pub enum GetUserRepositoryError {
    #[error("User with this username not found ({0})")]
    NotFound(String),
    #[error("Database error: {0}")]
    DatabaseError(String),
}

#[async_trait]
pub trait AuthenticationRepository: Send + Sync + 'static {
    async fn create_user(&self, new_user: NewUser) -> Result<User, CreateUserRepositoryError>;
    async fn get_user_by_username<'a>(
        &self,
        username: &'a str,
    ) -> Result<User, GetUserRepositoryError>;
}
