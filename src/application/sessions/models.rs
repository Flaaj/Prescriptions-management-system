use std::net::IpAddr;

use chrono::{DateTime, Utc};
use uuid::Uuid;

#[derive(Debug, PartialEq, Clone)]
pub struct NewSession {
    pub id: Uuid,
    pub user_id: Uuid,
    pub doctor_id: Option<Uuid>,
    pub pharmacist_id: Option<Uuid>,
    pub ip_address: IpAddr,
    pub user_agent: String,
    pub expires_at: DateTime<Utc>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct Session {
    pub id: Uuid,
    pub user_id: Uuid,
    pub doctor_id: Option<Uuid>,
    pub pharmacist_id: Option<Uuid>,
    pub ip_address: IpAddr,
    pub user_agent: String,
    pub expires_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub invalidated_at: Option<DateTime<Utc>>,
}

impl PartialEq<NewSession> for Session {
    fn eq(&self, other: &NewSession) -> bool {
        self.id == other.id
            && self.user_id == other.user_id
            && self.doctor_id == other.doctor_id
            && self.pharmacist_id == other.pharmacist_id
            && self.ip_address == other.ip_address
            && self.user_agent == other.user_agent
            && self.expires_at == other.expires_at
    }
}

impl PartialEq<Session> for NewSession {
    fn eq(&self, other: &Session) -> bool {
        other.eq(self)
    }
}
