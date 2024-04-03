use async_trait::async_trait;
use uuid::Uuid;

use crate::domain::doctors::use_cases::{create_doctor::NewDoctor, get_doctors::Doctor};

#[async_trait]
pub trait DoctorsRepositoryTrait {
    async fn create_doctor(&self, doctor: NewDoctor) -> anyhow::Result<()>;
    async fn get_doctors(&self) -> anyhow::Result<Vec<Doctor>>;
    async fn get_doctor_by_id(&self, doctor_id: Uuid) -> anyhow::Result<Doctor>;
}
