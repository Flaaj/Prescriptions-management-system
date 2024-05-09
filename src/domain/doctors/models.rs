use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Clone, Debug)]
pub struct NewDoctor {
    pub id: Uuid,
    pub name: String,
    pub pwz_number: String,
    pub pesel_number: String,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct Doctor {
    pub id: Uuid,
    pub name: String,
    pub pwz_number: String,
    pub pesel_number: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
