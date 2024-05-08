use async_trait::async_trait;
use uuid::Uuid;

use crate::domain::pharmacists::models::{NewPharmacist, Pharmacist};

use super::pharmacists_repository_trait::PharmacistsRepositoryTrait;

pub struct PharmacistsRepository<'a> {
    pool: &'a sqlx::PgPool,
}

impl<'a> PharmacistsRepository<'a> {
    pub fn new(pool: &'a sqlx::PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl<'a> PharmacistsRepositoryTrait for PharmacistsRepository<'a> {
    async fn create_pharmacist(&self, pharmacist: NewPharmacist) -> anyhow::Result<()> {
        sqlx::query!(
            r#"INSERT INTO pharmacists (id, name, pesel_number) VALUES ($1, $2, $3)"#,
            pharmacist.id,
            pharmacist.name,
            pharmacist.pesel_number
        )
        .execute(self.pool)
        .await?;

        Ok(())
    }

    async fn get_pharmacists(&self) -> anyhow::Result<Vec<Pharmacist>> {
        let pharmacists_from_db = sqlx::query!(
            r#"SELECT id, name, pesel_number, created_at, updated_at FROM pharmacists"#,
        )
        .fetch_all(self.pool)
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
        .fetch_one(self.pool)
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
        let repo = PharmacistsRepository::new(&pool);

        let pharmacist = NewPharmacist::new("John Doe".into(), "96021817257".into()).unwrap();

        repo.create_pharmacist(pharmacist).await.unwrap();

        let pharmacists = repo.get_pharmacists().await.unwrap();
        let first_pharmacist = pharmacists.first().unwrap();

        assert_eq!(first_pharmacist.name, "John Doe");
        assert_eq!(first_pharmacist.pesel_number, "96021817257");
    }

    #[sqlx::test]
    async fn create_and_read_pharmacist_by_id(pool: sqlx::PgPool) {
        create_tables(&pool, true).await.unwrap();
        let repo = PharmacistsRepository::new(&pool);

        let pharmacist = NewPharmacist::new("John Doe".into(), "96021817257".into()).unwrap();

        repo.create_pharmacist(pharmacist.clone()).await.unwrap();

        let pharmacist_from_repo = repo.get_pharmacist_by_id(pharmacist.id).await.unwrap();

        assert_eq!(pharmacist_from_repo.id, pharmacist.id);
    }
}
