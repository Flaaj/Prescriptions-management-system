use uuid::Uuid;

use super::{
    entities::{Doctor, NewDoctor},
    repository::{
        CreateDoctorRepositoryError, DoctorsRepository, GetDoctorByIdRepositoryError,
        GetDoctorsRepositoryError,
    },
};

#[derive(Debug)]
pub enum CreateDoctorError {
    DomainError(String),
    RepositoryError(CreateDoctorRepositoryError),
}

#[derive(Debug)]
pub enum GetDoctorByIdError {
    RepositoryError(GetDoctorByIdRepositoryError),
}

#[derive(Debug)]
pub enum GetDoctorsWithPaginationError {
    RepositoryError(GetDoctorsRepositoryError),
}

pub struct DoctorsService {
    repository: Box<dyn DoctorsRepository>,
}

impl DoctorsService {
    pub fn new(repository: Box<dyn DoctorsRepository>) -> Self {
        Self { repository }
    }

    pub async fn create_doctor(
        &self,
        name: String,
        pesel_number: String,
        pwz_number: String,
    ) -> Result<Doctor, CreateDoctorError> {
        let new_doctor = NewDoctor::new(name, pwz_number, pesel_number)
            .map_err(|err| CreateDoctorError::DomainError(err.to_string()))?;

        let created_doctor = self
            .repository
            .create_doctor(new_doctor)
            .await
            .map_err(|err| CreateDoctorError::RepositoryError(err))?;

        Ok(created_doctor)
    }

    pub async fn get_doctor_by_id(&self, doctor_id: Uuid) -> Result<Doctor, GetDoctorByIdError> {
        let doctor = self
            .repository
            .get_doctor_by_id(doctor_id)
            .await
            .map_err(|err| GetDoctorByIdError::RepositoryError(err))?;

        Ok(doctor)
    }

    pub async fn get_doctors_with_pagination(
        &self,
        page: Option<i64>,
        page_size: Option<i64>,
    ) -> Result<Vec<Doctor>, GetDoctorsWithPaginationError> {
        let doctors = self
            .repository
            .get_doctors(page, page_size)
            .await
            .map_err(|err| GetDoctorsWithPaginationError::RepositoryError(err))?;

        Ok(doctors)
    }
}

#[cfg(test)]
mod tests {
    use uuid::Uuid;

    use super::{CreateDoctorError, DoctorsService, GetDoctorByIdError};
    use crate::infrastructure::postgres_repository_impl::{
        create_tables::create_tables, doctors::PostgresDoctorsRepository,
    };

    async fn setup_service(pool: sqlx::PgPool) -> DoctorsService {
        create_tables(&pool, true).await.unwrap();
        DoctorsService::new(Box::new(PostgresDoctorsRepository::new(pool)))
    }

    #[sqlx::test]
    async fn creates_doctor_and_reads_by_id(pool: sqlx::PgPool) {
        let service = setup_service(pool).await;

        let created_doctor = service
            .create_doctor("John Doex".into(), "96021807250".into(), "5425740".into())
            .await
            .unwrap();

        assert_eq!(created_doctor.name, "John Doex");
        assert_eq!(created_doctor.pesel_number, "96021807250");
        assert_eq!(created_doctor.pwz_number, "5425740");

        let doctor_from_repository = service.get_doctor_by_id(created_doctor.id).await.unwrap();

        assert_eq!(doctor_from_repository.name, "John Doex");
        assert_eq!(doctor_from_repository.pesel_number, "96021807250");
        assert_eq!(doctor_from_repository.pwz_number, "5425740");
    }

    #[sqlx::test]
    async fn create_doctor_returns_error_if_body_is_incorrect(pool: sqlx::PgPool) {
        let service = setup_service(pool).await;

        let result = service
            .create_doctor("John Doex".into(), "96021807251".into(), "5425740".into()) // invalid pesel
            .await;

        assert!(match result {
            Err(CreateDoctorError::DomainError(_)) => true,
            _ => false,
        });
    }

    #[sqlx::test]
    async fn create_doctor_returns_error_if_pwz_or_pesel_numbers_are_duplicated(
        pool: sqlx::PgPool,
    ) {
        let service = setup_service(pool).await;

        service
            .create_doctor("John Doex".into(), "96021807250".into(), "5425740".into())
            .await
            .unwrap();

        let duplicated_pesel_number_result = service
            .create_doctor("John Doex".into(), "96021807250".into(), "8463856".into())
            .await;

        assert!(match duplicated_pesel_number_result {
            Err(CreateDoctorError::RepositoryError(_)) => true,
            _ => false,
        });

        let duplicated_pwz_number_result = service
            .create_doctor("John Doex".into(), "99031301347".into(), "5425740".into())
            .await;

        assert!(match duplicated_pwz_number_result {
            Err(CreateDoctorError::RepositoryError(_)) => true,
            _ => false,
        });
    }

    #[sqlx::test]
    async fn get_doctor_by_id_returns_error_if_such_doctor_does_not_exist(pool: sqlx::PgPool) {
        let service = setup_service(pool).await;

        let result = service.get_doctor_by_id(Uuid::new_v4()).await;

        assert!(match result {
            Err(GetDoctorByIdError::RepositoryError(_)) => true,
            _ => false,
        });
    }

    #[sqlx::test]
    async fn gets_doctors_with_pagination(pool: sqlx::PgPool) {
        let service = setup_service(pool).await;

        service
            .create_doctor("John Doex".into(), "96021817257".into(), "5425740".into())
            .await
            .unwrap();
        service
            .create_doctor("John Doey".into(), "99031301347".into(), "8463856".into())
            .await
            .unwrap();
        service
            .create_doctor("John Doez".into(), "92022900002".into(), "3123456".into())
            .await
            .unwrap();
        service
            .create_doctor("John Doeq".into(), "96021807250".into(), "5425751".into())
            .await
            .unwrap();

        let doctors = service
            .get_doctors_with_pagination(Some(1), Some(2))
            .await
            .unwrap();

        assert_eq!(doctors.len(), 2);

        let doctors = service
            .get_doctors_with_pagination(Some(1), Some(3))
            .await
            .unwrap();

        assert_eq!(doctors.len(), 1);

        let doctors = service
            .get_doctors_with_pagination(None, Some(10))
            .await
            .unwrap();

        assert_eq!(doctors.len(), 4);

        let doctors = service
            .get_doctors_with_pagination(Some(1), None)
            .await
            .unwrap();

        assert_eq!(doctors.len(), 0);

        let doctors = service
            .get_doctors_with_pagination(None, None)
            .await
            .unwrap();

        assert_eq!(doctors.len(), 4);

        let doctors = service
            .get_doctors_with_pagination(Some(2), Some(3))
            .await
            .unwrap();

        assert_eq!(doctors.len(), 0);
    }

    #[sqlx::test]
    async fn get_doctors_with_pagination_returns_error_if_params_are_invalid(pool: sqlx::PgPool) {
        let service = setup_service(pool).await;

        assert!(service
            .get_doctors_with_pagination(Some(-1), None)
            .await
            .is_err());

        assert!(service
            .get_doctors_with_pagination(None, Some(0))
            .await
            .is_err());
    }
}
