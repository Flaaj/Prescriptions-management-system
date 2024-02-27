use chrono::{DateTime, Utc};
use uuid::Uuid;

use super::prescription_type::PrescriptionType;

#[derive(Debug, PartialEq, Clone)]
pub struct PrescribedDrug {
    pub id: Uuid,
    pub prescription_id: Uuid,
    pub drug_id: Uuid,
    pub quantity: i32,
}

#[derive(Debug, PartialEq, Clone)]
pub struct Prescription {
    pub id: Uuid,
    pub doctor_id: Uuid,
    pub patient_id: Uuid,
    pub prescribed_drugs: Vec<PrescribedDrug>,
    pub prescription_type: PrescriptionType,
    pub start_date: DateTime<Utc>,
    pub end_date: DateTime<Utc>,
}
