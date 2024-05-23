use chrono::{DateTime, Utc};
use uuid::Uuid;

#[derive(Debug, PartialEq, sqlx::Type, Clone, Copy)]
#[sqlx(type_name = "prescription_type", rename_all = "snake_case")]
pub enum PrescriptionType {
    Regular,
    ForAntibiotics,
    ForImmunologicalDrugs,
    ForChronicDiseaseDrugs,
}

#[derive(Debug, PartialEq, Clone)]
pub struct NewPrescribedDrug {
    pub drug_id: Uuid,
    pub quantity: u32,
}

#[derive(Debug, PartialEq, Clone)]
pub struct NewPrescription {
    pub id: Uuid,
    pub doctor_id: Uuid,
    pub patient_id: Uuid,
    pub prescribed_drugs: Vec<NewPrescribedDrug>,
    pub prescription_type: PrescriptionType,
    pub code: String,
    pub start_date: DateTime<Utc>,
    pub end_date: DateTime<Utc>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct PrescribedDrug {
    pub id: Uuid,
    pub prescription_id: Uuid,
    pub drug_id: Uuid,
    pub quantity: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct PrescriptionPatient {
    pub id: Uuid,
    pub name: String,
    pub pesel_number: String,
}

#[derive(Debug, PartialEq, Clone)]
pub struct PrescriptionDoctor {
    pub id: Uuid,
    pub name: String,
    pub pesel_number: String,
    pub pwz_number: String,
}

#[derive(Debug, PartialEq, Clone)]
pub struct Prescription {
    pub id: Uuid,
    pub doctor: PrescriptionDoctor,
    pub patient: PrescriptionPatient,
    pub prescribed_drugs: Vec<PrescribedDrug>,
    pub prescription_type: PrescriptionType,
    pub code: String,
    pub fill: Option<PrescriptionFill>,
    pub start_date: DateTime<Utc>,
    pub end_date: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl PartialEq<NewPrescription> for Prescription {
    fn eq(&self, other: &NewPrescription) -> bool {
        self.id == other.id
            && self.doctor.id == other.doctor_id
            && self.patient.id == other.patient_id
            && self.prescription_type == other.prescription_type
            && self.code == other.code
            && self.start_date == other.start_date
            && self.end_date == other.end_date
            && self.prescribed_drugs.len() == other.prescribed_drugs.len()
    }
}

impl PartialEq<Prescription> for NewPrescription {
    fn eq(&self, other: &Prescription) -> bool {
        other.eq(self)
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct NewPrescriptionFill {
    pub id: Uuid,
    pub prescription_id: Uuid,
    pub pharmacist_id: Uuid,
}

#[derive(Debug, PartialEq, Clone)]
pub struct PrescriptionFill {
    pub id: Uuid,
    pub prescription_id: Uuid,
    pub pharmacist_id: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl PartialEq<NewPrescriptionFill> for PrescriptionFill {
    fn eq(&self, other: &NewPrescriptionFill) -> bool {
        self.id == other.id
            && self.prescription_id == other.prescription_id
            && self.pharmacist_id == other.pharmacist_id
    }
}

impl PartialEq<PrescriptionFill> for NewPrescriptionFill {
    fn eq(&self, other: &PrescriptionFill) -> bool {
        other.eq(self)
    }
}
