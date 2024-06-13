use std::sync::RwLock;

use chrono::Utc;
use rocket::async_trait;
use uuid::Uuid;

use super::models::{NewSession, Session};

#[derive(thiserror::Error, Debug, PartialEq)]
pub enum CreateUserRepositoryError {
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
    ) -> Result<Session, CreateUserRepositoryError>;
    async fn get_session_by_id(&self, id: Uuid) -> Result<Session, GetSessionRepositoryError>;
    async fn update_session(
        &self,
        session: Session,
    ) -> Result<Session, UpdateSessionRepositoryError>;
}

pub struct SessionsRepositoryFake {
    sessions: RwLock<Vec<Session>>,
}

impl SessionsRepositoryFake {
    #[allow(dead_code)]
    pub fn new() -> Self {
        Self {
            sessions: RwLock::new(Vec::new()),
        }
    }
}

#[async_trait]
impl SessionsRepository for SessionsRepositoryFake {
    async fn create_session(
        &self,
        new_session: NewSession,
    ) -> Result<Session, CreateUserRepositoryError> {
        let session = Session {
            id: new_session.id,
            user_id: new_session.user_id,
            doctor_id: new_session.doctor_id,
            pharmacist_id: new_session.pharmacist_id,
            ip_address: new_session.ip_address,
            user_agent: new_session.user_agent,
            expires_at: new_session.expires_at,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            invalidated_at: None,
        };

        self.sessions.write().unwrap().push(session.clone());

        Ok(session)
    }

    async fn get_session_by_id(&self, id: Uuid) -> Result<Session, GetSessionRepositoryError> {
        match self
            .sessions
            .read()
            .unwrap()
            .iter()
            .find(|session| session.id == id)
        {
            Some(session) => return Ok(session.clone()),
            None => Err(GetSessionRepositoryError::NotFound(id)),
        }
    }

    async fn update_session(
        &self,
        updated_session: Session,
    ) -> Result<Session, UpdateSessionRepositoryError> {
        match self
            .sessions
            .write()
            .unwrap()
            .iter_mut()
            .find(|session| session.id == updated_session.id)
        {
            Some(session) => {
                session.invalidated_at = updated_session.invalidated_at;
                session.expires_at = updated_session.expires_at;
                session.updated_at = updated_session.updated_at;
                Ok(session.clone())
            }
            None => Err(UpdateSessionRepositoryError::NotFound(updated_session.id)),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::{
        net::{IpAddr, Ipv4Addr},
        str::FromStr,
    };

    use uuid::Uuid;

    use super::{SessionsRepository, SessionsRepositoryFake};
    use crate::application::sessions::models::NewSession;

    fn setup_repository() -> SessionsRepositoryFake {
        SessionsRepositoryFake::new()
    }

    fn create_mock_new_session() -> NewSession {
        NewSession::new(
            Uuid::new_v4(),
            Some(Uuid::new_v4()),
            None,
            IpAddr::V4(Ipv4Addr::from_str("127.0.0.1").unwrap()),
            "Mozilla/5.0".to_string(),
        )
        .unwrap()
    }

    #[tokio::test]
    async fn creates_new_session_and_reads_by_id() {
        let repository = setup_repository();
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

    #[tokio::test]
    async fn updates_session() {
        let repository = setup_repository();
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

        created_session_by_id.invalidate();

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
