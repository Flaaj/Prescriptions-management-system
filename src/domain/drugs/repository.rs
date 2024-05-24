use async_trait::async_trait;
use uuid::Uuid;

use crate::domain::drugs::models::{Drug, NewDrug};

#[async_trait]
pub trait DrugsRepository {
    async fn create_drug(&self, drug: NewDrug) -> anyhow::Result<Drug>;
    async fn get_drugs(
        &self,
        page: Option<i64>,
        page_size: Option<i64>,
    ) -> anyhow::Result<Vec<Drug>>;
    async fn get_drug_by_id(&self, drug_id: Uuid) -> anyhow::Result<Drug>;
}
