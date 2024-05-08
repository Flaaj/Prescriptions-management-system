use async_trait::async_trait;
use uuid::Uuid;

use crate::domain::patients::models::{NewPatient, Patient};

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

    async fn get_patients(&self) -> anyhow::Result<Vec<Patient>> {
        let patients_from_db = sqlx::query!(
            r#"SELECT id, name, pesel_number, created_at, updated_at FROM patients"#,
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

        let patient = NewPatient::new("John Doe".into(), "96021817257".into()).unwrap();

        repo.create_patient(patient).await.unwrap();

        let patients = repo.get_patients().await.unwrap();
        let first_patient = patients.first().unwrap();

        assert_eq!(first_patient.name, "John Doe");
        assert_eq!(first_patient.pesel_number, "96021817257");
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
}
