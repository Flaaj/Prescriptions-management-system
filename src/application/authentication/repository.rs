use std::sync::RwLock;

use chrono::Utc;
use rocket::async_trait;

use super::models::{NewUser, User};
use crate::domain::{doctors::models::Doctor, pharmacists::models::Pharmacist};

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

pub struct AuthenticationRepositoryFake {
    users: RwLock<Vec<User>>,
}

impl AuthenticationRepositoryFake {
    #[allow(dead_code)]
    pub fn new() -> Self {
        Self {
            users: RwLock::new(Vec::new()),
        }
    }
}

#[async_trait]
impl AuthenticationRepository for AuthenticationRepositoryFake {
    async fn create_user(&self, new_user: NewUser) -> Result<User, CreateUserRepositoryError> {
        let user = User {
            id: new_user.id,
            username: new_user.username,
            password_hash: new_user.password_hash,
            email: new_user.email,
            phone_number: new_user.phone_number,
            role: new_user.role,
            doctor: new_user.doctor_id.map(|id| Doctor {
                id,
                name: "Joe Doctor".to_string(),
                pwz_number: "8463856".to_string(),
                pesel_number: "92022900002".to_string(),
                created_at: Utc::now(),
                updated_at: Utc::now(),
            }),
            pharmacist: new_user.pharmacist_id.map(|id| Pharmacist {
                id,
                name: "Joe Pharmacist".to_string(),
                pesel_number: "92022900002".to_string(),
                created_at: Utc::now(),
                updated_at: Utc::now(),
            }),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        self.users.write().unwrap().push(user.clone());

        Ok(user)
    }

    async fn get_user_by_username<'a>(
        &self,
        username: &'a str,
    ) -> Result<User, GetUserRepositoryError> {
        match self
            .users
            .read()
            .unwrap()
            .iter()
            .find(|user| user.username == username)
        {
            Some(user) => return Ok(user.clone()),
            None => Err(GetUserRepositoryError::NotFound(username.to_owned())),
        }
    }
}

#[cfg(test)]
mod tests {
    use uuid::Uuid;

    use super::{AuthenticationRepository, AuthenticationRepositoryFake};
    use crate::application::authentication::models::{NewUser, UserRole};

    fn setup_repository() -> AuthenticationRepositoryFake {
        AuthenticationRepositoryFake::new()
    }

    fn create_mock_new_user() -> NewUser {
        NewUser::new(
            "username".to_string(), //
            "password".to_string(),
            "john.doe@gmail.com".to_string(),
            "123456789".to_string(),
            UserRole::Doctor,
            Some(Uuid::default()),
            None,
        )
        .unwrap()
    }

    #[tokio::test]
    async fn creates_new_user_and_reads_by_username() {
        let repository = setup_repository();
        let mock_new_user = create_mock_new_user();

        let created_user = repository.create_user(mock_new_user.clone()).await.unwrap();

        assert_eq!(created_user, mock_new_user);

        let user_by_username = repository
            .get_user_by_username(&created_user.username)
            .await
            .unwrap();

        assert_eq!(created_user, user_by_username);
    }
}
