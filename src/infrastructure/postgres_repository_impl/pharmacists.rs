use async_trait::async_trait;
use uuid::Uuid;

use crate::domain::{
    pharmacists::{
        models::{NewPharmacist, Pharmacist},
        repository::{
            CreatePharmacistRepositoryError, GetPharmacistByIdRepositoryError,
            GetPharmacistsRepositoryError, PharmacistsRepository,
        },
    },
    utils::pagination::get_pagination_params,
};

pub struct PostgresPharmacistsRepository {
    pool: sqlx::PgPool,
}

impl PostgresPharmacistsRepository {
    pub fn new(pool: sqlx::PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl PharmacistsRepository for PostgresPharmacistsRepository {
    async fn create_pharmacist(
        &self,
        pharmacist: NewPharmacist,
    ) -> Result<Pharmacist, CreatePharmacistRepositoryError> {
        let result = sqlx::query!(
            r#"INSERT INTO pharmacists (id, name, pesel_number) VALUES ($1, $2, $3) RETURNING id, name, pesel_number, created_at, updated_at"#,
            pharmacist.id,
            pharmacist.name,
            pharmacist.pesel_number
        )
        .fetch_one(&self.pool)
        .await.
        map_err(|_| CreatePharmacistRepositoryError::DuplicatedPeselNumber)?;

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
    ) -> Result<Vec<Pharmacist>, GetPharmacistsRepositoryError> {
        let (page_size, offset) = get_pagination_params(page, page_size).map_err(|err| {
            GetPharmacistsRepositoryError::InvalidPaginationParams(err.to_string())
        })?;

        let pharmacists_from_db = sqlx::query!(
            r#"SELECT id, name, pesel_number, created_at, updated_at FROM pharmacists LIMIT $1 OFFSET $2"#,
            page_size,
            offset
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|err| GetPharmacistsRepositoryError::DatabaseError(err.to_string()))?;

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

    async fn get_pharmacist_by_id(
        &self,
        pharmacist_id: Uuid,
    ) -> Result<Pharmacist, GetPharmacistByIdRepositoryError> {
        let pharmacist_from_db = sqlx::query!(
            r#"SELECT id, name, pesel_number, created_at, updated_at FROM pharmacists WHERE id = $1"#,
            pharmacist_id
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|_| GetPharmacistByIdRepositoryError::NotFound(pharmacist_id))?;

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
mod tests {
    use std::assert_matches::assert_matches;

    use uuid::Uuid;

    use super::PostgresPharmacistsRepository;
    use crate::{
        create_tables::create_tables,
        domain::pharmacists::{
            models::NewPharmacist,
            repository::{
                CreatePharmacistRepositoryError, GetPharmacistByIdRepositoryError,
                GetPharmacistsRepositoryError, PharmacistsRepository,
            },
        },
    };

    async fn setup_repository(pool: sqlx::PgPool) -> PostgresPharmacistsRepository {
        create_tables(&pool, true).await.unwrap();
        PostgresPharmacistsRepository::new(pool)
    }

    #[sqlx::test]
    async fn create_and_read_pharmacist_by_id(pool: sqlx::PgPool) {
        let repository = setup_repository(pool).await;

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
    async fn returns_error_if_pharmacists_with_given_id_doesnt_exist(pool: sqlx::PgPool) {
        let repository = setup_repository(pool).await;
        let pharmacist_id = Uuid::new_v4();

        let pharmacist_from_repo = repository.get_pharmacist_by_id(pharmacist_id).await;

        assert_eq!(
            pharmacist_from_repo,
            Err(GetPharmacistByIdRepositoryError::NotFound(pharmacist_id))
        );
    }

    #[sqlx::test]
    async fn create_and_read_pharmacists_from_database(pool: sqlx::PgPool) {
        let repository = setup_repository(pool).await;

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

        assert_eq!(pharmacists.len(), 4);
        assert_eq!(pharmacists[0], new_pharmacist_0);
        assert_eq!(pharmacists[1], new_pharmacist_1);
        assert_eq!(pharmacists[2], new_pharmacist_2);
        assert_eq!(pharmacists[3], new_pharmacist_3);

        let pharmacists = repository.get_pharmacists(None, Some(2)).await.unwrap();

        assert_eq!(pharmacists.len(), 2);
        assert_eq!(pharmacists[0], new_pharmacist_0);
        assert_eq!(pharmacists[1], new_pharmacist_1);

        let pharmacists = repository.get_pharmacists(Some(1), Some(3)).await.unwrap();

        assert_eq!(pharmacists.len(), 1);
        assert_eq!(pharmacists[0], new_pharmacist_3);

        let pharmacists = repository.get_pharmacists(Some(2), Some(3)).await.unwrap();

        assert_eq!(pharmacists.len(), 0);
    }

    #[sqlx::test]
    async fn get_patients_returns_error_if_pagination_params_are_incorrect(pool: sqlx::PgPool) {
        let repository = setup_repository(pool).await;

        assert_matches!(
            repository.get_pharmacists(Some(-1), Some(10)).await,
            Err(GetPharmacistsRepositoryError::InvalidPaginationParams(_))
        );

        assert_matches!(
            repository.get_pharmacists(Some(0), Some(0)).await,
            Err(GetPharmacistsRepositoryError::InvalidPaginationParams(_))
        );
    }

    #[sqlx::test]
    async fn doesnt_create_pharmacist_if_pesel_number_is_duplicated(pool: sqlx::PgPool) {
        let repository = setup_repository(pool).await;

        let pharmacist = NewPharmacist::new("John Doe".into(), "96021817257".into()).unwrap();

        assert!(repository.create_pharmacist(pharmacist).await.is_ok());

        let pharmacist_with_duplicated_pesel_number =
            NewPharmacist::new("John Doe".into(), "96021817257".into()).unwrap();

        assert_eq!(
            repository
                .create_pharmacist(pharmacist_with_duplicated_pesel_number)
                .await,
            Err(CreatePharmacistRepositoryError::DuplicatedPeselNumber)
        );
    }
}
