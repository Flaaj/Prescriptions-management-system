use crate::domain::{
    drugs::{
        models::{Drug, NewDrug},
        repository::{
            CreateDrugRepositoryError, DrugsRepository, GetDrugByIdRepositoryError,
            GetDrugsRepositoryError,
        },
    },
    utils::pagination::get_pagination_params,
};
use async_trait::async_trait;
use sqlx::{postgres::PgRow, Row};
use uuid::Uuid;

pub struct PostgresDrugsRepository {
    pool: sqlx::PgPool,
}

impl PostgresDrugsRepository {
    pub fn new(pool: sqlx::PgPool) -> Self {
        Self { pool }
    }

    fn get_drug_from_pg_row(&self, row: PgRow) -> Result<Drug, sqlx::Error> {
        Ok(Drug {
            id: row.try_get(0)?,
            name: row.try_get(1)?,
            content_type: row.try_get(2)?,
            pills_count: row.try_get(3)?,
            mg_per_pill: row.try_get(4)?,
            ml_per_pill: row.try_get(5)?,
            volume_ml: row.try_get(6)?,
            created_at: row.try_get(7)?,
            updated_at: row.try_get(8)?,
        })
    }
}

#[async_trait]
impl DrugsRepository for PostgresDrugsRepository {
    async fn create_drug(&self, drug: NewDrug) -> Result<Drug, CreateDrugRepositoryError> {
        let result = sqlx::query(
                r#"INSERT INTO drugs (id, name, content_type, pills_count, mg_per_pill, ml_per_pill, volume_ml) VALUES ($1, $2, $3, $4, $5, $6, $7) RETURNING id, name, content_type, pills_count, mg_per_pill, ml_per_pill, volume_ml, created_at, updated_at"#
            )
            .bind(drug.id)
            .bind(drug.name)
            .bind(drug.content_type)
            .bind(drug.pills_count)
            .bind(drug.mg_per_pill)
            .bind(drug.ml_per_pill)
            .bind(drug.volume_ml)
            .fetch_one(&self.pool).await
            .map_err(|err| CreateDrugRepositoryError::DatabaseError(err.to_string()))?;

        Ok(self
            .get_drug_from_pg_row(result)
            .map_err(|err| CreateDrugRepositoryError::DatabaseError(err.to_string()))?)
    }

    async fn get_drugs(
        &self,
        page: Option<i64>,
        page_size: Option<i64>,
    ) -> Result<Vec<Drug>, GetDrugsRepositoryError> {
        let (page_size, offset) = get_pagination_params(page, page_size)
            .map_err(|err| GetDrugsRepositoryError::InvalidPaginationParams(err.to_string()))?;

        let drugs_from_db = sqlx::query(
                r#"SELECT id, name, content_type, pills_count, mg_per_pill, ml_per_pill, volume_ml, created_at, updated_at FROM drugs LIMIT $1 OFFSET $2"#
            )
            .bind(page_size)
            .bind(offset)
            .fetch_all(&self.pool).await
            .map_err(|err| GetDrugsRepositoryError::DatabaseError(err.to_string()))?;

        let mut drugs = vec![];
        for record in drugs_from_db {
            let drug = self
                .get_drug_from_pg_row(record)
                .map_err(|err| GetDrugsRepositoryError::DatabaseError(err.to_string()))?;
            drugs.push(drug);
        }

        Ok(drugs)
    }

    async fn get_drug_by_id(&self, drug_id: Uuid) -> Result<Drug, GetDrugByIdRepositoryError> {
        let drug_from_db = sqlx::query(
                r#"SELECT id, name, content_type, pills_count, mg_per_pill, ml_per_pill, volume_ml, created_at, updated_at FROM drugs WHERE id = $1"#
            )
            .bind(drug_id)
            .fetch_one(&self.pool).await
            .map_err(|err| {
                match err {
                    sqlx::Error::RowNotFound => GetDrugByIdRepositoryError::NotFound(drug_id),
                    _ => GetDrugByIdRepositoryError::DatabaseError(err.to_string()),
                }
            })?;

        Ok(self
            .get_drug_from_pg_row(drug_from_db)
            .map_err(|err| GetDrugByIdRepositoryError::DatabaseError(err.to_string()))?)
    }
}

#[cfg(test)]
mod tests {
    use std::assert_matches::assert_matches;

    use uuid::Uuid;

    use super::{DrugsRepository, PostgresDrugsRepository};
    use crate::{
        create_tables::create_tables,
        domain::drugs::{
            models::{DrugContentType, NewDrug},
            repository::{GetDrugByIdRepositoryError, GetDrugsRepositoryError},
        },
    };

    async fn setup_repository(pool: sqlx::PgPool) -> PostgresDrugsRepository {
        create_tables(&pool, true).await.unwrap();
        PostgresDrugsRepository::new(pool)
    }

    #[sqlx::test]
    async fn create_and_read_drug_by_id(pool: sqlx::PgPool) {
        let repository = setup_repository(pool).await;

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
    async fn returns_error_if_drug_with_given_id_doesnt_exist(pool: sqlx::PgPool) {
        let repository = setup_repository(pool).await;
        let drug_id = Uuid::new_v4();

        let drug_from_repo = repository.get_drug_by_id(drug_id).await;

        assert_eq!(
            drug_from_repo,
            Err(GetDrugByIdRepositoryError::NotFound(drug_id))
        );
    }

    #[sqlx::test]
    async fn create_and_read_drugs_from_database(pool: sqlx::PgPool) {
        let repository = setup_repository(pool).await;

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

    #[sqlx::test]
    async fn get_drugs_returns_error_if_pagination_params_are_incorrect(pool: sqlx::PgPool) {
        let repository = setup_repository(pool).await;

        assert_matches!(
            repository.get_drugs(Some(-1), Some(10)).await,
            Err(GetDrugsRepositoryError::InvalidPaginationParams(_))
        );

        assert_matches!(
            repository.get_drugs(Some(0), Some(0)).await,
            Err(GetDrugsRepositoryError::InvalidPaginationParams(_))
        );
    }
}
