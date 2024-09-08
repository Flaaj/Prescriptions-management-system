use std::sync::RwLock;

use async_trait::async_trait;
use chrono::Utc;
use uuid::Uuid;

use crate::domain::{
    drugs::entities::{Drug, NewDrug},
    utils::pagination::get_pagination_params,
};

#[derive(thiserror::Error, Debug, PartialEq)]
pub enum CreateDrugRepositoryError {
    #[error("Database error: {0}")]
    DatabaseError(String),
}

#[derive(thiserror::Error, Debug, PartialEq)]
pub enum GetDrugsRepositoryError {
    #[error("Invalid pagination parameters: {0}")]
    InvalidPaginationParams(String),
    #[error("Database error: {0}")]
    DatabaseError(String),
}

#[derive(thiserror::Error, Debug, PartialEq)]
pub enum GetDrugByIdRepositoryError {
    #[error("Drug with this id not found ({0})")]
    NotFound(Uuid),
    #[error("Database error: {0}")]
    DatabaseError(String),
}

#[async_trait]
pub trait DrugsRepository: Send + Sync + 'static {
    async fn create_drug(&self, drug: NewDrug) -> Result<Drug, CreateDrugRepositoryError>;
    async fn get_drugs(
        &self,
        page: Option<i64>,
        page_size: Option<i64>,
    ) -> Result<Vec<Drug>, GetDrugsRepositoryError>;
    async fn get_drug_by_id(&self, drug_id: Uuid) -> Result<Drug, GetDrugByIdRepositoryError>;
}
