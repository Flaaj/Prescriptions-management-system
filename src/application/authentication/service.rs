use uuid::Uuid;

use super::{
    entities::{NewUser, User, UserRole},
    repository::{AuthenticationRepository, CreateUserRepositoryError},
};
use crate::application::helpers::hashing::Hasher;

#[derive(Debug)]
pub enum CreateUserError {
    DomainError(String),
    RepositoryError(CreateUserRepositoryError),
}

#[derive(Debug, PartialEq, thiserror::Error)]
pub enum AuthenticationWithCredentialsError {
    #[error("Invalid credentials")]
    InvalidCredentials,
}

pub struct AuthenticationService {
    authentication_repository: Box<dyn AuthenticationRepository>,
}

impl AuthenticationService {
    pub fn new(authentication_repository: Box<dyn AuthenticationRepository>) -> Self {
        Self {
            authentication_repository,
        }
    }

    pub async fn register_user(
        &self,
        username: String,
        password: String,
        email: String,
        phone_number: String,
        user_role: UserRole,
        doctor_id: Option<Uuid>,
        pharmacist_id: Option<Uuid>,
    ) -> Result<User, CreateUserError> {
        let new_user = NewUser::new(
            username,
            password,
            email,
            phone_number,
            user_role,
            doctor_id,
            pharmacist_id,
        )
        .map_err(|err| CreateUserError::DomainError(err.to_string()))?;

        let created_user = self
            .authentication_repository
            .create_user(new_user)
            .await
            .map_err(|err| CreateUserError::RepositoryError(err))?;

        Ok(created_user)
    }

    fn verify_user_password(&self, pass: &str, user: &User) -> bool {
        Hasher::verify_password(pass, &user.password_hash)
    }

    pub async fn authenticate_with_credentials(
        &self,
        username: String,
        pass: String,
        role: UserRole,
    ) -> Result<User, AuthenticationWithCredentialsError> {
        let user = self
            .authentication_repository
            .get_user_by_username(&username)
            .await
            .map_err(|_| AuthenticationWithCredentialsError::InvalidCredentials)?;

        if user.role != role {
            Err(AuthenticationWithCredentialsError::InvalidCredentials)?;
        }

        if !self.verify_user_password(&pass, &user) {
            Err(AuthenticationWithCredentialsError::InvalidCredentials)?;
        }

        Ok(user)
    }
}

#[cfg(test)]
mod tests {
    use uuid::Uuid;

    use super::AuthenticationService;
    use crate::{
        application::authentication::entities::UserRole,
        infrastructure::postgres_repository_impl::{
            authentication::PostgresAuthenticationRepository, create_tables::create_tables,
        },
    };

    async fn setup_service(pool: sqlx::PgPool) -> AuthenticationService {
        create_tables(&pool, true).await.unwrap();
        AuthenticationService::new(Box::new(PostgresAuthenticationRepository::new(pool)))
    }

    #[sqlx::test]
    async fn registers_user(pool: sqlx::PgPool) {
        let service = setup_service(pool).await;

        service
            .register_user(
                "username".to_string(), //
                "password".to_string(),
                "john.doe@gmail.com".to_string(),
                "123456789".to_string(),
                UserRole::Doctor,
                Some(Uuid::default()),
                None,
            )
            .await
            .unwrap();
    }

    #[sqlx::test]
    async fn authenticates_user_by_credentials(pool: sqlx::PgPool) {
        let service = setup_service(pool).await;
        let seed_user = service
            .register_user(
                "username".to_string(), //
                "password123".to_string(),
                "john.doe@gmail.com".to_string(),
                "123456789".to_string(),
                UserRole::Doctor,
                Some(Uuid::default()),
                None,
            )
            .await
            .unwrap();

        let result = service
            .authenticate_with_credentials(
                "username".to_string(),
                "password123".to_string(),
                UserRole::Doctor,
            )
            .await;

        assert_eq!(result, Ok(seed_user))
    }

    #[sqlx::test]
    async fn doesnt_authenticate_with_wrong_credentials(pool: sqlx::PgPool) {
        let service = setup_service(pool).await;
        service
            .register_user(
                "username".to_string(), //
                "password123".to_string(),
                "john.doe@gmail.com".to_string(),
                "123456789".to_string(),
                UserRole::Doctor,
                Some(Uuid::default()),
                None,
            )
            .await
            .unwrap();

        service
            .authenticate_with_credentials(
                "username".to_string(),
                "password124".to_string(),
                UserRole::Doctor,
            )
            .await
            .unwrap_err();
    }
}
