use async_trait::async_trait;
use uuid::Uuid;

use crate::domain::patients::entities::{NewPatient, Patient};

#[derive(thiserror::Error, Debug, PartialEq)]
pub enum CreatePatientRepositoryError {
    #[error("PESEL number already exists")]
    DuplicatedPeselNumber,
    #[error("Database error: {0}")]
    DatabaseError(String),
}

#[derive(thiserror::Error, Debug, PartialEq)]
pub enum GetPatientsRepositoryError {
    #[error("Invalid pagination parameters: {0}")]
    InvalidPaginationParams(String),
    #[error("Database error: {0}")]
    DatabaseError(String),
}

#[derive(thiserror::Error, Debug, PartialEq)]
pub enum GetPatientByIdRepositoryError {
    #[error("Patient with this id not found ({0})")]
    NotFound(Uuid),
    #[error("Database error: {0}")]
    DatabaseError(String),
}

#[async_trait]
pub trait PatientsRepository: Send + Sync + 'static {
    async fn create_patient(
        &self,
        patient: NewPatient,
    ) -> Result<Patient, CreatePatientRepositoryError>;
    async fn get_patients(
        &self,
        page: Option<i64>,
        page_size: Option<i64>,
    ) -> Result<Vec<Patient>, GetPatientsRepositoryError>;
    async fn get_patient_by_id(
        &self,
        patient_id: Uuid,
    ) -> Result<Patient, GetPatientByIdRepositoryError>;
}
