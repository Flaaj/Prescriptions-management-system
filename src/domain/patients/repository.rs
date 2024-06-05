use std::sync::RwLock;

use async_trait::async_trait;
use chrono::Utc;
use uuid::Uuid;

use crate::domain::{
    patients::models::{NewPatient, Patient},
    utils::pagination::get_pagination_params,
};

#[derive(thiserror::Error, Debug, PartialEq)]
pub enum CreatePatientRepositoryError {
    #[error("PESEL number already exists")]
    DuplicatedPeselNumber,
    #[error("Database error: {0}")]
    DatabaseError(String),
}

#[derive(thiserror::Error, Debug, PartialEq)]
pub enum GetPatientsRepositoryError {
    #[error("Invalid pagination parameters: {0}")]
    InvalidPaginationParams(String),
    #[error("Database error: {0}")]
    DatabaseError(String),
}

#[derive(thiserror::Error, Debug, PartialEq)]
pub enum GetPatientByIdRepositoryError {
    #[error("Patient with this id not found ({0})")]
    NotFound(Uuid),
    #[error("Database error: {0}")]
    DatabaseError(String),
}

#[async_trait]
pub trait PatientsRepository: Send + Sync + 'static {
    async fn create_patient(
        &self,
        patient: NewPatient,
    ) -> Result<Patient, CreatePatientRepositoryError>;
    async fn get_patients(
        &self,
        page: Option<i64>,
        page_size: Option<i64>,
    ) -> Result<Vec<Patient>, GetPatientsRepositoryError>;
    async fn get_patient_by_id(
        &self,
        patient_id: Uuid,
    ) -> Result<Patient, GetPatientByIdRepositoryError>;
}

pub struct PatientsRepositoryFake {
    patients: RwLock<Vec<Patient>>,
}

impl PatientsRepositoryFake {
    #[allow(dead_code)]
    pub fn new() -> Self {
        Self {
            patients: RwLock::new(Vec::new()),
        }
    }
}

#[async_trait]
impl PatientsRepository for PatientsRepositoryFake {
    async fn create_patient(
        &self,
        new_patient: NewPatient,
    ) -> Result<Patient, CreatePatientRepositoryError> {
        let does_pesel_number_exist = self
            .patients
            .read()
            .unwrap()
            .iter()
            .any(|patient| patient.pesel_number == new_patient.pesel_number);

        if does_pesel_number_exist {
            return Err(CreatePatientRepositoryError::DuplicatedPeselNumber);
        }

        let patient = Patient {
            id: new_patient.id,
            name: new_patient.name,
            pesel_number: new_patient.pesel_number,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        self.patients.write().unwrap().push(patient.clone());

        Ok(patient)
    }

    async fn get_patients(
        &self,
        page: Option<i64>,
        page_size: Option<i64>,
    ) -> Result<Vec<Patient>, GetPatientsRepositoryError> {
        let (page_size, offset) = get_pagination_params(page, page_size)
            .map_err(|err| GetPatientsRepositoryError::InvalidPaginationParams(err.to_string()))?;
        let a = offset;
        let b = offset + page_size;

        let mut patients: Vec<Patient> = vec![];
        for i in a..b {
            match self.patients.read().unwrap().get(i as usize) {
                Some(patient) => patients.push(patient.clone()),
                None => {}
            }
        }

        Ok(patients)
    }

    async fn get_patient_by_id(
        &self,
        patient_id: Uuid,
    ) -> Result<Patient, GetPatientByIdRepositoryError> {
        match self
            .patients
            .read()
            .unwrap()
            .iter()
            .find(|patient| patient.id == patient_id)
        {
            Some(patient) => Ok(patient.clone()),
            None => Err(GetPatientByIdRepositoryError::NotFound(patient_id)),
        }
    }
}

#[cfg(test)]
mod tests {
    use uuid::Uuid;

    use super::PatientsRepositoryFake;
    use crate::domain::patients::{
        models::NewPatient,
        repository::{
            CreatePatientRepositoryError, GetPatientByIdRepositoryError,
            GetPatientsRepositoryError, PatientsRepository,
        },
    };

    fn setup_repository() -> PatientsRepositoryFake {
        PatientsRepositoryFake::new()
    }

    #[tokio::test]
    async fn create_and_read_patient_by_id() {
        let repository = setup_repository();

        let new_patient = NewPatient::new("John Doe".into(), "96021817257".into()).unwrap();

        repository
            .create_patient(new_patient.clone())
            .await
            .unwrap();

        let patient_from_repo = repository.get_patient_by_id(new_patient.id).await.unwrap();

        assert_eq!(patient_from_repo, new_patient);
    }

    #[tokio::test]
    async fn returns_error_if_patients_with_given_id_doesnt_exist() {
        let repository = setup_repository();
        let patient_id = Uuid::new_v4();

        let patient_from_repo = repository.get_patient_by_id(patient_id).await;

        assert_eq!(
            patient_from_repo,
            Err(GetPatientByIdRepositoryError::NotFound(patient_id))
        );
    }

    #[tokio::test]
    async fn create_and_read_patients_from_database() {
        let repository = setup_repository();

        let new_patient_0 = NewPatient::new("John Doe".into(), "96021817257".into()).unwrap();
        let new_patient_1 = NewPatient::new("John Doe".into(), "99031301347".into()).unwrap();
        let new_patient_2 = NewPatient::new("John Doe".into(), "92022900002".into()).unwrap();
        let new_patient_3 = NewPatient::new("John Doe".into(), "96021807250".into()).unwrap();

        repository
            .create_patient(new_patient_0.clone())
            .await
            .unwrap();
        repository
            .create_patient(new_patient_1.clone())
            .await
            .unwrap();
        repository
            .create_patient(new_patient_2.clone())
            .await
            .unwrap();
        repository
            .create_patient(new_patient_3.clone())
            .await
            .unwrap();

        let patients = repository.get_patients(None, Some(10)).await.unwrap();

        assert_eq!(patients.len(), 4);
        assert_eq!(patients[0], new_patient_0);
        assert_eq!(patients[1], new_patient_1);
        assert_eq!(patients[2], new_patient_2);
        assert_eq!(patients[3], new_patient_3);

        let patients = repository.get_patients(None, Some(2)).await.unwrap();

        assert_eq!(patients.len(), 2);
        assert_eq!(patients[0], new_patient_0);
        assert_eq!(patients[1], new_patient_1);

        let patients = repository.get_patients(Some(1), Some(3)).await.unwrap();

        assert_eq!(patients.len(), 1);
        assert_eq!(patients[0], new_patient_3);

        let patients = repository.get_patients(Some(2), Some(3)).await.unwrap();

        assert_eq!(patients.len(), 0);
    }

    #[tokio::test]
    async fn get_patients_returns_error_if_pagination_params_are_incorrect() {
        let repository = setup_repository();

        assert!(match repository.get_patients(Some(-1), Some(10)).await {
            Err(GetPatientsRepositoryError::InvalidPaginationParams(_)) => true,
            _ => false,
        });

        assert!(match repository.get_patients(Some(0), Some(0)).await {
            Err(GetPatientsRepositoryError::InvalidPaginationParams(_)) => true,
            _ => false,
        });
    }

    #[tokio::test]
    async fn doesnt_create_patient_if_pesel_number_is_duplicated() {
        let repository = setup_repository();

        let patient = NewPatient::new("John Doe".into(), "96021817257".into()).unwrap();
        assert!(repository.create_patient(patient).await.is_ok());

        let patient_with_duplicated_pesel_number =
            NewPatient::new("John Doe".into(), "96021817257".into()).unwrap();
        assert_eq!(
            repository
                .create_patient(patient_with_duplicated_pesel_number)
                .await,
            Err(CreatePatientRepositoryError::DuplicatedPeselNumber)
        );
    }
}
