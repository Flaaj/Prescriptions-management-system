use async_trait::async_trait;
use uuid::Uuid;

use crate::{
    domain::patients::models::{NewPatient, Patient},
    utils::pagination::get_pagination_params,
};

use super::patients_repository_trait::PatientsRepositoryTrait;

pub struct PatientsRepository<'a> {
    pool: &'a sqlx::PgPool,
}

impl<'a> PatientsRepository<'a> {
    pub fn new(pool: &'a sqlx::PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl<'a> PatientsRepositoryTrait for PatientsRepository<'a> {
    async fn create_patient(&self, patient: NewPatient) -> anyhow::Result<()> {
        sqlx::query!(
            r#"INSERT INTO patients (id, name, pesel_number) VALUES ($1, $2, $3)"#,
            patient.id,
            patient.name,
            patient.pesel_number
        )
        .execute(self.pool)
        .await?;

        Ok(())
    }

    async fn get_patients(
        &self,
        page: Option<i64>,
        page_size: Option<i64>,
    ) -> anyhow::Result<Vec<Patient>> {
        let (page_size, offset) = get_pagination_params(page, page_size)?;

        let patients_from_db = sqlx::query!(
            r#"SELECT id, name, pesel_number, created_at, updated_at FROM patients LIMIT $1 OFFSET $2"#,
            page_size,
            offset
        )
        .fetch_all(self.pool)
        .await?;

        let patients = patients_from_db
            .into_iter()
            .map(|record| Patient {
                id: record.id,
                name: record.name,
                pesel_number: record.pesel_number,
                created_at: record.created_at,
                updated_at: record.updated_at,
            })
            .collect();

        Ok(patients)
    }

    async fn get_patient_by_id(&self, patient_id: Uuid) -> anyhow::Result<Patient> {
        let patient_from_db = sqlx::query!(
            r#"SELECT id, name, pesel_number, created_at, updated_at FROM patients WHERE id = $1"#,
            patient_id
        )
        .fetch_one(self.pool)
        .await?;

        Ok(Patient {
            id: patient_from_db.id,
            name: patient_from_db.name,
            pesel_number: patient_from_db.pesel_number,
            created_at: patient_from_db.created_at,
            updated_at: patient_from_db.updated_at,
        })
    }
}

#[cfg(test)]
mod integration_tests {
    use crate::{
        create_tables::create_tables,
        domain::patients::{
            models::NewPatient,
            repository::{
                patients_repository_impl::PatientsRepository,
                patients_repository_trait::PatientsRepositoryTrait,
            },
        },
    };

    #[sqlx::test]
    async fn create_and_read_patients_from_database(pool: sqlx::PgPool) {
        create_tables(&pool, true).await.unwrap();
        let repo = PatientsRepository::new(&pool);

        repo.create_patient(NewPatient::new("John Doe".into(), "96021817257".into()).unwrap())
            .await
            .unwrap();
        repo.create_patient(NewPatient::new("John Doe".into(), "99031301347".into()).unwrap())
            .await
            .unwrap();
        repo.create_patient(NewPatient::new("John Doe".into(), "92022900002".into()).unwrap())
            .await
            .unwrap();
        repo.create_patient(NewPatient::new("John Doe".into(), "96021807250".into()).unwrap())
            .await
            .unwrap();

        let patients = repo.get_patients(None, Some(2)).await.unwrap();
        assert_eq!(patients.len(), 2);

        let patients = repo.get_patients(None, Some(10)).await.unwrap();
        assert!(patients.len() == 4);

        let patients = repo.get_patients(Some(1), Some(3)).await.unwrap();
        assert!(patients.len() == 1);

        let patients = repo.get_patients(Some(2), Some(3)).await.unwrap();
        assert!(patients.len() == 0);
    }

    #[sqlx::test]
    async fn create_and_read_patient_by_id(pool: sqlx::PgPool) {
        create_tables(&pool, true).await.unwrap();
        let repo = PatientsRepository::new(&pool);

        let patient = NewPatient::new("John Doe".into(), "96021817257".into()).unwrap();

        repo.create_patient(patient.clone()).await.unwrap();

        let patient_from_repo = repo.get_patient_by_id(patient.id).await.unwrap();

        assert_eq!(patient_from_repo.id, patient.id);
    }

    #[sqlx::test]
    async fn doesnt_create_patient_if_pesel_number_is_duplicated(pool: sqlx::PgPool) {
        create_tables(&pool, true).await.unwrap();
        let repo = PatientsRepository::new(&pool);

        let patient = NewPatient::new("John Doe".into(), "96021817257".into()).unwrap();
        assert!(repo.create_patient(patient).await.is_ok());

        let patient_with_duplicated_pesel_number =
            NewPatient::new("John Doe".into(), "96021817257".into()).unwrap();
        assert!(repo
            .create_patient(patient_with_duplicated_pesel_number)
            .await
            .is_err());
    }
}