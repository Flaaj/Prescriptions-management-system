use chrono::{DateTime, Utc};
use uuid::Uuid;

#[derive(Clone, Debug)]
pub struct NewPatient {
    pub id: Uuid,
    pub name: String,
    pub pesel_number: String,
}

#[derive(Debug)]
pub struct Patient {
    pub id: Uuid,
    pub name: String,
    pub pesel_number: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
