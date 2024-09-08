use chrono::{DateTime, Utc};
use rocket::async_trait;
use sqlx::Row;
use uuid::Uuid;

use crate::{
    application::authentication::{
        entities::{NewUser, User, UserRole},
        repository::{AuthenticationRepository, CreateUserRepositoryError, GetUserRepositoryError},
    },
    domain::{doctors::entities::Doctor, pharmacists::entities::Pharmacist},
};

pub struct PostgresAuthenticationRepository {
    pool: sqlx::PgPool,
}

struct UsersRow {
    user_id: Uuid,
    user_username: String,
    user_password_hash: String,
    user_email: String,
    user_phone_number: String,
    user_role: UserRole,
    user_created_at: DateTime<Utc>,
    user_updated_at: DateTime<Utc>,
    doctor_id: Option<Uuid>,
    doctor_name: Option<String>,
    doctor_pwz_number: Option<String>,
    doctor_pesel_number: Option<String>,
    doctor_created_at: Option<DateTime<Utc>>,
    doctor_updated_at: Option<DateTime<Utc>>,
    pharmacist_id: Option<Uuid>,
    pharmacist_name: Option<String>,
    pharmacist_pesel_number: Option<String>,
    pharmacist_created_at: Option<DateTime<Utc>>,
    pharmacist_updated_at: Option<DateTime<Utc>>,
}

impl PostgresAuthenticationRepository {
    pub fn new(pool: sqlx::PgPool) -> Self {
        Self { pool }
    }

    fn parse_users_row(&self, row: sqlx::postgres::PgRow) -> Result<User, sqlx::Error> {
        let users_row = UsersRow {
            user_id: row.try_get(0)?,
            user_username: row.try_get(1)?,
            user_password_hash: row.try_get(2)?,
            user_email: row.try_get(3)?,
            user_phone_number: row.try_get(4)?,
            user_role: row.try_get(5)?,
            user_created_at: row.try_get(6)?,
            user_updated_at: row.try_get(7)?,
            doctor_id: row.try_get(8)?,
            doctor_name: row.try_get(9)?,
            doctor_pwz_number: row.try_get(10)?,
            doctor_pesel_number: row.try_get(11)?,
            doctor_created_at: row.try_get(12)?,
            doctor_updated_at: row.try_get(13)?,
            pharmacist_id: row.try_get(14)?,
            pharmacist_name: row.try_get(15)?,
            pharmacist_pesel_number: row.try_get(16)?,
            pharmacist_created_at: row.try_get(17)?,
            pharmacist_updated_at: row.try_get(18)?,
        };

        Ok(User {
            id: users_row.user_id,
            username: users_row.user_username,
            password_hash: users_row.user_password_hash,
            email: users_row.user_email,
            phone_number: users_row.user_phone_number,
            role: users_row.user_role,
            created_at: users_row.user_created_at,
            updated_at: users_row.user_updated_at,
            doctor: users_row.doctor_id.map(|id| Doctor {
                id,
                name: users_row.doctor_name.unwrap(),
                pwz_number: users_row.doctor_pwz_number.unwrap(),
                pesel_number: users_row.doctor_pesel_number.unwrap(),
                created_at: users_row.doctor_created_at.unwrap(),
                updated_at: users_row.doctor_updated_at.unwrap(),
            }),
            pharmacist: users_row.pharmacist_id.map(|id| Pharmacist {
                id,
                name: users_row.pharmacist_name.unwrap(),
                pesel_number: users_row.pharmacist_pesel_number.unwrap(),
                created_at: users_row.pharmacist_created_at.unwrap(),
                updated_at: users_row.pharmacist_updated_at.unwrap(),
            }),
        })
    }
}

#[async_trait]
impl AuthenticationRepository for PostgresAuthenticationRepository {
    async fn create_user(&self, new_user: NewUser) -> Result<User, CreateUserRepositoryError> {
        let transaction = self
            .pool
            .begin()
            .await
            .map_err(|err| CreateUserRepositoryError::DatabaseError(err.to_string()))?;

        sqlx::query(
            r#"INSERT INTO users (id, username, password_hash, email, phone_number, role, doctor_id, pharmacist_id) VALUES ($1, $2, $3, $4, $5, $6, $7, $8)"#,
        )
        .bind(new_user.id)
        .bind(new_user.username.clone())
        .bind(new_user.password_hash)
        .bind(new_user.email)
        .bind(new_user.phone_number)
        .bind(new_user.role)
        .bind(new_user.doctor_id)
        .bind(new_user.pharmacist_id)
        .execute(&self.pool)
        .await
        .map_err(|err| CreateUserRepositoryError::DatabaseError(err.to_string()))?;

        let user = self
            .get_user_by_username(&new_user.username)
            .await
            .map_err(|err| CreateUserRepositoryError::DatabaseError(err.to_string()))?;

        transaction
            .commit()
            .await
            .map_err(|err| CreateUserRepositoryError::DatabaseError(err.to_string()))?;

        Ok(user)
    }

    async fn get_user_by_username<'a>(
        &self,
        username: &'a str,
    ) -> Result<User, GetUserRepositoryError> {
        let row = sqlx::query(
            r#"
            SELECT 
                users.id, 
                users.username, 
                users.password_hash,
                users.email, 
                users.phone_number, 
                users.role, 
                users.created_at,
                users.updated_at,
                doctors.id,
                doctors.name,
                doctors.pwz_number,
                doctors.pesel_number,
                doctors.created_at,
                doctors.updated_at,
                pharmacists.id,
                pharmacists.name,
                pharmacists.pesel_number,
                pharmacists.created_at,
                pharmacists.updated_at
            FROM users 
            LEFT JOIN doctors ON users.doctor_id = doctors.id
            LEFT JOIN pharmacists ON users.pharmacist_id = pharmacists.id
            WHERE username = $1
        "#,
        )
        .bind(username)
        .fetch_one(&self.pool)
        .await
        .map_err(|err| GetUserRepositoryError::DatabaseError(err.to_string()))?;

        let user = self
            .parse_users_row(row)
            .map_err(|err| GetUserRepositoryError::DatabaseError(err.to_string()))?;

        Ok(user)
    }
}

#[cfg(test)]
mod tests {
    use uuid::Uuid;

    use super::PostgresAuthenticationRepository;
    use crate::{
        application::authentication::{
            entities::{NewUser, UserRole},
            repository::AuthenticationRepository,
        },
        infrastructure::postgres_repository_impl::create_tables::create_tables,
    };

    async fn setup_repository(pool: sqlx::PgPool) -> PostgresAuthenticationRepository {
        create_tables(&pool, true).await.unwrap();
        PostgresAuthenticationRepository::new(pool)
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

    #[sqlx::test]
    async fn creates_new_user_and_reads_by_username(pool: sqlx::PgPool) {
        let repository = setup_repository(pool).await;
        let mock_new_user = create_mock_new_user();

        let created_user = repository.create_user(mock_new_user.clone()).await.unwrap();

        assert_eq!(created_user, mock_new_user);

        let user_by_username = repository
            .get_user_by_username(&mock_new_user.username)
            .await
            .unwrap();

        assert_eq!(created_user, user_by_username);
    }
}
