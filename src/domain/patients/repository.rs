use crate::domain::{
    patients::models::{NewPatient, Patient},
    utils::pagination::get_pagination_params,
};
use async_trait::async_trait;
use chrono::Utc;
use std::sync::RwLock;
use uuid::Uuid;

#[async_trait]
pub trait PatientsRepository {
    async fn create_patient(&self, patient: NewPatient) -> anyhow::Result<Patient>;
    async fn get_patients(
        &self,
        page: Option<i64>,
        page_size: Option<i64>,
    ) -> anyhow::Result<Vec<Patient>>;
    async fn get_patient_by_id(&self, patient_id: Uuid) -> anyhow::Result<Patient>;
}

/// Used to test the service layer in isolation
pub struct InMemoryPatientsRepository {
    patients: RwLock<Vec<Patient>>,
}

impl InMemoryPatientsRepository {
    #[allow(dead_code)]
    pub fn new() -> Self {
        Self {
            patients: RwLock::new(Vec::new()),
        }
    }
}

#[async_trait]
impl PatientsRepository for InMemoryPatientsRepository {
    async fn create_patient(&self, new_patient: NewPatient) -> anyhow::Result<Patient> {
        let does_pesel_number_exist = self
            .patients
            .read()
            .unwrap()
            .iter()
            .any(|patient| patient.pesel_number == new_patient.pesel_number);

        if does_pesel_number_exist {
            return Err(anyhow::anyhow!("PWZ or PESEL number already exists"));
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
    ) -> anyhow::Result<Vec<Patient>> {
        let (page_size, offset) = get_pagination_params(page, page_size)?;
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

    async fn get_patient_by_id(&self, patient_id: Uuid) -> anyhow::Result<Patient> {
        match self
            .patients
            .read()
            .unwrap()
            .iter()
            .find(|patient| patient.id == patient_id)
        {
            Some(patient) => Ok(patient.clone()),
            None => Err(anyhow::anyhow!("Patient not found")),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::InMemoryPatientsRepository;
    use crate::domain::patients::{models::NewPatient, repository::PatientsRepository};
    use uuid::Uuid;

    async fn setup_repository() -> InMemoryPatientsRepository {
        InMemoryPatientsRepository::new()
    }

    #[tokio::test]
    async fn create_and_read_patient_by_id() {
        let repository = setup_repository().await;

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
        let repository = setup_repository().await;

        let patient_from_repo = repository.get_patient_by_id(Uuid::new_v4()).await;

        assert!(patient_from_repo.is_err());
    }

    #[tokio::test]
    async fn create_and_read_patients_from_database() {
        let repository = setup_repository().await;

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
    async fn doesnt_create_patient_if_pesel_number_is_duplicated() {
        let repository = setup_repository().await;

        let patient = NewPatient::new("John Doe".into(), "96021817257".into()).unwrap();
        assert!(repository.create_patient(patient).await.is_ok());

        let patient_with_duplicated_pesel_number =
            NewPatient::new("John Doe".into(), "96021817257".into()).unwrap();
        assert!(repository
            .create_patient(patient_with_duplicated_pesel_number)
            .await
            .is_err());
    }
}
