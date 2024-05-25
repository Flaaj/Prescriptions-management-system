use std::sync::RwLock;

use async_trait::async_trait;
use chrono::Utc;
use uuid::Uuid;

use crate::domain::{
    drugs::models::{Drug, NewDrug},
    utils::pagination::get_pagination_params,
};

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

/// Used to test the service layer in isolation
pub struct InMemoryDrugsRepository {
    drugs: RwLock<Vec<Drug>>,
}

impl InMemoryDrugsRepository {
    #[allow(dead_code)]
    pub fn new() -> Self {
        Self {
            drugs: RwLock::new(Vec::new()),
        }
    }
}

#[async_trait]
impl DrugsRepository for InMemoryDrugsRepository {
    async fn create_drug(&self, new_drug: NewDrug) -> anyhow::Result<Drug> {
        let drug = Drug {
            id: new_drug.id,
            name: new_drug.name,
            content_type: new_drug.content_type,
            mg_per_pill: new_drug.mg_per_pill,
            ml_per_pill: new_drug.ml_per_pill,
            pills_count: new_drug.pills_count,
            volume_ml: new_drug.volume_ml,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        self.drugs.write().unwrap().push(drug.clone());

        Ok(drug)
    }

    async fn get_drugs(
        &self,
        page: Option<i64>,
        page_size: Option<i64>,
    ) -> anyhow::Result<Vec<Drug>> {
        let (page_size, offset) = get_pagination_params(page, page_size)?;
        let a = offset;
        let b = offset + page_size;

        let mut drugs: Vec<Drug> = vec![];
        for i in a..b {
            match self.drugs.read().unwrap().get(i as usize) {
                Some(drug) => drugs.push(drug.clone()),
                None => {}
            }
        }

        Ok(drugs)
    }

    async fn get_drug_by_id(&self, drug_id: Uuid) -> anyhow::Result<Drug> {
        match self
            .drugs
            .read()
            .unwrap()
            .iter()
            .find(|drug| drug.id == drug_id)
        {
            Some(drug) => Ok(drug.clone()),
            None => Err(anyhow::anyhow!("Drug not found")),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{DrugsRepository, InMemoryDrugsRepository};
    use crate::domain::drugs::models::{DrugContentType, NewDrug};

    async fn setup_repository() -> InMemoryDrugsRepository {
        InMemoryDrugsRepository::new()
    }

    #[sqlx::test]
    async fn create_and_read_drug_by_id() {
        let repository = setup_repository().await;

        let drug = NewDrug::new(
            "Gripex Max".into(),
            DrugContentType::SolidPills,
            Some(20),
            Some(300),
            None,
            None,
        )
        .unwrap();

        let created_drug = repository.create_drug(drug.clone()).await.unwrap();

        assert_eq!(drug, created_drug);

        let drug_from_repo = repository.get_drug_by_id(drug.id).await.unwrap();

        assert_eq!(drug, drug_from_repo);
    }

    #[sqlx::test]
    async fn create_and_read_drugs_from_database() {
        let repository = setup_repository().await;

        let new_drug_0 = NewDrug::new(
            "Gripex".into(),
            DrugContentType::SolidPills,
            Some(20),
            Some(300),
            None,
            None,
        )
        .unwrap();
        let new_drug_1 = NewDrug::new(
            "Apap".into(),
            DrugContentType::SolidPills,
            Some(10),
            Some(400),
            None,
            None,
        )
        .unwrap();
        let new_drug_2 = NewDrug::new(
            "Aspirin".into(),
            DrugContentType::SolidPills,
            Some(30),
            Some(200),
            None,
            None,
        )
        .unwrap();
        let new_drug_3 = NewDrug::new(
            "Flegamax".into(),
            DrugContentType::BottleOfLiquid,
            None,
            None,
            None,
            Some(400),
        )
        .unwrap();

        repository.create_drug(new_drug_0.clone()).await.unwrap();
        repository.create_drug(new_drug_1.clone()).await.unwrap();
        repository.create_drug(new_drug_2.clone()).await.unwrap();
        repository.create_drug(new_drug_3.clone()).await.unwrap();

        let drugs = repository.get_drugs(None, Some(10)).await.unwrap();

        assert_eq!(drugs.len(), 4);
        assert_eq!(drugs[0], new_drug_0);
        assert_eq!(drugs[1], new_drug_1);
        assert_eq!(drugs[2], new_drug_2);
        assert_eq!(drugs[3], new_drug_3);

        let drugs = repository.get_drugs(None, Some(2)).await.unwrap();

        assert_eq!(drugs.len(), 2);
        assert_eq!(drugs[0], new_drug_0);
        assert_eq!(drugs[1], new_drug_1);

        let drugs = repository.get_drugs(Some(1), Some(3)).await.unwrap();

        assert_eq!(drugs.len(), 1);
        assert_eq!(drugs[0], new_drug_3);

        let drugs = repository.get_drugs(Some(2), Some(3)).await.unwrap();

        assert_eq!(drugs.len(), 0);
    }
}
