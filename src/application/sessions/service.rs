use std::net::IpAddr;

use uuid::Uuid;

use super::{
    models::{NewSession, Session},
    repository::{
        CreateSessionRepositoryError, GetSessionRepositoryError, SessionsRepository,
        UpdateSessionRepositoryError,
    },
    use_cases::invalidate_session::InvalidateSessionDomainError,
};

pub struct SessionsService {
    sessions_repository: Box<dyn SessionsRepository>,
}

#[derive(Debug)]
pub enum CreateSessionError {
    RepositoryError(CreateSessionRepositoryError),
}

#[derive(Debug)]
pub enum InvalidateSessionError {
    DomainError(InvalidateSessionDomainError),
    RepositoryError(UpdateSessionRepositoryError),
}

#[derive(Debug)]
pub enum GetSessionByIdError {
    RepositoryError(GetSessionRepositoryError),
}

impl SessionsService {
    pub fn new(sessions_repository: Box<dyn SessionsRepository>) -> Self {
        Self {
            sessions_repository,
        }
    }

    pub async fn create_session(
        &self,
        user_id: Uuid,
        doctor_id: Option<Uuid>,
        pharmacist_id: Option<Uuid>,
        ip_address: IpAddr,
        user_agent: String,
    ) -> Result<Session, CreateSessionError> {
        let new_session =
            NewSession::new(user_id, doctor_id, pharmacist_id, ip_address, user_agent);

        let created_session = self
            .sessions_repository
            .create_session(new_session)
            .await
            .map_err(|err| CreateSessionError::RepositoryError(err))?;

        Ok(created_session)
    }

    pub async fn get_session_by_id(
        &self,
        session_id: Uuid,
    ) -> Result<Session, GetSessionByIdError> {
        self.sessions_repository
            .get_session_by_id(session_id)
            .await
            .map_err(|err| GetSessionByIdError::RepositoryError(err))
    }

    pub async fn invalidate_session(
        &self,
        mut session: Session,
    ) -> Result<Session, InvalidateSessionError> {
        session
            .invalidate()
            .map_err(|err| InvalidateSessionError::DomainError(err))?;

        self.sessions_repository
            .update_session(session)
            .await
            .map_err(|err| InvalidateSessionError::RepositoryError(err))
    }
}

#[cfg(test)]
mod tests {
    use std::{
        net::{IpAddr, Ipv4Addr},
        str::FromStr,
    };

    use uuid::Uuid;

    use super::SessionsService;
    use crate::application::sessions::repository::SessionsRepositoryFake;

    fn setup_service() -> SessionsService {
        SessionsService::new(Box::new(SessionsRepositoryFake::new()))
    }

    #[tokio::test]
    async fn creates_new_session() {
        let service = setup_service();

        service
            .create_session(
                Uuid::new_v4(),
                Some(Uuid::new_v4()),
                None,
                IpAddr::V4(Ipv4Addr::from_str("127.0.0.1").unwrap()),
                "Mozilla/5.0".to_string(),
            )
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn invalidates_session() {
        let service = setup_service();
        let session = service
            .create_session(
                Uuid::new_v4(),
                Some(Uuid::new_v4()),
                None,
                IpAddr::V4(Ipv4Addr::from_str("127.0.0.1").unwrap()),
                "Mozilla/5.0".to_string(),
            )
            .await
            .unwrap();

        assert!(session.invalidated_at.is_none());

        let invalidated_session = service.invalidate_session(session).await.unwrap();

        let invalidated_session_by_id = service.get_session_by_id(invalidated_session.id).await.unwrap();

        assert!(invalidated_session_by_id.invalidated_at.is_some());
    }
}
