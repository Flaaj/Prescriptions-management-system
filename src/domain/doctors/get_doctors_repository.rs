use uuid::Uuid;

use super::get_doctors::Doctor;

pub struct DoctorsRepository {}

impl DoctorsRepository {
    pub async fn get_doctors(pool: &sqlx::PgPool) -> anyhow::Result<Vec<Doctor>> {
        let doctors_from_db = sqlx::query!(
            r#"SELECT id, name, pwz_number, pesel_number, created_at, updated_at FROM doctors"#,
        )
        .fetch_all(pool)
        .await?;

        let doctors = doctors_from_db
            .into_iter()
            .map(|record| Doctor {
                id: record.id,
                name: record.name,
                pwz_number: record.pwz_number,
                pesel_number: record.pesel_number,
                created_at: record.created_at,
                updated_at: record.updated_at,
            })
            .collect();

        Ok(doctors)
    }

    pub async fn get_doctor_by_id(pool: &sqlx::PgPool, id: &Uuid) -> anyhow::Result<Doctor> {
        let doctor_from_db = sqlx::query!(
            r#"SELECT id, name, pwz_number, pesel_number, created_at, updated_at FROM doctors WHERE id = $1"#,
            id
        )
        .fetch_one(pool)
        .await?;

        Ok(Doctor {
            id: doctor_from_db.id,
            name: doctor_from_db.name,
            pwz_number: doctor_from_db.pwz_number,
            pesel_number: doctor_from_db.pesel_number,
            created_at: doctor_from_db.created_at,
            updated_at: doctor_from_db.updated_at,
        })
    }
}
