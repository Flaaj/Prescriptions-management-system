use async_trait::async_trait;
use uuid::Uuid;

use crate::domain::pharmacists::models::{Pharmacist, NewPharmacist};

#[async_trait]
pub trait PharmacistsRepositoryTrait {
    async fn create_pharmacist(&self, pharmacist: NewPharmacist) -> anyhow::Result<()>;
    async fn get_pharmacists(&self) -> anyhow::Result<Vec<Pharmacist>>;
    async fn get_pharmacist_by_id(&self, pharmacist_id: Uuid) -> anyhow::Result<Pharmacist>;
}
