use rocket::async_trait;
use sqlx::Row;
use uuid::Uuid;

use crate::application::sessions::{
    models::{NewSession, Session},
    repository::{
        CreateSessionRepositoryError, GetSessionRepositoryError, SessionsRepository,
        UpdateSessionRepositoryError,
    },
};

pub struct PostgresSessionsRepository {
    pool: sqlx::PgPool,
}

impl PostgresSessionsRepository {
    pub fn new(pool: sqlx::PgPool) -> Self {
        Self { pool }
    }

    fn parse_sessions_row(&self, row: sqlx::postgres::PgRow) -> Result<Session, sqlx::Error> {
        Ok(Session {
            id: row.try_get(0)?,
            user_id: row.try_get(1)?,
            doctor_id: row.try_get(2)?,
            pharmacist_id: row.try_get(3)?,
            ip_address: row.try_get(4).map(|ip: String| ip.parse().unwrap())?,
            user_agent: row.try_get(5)?,
            created_at: row.try_get(6)?,
            updated_at: row.try_get(7)?,
            expires_at: row.try_get(8)?,
            invalidated_at: row.try_get(9)?,
        })
    }
}

#[async_trait]
impl SessionsRepository for PostgresSessionsRepository {
    async fn create_session(
        &self,
        new_session: NewSession,
    ) -> Result<Session, CreateSessionRepositoryError> {
        let row = sqlx::query(r#"INSERT INTO sessions (id, user_id, doctor_id, pharmacist_id, ip_address, user_agent, expires_at) VALUES ($1, $2, $3, $4, $5, $6, $7) RETURNING id, user_id, doctor_id, pharmacist_id, ip_address, user_agent, created_at, updated_at, expires_at, invalidated_at"#)
            .bind(new_session.id)
            .bind(new_session.user_id)
            .bind(new_session.doctor_id)
            .bind(new_session.pharmacist_id)
            .bind(new_session.ip_address.to_string())
            .bind(new_session.user_agent)
            .bind(new_session.expires_at)
            .fetch_one(&self.pool)
            .await
            .map_err(|err| CreateSessionRepositoryError::DatabaseError(err.to_string()))?;

        let session = self
            .parse_sessions_row(row)
            .map_err(|err| CreateSessionRepositoryError::DatabaseError(err.to_string()))?;

        Ok(session)
    }

    async fn get_session_by_id(&self, id: Uuid) -> Result<Session, GetSessionRepositoryError> {
        let row = sqlx::query(r#"SELECT id, user_id, doctor_id, pharmacist_id, ip_address, user_agent, created_at, updated_at, expires_at, invalidated_at FROM sessions WHERE id = $1"#)
            .bind(id)
            .fetch_one(&self.pool)
            .await
            .map_err(|err| GetSessionRepositoryError::DatabaseError(err.to_string()))?;

        let session = self
            .parse_sessions_row(row)
            .map_err(|err| GetSessionRepositoryError::DatabaseError(err.to_string()))?;

        Ok(session)
    }

    async fn update_session(
        &self,
        session: Session,
    ) -> Result<Session, UpdateSessionRepositoryError> {
        let row = sqlx::query(r#"UPDATE sessions SET updated_at = $1, expires_at = $2, invalidated_at = $3 WHERE id = $4 RETURNING id, user_id, doctor_id, pharmacist_id, ip_address, user_agent, created_at, updated_at, expires_at, invalidated_at"#)
            .bind(session.updated_at)
            .bind(session.expires_at)
            .bind(session.invalidated_at)
            .bind(session.id)
            .fetch_one(&self.pool)
            .await
            .map_err(|err| UpdateSessionRepositoryError::DatabaseError(err.to_string()))?;

        let session = self
            .parse_sessions_row(row)
            .map_err(|err| UpdateSessionRepositoryError::DatabaseError(err.to_string()))?;

        Ok(session)
    }
}

#[cfg(test)]
mod tests {
    use std::{
        net::{IpAddr, Ipv4Addr},
        str::FromStr,
    };

    use uuid::Uuid;

    use super::PostgresSessionsRepository;
    use crate::{
        application::sessions::{models::NewSession, repository::SessionsRepository},
        infrastructure::postgres_repository_impl::create_tables::create_tables,
    };

    async fn setup_repository(pool: sqlx::PgPool) -> PostgresSessionsRepository {
        create_tables(&pool, true).await.unwrap();
        PostgresSessionsRepository::new(pool)
    }

    fn create_mock_new_session() -> NewSession {
        NewSession::new(
            Uuid::new_v4(),
            Some(Uuid::new_v4()),
            None,
            IpAddr::V4(Ipv4Addr::from_str("127.0.0.1").unwrap()),
            "Mozilla/5.0".to_string(),
        )
    }

    #[sqlx::test]
    async fn creates_new_session_and_reads_by_id(pool: sqlx::PgPool) {
        let repository = setup_repository(pool).await;
        let mock_new_session = create_mock_new_session();

        let created_session = repository
            .create_session(mock_new_session.clone())
            .await
            .unwrap();

        assert_eq!(created_session, mock_new_session);

        let session_by_id = repository
            .get_session_by_id(created_session.id)
            .await
            .unwrap();

        assert_eq!(created_session, session_by_id);
    }

    #[sqlx::test]
    async fn updates_session(pool: sqlx::PgPool) {
        let repository = setup_repository(pool).await;
        let mock_new_session = create_mock_new_session();

        let created_session = repository
            .create_session(mock_new_session.clone())
            .await
            .unwrap();

        let mut created_session_by_id = repository
            .get_session_by_id(created_session.id)
            .await
            .unwrap();

        assert!(created_session.invalidated_at.is_none());

        created_session_by_id.invalidate().unwrap();

        repository
            .update_session(created_session_by_id.clone())
            .await
            .unwrap();

        let invalidated_session = repository
            .get_session_by_id(created_session_by_id.id)
            .await
            .unwrap();

        assert!(invalidated_session.invalidated_at.is_some());
    }
}
