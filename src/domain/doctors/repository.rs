use async_trait::async_trait;
use uuid::Uuid;

use crate::domain::doctors::models::{Doctor, NewDoctor};

#[async_trait]
pub trait DoctorsRepository {
    async fn create_doctor(&self, doctor: NewDoctor) -> anyhow::Result<Doctor>;
    async fn get_doctors(
        &self,
        page: Option<i64>,
        page_size: Option<i64>,
    ) -> anyhow::Result<Vec<Doctor>>;
    async fn get_doctor_by_id(&self, doctor_id: Uuid) -> anyhow::Result<Doctor>;
}
