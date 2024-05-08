use async_trait::async_trait;
use uuid::Uuid;

use crate::{
    domain::doctors::models::{Doctor, NewDoctor},
    utils::pagination::get_pagination_params,
};

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

    async fn get_doctors(
        &self,
        page: Option<i64>,
        page_size: Option<i64>,
    ) -> anyhow::Result<Vec<Doctor>> {
        let (page_size, offset) = get_pagination_params(page, page_size)?;

        let doctors_from_db = sqlx::query!(
            r#"SELECT 
                id, 
                name, 
                pwz_number, 
                pesel_number, 
                created_at, 
                updated_at 
            FROM doctors 
            LIMIT $1 OFFSET $2"#,
            page_size,
            offset
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
        domain::doctors::{
            models::NewDoctor,
            repository::{
                doctors_repository_impl::DoctorsRepository,
                doctors_repository_trait::DoctorsRepositoryTrait,
            },
        },
    };

    #[sqlx::test]
    async fn create_and_read_doctors_from_database(pool: sqlx::PgPool) {
        create_tables(&pool, true).await.unwrap();
        let repo = DoctorsRepository::new(&pool);

        repo.create_doctor(
            NewDoctor::new("John Doe".into(), "5425740".into(), "96021817257".into()).unwrap(),
        )
        .await
        .unwrap();
        repo.create_doctor(
            NewDoctor::new("John Doe".into(), "8463856".into(), "99031301347".into()).unwrap(),
        )
        .await
        .unwrap();
        repo.create_doctor(
            NewDoctor::new("John Doe".into(), "3123456".into(), "92022900002".into()).unwrap(),
        )
        .await
        .unwrap();
        repo.create_doctor(
            NewDoctor::new("John Doe".into(), "5425751".into(), "96021807250".into()).unwrap(),
        )
        .await
        .unwrap();

        let doctors = repo.get_doctors(None, Some(2)).await.unwrap();
        assert_eq!(doctors.len(), 2);

        let doctors = repo.get_doctors(None, Some(10)).await.unwrap();
        assert!(doctors.len() == 4);

        let doctors = repo.get_doctors(Some(1), Some(3)).await.unwrap();
        assert!(doctors.len() == 1);

        let doctors = repo.get_doctors(Some(2), Some(3)).await.unwrap();
        assert!(doctors.len() == 0);
    }

    #[sqlx::test]
    async fn create_and_read_doctor_by_id(pool: sqlx::PgPool) {
        create_tables(&pool, true).await.unwrap();
        let repo = DoctorsRepository::new(&pool);

        let doctor =
            NewDoctor::new("John Doe".into(), "5425740".into(), "96021817257".into()).unwrap();

        repo.create_doctor(doctor.clone()).await.unwrap();

        let doctor_from_repo = repo.get_doctor_by_id(doctor.id).await.unwrap();

        assert_eq!(doctor_from_repo.id, doctor.id);
    }

    #[sqlx::test]
    async fn doesnt_create_doctor_if_pwz_or_pesel_numbers_are_duplicated(pool: sqlx::PgPool) {
        create_tables(&pool, true).await.unwrap();
        let repo = DoctorsRepository::new(&pool);

        let doctor =
            NewDoctor::new("John Doe".into(), "5425740".into(), "96021817257".into()).unwrap();
        assert!(repo.create_doctor(doctor).await.is_ok());

        let doctor_with_duplicated_pwz_number =
            NewDoctor::new("John Doe".into(), "5425740".into(), "99031301347".into()).unwrap();
        assert!(repo
            .create_doctor(doctor_with_duplicated_pwz_number)
            .await
            .is_err());

        let doctor_with_duplicated_pesel_number =
            NewDoctor::new("John Doe".into(), "3123456".into(), "96021817257".into()).unwrap();
        assert!(repo
            .create_doctor(doctor_with_duplicated_pesel_number)
            .await
            .is_err());
    }
}
