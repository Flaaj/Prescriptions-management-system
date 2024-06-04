use async_trait::async_trait;
use sqlx::Row;
use uuid::Uuid;

use crate::domain::{
    patients::{
        models::{NewPatient, Patient},
        repository::{
            CreatePatientRepositoryError, GetPatientByIdRepositoryError,
            GetPatientsRepositoryError, PatientsRepository,
        },
    },
    utils::pagination::get_pagination_params,
};

pub struct PostgresPatientsRepository {
    pool: sqlx::PgPool,
}

impl PostgresPatientsRepository {
    pub fn new(pool: sqlx::PgPool) -> Self {
        Self { pool }
    }

    fn parse_patients_row(&self, row: sqlx::postgres::PgRow) -> Result<Patient, sqlx::Error> {
        Ok(Patient {
            id: row.try_get(0)?,
            name: row.try_get(1)?,
            pesel_number: row.try_get(2)?,
            created_at: row.try_get(3)?,
            updated_at: row.try_get(4)?,
        })
    }
}

#[async_trait]
impl PatientsRepository for PostgresPatientsRepository {
    async fn create_patient(
        &self,
        patient: NewPatient,
    ) -> Result<Patient, CreatePatientRepositoryError> {
        let result = sqlx::query(
                r#"INSERT INTO patients (id, name, pesel_number) VALUES ($1, $2, $3) RETURNING id, name, pesel_number, created_at, updated_at"#
            )
            .bind(patient.id)
            .bind(patient.name)
            .bind(patient.pesel_number)
            .fetch_one(&self.pool).await
            .map_err(|err| {
                match err {
                    sqlx::Error::Database(err) if err.is_unique_violation() => {
                        match err.constraint() {
                            Some("patients_pesel_number_key") => {
                                CreatePatientRepositoryError::DuplicatedPeselNumber
                            }
                            _ => CreatePatientRepositoryError::DatabaseError(err.to_string()),
                        }
                    }
                    _ => CreatePatientRepositoryError::DatabaseError(err.to_string()),
                }
            })?;

        let patient = self
            .parse_patients_row(result)
            .map_err(|err| CreatePatientRepositoryError::DatabaseError(err.to_string()))?;
        Ok(patient)
    }

    async fn get_patients(
        &self,
        page: Option<i64>,
        page_size: Option<i64>,
    ) -> Result<Vec<Patient>, GetPatientsRepositoryError> {
        let (page_size, offset) = get_pagination_params(page, page_size)
            .map_err(|err| GetPatientsRepositoryError::InvalidPaginationParams(err.to_string()))?;

        let patients_from_db = sqlx::query(
                r#"SELECT id, name, pesel_number, created_at, updated_at FROM patients LIMIT $1 OFFSET $2"#
            )
            .bind(page_size)
            .bind(offset)
            .fetch_all(&self.pool).await
            .map_err(|err| GetPatientsRepositoryError::DatabaseError(err.to_string()))?;

        let mut patients: Vec<Patient> = Vec::new();
        for record in patients_from_db {
            let patient = self
                .parse_patients_row(record)
                .map_err(|err| GetPatientsRepositoryError::DatabaseError(err.to_string()))?;
            patients.push(patient);
        }

        Ok(patients)
    }

    async fn get_patient_by_id(
        &self,
        patient_id: Uuid,
    ) -> Result<Patient, GetPatientByIdRepositoryError> {
        let patient_from_db = sqlx::query(
            r#"SELECT id, name, pesel_number, created_at, updated_at FROM patients WHERE id = $1"#,
        )
        .bind(patient_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|err| match err {
            sqlx::Error::RowNotFound => GetPatientByIdRepositoryError::NotFound(patient_id),
            _ => GetPatientByIdRepositoryError::DatabaseError(err.to_string()),
        })?;

        let patient = self
            .parse_patients_row(patient_from_db)
            .map_err(|err| GetPatientByIdRepositoryError::DatabaseError(err.to_string()))?;
        Ok(patient)
    }
}

#[cfg(test)]
mod tests {
    use uuid::Uuid;

    use super::PostgresPatientsRepository;
    use crate::{
        domain::patients::{
            models::NewPatient,
            repository::{
                CreatePatientRepositoryError, GetPatientByIdRepositoryError,
                GetPatientsRepositoryError, PatientsRepository,
            },
        },
        infrastructure::postgres_repository_impl::create_tables::create_tables,
    };

    async fn setup_repository(pool: sqlx::PgPool) -> PostgresPatientsRepository {
        create_tables(&pool, true).await.unwrap();
        PostgresPatientsRepository::new(pool)
    }

    #[sqlx::test]
    async fn create_and_read_patient_by_id(pool: sqlx::PgPool) {
        let repository = setup_repository(pool).await;

        let new_patient = NewPatient::new("John Doe".into(), "96021817257".into()).unwrap();

        repository
            .create_patient(new_patient.clone())
            .await
            .unwrap();

        let patient_from_repo = repository.get_patient_by_id(new_patient.id).await.unwrap();

        assert_eq!(patient_from_repo, new_patient);
    }

    #[sqlx::test]
    async fn returns_error_if_patients_with_given_id_doesnt_exist(pool: sqlx::PgPool) {
        let repository = setup_repository(pool).await;
        let patient_id = Uuid::new_v4();

        let patient_from_repo = repository.get_patient_by_id(patient_id).await;

        assert_eq!(
            patient_from_repo,
            Err(GetPatientByIdRepositoryError::NotFound(patient_id))
        );
    }

    #[sqlx::test]
    async fn create_and_read_patients_from_database(pool: sqlx::PgPool) {
        let repository = setup_repository(pool).await;

        let new_patient_0 = NewPatient::new("John Doe".into(), "96021817257".into()).unwrap();
        let new_patient_1 = NewPatient::new("John Doe".into(), "99031301347".into()).unwrap();
        let new_patient_2 = NewPatient::new("John Doe".into(), "92022900002".into()).unwrap();
        let new_patient_3 = NewPatient::new("John Doe".into(), "96021807250".into()).unwrap();

        repository
            .create_patient(new_patient_0.clone())
            .await
            .unwrap();
        repository
            .create_patient(new_patient_1.clone())
            .await
            .unwrap();
        repository
            .create_patient(new_patient_2.clone())
            .await
            .unwrap();
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
        assert_eq!(patients[0], new_patient_0);
        assert_eq!(patients[1], new_patient_1);

        let patients = repository.get_patients(Some(1), Some(3)).await.unwrap();

        assert_eq!(patients.len(), 1);
        assert_eq!(patients[0], new_patient_3);

        let patients = repository.get_patients(Some(2), Some(3)).await.unwrap();

        assert_eq!(patients.len(), 0);
    }

    #[sqlx::test]
    async fn get_patients_returns_error_if_pagination_params_are_incorrect(pool: sqlx::PgPool) {
        let repository = setup_repository(pool).await;

        assert!(match repository.get_patients(Some(-1), Some(10)).await {
            Err(GetPatientsRepositoryError::InvalidPaginationParams(_)) => true,
            _ => false,
        });

        assert!(match repository.get_patients(Some(0), Some(0)).await {
            Err(GetPatientsRepositoryError::InvalidPaginationParams(_)) => true,
            _ => false,
        });
    }

    #[sqlx::test]
    async fn doesnt_create_patient_if_pesel_number_is_duplicated(pool: sqlx::PgPool) {
        let repository = setup_repository(pool).await;

        let patient = NewPatient::new("John Doe".into(), "96021817257".into()).unwrap();
        assert!(repository.create_patient(patient).await.is_ok());

        let patient_with_duplicated_pesel_number =
            NewPatient::new("John Doe".into(), "96021817257".into()).unwrap();
        assert_eq!(
            repository
                .create_patient(patient_with_duplicated_pesel_number)
                .await,
            Err(CreatePatientRepositoryError::DuplicatedPeselNumber)
        )
    }
}
