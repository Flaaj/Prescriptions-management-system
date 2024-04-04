use async_trait::async_trait;
use uuid::Uuid;

use crate::domain::doctors::models::{Doctor, NewDoctor};

use super::doctors_repository_trait::DoctorsRepositoryTrait;

pub struct DoctorsRepository<'a> {
    pool: &'a sqlx::PgPool,
}

impl<'a> DoctorsRepository<'a> {
    pub fn new(pool: &'a sqlx::PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl<'a> DoctorsRepositoryTrait for DoctorsRepository<'a> {
    async fn create_doctor(&self, doctor: NewDoctor) -> anyhow::Result<()> {
        sqlx::query!(
            r#"INSERT INTO doctors (id, name, pwz_number, pesel_number) VALUES ($1, $2, $3, $4)"#,
            doctor.id,
            doctor.name,
            doctor.pwz_number,
            doctor.pesel_number
        )
        .execute(self.pool)
        .await?;

        Ok(())
    }

    async fn get_doctors(&self) -> anyhow::Result<Vec<Doctor>> {
        let doctors_from_db = sqlx::query!(
            r#"SELECT id, name, pwz_number, pesel_number, created_at, updated_at FROM doctors"#,
        )
        .fetch_all(self.pool)
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

    async fn get_doctor_by_id(&self, doctor_id: Uuid) -> anyhow::Result<Doctor> {
        let doctor_from_db = sqlx::query!(
            r#"SELECT id, name, pwz_number, pesel_number, created_at, updated_at FROM doctors WHERE id = $1"#,
            doctor_id
        )
        .fetch_one(self.pool)
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

#[cfg(test)]
mod integration_tests {
    use crate::{
        create_tables::create_tables,
        domain::doctors::{models::NewDoctor, repository::{
            doctors_repository_impl::DoctorsRepository,
            doctors_repository_trait::DoctorsRepositoryTrait,
        }},
    };

    #[sqlx::test]
    async fn create_and_read_doctors_from_database(pool: sqlx::PgPool) {
        create_tables(&pool, true).await.unwrap();
        let repo = DoctorsRepository::new(&pool);

        let doctor =
            NewDoctor::new("John Doe".into(), "5425740".into(), "96021817257".into()).unwrap();

        repo.create_doctor(doctor).await.unwrap();

        let doctors = repo.get_doctors().await.unwrap();
        let first_doctor = doctors.first().unwrap();

        assert_eq!(first_doctor.name, "John Doe");
        assert_eq!(first_doctor.pwz_number, "5425740");
        assert_eq!(first_doctor.pesel_number, "96021817257");
    }

    #[sqlx::test]
    async fn create_and_read_doctor_by_id(pool: sqlx::PgPool) {
        create_tables(&pool, true).await.unwrap();
        let repo = DoctorsRepository::new(&pool);

        let doctor =
            NewDoctor::new("John Doe".into(), "5425740".into(), "96021817257".into()).unwrap();

        repo.create_doctor(doctor.clone()).await.unwrap();

        let doctor_from_repo = repo.get_doctor_by_id(doctor.id).await.unwrap();

        assert_eq!(doctor_from_repo.name, "John Doe");
        assert_eq!(doctor_from_repo.pwz_number, "5425740");
        assert_eq!(doctor_from_repo.pesel_number, "96021817257");
    }
}
