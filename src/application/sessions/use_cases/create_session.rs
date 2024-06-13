use std::net::IpAddr;

use chrono::{Duration, Utc};
use uuid::Uuid;

use crate::application::sessions::models::NewSession;

impl NewSession {
    pub fn new(
        user_id: Uuid,
        doctor_id: Option<Uuid>,
        pharmacist_id: Option<Uuid>,
        ip_address: IpAddr,
        user_agent: String,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            user_id,
            doctor_id,
            pharmacist_id,
            ip_address,
            user_agent,
            expires_at: Utc::now() + Duration::days(2),
        }
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

    use crate::application::sessions::models::NewSession;

    #[test]
    fn creates_new_session() {
        let now = Utc::now();

        let new_session = NewSession::new(
            Uuid::new_v4(),
            Some(Uuid::new_v4()),
            None,
            IpAddr::V4(Ipv4Addr::from_str("127.0.0.1").unwrap()),
            "Mozilla/5.0".to_string(),
        );

        let session_duration = new_session.expires_at - now;

        assert_eq!(session_duration.num_hours(), 48);
    }
}
