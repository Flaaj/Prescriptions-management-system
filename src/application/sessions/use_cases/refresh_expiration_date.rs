use chrono::{Duration, Utc};

use crate::application::sessions::entities::Session;

impl Session {
    pub fn refresh_expiration_date(&mut self) {
        let now = Utc::now();
        self.expires_at = now + Duration::days(2);
        self.updated_at = now;
    }
}

#[cfg(test)]
mod tests {
    use std::{
        net::{IpAddr, Ipv4Addr},
        str::FromStr,
    };

    use chrono::{Duration, Utc};
    use uuid::Uuid;

    use crate::application::sessions::entities::Session;

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
    fn refreshes_expiration_date() {
        let now = Utc::now();
        let mut session = create_mock_session();
        session.expires_at = Utc::now() + Duration::hours(1);

        session.refresh_expiration_date();

        let session_duration = session.expires_at - now;

        assert_eq!(session_duration.num_hours(), 48);
        assert_eq!(session.expires_at, session.updated_at + Duration::days(2))
    }
}
