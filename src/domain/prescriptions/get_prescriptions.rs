use chrono::{DateTime, Utc};
use uuid::Uuid;

use super::prescription_type::PrescriptionType;

#[derive(Debug, PartialEq)]
pub struct PrescribedDrug {
    pub drug_id: Uuid,
    pub quantity: u16,
}

#[derive(Debug, PartialEq)]
pub struct Prescription {
    pub id: Uuid,
    pub doctor_id: Uuid,
    pub patient_id: Uuid,
    pub prescribed_drugs: Vec<PrescribedDrug>,
    pub prescription_type: PrescriptionType,
    pub start_date: DateTime<Utc>,
    pub end_date: DateTime<Utc>,
}