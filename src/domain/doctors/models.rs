use chrono::{DateTime, Utc};
use uuid::Uuid;

#[derive(Clone, Debug)]
pub struct NewDoctor {
    pub id: Uuid,
    pub name: String,
    pub pwz_number: String,
    pub pesel_number: String,
}

#[derive(Debug)]
pub struct Doctor {
    pub id: Uuid,
    pub name: String,
    pub pwz_number: String,
    pub pesel_number: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
