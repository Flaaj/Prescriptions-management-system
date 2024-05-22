use async_trait::async_trait;
use uuid::Uuid;

use crate::{
    domain::pharmacists::models::{NewPharmacist, Pharmacist},
    utils::pagination::get_pagination_params,
};

use super::pharmacists_repository_trait::PharmacistsRepositoryTrait;

pub struct PharmacistsRepository {
    pool: sqlx::PgPool,
}

impl PharmacistsRepository {
    pub fn new(pool: sqlx::PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl PharmacistsRepositoryTrait for PharmacistsRepository {
    async fn create_pharmacist(&self, pharmacist: NewPharmacist) -> anyhow::Result<Pharmacist> {
        let result = sqlx::query!(
            r#"INSERT INTO pharmacists (id, name, pesel_number) VALUES ($1, $2, $3) RETURNING id, name, pesel_number, created_at, updated_at"#,
            pharmacist.id,
            pharmacist.name,
            pharmacist.pesel_number
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(Pharmacist {
            id: result.id,
            name: result.name,
            pesel_number: result.pesel_number,
            created_at: result.created_at,
            updated_at: result.updated_at,
        })
    }

    async fn get_pharmacists(
        &self,
        page: Option<i64>,
        page_size: Option<i64>,
    ) -> anyhow::Result<Vec<Pharmacist>> {
        let (page_size, offset) = get_pagination_params(page, page_size)?;

        let pharmacists_from_db = sqlx::query!(
            r#"SELECT id, name, pesel_number, created_at, updated_at FROM pharmacists LIMIT $1 OFFSET $2"#,
            page_size,
            offset
        )
        .fetch_all(&self.pool)
        .await?;

        let pharmacists = pharmacists_from_db
            .into_iter()
            .map(|record| Pharmacist {
                id: record.id,
                name: record.name,
                pesel_number: record.pesel_number,
                created_at: record.created_at,
                updated_at: record.updated_at,
            })
            .collect();

        Ok(pharmacists)
    }

    async fn get_pharmacist_by_id(&self, pharmacist_id: Uuid) -> anyhow::Result<Pharmacist> {
        let pharmacist_from_db = sqlx::query!(
            r#"SELECT id, name, pesel_number, created_at, updated_at FROM pharmacists WHERE id = $1"#,
            pharmacist_id
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(Pharmacist {
            id: pharmacist_from_db.id,
            name: pharmacist_from_db.name,
            pesel_number: pharmacist_from_db.pesel_number,
            created_at: pharmacist_from_db.created_at,
            updated_at: pharmacist_from_db.updated_at,
        })
    }
}

#[cfg(test)]
mod integration_tests {
    use crate::{
        create_tables::create_tables,
        domain::pharmacists::{
            models::NewPharmacist,
            repository::{
                pharmacists_repository_impl::PharmacistsRepository,
                pharmacists_repository_trait::PharmacistsRepositoryTrait,
            },
        },
    };

    #[sqlx::test]
    async fn create_and_read_pharmacists_from_database(pool: sqlx::PgPool) {
        create_tables(&pool, true).await.unwrap();
        let repository = PharmacistsRepository::new(pool);

        repository
            .create_pharmacist(NewPharmacist::new("John Doe".into(), "96021817257".into()).unwrap())
            .await
            .unwrap();
        repository
            .create_pharmacist(NewPharmacist::new("John Doe".into(), "99031301347".into()).unwrap())
            .await
            .unwrap();
        repository
            .create_pharmacist(NewPharmacist::new("John Doe".into(), "92022900002".into()).unwrap())
            .await
            .unwrap();
        repository
            .create_pharmacist(NewPharmacist::new("John Doe".into(), "96021807250".into()).unwrap())
            .await
            .unwrap();

        let pharmacists = repository.get_pharmacists(None, Some(2)).await.unwrap();
        assert_eq!(pharmacists.len(), 2);

        let pharmacists = repository.get_pharmacists(None, Some(10)).await.unwrap();
        assert!(pharmacists.len() == 4);

        let pharmacists = repository.get_pharmacists(Some(1), Some(3)).await.unwrap();
        assert!(pharmacists.len() == 1);

        let pharmacists = repository.get_pharmacists(Some(2), Some(3)).await.unwrap();
        assert!(pharmacists.len() == 0);
    }

    #[sqlx::test]
    async fn create_and_read_pharmacist_by_id(pool: sqlx::PgPool) {
        create_tables(&pool, true).await.unwrap();
        let repository = PharmacistsRepository::new(pool);

        let pharmacist = NewPharmacist::new("John Doe".into(), "96021817257".into()).unwrap();

        let created_pharmacist = repository
            .create_pharmacist(pharmacist.clone())
            .await
            .unwrap();

        assert_eq!(created_pharmacist.name, "John Doe");
        assert_eq!(created_pharmacist.pesel_number, "96021817257");

        let pharmacist_from_repo = repository
            .get_pharmacist_by_id(pharmacist.id)
            .await
            .unwrap();

        assert_eq!(pharmacist_from_repo.name, "John Doe");
        assert_eq!(pharmacist_from_repo.pesel_number, "96021817257")
    }

    #[sqlx::test]
    async fn doesnt_create_pharmacist_if_pesel_number_is_duplicated(pool: sqlx::PgPool) {
        create_tables(&pool, true).await.unwrap();
        let repository = PharmacistsRepository::new(pool);

        let pharmacist = NewPharmacist::new("John Doe".into(), "96021817257".into()).unwrap();

        assert!(repository.create_pharmacist(pharmacist).await.is_ok());

        let pharmacist_with_duplicated_pesel_number =
            NewPharmacist::new("John Doe".into(), "96021817257".into()).unwrap();

        assert!(repository
            .create_pharmacist(pharmacist_with_duplicated_pesel_number)
            .await
            .is_err());
    }
}
