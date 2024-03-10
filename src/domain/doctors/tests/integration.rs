#[cfg(test)]
mod integration_tests {
    use crate::{
        create_tables::create_tables,
        domain::doctors::{create_doctor::NewDoctor, get_doctors_repository::DoctorsRepository},
    };

    #[sqlx::test]
    async fn create_and_read_doctors_from_database(pool: sqlx::PgPool) {
        create_tables(&pool, true).await.unwrap();

        let doctor =
            NewDoctor::new("John Doe".into(), "5425740".into(), "96021817257".into()).unwrap();

        doctor.commit_to_repository(&pool).await.unwrap();

        let doctors = DoctorsRepository::get_doctors(&pool).await.unwrap();
        let first_doctor = doctors.first().unwrap();

        assert_eq!(first_doctor.name, "John Doe");
        assert_eq!(first_doctor.pwz_number, "5425740");
        assert_eq!(first_doctor.pesel_number, "96021817257");
    }

    #[sqlx::test]
    async fn create_and_read_doctor_by_id(pool: sqlx::PgPool) {
        create_tables(&pool, true).await.unwrap();

        let doctor =
            NewDoctor::new("John Doe".into(), "5425740".into(), "96021817257".into()).unwrap();

        doctor.clone().commit_to_repository(&pool).await.unwrap();

        let doctor_from_repo = DoctorsRepository::get_doctor_by_id(&pool, &doctor.id)
            .await
            .unwrap();

        assert_eq!(doctor_from_repo.name, "John Doe");
        assert_eq!(doctor_from_repo.pwz_number, "5425740");
        assert_eq!(doctor_from_repo.pesel_number, "96021817257");
    }
}
