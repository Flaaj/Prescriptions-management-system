use async_trait::async_trait;
use uuid::Uuid;

use crate::{
    domain::patients::models::{NewPatient, Patient},
    utils::pagination::get_pagination_params,
};

use super::patients_repository_trait::PatientsRepositoryTrait;

pub struct PatientsRepository {
    pool: sqlx::PgPool,
}

impl PatientsRepository {
    pub fn new(pool: sqlx::PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl PatientsRepositoryTrait for PatientsRepository {
    async fn create_patient(&self, patient: NewPatient) -> anyhow::Result<Patient> {
        let result = sqlx::query!(
            r#"INSERT INTO patients (id, name, pesel_number) VALUES ($1, $2, $3) RETURNING id, name, pesel_number, created_at, updated_at"#,
            patient.id,
            patient.name,
            patient.pesel_number
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(Patient {
            id: result.id,
            name: result.name,
            pesel_number: result.pesel_number,
            created_at: result.created_at,
            updated_at: result.updated_at,
        })
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
        .fetch_all(&self.pool)
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
        .fetch_one(&self.pool)
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
        let repository = PatientsRepository::new(pool);

        let new_patient_0 = NewPatient::new("John Doe".into(), "96021817257".into()).unwrap();
        repository
            .create_patient(new_patient_0.clone())
            .await
            .unwrap();
        let new_patient_1 = NewPatient::new("John Doe".into(), "99031301347".into()).unwrap();
        repository
            .create_patient(new_patient_1.clone())
            .await
            .unwrap();
        let new_patient_2 = NewPatient::new("John Doe".into(), "92022900002".into()).unwrap();
        repository
            .create_patient(new_patient_2.clone())
            .await
            .unwrap();
        let new_patient_3 = NewPatient::new("John Doe".into(), "96021807250".into()).unwrap();
        repository
            .create_patient(new_patient_3.clone())
            .await
            .unwrap();

        let patients = repository.get_patients(None, Some(10)).await.unwrap();
        assert_eq!(patients.len(), 4);

        assert_eq!(patients[0], new_patient_0);
        assert_eq!(patients[1], new_patient_1);
        assert_eq!(patients[2], new_patient_2);
        assert_eq!(patients[3], new_patient_3);

        let patients = repository.get_patients(None, Some(2)).await.unwrap();
        assert_eq!(patients.len(), 2);

        let patients = repository.get_patients(Some(1), Some(3)).await.unwrap();
        assert_eq!(patients.len(), 1);

        let patients = repository.get_patients(Some(2), Some(3)).await.unwrap();
        assert_eq!(patients.len(), 0);
    }

    #[sqlx::test]
    async fn create_and_read_patient_by_id(pool: sqlx::PgPool) {
        create_tables(&pool, true).await.unwrap();
        let repository = PatientsRepository::new(pool);

        let new_patient = NewPatient::new("John Doe".into(), "96021817257".into()).unwrap();

        repository
            .create_patient(new_patient.clone())
            .await
            .unwrap();

        let patient_from_repo = repository.get_patient_by_id(new_patient.id).await.unwrap();

        assert_eq!(patient_from_repo, new_patient);
    }

    #[sqlx::test]
    async fn doesnt_create_patient_if_pesel_number_is_duplicated(pool: sqlx::PgPool) {
        create_tables(&pool, true).await.unwrap();
        let repository = PatientsRepository::new(pool);

        let patient = NewPatient::new("John Doe".into(), "96021817257".into()).unwrap();
        assert!(repository.create_patient(patient).await.is_ok());

        let patient_with_duplicated_pesel_number =
            NewPatient::new("John Doe".into(), "96021817257".into()).unwrap();
        assert!(repository
            .create_patient(patient_with_duplicated_pesel_number)
            .await
            .is_err());
    }
}
