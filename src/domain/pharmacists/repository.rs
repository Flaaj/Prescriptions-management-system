use std::sync::RwLock;

use async_trait::async_trait;
use chrono::Utc;
use uuid::Uuid;

use crate::domain::{
    pharmacists::models::{NewPharmacist, Pharmacist},
    utils::pagination::get_pagination_params,
};

#[derive(thiserror::Error, Debug, PartialEq)]
pub enum CreatePharmacistRepositoryError {
    #[error("PESEL number already exists")]
    DuplicatedPeselNumber,
    #[error("Database error: {0}")]
    DatabaseError(String),
}

#[derive(thiserror::Error, Debug, PartialEq)]
pub enum GetPharmacistsRepositoryError {
    #[error("Invalid pagination parameters: {0}")]
    InvalidPaginationParams(String),
    #[error("Database error: {0}")]
    DatabaseError(String),
}

#[derive(thiserror::Error, Debug, PartialEq)]
pub enum GetPharmacistByIdRepositoryError {
    #[error("Pharmacist with this id not found ({0})")]
    NotFound(Uuid),
    #[error("Database error: {0}")]
    DatabaseError(String),
}

#[async_trait]
pub trait PharmacistsRepository {
    async fn create_pharmacist(
        &self,
        pharmacist: NewPharmacist,
    ) -> Result<Pharmacist, CreatePharmacistRepositoryError>;
    async fn get_pharmacists(
        &self,
        page: Option<i64>,
        page_size: Option<i64>,
    ) -> Result<Vec<Pharmacist>, GetPharmacistsRepositoryError>;
    async fn get_pharmacist_by_id(
        &self,
        pharmacist_id: Uuid,
    ) -> Result<Pharmacist, GetPharmacistByIdRepositoryError>;
}

/// Used to test the service layer in isolation
pub struct PharmacistsRepositoryFake {
    pharmacists: RwLock<Vec<Pharmacist>>,
}

impl PharmacistsRepositoryFake {
    #[allow(dead_code)]
    pub fn new() -> Self {
        Self {
            pharmacists: RwLock::new(Vec::new()),
        }
    }
}

#[async_trait]
impl PharmacistsRepository for PharmacistsRepositoryFake {
    async fn create_pharmacist(
        &self,
        new_pharmacist: NewPharmacist,
    ) -> Result<Pharmacist, CreatePharmacistRepositoryError> {
        let does_pesel_number_exist = self
            .pharmacists
            .read()
            .unwrap()
            .iter()
            .any(|pharmacist| pharmacist.pesel_number == new_pharmacist.pesel_number);

        if does_pesel_number_exist {
            return Err(CreatePharmacistRepositoryError::DuplicatedPeselNumber);
        }

        let pharmacist = Pharmacist {
            id: new_pharmacist.id,
            name: new_pharmacist.name,
            pesel_number: new_pharmacist.pesel_number,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        self.pharmacists.write().unwrap().push(pharmacist.clone());

        Ok(pharmacist)
    }

    async fn get_pharmacists(
        &self,
        page: Option<i64>,
        page_size: Option<i64>,
    ) -> Result<Vec<Pharmacist>, GetPharmacistsRepositoryError> {
        let (page_size, offset) = get_pagination_params(page, page_size).map_err(|err| {
            GetPharmacistsRepositoryError::InvalidPaginationParams(err.to_string())
        })?;
        let a = offset;
        let b = offset + page_size;

        let mut pharmacists: Vec<Pharmacist> = vec![];
        for i in a..b {
            match self.pharmacists.read().unwrap().get(i as usize) {
                Some(pharmacist) => pharmacists.push(pharmacist.clone()),
                None => {}
            }
        }

        Ok(pharmacists)
    }

    async fn get_pharmacist_by_id(
        &self,
        pharmacist_id: Uuid,
    ) -> Result<Pharmacist, GetPharmacistByIdRepositoryError> {
        match self
            .pharmacists
            .read()
            .unwrap()
            .iter()
            .find(|pharmacist| pharmacist.id == pharmacist_id)
        {
            Some(pharmacist) => Ok(pharmacist.clone()),
            None => Err(GetPharmacistByIdRepositoryError::NotFound(pharmacist_id)),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::assert_matches::assert_matches;

    use uuid::Uuid;

    use super::{
        CreatePharmacistRepositoryError, GetPharmacistByIdRepositoryError,
        GetPharmacistsRepositoryError, PharmacistsRepositoryFake, PharmacistsRepository,
    };
    use crate::domain::pharmacists::models::NewPharmacist;

    async fn setup_repository() -> PharmacistsRepositoryFake {
        PharmacistsRepositoryFake::new()
    }

    #[sqlx::test]
    async fn create_and_read_pharmacist_by_id() {
        let repository = setup_repository().await;

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
    async fn returns_error_if_pharmacists_with_given_id_doesnt_exist() {
        let repository = setup_repository().await;
        let pharmacist_id = Uuid::new_v4();

        let pharmacist_from_repo = repository.get_pharmacist_by_id(pharmacist_id).await;

        assert_eq!(
            pharmacist_from_repo,
            Err(GetPharmacistByIdRepositoryError::NotFound(pharmacist_id))
        );
    }

    #[sqlx::test]
    async fn create_and_read_pharmacists_from_database() {
        let repository = setup_repository().await;

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

    #[tokio::test]
    async fn get_patients_returns_error_if_pagination_params_are_incorrect() {
        let repository = setup_repository().await;

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
    async fn doesnt_create_pharmacist_if_pesel_number_is_duplicated() {
        let repository = setup_repository().await;

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
