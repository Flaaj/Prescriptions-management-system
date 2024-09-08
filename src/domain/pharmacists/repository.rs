use std::sync::RwLock;

use async_trait::async_trait;
use chrono::Utc;
use uuid::Uuid;

use crate::domain::{
    pharmacists::entities::{NewPharmacist, Pharmacist},
    utils::pagination::get_pagination_params,
};

#[derive(thiserror::Error, Debug, PartialEq)]
pub enum CreatePharmacistRepositoryError {
    #[error("PESEL number already exists")]
    DuplicatedPeselNumber,
    #[error("Database error: {0}")]
    DatabaseError(String),
}

#[derive(thiserror::Error, Debug, PartialEq)]
pub enum GetPharmacistsRepositoryError {
    #[error("Invalid pagination parameters: {0}")]
    InvalidPaginationParams(String),
    #[error("Database error: {0}")]
    DatabaseError(String),
}

#[derive(thiserror::Error, Debug, PartialEq)]
pub enum GetPharmacistByIdRepositoryError {
    #[error("Pharmacist with this id not found ({0})")]
    NotFound(Uuid),
    #[error("Database error: {0}")]
    DatabaseError(String),
}

#[async_trait]
pub trait PharmacistsRepository: Send + Sync + 'static {
    async fn create_pharmacist(
        &self,
        pharmacist: NewPharmacist,
    ) -> Result<Pharmacist, CreatePharmacistRepositoryError>;
    async fn get_pharmacists(
        &self,
        page: Option<i64>,
        page_size: Option<i64>,
    ) -> Result<Vec<Pharmacist>, GetPharmacistsRepositoryError>;
    async fn get_pharmacist_by_id(
        &self,
        pharmacist_id: Uuid,
    ) -> Result<Pharmacist, GetPharmacistByIdRepositoryError>;
}
