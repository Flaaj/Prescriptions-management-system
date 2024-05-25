use crate::domain::pharmacists::{
    models::{NewPharmacist, Pharmacist},
    repository::PharmacistsRepository,
};
use uuid::Uuid;

#[derive(Debug)]
pub enum CreatePharmacistError {
    DomainError(String),
    RepositoryError(String),
}

#[derive(Debug)]
pub enum GetPharmacistByIdError {
    DomainError,
    RepositoryError(String),
}

#[derive(Debug)]
pub enum GetPharmacistWithPaginationError {
    DomainError(String),
}

#[derive(Clone)]
pub struct PharmacistsService<R: PharmacistsRepository> {
    repository: R,
}

impl<R: PharmacistsRepository> PharmacistsService<R> {
    pub fn new(repository: R) -> Self {
        Self { repository }
    }

    pub async fn create_pharmacist(
        &self,
        name: String,
        pesel_number: String,
    ) -> Result<Pharmacist, CreatePharmacistError> {
        let new_pharmacist = NewPharmacist::new(name, pesel_number)
            .map_err(|err| CreatePharmacistError::DomainError(err.to_string()))?;

        let created_pharmacist = self
            .repository
            .create_pharmacist(new_pharmacist)
            .await
            .map_err(|err| CreatePharmacistError::RepositoryError(err.to_string()))?;

        Ok(created_pharmacist)
    }

    pub async fn get_pharmacist_by_id(
        &self,
        pharmacist_id: Uuid,
    ) -> Result<Pharmacist, GetPharmacistByIdError> {
        let pharmacist = self
            .repository
            .get_pharmacist_by_id(pharmacist_id)
            .await
            .map_err(|err| GetPharmacistByIdError::RepositoryError(err.to_string()))?;

        Ok(pharmacist)
    }

    pub async fn get_pharmacists_with_pagination(
        &self,
        page: Option<i64>,
        page_size: Option<i64>,
    ) -> Result<Vec<Pharmacist>, GetPharmacistWithPaginationError> {
        let pharmacists = self
            .repository
            .get_pharmacists(page, page_size)
            .await
            .map_err(|err| GetPharmacistWithPaginationError::DomainError(err.to_string()))?;

        Ok(pharmacists)
    }
}

#[cfg(test)]
mod tests {
    use super::PharmacistsService;
    use crate::{
        create_tables::create_tables, domain::pharmacists::repository::PharmacistsRepository,
        infrastructure::postgres_repository_impl::pharmacists::PharmacistsPostgresRepository,
    };
    use uuid::Uuid;

    async fn setup_service(pool: sqlx::PgPool) -> PharmacistsService<impl PharmacistsRepository> {
        create_tables(&pool, true).await.unwrap();
        PharmacistsService::new(PharmacistsPostgresRepository::new(pool))
    }

    #[sqlx::test]
    async fn creates_pharmacist_and_reads_by_id(pool: sqlx::PgPool) {
        let service = setup_service(pool).await;

        let created_pharmacist = service
            .create_pharmacist("John Doex".into(), "96021807250".into())
            .await
            .unwrap();

        assert_eq!(created_pharmacist.name, "John Doex");
        assert_eq!(created_pharmacist.pesel_number, "96021807250");

        let pharmacist_from_repository = service
            .get_pharmacist_by_id(created_pharmacist.id)
            .await
            .unwrap();

        assert_eq!(pharmacist_from_repository.name, "John Doex");
        assert_eq!(pharmacist_from_repository.pesel_number, "96021807250");
    }

    #[sqlx::test]
    async fn create_pharmacist_returns_error_if_body_is_incorrect(pool: sqlx::PgPool) {
        let service = setup_service(pool).await;

        let result = service
            .create_pharmacist("John Doex".into(), "96021807251".into()) // invalid pesel
            .await;

        assert!(result.is_err());
    }

    #[sqlx::test]
    async fn create_pharmacist_returns_error_if_pesel_number_is_duplicated(pool: sqlx::PgPool) {
        let service = setup_service(pool).await;

        service
            .create_pharmacist("John Doex".into(), "96021807250".into())
            .await
            .unwrap();

        let duplicated_pesel_number_result = service
            .create_pharmacist("John Doex".into(), "96021807250".into())
            .await;

        assert!(duplicated_pesel_number_result.is_err());
    }

    #[sqlx::test]
    async fn get_pharmacist_by_id_returns_error_if_such_pharmacist_does_not_exist(
        pool: sqlx::PgPool,
    ) {
        let service = setup_service(pool).await;

        let result = service.get_pharmacist_by_id(Uuid::new_v4()).await;

        assert!(result.is_err());
    }

    #[sqlx::test]
    async fn gets_pharmacists_with_pagination(pool: sqlx::PgPool) {
        let service = setup_service(pool).await;

        service
            .create_pharmacist("John Doex".into(), "96021817257".into())
            .await
            .unwrap();
        service
            .create_pharmacist("John Doey".into(), "99031301347".into())
            .await
            .unwrap();
        service
            .create_pharmacist("John Doez".into(), "92022900002".into())
            .await
            .unwrap();
        service
            .create_pharmacist("John Doeq".into(), "96021807250".into())
            .await
            .unwrap();

        let pharmacists = service
            .get_pharmacists_with_pagination(Some(1), Some(2))
            .await
            .unwrap();

        assert_eq!(pharmacists.len(), 2);

        let pharmacists = service
            .get_pharmacists_with_pagination(Some(1), Some(3))
            .await
            .unwrap();

        assert_eq!(pharmacists.len(), 1);

        let pharmacists = service
            .get_pharmacists_with_pagination(None, Some(10))
            .await
            .unwrap();

        assert_eq!(pharmacists.len(), 4);

        let pharmacists = service
            .get_pharmacists_with_pagination(Some(1), None)
            .await
            .unwrap();

        assert_eq!(pharmacists.len(), 0);

        let pharmacists = service
            .get_pharmacists_with_pagination(None, None)
            .await
            .unwrap();

        assert_eq!(pharmacists.len(), 4);

        let pharmacists = service
            .get_pharmacists_with_pagination(Some(2), Some(3))
            .await
            .unwrap();

        assert_eq!(pharmacists.len(), 0);
    }

    #[sqlx::test]
    async fn get_pharmacists_with_pagination_returns_error_if_params_are_invalid(
        pool: sqlx::PgPool,
    ) {
        let service = setup_service(pool).await;

        assert!(service
            .get_pharmacists_with_pagination(Some(-1), None)
            .await
            .is_err());

        assert!(service
            .get_pharmacists_with_pagination(None, Some(0))
            .await
            .is_err());
    }
}
