use super::{
    models::{Doctor, NewDoctor},
    repository::{
        CreateDoctorRepositoryError, DoctorsRepository, GetDoctorByIdRepositoryError,
        GetDoctorsRepositoryError,
    },
};
use uuid::Uuid;

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
pub enum GetDoctorWithPaginationError {
    RepositoryError(GetDoctorsRepositoryError),
}

#[derive(Clone)]
pub struct DoctorsService<R: DoctorsRepository> {
    repository: R,
}

impl<R: DoctorsRepository> DoctorsService<R> {
    pub fn new(repository: R) -> Self {
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
    ) -> Result<Vec<Doctor>, GetDoctorWithPaginationError> {
        let doctors = self
            .repository
            .get_doctors(page, page_size)
            .await
            .map_err(|err| GetDoctorWithPaginationError::RepositoryError(err))?;

        Ok(doctors)
    }
}

#[cfg(test)]
mod tests {
    use super::DoctorsService;
    use crate::domain::doctors::{
        repository::DoctorsRepository, repository::InMemoryDoctorsRepository,
    };
    use uuid::Uuid;

    fn setup_service() -> DoctorsService<impl DoctorsRepository> {
        DoctorsService::new(InMemoryDoctorsRepository::new())
    }

    #[tokio::test]
    async fn creates_doctor_and_reads_by_id() {
        let service = setup_service();

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

    #[tokio::test]
    async fn create_doctor_returns_error_if_body_is_incorrect() {
        let service = setup_service();

        let result = service
            .create_doctor("John Doex".into(), "96021807251".into(), "5425740".into()) // invalid pesel
            .await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn create_doctor_returns_error_if_pwz_or_pesel_numbers_are_duplicated() {
        let service = setup_service();

        service
            .create_doctor("John Doex".into(), "96021807250".into(), "5425740".into())
            .await
            .unwrap();

        let duplicated_pesel_number_result = service
            .create_doctor("John Doex".into(), "96021807250".into(), "8463856".into())
            .await;

        assert!(duplicated_pesel_number_result.is_err());

        let duplicated_pwz_number_result = service
            .create_doctor("John Doex".into(), "99031301347".into(), "5425740".into())
            .await;

        assert!(duplicated_pwz_number_result.is_err());
    }

    #[tokio::test]
    async fn get_doctor_by_id_returns_error_if_such_doctor_does_not_exist() {
        let service = setup_service();

        let result = service.get_doctor_by_id(Uuid::new_v4()).await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn gets_doctors_with_pagination() {
        let service = setup_service();

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

    #[tokio::test]
    async fn get_doctors_with_pagination_returns_error_if_params_are_invalid() {
        let service = setup_service();

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
