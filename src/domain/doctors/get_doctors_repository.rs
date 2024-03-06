use super::get_doctors::Doctor;

pub struct DoctorsRepository {}

impl DoctorsRepository {
    pub async fn get_doctors(pool: &sqlx::PgPool) -> anyhow::Result<Vec<Doctor>> {
        let doctors_from_db = sqlx::query_as(
            r#"SELECT id, name, pwz_number, pesel_number, created_at, updated_at FROM doctors"#,
        )
        .fetch_all(pool)
        .await?;

        let doctors = doctors_from_db
            .into_iter()
            .map(
                |(id, name, pwz_number, pesel_number, created_at, updated_at)| Doctor {
                    id,
                    name,
                    pwz_number,
                    pesel_number,
                    created_at,
                    updated_at,
                },
            )
            .collect();

        Ok(doctors)
    }
}