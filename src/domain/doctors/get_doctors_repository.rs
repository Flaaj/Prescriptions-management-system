use chrono::{DateTime, Utc};
use uuid::Uuid;

use super::get_doctors::Doctor;

pub struct DoctorsRepository {}

impl DoctorsRepository {
    pub async fn get_doctors(pool: &sqlx::PgPool) -> anyhow::Result<Vec<Doctor>> {
        let rows =
            sqlx::query_as::<_, (Uuid, String, String, String, DateTime<Utc>, DateTime<Utc>)>(
                r#"SELECT id, name, pwz_number, pesel_number, created_at, updated_at FROM doctors"#,
            )
            .fetch_all(pool)
            .await?;

        let doctors = rows
            .into_iter()
            .map(|row| {
                let (id, name, pwz_number, pesel_number, created_at, updated_at) = row;
                Doctor {
                    id,
                    name,
                    pwz_number,
                    pesel_number,
                    created_at,
                    updated_at,
                }
            })
            .collect();

        Ok(doctors)
    }
}
