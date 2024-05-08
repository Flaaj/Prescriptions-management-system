use async_trait::async_trait;
use uuid::Uuid;

use crate::domain::patients::models::{Patient, NewPatient};

#[async_trait]
pub trait PatientsRepositoryTrait {
    async fn create_patient(&self, patient: NewPatient) -> anyhow::Result<()>;
    async fn get_patients(&self) -> anyhow::Result<Vec<Patient>>;
    async fn get_patient_by_id(&self, patient_id: Uuid) -> anyhow::Result<Patient>;
}
