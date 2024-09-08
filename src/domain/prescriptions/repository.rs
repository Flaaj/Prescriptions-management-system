use async_trait::async_trait;
use uuid::Uuid;

use crate::domain::prescriptions::entities::{
    NewPrescription, NewPrescriptionFill, Prescription, PrescriptionFill,
};

#[derive(thiserror::Error, Debug, PartialEq)]
pub enum CreatePrescriptionRepositoryError {
    #[error("Doctor with id {0} not found")]
    DoctorNotFound(Uuid),
    #[error("Patient with id {0} not found")]
    PatientNotFound(Uuid),
    #[error("Drug with id {0} not found")]
    DrugNotFound(Uuid),
    #[error("Database error: {0}")]
    DatabaseError(String),
}

#[derive(thiserror::Error, Debug, PartialEq)]
pub enum GetPrescriptionsRepositoryError {
    #[error("Invalid pagination parameters: {0}")]
    InvalidPaginationParams(String),
    #[error("Database error: {0}")]
    DatabaseError(String),
}

#[derive(thiserror::Error, Debug, PartialEq)]
pub enum GetPrescriptionByIdRepositoryError {
    #[error("Prescription with id {0} not found")]
    NotFound(Uuid),
    #[error("Database error: {0}")]
    DatabaseError(String),
}

#[derive(thiserror::Error, Debug, PartialEq)]
pub enum FillPrescriptionRepositoryError {
    #[error("Pharmacist with id {0} not found")]
    PharmacistNotFound(Uuid),
    #[error("Prescription with id {0} not found")]
    PrescriptionNotFound(Uuid),
    #[error("Database error: {0}")]
    DatabaseError(String),
}

#[async_trait]
pub trait PrescriptionsRepository: Send + Sync + 'static {
    async fn create_prescription(
        &self,
        prescription: NewPrescription,
    ) -> Result<Prescription, CreatePrescriptionRepositoryError>;
    async fn get_prescriptions(
        &self,
        page: Option<i64>,
        page_size: Option<i64>,
    ) -> Result<Vec<Prescription>, GetPrescriptionsRepositoryError>;
    async fn get_prescription_by_id(
        &self,
        prescription_id: Uuid,
    ) -> Result<Prescription, GetPrescriptionByIdRepositoryError>;
    async fn fill_prescription(
        &self,
        prescription_fill: NewPrescriptionFill,
    ) -> Result<PrescriptionFill, FillPrescriptionRepositoryError>;
    // async fn get_prescriptions_by_prescription_id(&self, prescription_id: Uuid) ->
    // Result<Vec<Prescription>>; async fn get_prescriptions_by_patient_id(&self, patient_id:
    // Uuid) -> Result<Vec<Prescription>>; async fn update_prescription(&self, prescription:
    // Prescription) -> Result<()>; async fn delete_prescription(&self, prescription_id: Uuid)
    // -> Result<()>;
}
