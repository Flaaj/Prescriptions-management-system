use async_trait::async_trait;
use uuid::Uuid;

use crate::domain::pharmacists::models::{NewPharmacist, Pharmacist};

#[async_trait]
pub trait PharmacistsRepositoryTrait {
    async fn create_pharmacist(&self, pharmacist: NewPharmacist) -> anyhow::Result<Pharmacist>;
    async fn get_pharmacists(
        &self,
        page: Option<i64>,
        page_size: Option<i64>,
    ) -> anyhow::Result<Vec<Pharmacist>>;
    async fn get_pharmacist_by_id(&self, pharmacist_id: Uuid) -> anyhow::Result<Pharmacist>;
}
