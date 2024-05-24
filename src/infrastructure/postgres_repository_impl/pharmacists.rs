use async_trait::async_trait;
use uuid::Uuid;

use crate::domain::{
    pharmacists::{
        models::{NewPharmacist, Pharmacist},
        repository::PharmacistsRepository,
    },
    utils::pagination::get_pagination_params,
};

pub struct PharmacistsPostgresRepository {
    pool: sqlx::PgPool,
}

impl PharmacistsPostgresRepository {
    pub fn new(pool: sqlx::PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl PharmacistsRepository for PharmacistsPostgresRepository {
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
    use super::PharmacistsPostgresRepository;
    use crate::{
        create_tables::create_tables,
        domain::pharmacists::{models::NewPharmacist, repository::PharmacistsRepository},
    };

    #[sqlx::test]
    async fn create_and_read_pharmacists_from_database(pool: sqlx::PgPool) {
        create_tables(&pool, true).await.unwrap();
        let repository = PharmacistsPostgresRepository::new(pool);

        let new_pharmacist_0 = NewPharmacist::new("John Doe".into(), "96021817257".into()).unwrap();
        let new_pharmacist_1 = NewPharmacist::new("John Doe".into(), "99031301347".into()).unwrap();
        let new_pharmacist_2 = NewPharmacist::new("John Doe".into(), "92022900002".into()).unwrap();
        let new_pharmacist_3 = NewPharmacist::new("John Doe".into(), "96021807250".into()).unwrap();

        repository
            .create_pharmacist(new_pharmacist_0.clone())
            .await
            .unwrap();
        repository
            .create_pharmacist(new_pharmacist_1.clone())
            .await
            .unwrap();
        repository
            .create_pharmacist(new_pharmacist_2.clone())
            .await
            .unwrap();
        repository
            .create_pharmacist(new_pharmacist_3.clone())
            .await
            .unwrap();

        let pharmacists = repository.get_pharmacists(None, Some(10)).await.unwrap();

        assert!(pharmacists.len() == 4);
        assert_eq!(pharmacists[0], new_pharmacist_0);
        assert_eq!(pharmacists[1], new_pharmacist_1);
        assert_eq!(pharmacists[2], new_pharmacist_2);
        assert_eq!(pharmacists[3], new_pharmacist_3);

        let pharmacists = repository.get_pharmacists(None, Some(2)).await.unwrap();

        assert_eq!(pharmacists.len(), 2);
        assert_eq!(pharmacists[0], new_pharmacist_0);
        assert_eq!(pharmacists[1], new_pharmacist_1);

        let pharmacists = repository.get_pharmacists(Some(1), Some(3)).await.unwrap();

        assert!(pharmacists.len() == 1);
        assert_eq!(pharmacists[0], new_pharmacist_3);

        let pharmacists = repository.get_pharmacists(Some(2), Some(3)).await.unwrap();

        assert!(pharmacists.len() == 0);
    }

    #[sqlx::test]
    async fn create_and_read_pharmacist_by_id(pool: sqlx::PgPool) {
        create_tables(&pool, true).await.unwrap();
        let repository = PharmacistsPostgresRepository::new(pool);

        let new_pharmacist = NewPharmacist::new("John Doe".into(), "96021817257".into()).unwrap();

        let created_pharmacist = repository
            .create_pharmacist(new_pharmacist.clone())
            .await
            .unwrap();

        assert_eq!(created_pharmacist, new_pharmacist);

        let pharmacist_from_repo = repository
            .get_pharmacist_by_id(new_pharmacist.id)
            .await
            .unwrap();

        assert_eq!(pharmacist_from_repo, new_pharmacist);
    }

    #[sqlx::test]
    async fn doesnt_create_pharmacist_if_pesel_number_is_duplicated(pool: sqlx::PgPool) {
        create_tables(&pool, true).await.unwrap();
        let repository = PharmacistsPostgresRepository::new(pool);

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
