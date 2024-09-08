use async_trait::async_trait;
use uuid::Uuid;

use crate::domain::doctors::entities::{Doctor, NewDoctor};

#[derive(thiserror::Error, Debug, PartialEq)]
pub enum CreateDoctorRepositoryError {
    #[error("PWZ number already exists")]
    DuplicatedPwzNumber,
    #[error("PESEL number already exists")]
    DuplicatedPeselNumber,
    #[error("Database error: {0}")]
    DatabaseError(String),
}

#[derive(thiserror::Error, Debug, PartialEq)]
pub enum GetDoctorsRepositoryError {
    #[error("Invalid pagination parameters: {0}")]
    InvalidPaginationParams(String),
    #[error("Database error: {0}")]
    DatabaseError(String),
}

#[derive(thiserror::Error, Debug, PartialEq)]
pub enum GetDoctorByIdRepositoryError {
    #[error("Doctor with this id not found ({0})")]
    NotFound(Uuid),
    #[error("Database error: {0}")]
    DatabaseError(String),
}

#[async_trait]
pub trait DoctorsRepository: Send + Sync + 'static {
    async fn create_doctor(&self, doctor: NewDoctor)
        -> Result<Doctor, CreateDoctorRepositoryError>;
    async fn get_doctors(
        &self,
        page: Option<i64>,
        page_size: Option<i64>,
    ) -> Result<Vec<Doctor>, GetDoctorsRepositoryError>;
    async fn get_doctor_by_id(
        &self,
        doctor_id: Uuid,
    ) -> Result<Doctor, GetDoctorByIdRepositoryError>;
}
