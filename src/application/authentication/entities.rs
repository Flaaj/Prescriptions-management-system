use chrono::{DateTime, Utc};
use serde::Serialize;
use uuid::Uuid;

use crate::domain::{doctors::entities::Doctor, pharmacists::entities::Pharmacist};

#[derive(sqlx::Type, Debug, PartialEq, Clone, Copy, Serialize)]
#[sqlx(type_name = "user_role", rename_all = "snake_case")]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum UserRole {
    Doctor,
    Pharmacist,
}

#[derive(Debug, Clone)]
pub struct NewUser {
    pub id: Uuid,
    pub username: String,
    pub password_hash: String,
    pub email: String,
    pub phone_number: String,
    pub role: UserRole,
    pub doctor_id: Option<Uuid>,
    pub pharmacist_id: Option<Uuid>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct User {
    pub id: Uuid,
    pub username: String,
    #[serde(skip_serializing)]
    pub password_hash: String,
    pub email: String,
    pub phone_number: String,
    pub role: UserRole,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub doctor: Option<Doctor>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pharmacist: Option<Pharmacist>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl PartialEq<NewUser> for User {
    fn eq(&self, other: &NewUser) -> bool {
        self.id == other.id
            && self.username == other.username
            && self.email == other.email
            && self.phone_number == other.phone_number
            && self.role == other.role
    }
}

impl PartialEq<User> for NewUser {
    fn eq(&self, other: &User) -> bool {
        other.eq(self)
    }
}
