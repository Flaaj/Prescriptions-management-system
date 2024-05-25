use chrono::{DateTime, Utc};
use uuid::Uuid;

#[derive(Clone, Debug)]
pub struct NewPatient {
    pub id: Uuid,
    pub name: String,
    pub pesel_number: String,
}

#[derive(Debug, PartialEq, Clone)]
pub struct Patient {
    pub id: Uuid,
    pub name: String,
    pub pesel_number: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl PartialEq<NewPatient> for Patient {
    fn eq(&self, other: &NewPatient) -> bool {
        self.id == other.id && self.name == other.name && self.pesel_number == other.pesel_number
    }
}

impl PartialEq<Patient> for NewPatient {
    fn eq(&self, other: &Patient) -> bool {
        other.eq(self)
    }
}
