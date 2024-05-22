use async_trait::async_trait;
use sqlx::Row;
use uuid::Uuid;

use crate::{
    domain::drugs::models::{Drug, NewDrug},
    utils::pagination::get_pagination_params,
};

use super::drugs_repository_trait::DrugsRepositoryTrait;

pub struct DrugsRepository {
    pool: sqlx::PgPool,
}

impl DrugsRepository {
    pub fn new(pool: sqlx::PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl<'a> DrugsRepositoryTrait for DrugsRepository {
    async fn create_drug(&self, drug: NewDrug) -> anyhow::Result<Drug> {
        let result = sqlx::query(
            r#"INSERT INTO drugs (id, name, content_type, pills_count, mg_per_pill, ml_per_pill, volume_ml) VALUES ($1, $2, $3, $4, $5, $6, $7) RETURNING id, name, content_type, pills_count, mg_per_pill, ml_per_pill, volume_ml, created_at, updated_at"#,
        )
        .bind(drug.id)
        .bind(drug.name)
        .bind(drug.content_type)
        .bind(drug.pills_count)
        .bind(drug.mg_per_pill)
        .bind(drug.ml_per_pill)
        .bind(drug.volume_ml)
        .fetch_one(&self.pool)
        .await?;

        Ok(Drug {
            id: result.get(0),
            name: result.get(1),
            content_type: result.get(2),
            pills_count: result.get(3),
            mg_per_pill: result.get(4),
            ml_per_pill: result.get(5),
            volume_ml: result.get(6),
            created_at: result.get(7),
            updated_at: result.get(8),
        })
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
        .fetch_all(&self.pool)
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
        .fetch_one(&self.pool)
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
        let repository = DrugsRepository::new(pool);

        let result = repository
            .create_drug(
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

        assert_eq!(result.name, "Gripex");
        assert_eq!(result.content_type, DrugContentType::SolidPills);
        assert_eq!(result.pills_count, Some(20));
        assert_eq!(result.mg_per_pill, Some(300));
        assert_eq!(result.ml_per_pill, None);
        assert_eq!(result.volume_ml, None);

        repository
            .create_drug(
                NewDrug::new(
                    "Apap".into(),
                    DrugContentType::SolidPills,
                    Some(10),
                    Some(400),
                    None,
                    None,
                )
                .unwrap(),
            )
            .await
            .unwrap();
        repository
            .create_drug(
                NewDrug::new(
                    "Aspirin".into(),
                    DrugContentType::SolidPills,
                    Some(30),
                    Some(200),
                    None,
                    None,
                )
                .unwrap(),
            )
            .await
            .unwrap();
        repository
            .create_drug(
                NewDrug::new(
                    "Flegamax".into(),
                    DrugContentType::BottleOfLiquid,
                    None,
                    None,
                    None,
                    Some(400),
                )
                .unwrap(),
            )
            .await
            .unwrap();

        let drugs = repository.get_drugs(None, Some(10)).await.unwrap();

        assert_eq!(drugs.len(), 4);
        assert_eq!(drugs[0].name, "Gripex");
        assert_eq!(drugs[0].content_type, DrugContentType::SolidPills);
        assert_eq!(drugs[0].pills_count, Some(20));
        assert_eq!(drugs[0].mg_per_pill, Some(300));
        assert_eq!(drugs[0].ml_per_pill, None);
        assert_eq!(drugs[0].volume_ml, None);
        assert_eq!(drugs[1].name, "Apap");
        assert_eq!(drugs[1].content_type, DrugContentType::SolidPills);
        assert_eq!(drugs[1].pills_count, Some(10));
        assert_eq!(drugs[1].mg_per_pill, Some(400));
        assert_eq!(drugs[1].ml_per_pill, None);
        assert_eq!(drugs[1].volume_ml, None);
        assert_eq!(drugs[2].name, "Aspirin");
        assert_eq!(drugs[2].content_type, DrugContentType::SolidPills);
        assert_eq!(drugs[2].pills_count, Some(30));
        assert_eq!(drugs[2].mg_per_pill, Some(200));
        assert_eq!(drugs[2].ml_per_pill, None);
        assert_eq!(drugs[2].volume_ml, None);
        assert_eq!(drugs[3].name, "Flegamax");
        assert_eq!(drugs[3].content_type, DrugContentType::BottleOfLiquid);
        assert_eq!(drugs[3].pills_count, None);
        assert_eq!(drugs[3].mg_per_pill, None);
        assert_eq!(drugs[3].ml_per_pill, None);
        assert_eq!(drugs[3].volume_ml, Some(400));

        let drugs = repository.get_drugs(None, Some(2)).await.unwrap();

        assert_eq!(drugs.len(), 2);

        let drugs = repository.get_drugs(Some(1), Some(3)).await.unwrap();

        assert_eq!(drugs.len(), 1);

        let drugs = repository.get_drugs(Some(2), Some(3)).await.unwrap();

        assert_eq!(drugs.len(), 0);
    }

    #[sqlx::test]
    async fn create_and_read_drug_by_id(pool: sqlx::PgPool) {
        create_tables(&pool, true).await.unwrap();
        let repository = DrugsRepository::new(pool);

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

        assert_eq!(created_drug.name, "Gripex Max");
        assert_eq!(created_drug.content_type, DrugContentType::SolidPills);
        assert_eq!(created_drug.pills_count, Some(20));
        assert_eq!(created_drug.mg_per_pill, Some(300));
        assert_eq!(created_drug.ml_per_pill, None);
        assert_eq!(created_drug.volume_ml, None);

        let drug_from_repo = repository.get_drug_by_id(drug.id).await.unwrap();

        assert_eq!(drug_from_repo.name, "Gripex Max");
        assert_eq!(drug_from_repo.content_type, DrugContentType::SolidPills);
        assert_eq!(drug_from_repo.pills_count, Some(20));
        assert_eq!(drug_from_repo.mg_per_pill, Some(300));
        assert_eq!(drug_from_repo.ml_per_pill, None);
        assert_eq!(drug_from_repo.volume_ml, None);
    }
}
