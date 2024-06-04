use async_trait::async_trait;
use sqlx::Row;
use uuid::Uuid;

use crate::domain::{
    doctors::{
        models::{Doctor, NewDoctor},
        repository::{
            CreateDoctorRepositoryError, DoctorsRepository, GetDoctorByIdRepositoryError,
            GetDoctorsRepositoryError,
        },
    },
    utils::pagination::get_pagination_params,
};

#[derive(Clone)]
pub struct PostgresDoctorsRepository {
    pool: sqlx::PgPool,
}

impl PostgresDoctorsRepository {
    pub fn new(pool: sqlx::PgPool) -> Self {
        Self { pool }
    }

    fn parse_doctors_row(&self, row: sqlx::postgres::PgRow) -> Result<Doctor, sqlx::Error> {
        Ok(Doctor {
            id: row.try_get(0)?,
            name: row.try_get(1)?,
            pwz_number: row.try_get(2)?,
            pesel_number: row.try_get(3)?,
            created_at: row.try_get(4)?,
            updated_at: row.try_get(5)?,
        })
    }
}

#[async_trait]
impl DoctorsRepository for PostgresDoctorsRepository {
    async fn create_doctor(
        &self,
        doctor: NewDoctor,
    ) -> Result<Doctor, CreateDoctorRepositoryError> {
        let result = sqlx::query(
                r#"INSERT INTO doctors (id, name, pwz_number, pesel_number) VALUES ($1, $2, $3, $4) RETURNING id, name, pwz_number, pesel_number, created_at, updated_at"#
            )
            .bind(doctor.id)
            .bind(doctor.name)
            .bind(doctor.pwz_number)
            .bind(doctor.pesel_number)
            .fetch_one(&self.pool).await
            .map_err(|err| {
                match err {
                    sqlx::Error::Database(err) if err.is_unique_violation() => {
                        match err.constraint() {
                            Some("doctors_pwz_number_key") => {
                                CreateDoctorRepositoryError::DuplicatedPwzNumber
                            }
                            Some("doctors_pesel_number_key") => {
                                CreateDoctorRepositoryError::DuplicatedPeselNumber
                            }
                            _ => CreateDoctorRepositoryError::DatabaseError(err.to_string()),
                        }
                    },
                    _ => CreateDoctorRepositoryError::DatabaseError(err.to_string()),
                }
            })?;

        let doctor = self
            .parse_doctors_row(result)
            .map_err(|err| CreateDoctorRepositoryError::DatabaseError(err.to_string()))?;

        Ok(doctor)
    }

    async fn get_doctors(
        &self,
        page: Option<i64>,
        page_size: Option<i64>,
    ) -> Result<Vec<Doctor>, GetDoctorsRepositoryError> {
        let (page_size, offset) = get_pagination_params(page, page_size)
            .map_err(|err| GetDoctorsRepositoryError::InvalidPaginationParams(err.to_string()))?;

        let doctors_from_db = sqlx::query(
                r#"SELECT id, name, pwz_number, pesel_number, created_at, updated_at FROM doctors LIMIT $1 OFFSET $2"#
            )
            .bind(page_size)
            .bind(offset)
            .fetch_all(&self.pool).await
            .map_err(|err| GetDoctorsRepositoryError::DatabaseError(err.to_string()))?;

        let mut doctors: Vec<Doctor> = Vec::new();
        for record in doctors_from_db {
            let doctor = self
                .parse_doctors_row(record)
                .map_err(|err| GetDoctorsRepositoryError::DatabaseError(err.to_string()))?;
            doctors.push(doctor);
        }

        Ok(doctors)
    }

    async fn get_doctor_by_id(
        &self,
        doctor_id: Uuid,
    ) -> Result<Doctor, GetDoctorByIdRepositoryError> {
        let doctor_from_db = sqlx::query(
                r#"SELECT id, name, pwz_number, pesel_number, created_at, updated_at FROM doctors WHERE id = $1"#
            )
            .bind(doctor_id)
            .fetch_one(&self.pool).await
            .map_err(|err| {
                match err {
                    sqlx::Error::RowNotFound => GetDoctorByIdRepositoryError::NotFound(doctor_id),
                    _ => GetDoctorByIdRepositoryError::DatabaseError(err.to_string()),
                }
            })?;

        let doctor = self
            .parse_doctors_row(doctor_from_db)
            .map_err(|err| GetDoctorByIdRepositoryError::DatabaseError(err.to_string()))?;

        Ok(doctor)
    }
}

#[cfg(test)]
mod tests {
    use uuid::Uuid;

    use super::PostgresDoctorsRepository;
    use crate::{
        domain::doctors::{
            models::NewDoctor,
            repository::{
                CreateDoctorRepositoryError, DoctorsRepository, GetDoctorByIdRepositoryError,
                GetDoctorsRepositoryError,
            },
        },
        infrastructure::postgres_repository_impl::create_tables::create_tables,
    };

    async fn setup_repository(pool: sqlx::PgPool) -> PostgresDoctorsRepository {
        create_tables(&pool, true).await.unwrap();
        PostgresDoctorsRepository::new(pool)
    }

    #[sqlx::test]
    async fn create_and_read_doctor_by_id(pool: sqlx::PgPool) {
        let repository = setup_repository(pool).await;

        let new_doctor =
            NewDoctor::new("John Does".into(), "5425740".into(), "96021817257".into()).unwrap();

        repository.create_doctor(new_doctor.clone()).await.unwrap();

        let doctor_from_repo = repository.get_doctor_by_id(new_doctor.id).await.unwrap();

        assert_eq!(doctor_from_repo, new_doctor);
    }

    #[sqlx::test]
    async fn returns_error_if_doctor_with_given_id_doesnt_exist(pool: sqlx::PgPool) {
        let repository = setup_repository(pool).await;
        let doctor_id = Uuid::new_v4();

        let doctor_from_repo = repository.get_doctor_by_id(doctor_id).await;

        assert_eq!(
            doctor_from_repo,
            Err(GetDoctorByIdRepositoryError::NotFound(doctor_id))
        );
    }

    #[sqlx::test]
    async fn create_and_read_doctors_from_database(pool: sqlx::PgPool) {
        let repository = setup_repository(pool).await;

        let new_doctor_0 =
            NewDoctor::new("John First".into(), "5425740".into(), "96021817257".into()).unwrap();
        let new_doctor_1 =
            NewDoctor::new("John Second".into(), "8463856".into(), "99031301347".into()).unwrap();
        let new_doctor_2 =
            NewDoctor::new("John Third".into(), "3123456".into(), "92022900002".into()).unwrap();
        let new_doctor_3 =
            NewDoctor::new("John Fourth".into(), "5425751".into(), "96021807250".into()).unwrap();

        repository
            .create_doctor(new_doctor_0.clone())
            .await
            .unwrap();
        repository
            .create_doctor(new_doctor_1.clone())
            .await
            .unwrap();
        repository
            .create_doctor(new_doctor_2.clone())
            .await
            .unwrap();
        repository
            .create_doctor(new_doctor_3.clone())
            .await
            .unwrap();

        let doctors = repository.get_doctors(None, Some(10)).await.unwrap();

        assert_eq!(doctors.len(), 4);
        assert_eq!(doctors[0], new_doctor_0);
        assert_eq!(doctors[1], new_doctor_1);
        assert_eq!(doctors[2], new_doctor_2);
        assert_eq!(doctors[3], new_doctor_3);

        let doctors = repository.get_doctors(None, Some(2)).await.unwrap();

        assert_eq!(doctors.len(), 2);
        assert_eq!(doctors[0], new_doctor_0);
        assert_eq!(doctors[1], new_doctor_1);

        let doctors = repository.get_doctors(Some(1), Some(3)).await.unwrap();

        assert_eq!(doctors.len(), 1);
        assert_eq!(doctors[0], new_doctor_3);

        let doctors = repository.get_doctors(Some(2), Some(3)).await.unwrap();

        assert_eq!(doctors.len(), 0);
    }

    #[sqlx::test]
    async fn get_doctors_returns_error_if_pagination_params_are_incorrect(pool: sqlx::PgPool) {
        let repository = setup_repository(pool).await;

        assert!(match repository.get_doctors(Some(-1), Some(10)).await {
            Err(GetDoctorsRepositoryError::InvalidPaginationParams(_)) => true,
            _ => false,
        });

        assert!(match repository.get_doctors(Some(0), Some(0)).await {
            Err(GetDoctorsRepositoryError::InvalidPaginationParams(_)) => true,
            _ => false,
        });
    }

    #[sqlx::test]
    async fn doesnt_create_doctor_if_pwz_or_pesel_numbers_are_duplicated(pool: sqlx::PgPool) {
        let repository = setup_repository(pool).await;

        let doctor =
            NewDoctor::new("John Doe".into(), "5425740".into(), "96021817257".into()).unwrap();

        assert!(repository.create_doctor(doctor).await.is_ok());

        let doctor_with_duplicated_pwz_number =
            NewDoctor::new("John Doe".into(), "5425740".into(), "99031301347".into()).unwrap();

        assert_eq!(
            repository
                .create_doctor(doctor_with_duplicated_pwz_number)
                .await,
            Err(CreateDoctorRepositoryError::DuplicatedPwzNumber)
        );

        let doctor_with_duplicated_pesel_number =
            NewDoctor::new("John Doe".into(), "3123456".into(), "96021817257".into()).unwrap();

        assert_eq!(
            repository
                .create_doctor(doctor_with_duplicated_pesel_number)
                .await,
            Err(CreateDoctorRepositoryError::DuplicatedPeselNumber)
        );
    }
}
