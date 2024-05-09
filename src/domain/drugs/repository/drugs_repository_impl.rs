use async_trait::async_trait;
use sqlx::Row;
use uuid::Uuid;

use crate::{
    domain::drugs::models::{Drug, NewDrug},
    utils::pagination::get_pagination_params,
};

use super::drugs_repository_trait::DrugsRepositoryTrait;

pub struct DrugsRepository<'a> {
    pool: &'a sqlx::PgPool,
}

impl<'a> DrugsRepository<'a> {
    pub fn new(pool: &'a sqlx::PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl<'a> DrugsRepositoryTrait for DrugsRepository<'a> {
    async fn create_drug(&self, drug: NewDrug) -> anyhow::Result<()> {
        sqlx::query!(
            r#"INSERT INTO drugs (id, name, content_type, pills_count, mg_per_pill, ml_per_pill, volume_ml) VALUES ($1, $2, $3, $4, $5, $6, $7)"#,
            drug.id,
            drug.name,
            drug.content_type as _,
            drug.pills_count,
            drug.mg_per_pill,
            drug.ml_per_pill,
            drug.volume_ml
        )
        .execute(self.pool)
        .await?;

        Ok(())
    }

    async fn get_drugs(
        &self,
        page: Option<i64>,
        page_size: Option<i64>,
    ) -> anyhow::Result<Vec<Drug>> {
        let (page_size, offset) = get_pagination_params(page, page_size)?;

        let drugs_from_db = sqlx::query(
            r#"SELECT id, name, content_type, pills_count, mg_per_pill, ml_per_pill, volume_ml, created_at, updated_at FROM drugs LIMIT $1 OFFSET $2"#,
        )
        .bind(page_size)
        .bind(offset)
        .fetch_all(self.pool)
        .await?;

        let drugs = drugs_from_db
            .into_iter()
            .map(|record| Drug {
                id: record.get(0),
                name: record.get(1),
                content_type: record.get(2),
                pills_count: record.get(3),
                mg_per_pill: record.get(4),
                ml_per_pill: record.get(5),
                volume_ml: record.get(6),
                created_at: record.get(7),
                updated_at: record.get(8),
            })
            .collect();

        Ok(drugs)
    }

    async fn get_drug_by_id(&self, drug_id: Uuid) -> anyhow::Result<Drug> {
        let drug_from_db = sqlx::query(
            r#"SELECT id, name, content_type, pills_count, mg_per_pill, ml_per_pill, volume_ml, created_at, updated_at FROM drugs WHERE id = $1"#,
        )
        .bind(drug_id)
        .fetch_one(self.pool)
        .await?;

        Ok(Drug {
            id: drug_from_db.get(0),
            name: drug_from_db.get(1),
            content_type: drug_from_db.get(2),
            pills_count: drug_from_db.get(3),
            mg_per_pill: drug_from_db.get(4),
            ml_per_pill: drug_from_db.get(5),
            volume_ml: drug_from_db.get(6),
            created_at: drug_from_db.get(7),
            updated_at: drug_from_db.get(8),
        })
    }
}

#[cfg(test)]
mod integration_tests {
    use crate::{
        create_tables::create_tables,
        domain::drugs::{
            models::{DrugContentType, NewDrug},
            repository::{
                drugs_repository_impl::DrugsRepository,
                drugs_repository_trait::DrugsRepositoryTrait,
            },
        },
    };

    #[sqlx::test]
    async fn create_and_read_drugs_from_database(pool: sqlx::PgPool) {
        create_tables(&pool, true).await.unwrap();
        let repo = DrugsRepository::new(&pool);

        for _ in 0..4 {
            repo.create_drug(
                NewDrug::new(
                    "Gripex".into(),
                    DrugContentType::SolidPills,
                    Some(20),
                    Some(300),
                    None,
                    None,
                )
                .unwrap(),
            )
            .await
            .unwrap();
        }

        let drugs = repo.get_drugs(None, Some(2)).await.unwrap();
        assert_eq!(drugs.len(), 2);

        let drugs = repo.get_drugs(None, Some(10)).await.unwrap();
        assert_eq!(drugs.len(), 4);

        let drugs = repo.get_drugs(Some(1), Some(3)).await.unwrap();
        assert_eq!(drugs.len(), 1);

        let drugs = repo.get_drugs(Some(2), Some(3)).await.unwrap();
        assert_eq!(drugs.len(), 0);
    }

    #[sqlx::test]
    async fn create_and_read_drug_by_id(pool: sqlx::PgPool) {
        create_tables(&pool, true).await.unwrap();
        let repo = DrugsRepository::new(&pool);

        let drug = NewDrug::new(
            "Gripex".into(),
            DrugContentType::SolidPills,
            Some(20),
            Some(300),
            None,
            None,
        )
        .unwrap();

        repo.create_drug(drug.clone()).await.unwrap();

        let drug_from_repo = repo.get_drug_by_id(drug.id).await.unwrap();

        assert_eq!(drug_from_repo.id, drug.id);
    }
}
