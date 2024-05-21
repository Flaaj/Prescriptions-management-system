use crate::domain::doctors::{
    models::{Doctor, NewDoctor},
    repository::doctors_repository_trait::DoctorsRepositoryTrait,
};
use uuid::Uuid;

#[derive(Debug)]
pub enum CreateDoctorError {
    ValidationError(String),
    DatabaseError(String),
}

#[derive(Debug)]
pub enum GetDoctorByIdError {
    InputError,
    DatabaseError(String),
}

#[derive(Debug)]
pub enum GetDoctorWithPaginationError {
    InputError(String),
}

#[derive(Clone)]
pub struct DoctorsService<R: DoctorsRepositoryTrait> {
    repo: R,
}

impl<R: DoctorsRepositoryTrait> DoctorsService<R> {
    pub fn new(repo: R) -> Self {
        Self { repo }
    }

    pub async fn create_doctor(
        &self,
        name: String,
        pesel_number: String,
        pwz_number: String,
    ) -> Result<Doctor, CreateDoctorError> {
        let new_doctor = NewDoctor::new(name, pwz_number, pesel_number)
            .map_err(|err| CreateDoctorError::ValidationError(err.to_string()))?;

        let created_doctor = self
            .repo
            .create_doctor(new_doctor)
            .await
            .map_err(|err| CreateDoctorError::DatabaseError(err.to_string()))?;

        Ok(created_doctor)
    }

    pub async fn get_doctor_by_id(&self, doctor_id: Uuid) -> Result<Doctor, GetDoctorByIdError> {
        let doctor = self
            .repo
            .get_doctor_by_id(doctor_id)
            .await
            .map_err(|err| GetDoctorByIdError::DatabaseError(err.to_string()))?;

        Ok(doctor)
    }

    pub async fn get_doctors_with_pagination(
        &self,
        page: Option<i64>,
        page_size: Option<i64>,
    ) -> Result<Vec<Doctor>, GetDoctorWithPaginationError> {
        let doctors = self
            .repo
            .get_doctors(page, page_size)
            .await
            .map_err(|err| GetDoctorWithPaginationError::InputError(err.to_string()))?;

        Ok(doctors)
    }
}

#[cfg(test)]
mod integration_tests {
    use super::DoctorsService;
    use crate::{
        create_tables::create_tables,
        domain::doctors::repository::{
            doctors_repository_impl::DoctorsRepository,
            doctors_repository_trait::DoctorsRepositoryTrait,
        },
    };
    use uuid::Uuid;

    async fn create_doctors_service<'a>(
        pool: &'a sqlx::PgPool,
    ) -> DoctorsService<impl DoctorsRepositoryTrait + 'a> {
        create_tables(&pool, true).await.unwrap();
        DoctorsService::new(DoctorsRepository::new(pool))
    }

    #[sqlx::test]
    async fn creates_doctor_and_reads_by_id(pool: sqlx::PgPool) {
        let service = create_doctors_service(&pool).await;

        let create_doctor_result = service
            .create_doctor("John Doex".into(), "96021807250".into(), "5425740".into())
            .await;

        assert!(create_doctor_result.is_ok());

        let created_doctor = create_doctor_result.unwrap();

        assert_eq!(created_doctor.name, "John Doex");
        assert_eq!(created_doctor.pesel_number, "96021807250");
        assert_eq!(created_doctor.pwz_number, "5425740");

        let get_doctor_by_id_result = service.get_doctor_by_id(created_doctor.id).await;

        assert!(get_doctor_by_id_result.is_ok());

        let doctor = get_doctor_by_id_result.unwrap();

        assert_eq!(doctor.name, "John Doex");
        assert_eq!(doctor.pesel_number, "96021807250");
        assert_eq!(doctor.pwz_number, "5425740");
    }

    #[sqlx::test]
    async fn create_doctor_returns_error_if_body_is_incorrect(pool: sqlx::PgPool) {
        let service = create_doctors_service(&pool).await;

        let result = service
            .create_doctor("John Doex".into(), "96021807251".into(), "5425740".into()) // invalid pesel
            .await;

        assert!(result.is_err());
    }

    #[sqlx::test]
    async fn create_doctor_returns_error_if_pwz_or_pesel_numbers_are_duplicated(
        pool: sqlx::PgPool,
    ) {
        let service = create_doctors_service(&pool).await;

        let result = service
            .create_doctor("John Doex".into(), "96021807250".into(), "5425740".into())
            .await;

        assert!(result.is_ok());

        let duplicated_pesel_number_result = service
            .create_doctor("John Doex".into(), "96021807250".into(), "8463856".into())
            .await;

        assert!(duplicated_pesel_number_result.is_err());

        let duplicated_pwz_number_result = service
            .create_doctor("John Doex".into(), "99031301347".into(), "5425740".into())
            .await;

        assert!(duplicated_pwz_number_result.is_err());
    }

    #[sqlx::test]
    async fn get_doctor_by_id_returns_error_if_such_doctor_does_not_exist(pool: sqlx::PgPool) {
        let service = create_doctors_service(&pool).await;

        let result = service.get_doctor_by_id(Uuid::new_v4()).await;

        assert!(result.is_err());
    }

    #[sqlx::test]
    async fn gets_doctors_with_pagination(pool: sqlx::PgPool) {
        let service = create_doctors_service(&pool).await;

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
        let service = create_doctors_service(&pool).await;

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