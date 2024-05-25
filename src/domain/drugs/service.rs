use uuid::Uuid;

use super::{
    models::{Drug, DrugContentType, NewDrug},
    repository::DrugsRepository,
};

pub struct DrugsService<R: DrugsRepository> {
    repository: R,
}

#[derive(Debug)]
pub enum CreateDrugError {
    DomainError(String),
    RepositoryError(String),
}

#[derive(Debug)]
pub enum GetDrugByIdError {
    DomainError,
    RepositoryError(String),
}

#[derive(Debug)]
pub enum GetDrugsWithPaginationError {
    DomainError(String),
}

impl<R: DrugsRepository> DrugsService<R> {
    pub fn new(repository: R) -> Self {
        Self { repository }
    }

    pub async fn create_drug(
        &self,
        name: String,
        content_type: DrugContentType,
        pills_count: Option<i32>,
        mg_per_pill: Option<i32>,
        ml_per_pill: Option<i32>,
        volume_ml: Option<i32>,
    ) -> Result<Drug, CreateDrugError> {
        let new_drug = NewDrug::new(
            name,
            content_type,
            pills_count,
            mg_per_pill,
            ml_per_pill,
            volume_ml,
        )
        .map_err(|err| CreateDrugError::DomainError(err.to_string()))?;

        let created_drug = self
            .repository
            .create_drug(new_drug)
            .await
            .map_err(|err| CreateDrugError::RepositoryError(err.to_string()))?;

        Ok(created_drug)
    }

    pub async fn get_drug_by_id(&self, drug_id: Uuid) -> Result<Drug, GetDrugByIdError> {
        let doctor = self
            .repository
            .get_drug_by_id(drug_id)
            .await
            .map_err(|err| GetDrugByIdError::RepositoryError(err.to_string()))?;

        Ok(doctor)
    }

    pub async fn get_drugs_with_pagination(
        &self,
        page: Option<i64>,
        page_size: Option<i64>,
    ) -> Result<Vec<Drug>, GetDrugsWithPaginationError> {
        let result = self
            .repository
            .get_drugs(page, page_size)
            .await
            .map_err(|err| GetDrugsWithPaginationError::DomainError(err.to_string()))?;

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use uuid::Uuid;

    use crate::{
        create_tables::create_tables,
        domain::drugs::{models::DrugContentType, repository::DrugsRepository},
        infrastructure::postgres_repository_impl::drugs::DrugsPostgresRepository,
    };

    use super::DrugsService;

    async fn setup_service(pool: sqlx::PgPool) -> DrugsService<impl DrugsRepository> {
        create_tables(&pool, true).await.unwrap();
        DrugsService::new(DrugsPostgresRepository::new(pool))
    }

    #[sqlx::test]
    async fn creates_drug_and_reads_by_id(pool: sqlx::PgPool) {
        let service = setup_service(pool).await;

        let created_drug = service
            .create_drug(
                "Gripex".into(),
                DrugContentType::SolidPills,
                Some(20),
                Some(300),
                None,
                None,
            )
            .await
            .unwrap();

        assert_eq!(created_drug.name, "Gripex");
        assert_eq!(created_drug.content_type, DrugContentType::SolidPills);
        assert_eq!(created_drug.pills_count, Some(20));
        assert_eq!(created_drug.mg_per_pill, Some(300));
        assert_eq!(created_drug.ml_per_pill, None);
        assert_eq!(created_drug.volume_ml, None);

        let drug_from_repository = service.get_drug_by_id(created_drug.id).await.unwrap();

        assert_eq!(drug_from_repository.name, "Gripex");
        assert_eq!(
            drug_from_repository.content_type,
            DrugContentType::SolidPills
        );
        assert_eq!(drug_from_repository.pills_count, Some(20));
        assert_eq!(drug_from_repository.mg_per_pill, Some(300));
        assert_eq!(drug_from_repository.ml_per_pill, None);
        assert_eq!(drug_from_repository.volume_ml, None);
    }

    #[sqlx::test]
    fn get_drug_by_id_returns_error_if_drug_doesnt_exist(pool: sqlx::PgPool) {
        let service = setup_service(pool).await;

        let result = service.get_drug_by_id(Uuid::new_v4()).await;

        assert!(result.is_err());
    }

    #[sqlx::test]
    async fn gets_drugs_with_pagination(pool: sqlx::PgPool) {
        let service = setup_service(pool).await;

        let result = service
            .create_drug(
                "Gripex".into(),
                DrugContentType::SolidPills,
                Some(20),
                Some(300),
                None,
                None,
            )
            .await
            .unwrap();

        assert_eq!(result.name, "Gripex");
        assert_eq!(result.content_type, DrugContentType::SolidPills);
        assert_eq!(result.pills_count, Some(20));
        assert_eq!(result.mg_per_pill, Some(300));
        assert_eq!(result.ml_per_pill, None);
        assert_eq!(result.volume_ml, None);

        service
            .create_drug(
                "Apap".into(),
                DrugContentType::SolidPills,
                Some(10),
                Some(400),
                None,
                None,
            )
            .await
            .unwrap();
        service
            .create_drug(
                "Aspirin".into(),
                DrugContentType::SolidPills,
                Some(30),
                Some(200),
                None,
                None,
            )
            .await
            .unwrap();
        service
            .create_drug(
                "Flegamax".into(),
                DrugContentType::BottleOfLiquid,
                None,
                None,
                None,
                Some(400),
            )
            .await
            .unwrap();

        let drugs = service
            .get_drugs_with_pagination(Some(1), Some(2))
            .await
            .unwrap();

        assert_eq!(drugs.len(), 2);

        let drugs = service
            .get_drugs_with_pagination(Some(1), Some(3))
            .await
            .unwrap();

        assert_eq!(drugs.len(), 1);

        let drugs = service
            .get_drugs_with_pagination(None, Some(10))
            .await
            .unwrap();

        assert_eq!(drugs.len(), 4);

        let drugs = service
            .get_drugs_with_pagination(Some(1), None)
            .await
            .unwrap();

        assert_eq!(drugs.len(), 0);

        let drugs = service.get_drugs_with_pagination(None, None).await.unwrap();

        assert_eq!(drugs.len(), 4);

        let drugs = service
            .get_drugs_with_pagination(Some(2), Some(3))
            .await
            .unwrap();

        assert_eq!(drugs.len(), 0);
    }

    #[sqlx::test]
    async fn get_drugs_with_pagination_returns_error_if_params_are_invalid(pool: sqlx::PgPool) {
        let service = setup_service(pool).await;

        assert!(service
            .get_drugs_with_pagination(Some(-1), None)
            .await
            .is_err());

        assert!(service
            .get_drugs_with_pagination(None, Some(0))
            .await
            .is_err());
    }
}
