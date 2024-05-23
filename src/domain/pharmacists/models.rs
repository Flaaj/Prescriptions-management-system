use chrono::{DateTime, Utc};
use uuid::Uuid;

#[derive(Clone, Debug)]
pub struct NewPharmacist {
    pub id: Uuid,
    pub name: String,
    pub pesel_number: String,
}

#[derive(Debug)]
pub struct Pharmacist {
    pub id: Uuid,
    pub name: String,
    pub pesel_number: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl PartialEq<NewPharmacist> for Pharmacist {
    fn eq(&self, other: &NewPharmacist) -> bool {
        self.id == other.id && self.name == other.name && self.pesel_number == other.pesel_number
    }
}

impl PartialEq<Pharmacist> for NewPharmacist {
    fn eq(&self, other: &Pharmacist) -> bool {
        other.eq(self)
    }
}
