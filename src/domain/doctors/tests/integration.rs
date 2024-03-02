#[cfg(test)]
mod integration_tests {
    use chrono::{Duration, Utc};
    use uuid::Uuid;

    use crate::{
        create_tables::create_tables,
        domain::doctors::{create_doctor::NewDoctor, get_doctors_repository::DoctorsRepository},
    };

    #[sqlx::test]
    async fn create_and_read_doctors_from_database(pool: sqlx::PgPool) -> anyhow::Result<()> {
        create_tables(&pool).await?;

        let doctor_name = "John Doe";
        let pwz_number = "5425740";
        let pesel_number = "96021817257";

        let doctor = NewDoctor::new(doctor_name.into(), pwz_number.into(), pesel_number.into())?;

        doctor.commit_to_repository(&pool).await?;

        let doctors = DoctorsRepository::get_doctors(&pool).await?;
        let first_doctor = doctors.first().unwrap();

        assert_eq!(first_doctor.name, doctor_name);
        assert_eq!(first_doctor.pwz_number, pwz_number);
        assert_eq!(first_doctor.pesel_number, pesel_number);

        Ok(())
    }
}
