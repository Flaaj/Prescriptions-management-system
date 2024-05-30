use crate::domain::patients::{
    models::{NewPatient, Patient},
    repository::PatientsRepository,
};
use uuid::Uuid;

#[derive(Debug)]
pub enum CreatePatientError {
    DomainError(String),
    RepositoryError(String),
}

#[derive(Debug)]
pub enum GetPatientByIdError {
    DomainError,
    RepositoryError(String),
}

#[derive(Debug)]
pub enum GetPatientWithPaginationError {
    DomainError(String),
}

pub struct PatientsService {
    repository: Box<dyn PatientsRepository>,
}

impl PatientsService {
    pub fn new(repository: Box<dyn PatientsRepository>) -> Self {
        Self { repository }
    }

    pub async fn create_patient(
        &self,
        name: String,
        pesel_number: String,
    ) -> Result<Patient, CreatePatientError> {
        let new_patient = NewPatient::new(name, pesel_number)
            .map_err(|err| CreatePatientError::DomainError(err.to_string()))?;

        let created_patient = self
            .repository
            .create_patient(new_patient)
            .await
            .map_err(|err| CreatePatientError::RepositoryError(err.to_string()))?;

        Ok(created_patient)
    }

    pub async fn get_patient_by_id(
        &self,
        patient_id: Uuid,
    ) -> Result<Patient, GetPatientByIdError> {
        let patient = self
            .repository
            .get_patient_by_id(patient_id)
            .await
            .map_err(|err| GetPatientByIdError::RepositoryError(err.to_string()))?;

        Ok(patient)
    }

    pub async fn get_patients_with_pagination(
        &self,
        page: Option<i64>,
        page_size: Option<i64>,
    ) -> Result<Vec<Patient>, GetPatientWithPaginationError> {
        let patients = self
            .repository
            .get_patients(page, page_size)
            .await
            .map_err(|err| GetPatientWithPaginationError::DomainError(err.to_string()))?;

        Ok(patients)
    }
}

#[cfg(test)]
mod tests {
    use super::PatientsService;
    use crate::domain::patients::repository::PatientsRepositoryFake;
    use uuid::Uuid;

    fn setup_service() -> PatientsService {
        PatientsService::new(Box::new(PatientsRepositoryFake::new()))
    }

    #[tokio::test]
    async fn creates_patient_and_reads_by_id() {
        let service = setup_service();

        let created_patient = service
            .create_patient("John Doex".into(), "96021807250".into())
            .await
            .unwrap();

        assert_eq!(created_patient.name, "John Doex");
        assert_eq!(created_patient.pesel_number, "96021807250");

        let patient_from_repository = service.get_patient_by_id(created_patient.id).await.unwrap();

        assert_eq!(patient_from_repository.name, "John Doex");
        assert_eq!(patient_from_repository.pesel_number, "96021807250");
    }

    #[tokio::test]
    async fn create_patient_returns_error_if_body_is_incorrect() {
        let service = setup_service();

        let result = service
            .create_patient("John Doex".into(), "96021807251".into()) // invalid pesel
            .await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn create_patient_returns_error_if_pesel_number_is_duplicated() {
        let service = setup_service();

        service
            .create_patient("John Doex".into(), "96021807250".into())
            .await
            .unwrap();

        let duplicated_pesel_number_result = service
            .create_patient("John Doex".into(), "96021807250".into())
            .await;

        assert!(duplicated_pesel_number_result.is_err());
    }

    #[tokio::test]
    async fn get_patient_by_id_returns_error_if_such_patient_does_not_exist() {
        let service = setup_service();

        let result = service.get_patient_by_id(Uuid::new_v4()).await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn gets_patients_with_pagination() {
        let service = setup_service();

        service
            .create_patient("John Doex".into(), "96021817257".into())
            .await
            .unwrap();
        service
            .create_patient("John Doey".into(), "99031301347".into())
            .await
            .unwrap();
        service
            .create_patient("John Doez".into(), "92022900002".into())
            .await
            .unwrap();
        service
            .create_patient("John Doeq".into(), "96021807250".into())
            .await
            .unwrap();

        let patients = service
            .get_patients_with_pagination(Some(1), Some(2))
            .await
            .unwrap();

        assert_eq!(patients.len(), 2);

        let patients = service
            .get_patients_with_pagination(Some(1), Some(3))
            .await
            .unwrap();

        assert_eq!(patients.len(), 1);

        let patients = service
            .get_patients_with_pagination(None, Some(10))
            .await
            .unwrap();

        assert_eq!(patients.len(), 4);

        let patients = service
            .get_patients_with_pagination(Some(1), None)
            .await
            .unwrap();

        assert_eq!(patients.len(), 0);

        let patients = service
            .get_patients_with_pagination(None, None)
            .await
            .unwrap();

        assert_eq!(patients.len(), 4);

        let patients = service
            .get_patients_with_pagination(Some(2), Some(3))
            .await
            .unwrap();

        assert_eq!(patients.len(), 0);
    }

    #[tokio::test]
    async fn get_patients_with_pagination_returns_error_if_params_are_invalid() {
        let service = setup_service();

        assert!(service
            .get_patients_with_pagination(Some(-1), None)
            .await
            .is_err());

        assert!(service
            .get_patients_with_pagination(None, Some(0))
            .await
            .is_err());
    }
}
