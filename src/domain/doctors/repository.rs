use crate::domain::{
    doctors::models::{Doctor, NewDoctor},
    utils::pagination::get_pagination_params,
};
use async_trait::async_trait;
use chrono::Utc;
use std::sync::RwLock;
use uuid::Uuid;

#[derive(thiserror::Error, Debug, PartialEq)]
pub enum CreateDoctorRepositoryError {
    #[error("PWZ number already exists")]
    DuplicatedPwzNumber,
    #[error("PESEL number already exists")]
    DuplicatedPeselNumber,
    #[error("Database error: {0}")]
    DatabaseError(String),
}

#[derive(thiserror::Error, Debug, PartialEq)]
pub enum GetDoctorsRepositoryError {
    #[error("Invalid pagination parameters: {0}")]
    InvalidPaginationParams(String),
    #[error("Database error: {0}")]
    DatabaseError(String),
}

#[derive(thiserror::Error, Debug, PartialEq)]
pub enum GetDoctorByIdRepositoryError {
    #[error("Doctor with this id not found ({0})")]
    NotFound(Uuid),
    #[error("Database error: {0}")]
    DatabaseError(String),
}

#[async_trait]
pub trait DoctorsRepository {
    async fn create_doctor(&self, doctor: NewDoctor)
        -> Result<Doctor, CreateDoctorRepositoryError>;
    async fn get_doctors(
        &self,
        page: Option<i64>,
        page_size: Option<i64>,
    ) -> Result<Vec<Doctor>, GetDoctorsRepositoryError>;
    async fn get_doctor_by_id(
        &self,
        doctor_id: Uuid,
    ) -> Result<Doctor, GetDoctorByIdRepositoryError>;
}

/// Used to test the service layer in isolation
pub struct InMemoryDoctorsRepository {
    doctors: RwLock<Vec<Doctor>>,
}

impl InMemoryDoctorsRepository {
    #[allow(dead_code)]
    pub fn new() -> Self {
        Self {
            doctors: RwLock::new(Vec::new()),
        }
    }
}

#[async_trait]
impl DoctorsRepository for InMemoryDoctorsRepository {
    async fn create_doctor(
        &self,
        new_doctor: NewDoctor,
    ) -> Result<Doctor, CreateDoctorRepositoryError> {
        for doctor in self.doctors.read().unwrap().iter() {
            if doctor.pwz_number == new_doctor.pwz_number {
                return Err(CreateDoctorRepositoryError::DuplicatedPwzNumber);
            }
            if doctor.pesel_number == new_doctor.pesel_number {
                return Err(CreateDoctorRepositoryError::DuplicatedPeselNumber);
            }
        }

        let doctor = Doctor {
            id: new_doctor.id,
            name: new_doctor.name,
            pwz_number: new_doctor.pwz_number,
            pesel_number: new_doctor.pesel_number,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        self.doctors.write().unwrap().push(doctor.clone());

        Ok(doctor)
    }

    async fn get_doctors(
        &self,
        page: Option<i64>,
        page_size: Option<i64>,
    ) -> Result<Vec<Doctor>, GetDoctorsRepositoryError> {
        let (page_size, offset) = get_pagination_params(page, page_size)
            .map_err(|err| GetDoctorsRepositoryError::InvalidPaginationParams(err.to_string()))?;
        let a = offset;
        let b = offset + page_size;

        let mut doctors: Vec<Doctor> = vec![];
        for i in a..b {
            match self.doctors.read().unwrap().get(i as usize) {
                Some(doctor) => doctors.push(doctor.clone()),
                None => {}
            }
        }

        Ok(doctors)
    }

    async fn get_doctor_by_id(
        &self,
        doctor_id: Uuid,
    ) -> Result<Doctor, GetDoctorByIdRepositoryError> {
        match self
            .doctors
            .read()
            .unwrap()
            .iter()
            .find(|doctor| doctor.id == doctor_id)
        {
            Some(doctor) => Ok(doctor.clone()),
            None => Err(GetDoctorByIdRepositoryError::NotFound(doctor_id)),
        }
    }
}

#[cfg(test)]
// the same tests as in postgres_repository_impl/doctors.rs to make sure fake repo works the same way
mod tests {
    use uuid::Uuid;

    use super::InMemoryDoctorsRepository;
    use crate::domain::{
        doctors::{
            models::NewDoctor,
            repository::{
                CreateDoctorRepositoryError, DoctorsRepository, GetDoctorByIdRepositoryError,
                GetDoctorsRepositoryError,
            },
        },
        utils::pagination::PaginationError,
    };

    async fn setup_repository() -> InMemoryDoctorsRepository {
        InMemoryDoctorsRepository::new()
    }

    #[tokio::test]
    async fn create_and_read_doctor_by_id() {
        let repository = setup_repository().await;

        let new_doctor =
            NewDoctor::new("John Does".into(), "5425740".into(), "96021817257".into()).unwrap();

        repository.create_doctor(new_doctor.clone()).await.unwrap();

        let doctor_from_repo = repository.get_doctor_by_id(new_doctor.id).await.unwrap();

        assert_eq!(doctor_from_repo, new_doctor);
    }

    #[tokio::test]
    async fn returns_error_if_doctor_with_given_id_doesnt_exist() {
        let repository = setup_repository().await;
        let doctor_id = Uuid::new_v4();

        let doctor_from_repo = repository.get_doctor_by_id(doctor_id).await;

        assert_eq!(
            doctor_from_repo,
            Err(GetDoctorByIdRepositoryError::NotFound(doctor_id))
        );
    }

    #[tokio::test]
    async fn create_and_read_doctors_from_database() {
        let repository = setup_repository().await;

        let new_doctor_0 =
            NewDoctor::new("John First".into(), "5425740".into(), "96021817257".into()).unwrap();
        let new_doctor_1 =
            NewDoctor::new("John Second".into(), "8463856".into(), "99031301347".into()).unwrap();
        let new_doctor_2 =
            NewDoctor::new("John Third".into(), "3123456".into(), "92022900002".into()).unwrap();
        let new_doctor_3 =
            NewDoctor::new("John Fourth".into(), "5425751".into(), "96021807250".into()).unwrap();

        repository
            .create_doctor(new_doctor_0.clone())
            .await
            .unwrap();
        repository
            .create_doctor(new_doctor_1.clone())
            .await
            .unwrap();
        repository
            .create_doctor(new_doctor_2.clone())
            .await
            .unwrap();
        repository
            .create_doctor(new_doctor_3.clone())
            .await
            .unwrap();

        let doctors = repository.get_doctors(None, Some(10)).await.unwrap();

        assert!(doctors.len() == 4);
        assert_eq!(doctors[0], new_doctor_0);
        assert_eq!(doctors[1], new_doctor_1);
        assert_eq!(doctors[2], new_doctor_2);
        assert_eq!(doctors[3], new_doctor_3);

        let doctors = repository.get_doctors(None, Some(2)).await.unwrap();

        assert_eq!(doctors.len(), 2);
        assert_eq!(doctors[0], new_doctor_0);
        assert_eq!(doctors[1], new_doctor_1);

        let doctors = repository.get_doctors(Some(1), Some(3)).await.unwrap();

        assert!(doctors.len() == 1);
        assert_eq!(doctors[0], new_doctor_3);

        let doctors = repository.get_doctors(Some(2), Some(3)).await.unwrap();

        assert!(doctors.len() == 0);
    }

    #[tokio::test]
    async fn get_doctors_returns_error_if_pagination_params_are_incorrect() {
        let repository = setup_repository().await;

        assert_eq!(
            repository.get_doctors(Some(-1), Some(10)).await,
            Err(GetDoctorsRepositoryError::InvalidPaginationParams(
                PaginationError::InvalidPageOrPageSize.to_string()
            ))
        );

        assert_eq!(
            repository.get_doctors(Some(0), Some(0)).await,
            Err(GetDoctorsRepositoryError::InvalidPaginationParams(
                PaginationError::InvalidPageOrPageSize.to_string()
            ))
        );
    }

    #[tokio::test]
    async fn doesnt_create_doctor_if_pwz_or_pesel_numbers_are_duplicated() {
        let repository = setup_repository().await;

        let doctor =
            NewDoctor::new("John Doe".into(), "5425740".into(), "96021817257".into()).unwrap();

        assert!(repository.create_doctor(doctor).await.is_ok());

        let doctor_with_duplicated_pwz_number =
            NewDoctor::new("John Doe".into(), "5425740".into(), "99031301347".into()).unwrap();

        assert_eq!(
            repository
                .create_doctor(doctor_with_duplicated_pwz_number)
                .await,
            Err(CreateDoctorRepositoryError::DuplicatedPwzNumber)
        );

        let doctor_with_duplicated_pesel_number =
            NewDoctor::new("John Doe".into(), "3123456".into(), "96021817257".into()).unwrap();

        assert_eq!(
            repository
                .create_doctor(doctor_with_duplicated_pesel_number)
                .await,
            Err(CreateDoctorRepositoryError::DuplicatedPeselNumber)
        );
    }
}
