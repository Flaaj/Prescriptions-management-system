use chrono::Utc;

use crate::application::sessions::models::Session;

#[derive(Debug, PartialEq)]
pub enum SessionValidationError {
    SessionExpired,
    SessionInvalidated,
}

impl Session {
    pub fn validate(&self) -> Result<(), SessionValidationError> {
        if self.invalidated_at.is_some() {
            Err(SessionValidationError::SessionInvalidated)?;
        }

        if self.expires_at < Utc::now() {
            Err(SessionValidationError::SessionExpired)?;
        }

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

    use crate::application::sessions::{
        models::Session, use_cases::validate_session::SessionValidationError,
    };

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
    fn passes_validation_if_not_invalidated_and_expiration_date_is_in_the_future_and() {
        let mut session = create_mock_session();
        session.expires_at = Utc::now() + chrono::Duration::days(2);
        session.invalidated_at = None;

        let result = session.validate();

        assert!(result.is_ok());
    }

    #[test]
    fn fails_validation_if_session_is_invalidated() {
        let mut session = create_mock_session();
        session.invalidated_at = Some(Utc::now());

        let result = session.validate();

        assert_eq!(result, Err(SessionValidationError::SessionInvalidated));
    }

    #[test]
    fn fails_validation_if_session_is_expired() {
        let mut session = create_mock_session();
        session.expires_at = Utc::now() - chrono::Duration::minutes(10);

        let result = session.validate();

        assert_eq!(result, Err(SessionValidationError::SessionExpired));
    }
}
