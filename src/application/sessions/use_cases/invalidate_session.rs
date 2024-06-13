use chrono::Utc;

use crate::application::sessions::models::Session;

#[derive(thiserror::Error, Debug, PartialEq)]
pub enum InvalidateSessionDomainError {
    #[error("Session is already invalidated")]
    AlreadyInvalidated,
}

impl Session {
    pub fn invalidate(&mut self) -> Result<(), InvalidateSessionDomainError> {
        if self.invalidated_at.is_some() {
            Err(InvalidateSessionDomainError::AlreadyInvalidated)?;
        }
        let now = Utc::now();
        self.invalidated_at = Some(now);
        self.updated_at = now;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::{
        net::{IpAddr, Ipv4Addr},
        str::FromStr,
    };

    use chrono::Utc;
    use uuid::Uuid;

    use crate::application::sessions::models::Session;

    fn create_mock_session() -> Session {
        Session {
            id: Uuid::new_v4(),
            user_id: Uuid::new_v4(),
            doctor_id: Some(Uuid::new_v4()),
            pharmacist_id: None,
            ip_address: IpAddr::V4(Ipv4Addr::from_str("127.0.0.1").unwrap()),
            user_agent: "Mozilla/5.0".to_string(),
            expires_at: Utc::now() + chrono::Duration::days(2),
            created_at: Utc::now(),
            updated_at: Utc::now(),
            invalidated_at: None,
        }
    }

    #[test]
    fn invalidates_session() {
        let mut session = create_mock_session();

        session.invalidate().unwrap();

        assert_eq!(session.invalidated_at.unwrap(), session.updated_at)
    }

    #[test]
    fn returns_error_if_session_is_already_invalidated() {
        let mut session = create_mock_session();
        session.invalidated_at = Some(Utc::now());

        let result = session.invalidate();

        assert!(result.is_err());
    }
}
