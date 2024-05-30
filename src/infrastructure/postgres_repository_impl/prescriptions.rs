use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::{postgres::PgRow, Row};
use uuid::Uuid;

use crate::domain::{
    prescriptions::{
        models::{
            NewPrescription, NewPrescriptionFill, PrescribedDrug, Prescription, PrescriptionDoctor,
            PrescriptionFill, PrescriptionPatient, PrescriptionType,
        },
        repository::{
            CreatePrescriptionRepositoryError, FillPrescriptionRepositoryError,
            GetPrescriptionByIdRepositoryError, GetPrescriptionsRepositoryError,
            PrescriptionsRepository,
        },
    },
    utils::pagination::get_pagination_params,
};

pub struct PostgresPrescriptionsRepository {
    pool: sqlx::PgPool,
}

struct PrescriptionsRow {
    prescription_id: Uuid,
    prescription_code: String,
    prescription_prescription_type: PrescriptionType,
    prescription_start_date: DateTime<Utc>,
    prescription_end_date: DateTime<Utc>,
    prescription_created_at: DateTime<Utc>,
    prescription_updated_at: DateTime<Utc>,
    doctor_id: Uuid,
    doctor_name: String,
    doctor_pesel_number: String,
    doctor_pwz_number: String,
    patient_id: Uuid,
    patient_name: String,
    patient_pesel_number: String,
    prescribed_drug_id: Uuid,
    prescribed_drug_drug_id: Uuid,
    prescribed_drug_quantity: i32,
    prescribed_drug_created_at: DateTime<Utc>,
    prescribed_drug_updated_at: DateTime<Utc>,
    prescription_fill_id: Option<Uuid>,
    prescription_fill_pharmacist_id: Option<Uuid>,
    prescription_fill_created_at: Option<DateTime<Utc>>,
    prescription_fill_updated_at: Option<DateTime<Utc>>,
}

impl PostgresPrescriptionsRepository {
    pub fn new(pool: sqlx::PgPool) -> Self {
        Self { pool }
    }

    fn parse_prescription_row(&self, row: PgRow) -> Result<PrescriptionsRow, sqlx::Error> {
        Ok(PrescriptionsRow {
            prescription_id: row.try_get(0)?,
            prescription_code: row.try_get(1)?,
            prescription_prescription_type: row.try_get(2)?,
            prescription_start_date: row.try_get(3)?,
            prescription_end_date: row.try_get(4)?,
            prescription_created_at: row.try_get(5)?,
            prescription_updated_at: row.try_get(6)?,
            doctor_id: row.try_get(7)?,
            doctor_name: row.try_get(8)?,
            doctor_pesel_number: row.try_get(9)?,
            doctor_pwz_number: row.try_get(10)?,
            patient_id: row.try_get(11)?,
            patient_name: row.try_get(12)?,
            patient_pesel_number: row.try_get(13)?,
            prescribed_drug_id: row.try_get(14)?,
            prescribed_drug_drug_id: row.try_get(15)?,
            prescribed_drug_quantity: row.try_get(16)?,
            prescribed_drug_created_at: row.try_get(17)?,
            prescribed_drug_updated_at: row.try_get(18)?,
            prescription_fill_id: row.try_get(19)?,
            prescription_fill_pharmacist_id: row.try_get(20)?,
            prescription_fill_created_at: row.try_get(21)?,
            prescription_fill_updated_at: row.try_get(22)?,
        })
    }
}

#[async_trait]
impl PrescriptionsRepository for PostgresPrescriptionsRepository {
    async fn create_prescription(
        &self,
        prescription: NewPrescription,
    ) -> Result<Prescription, CreatePrescriptionRepositoryError> {
        let transaction = self
            .pool
            .begin()
            .await
            .map_err(|err| CreatePrescriptionRepositoryError::DatabaseError(err.to_string()))?;

        sqlx::query!(
            r#"INSERT INTO prescriptions (id, patient_id, doctor_id, code, prescription_type, start_date, end_date) VALUES ($1, $2, $3, $4, $5, $6, $7)"#,
            prescription.id,
            prescription.patient_id,
            prescription.doctor_id,
            prescription.code,
            prescription.prescription_type as _,
            prescription.start_date,
            prescription.end_date
        )
        .execute(&self.pool)
        .await.map_err(|err| match err {
            sqlx::Error::Database(err) if err.message().contains("insert or update on table \"prescriptions\" violates foreign key constraint \"prescriptions_doctor_id_fkey\"") => {
                CreatePrescriptionRepositoryError::DoctorNotFound(prescription.doctor_id)
            },
            sqlx::Error::Database(err) if err.message().contains("insert or update on table \"prescriptions\" violates foreign key constraint \"prescriptions_patient_id_fkey\"") => {
                CreatePrescriptionRepositoryError::PatientNotFound(prescription.patient_id)
            },
            err => CreatePrescriptionRepositoryError::DatabaseError(err.to_string()),
        })?;

        for prescribed_drug in &prescription.prescribed_drugs {
            sqlx::query!(
                r#"INSERT INTO prescribed_drugs (id, prescription_id, drug_id, quantity) VALUES ($1, $2, $3, $4)"#,
                Uuid::new_v4(),
                prescription.id,
                prescribed_drug.drug_id,
                prescribed_drug.quantity as i32
            )
            .execute(&self.pool)
            .await.map_err(|err| {
                match err {
                    sqlx::Error::Database(err) if err.message().contains("insert or update on table \"prescribed_drugs\" violates foreign key constraint \"prescribed_drugs_drug_id_fkey\"") => {
                        CreatePrescriptionRepositoryError::DrugNotFound(prescribed_drug.drug_id)
                    },
                    err => CreatePrescriptionRepositoryError::DatabaseError(err.to_string()),
                }
            })?;
        }

        let prescription = self
            .get_prescription_by_id(prescription.id)
            .await
            .map_err(|err| CreatePrescriptionRepositoryError::DatabaseError(err.to_string()))?;

        transaction
            .commit()
            .await
            .map_err(|err| CreatePrescriptionRepositoryError::DatabaseError(err.to_string()))?;

        Ok(prescription)
    }

    async fn get_prescriptions(
        &self,
        page: Option<i64>,
        page_size: Option<i64>,
    ) -> Result<Vec<Prescription>, GetPrescriptionsRepositoryError> {
        let (page_size, offset) = get_pagination_params(page, page_size).map_err(|err| {
            GetPrescriptionsRepositoryError::InvalidPaginationParams(err.to_string())
        })?;

        let prescriptions_from_db = sqlx::query(
            r#"
        SELECT 
            prescriptions.id, 
            prescriptions.code,
            prescriptions.prescription_type, 
            prescriptions.start_date, 
            prescriptions.end_date, 
            prescriptions.created_at,
            prescriptions.updated_at,
            doctors.id,
            doctors.name,
            doctors.pesel_number,
            doctors.pwz_number,
            patients.id,
            patients.name,
            patients.pesel_number,
            prescribed_drugs.id, 
            prescribed_drugs.drug_id, 
            prescribed_drugs.quantity,
            prescribed_drugs.created_at,
            prescribed_drugs.updated_at,
            prescription_fills.id,
            prescription_fills.pharmacist_id,
            prescription_fills.created_at,
            prescription_fills.updated_at
        FROM (
            SELECT * FROM prescriptions
            ORDER BY created_at ASC
            LIMIT $1 OFFSET $2
        ) AS prescriptions
        LEFT JOIN prescription_fills ON prescriptions.id = prescription_fills.prescription_id
        INNER JOIN prescribed_drugs ON prescriptions.id = prescribed_drugs.prescription_id
        INNER JOIN doctors ON prescriptions.doctor_id = doctors.id
        INNER JOIN patients ON prescriptions.patient_id = patients.id
    "#,
        )
        .bind(page_size)
        .bind(offset)
        .fetch_all(&self.pool)
        .await
        .map_err(|err| GetPrescriptionsRepositoryError::DatabaseError(err.to_string()))?;

        let mut prescriptions: Vec<Prescription> = vec![];

        for row in prescriptions_from_db {
            let PrescriptionsRow {
                prescription_id,
                prescription_code,
                prescription_prescription_type,
                prescription_start_date,
                prescription_end_date,
                prescription_created_at,
                prescription_updated_at,
                doctor_id,
                doctor_name,
                doctor_pesel_number,
                doctor_pwz_number,
                patient_id,
                patient_name,
                patient_pesel_number,
                prescribed_drug_id,
                prescribed_drug_drug_id,
                prescribed_drug_quantity,
                prescribed_drug_created_at,
                prescribed_drug_updated_at,
                prescription_fill_id,
                prescription_fill_pharmacist_id,
                prescription_fill_created_at,
                prescription_fill_updated_at,
            } = self
                .parse_prescription_row(row)
                .map_err(|err| GetPrescriptionsRepositoryError::DatabaseError(err.to_string()))?;

            let prescription = prescriptions.iter_mut().find(|p| p.id == prescription_id);

            let prescribed_drug = PrescribedDrug {
                id: prescribed_drug_id,
                prescription_id,
                drug_id: prescribed_drug_drug_id,
                quantity: prescribed_drug_quantity,
                created_at: prescribed_drug_created_at,
                updated_at: prescribed_drug_updated_at,
            };

            if let Some(prescription) = prescription {
                prescription.prescribed_drugs.push(prescribed_drug);
            } else {
                let fill = if let Some(prescription_fill_id) = prescription_fill_id {
                    Some(PrescriptionFill {
                        id: prescription_fill_id,
                        prescription_id,
                        pharmacist_id: prescription_fill_pharmacist_id.unwrap(),
                        created_at: prescription_fill_created_at.unwrap(),
                        updated_at: prescription_fill_updated_at.unwrap(),
                    })
                } else {
                    None
                };

                prescriptions.push(Prescription {
                    id: prescription_id,
                    patient: PrescriptionPatient {
                        id: patient_id,
                        name: patient_name,
                        pesel_number: patient_pesel_number,
                    },
                    doctor: PrescriptionDoctor {
                        id: doctor_id,
                        name: doctor_name,
                        pesel_number: doctor_pesel_number,
                        pwz_number: doctor_pwz_number,
                    },
                    code: prescription_code,
                    prescription_type: prescription_prescription_type,
                    start_date: prescription_start_date,
                    end_date: prescription_end_date,
                    prescribed_drugs: vec![prescribed_drug],
                    fill,
                    created_at: prescription_created_at,
                    updated_at: prescription_updated_at,
                });
            }
        }

        Ok(prescriptions)
    }

    async fn get_prescription_by_id(
        &self,
        id: Uuid,
    ) -> Result<Prescription, GetPrescriptionByIdRepositoryError> {
        let prescription_from_db = sqlx::query(
            r#"
        SELECT
            prescriptions.id, 
            prescriptions.code,
            prescriptions.prescription_type, 
            prescriptions.start_date, 
            prescriptions.end_date, 
            prescriptions.created_at,
            prescriptions.updated_at,
            doctors.id,
            doctors.name,
            doctors.pesel_number,
            doctors.pwz_number,
            patients.id,
            patients.name,
            patients.pesel_number,
            prescribed_drugs.id, 
            prescribed_drugs.drug_id, 
            prescribed_drugs.quantity,
            prescribed_drugs.created_at,
            prescribed_drugs.updated_at,
            prescription_fills.id,
            prescription_fills.pharmacist_id,
            prescription_fills.created_at,
            prescription_fills.updated_at
        FROM (
            SELECT * FROM prescriptions
            WHERE id = $1
        ) AS prescriptions
        LEFT JOIN prescription_fills ON prescriptions.id = prescription_fills.prescription_id
        INNER JOIN prescribed_drugs ON prescriptions.id = prescribed_drugs.prescription_id
        INNER JOIN doctors ON prescriptions.doctor_id = doctors.id
        INNER JOIN patients ON prescriptions.patient_id = patients.id
    "#,
        )
        .bind(id)
        .fetch_all(&self.pool)
        .await
        .map_err(|err| match err {
            sqlx::Error::RowNotFound => GetPrescriptionByIdRepositoryError::NotFound(id),
            _ => GetPrescriptionByIdRepositoryError::DatabaseError(err.to_string()),
        })?;

        let mut prescriptions: Vec<Prescription> = vec![];

        for row in prescription_from_db {
            let PrescriptionsRow {
                prescription_id,
                prescription_code,
                prescription_prescription_type,
                prescription_start_date,
                prescription_end_date,
                prescription_created_at,
                prescription_updated_at,
                doctor_id,
                doctor_name,
                doctor_pesel_number,
                doctor_pwz_number,
                patient_id,
                patient_name,
                patient_pesel_number,
                prescribed_drug_id,
                prescribed_drug_drug_id,
                prescribed_drug_quantity,
                prescribed_drug_created_at,
                prescribed_drug_updated_at,
                prescription_fill_id,
                prescription_fill_pharmacist_id,
                prescription_fill_created_at,
                prescription_fill_updated_at,
            } = self.parse_prescription_row(row).map_err(|err| {
                GetPrescriptionByIdRepositoryError::DatabaseError(err.to_string())
            })?;

            let prescription = prescriptions.iter_mut().find(|p| p.id == prescription_id);

            let prescribed_drug = PrescribedDrug {
                id: prescribed_drug_id,
                prescription_id,
                drug_id: prescribed_drug_drug_id,
                quantity: prescribed_drug_quantity,
                created_at: prescribed_drug_created_at,
                updated_at: prescribed_drug_updated_at,
            };

            if let Some(prescription) = prescription {
                prescription.prescribed_drugs.push(prescribed_drug);
            } else {
                let fill = if let Some(prescription_fill_id) = prescription_fill_id {
                    Some(PrescriptionFill {
                        id: prescription_fill_id,
                        prescription_id,
                        pharmacist_id: prescription_fill_pharmacist_id.unwrap(),
                        created_at: prescription_fill_created_at.unwrap(),
                        updated_at: prescription_fill_updated_at.unwrap(),
                    })
                } else {
                    None
                };

                prescriptions.push(Prescription {
                    id: prescription_id,
                    patient: PrescriptionPatient {
                        id: patient_id,
                        name: patient_name,
                        pesel_number: patient_pesel_number,
                    },
                    doctor: PrescriptionDoctor {
                        id: doctor_id,
                        name: doctor_name,
                        pesel_number: doctor_pesel_number,
                        pwz_number: doctor_pwz_number,
                    },
                    code: prescription_code,
                    prescription_type: prescription_prescription_type,
                    start_date: prescription_start_date,
                    end_date: prescription_end_date,
                    prescribed_drugs: vec![prescribed_drug],
                    fill,
                    created_at: prescription_created_at,
                    updated_at: prescription_updated_at,
                });
            }
        }

        let prescription = prescriptions
            .first()
            .ok_or(GetPrescriptionByIdRepositoryError::NotFound(id))?
            .to_owned();

        Ok(prescription)
    }

    async fn fill_prescription(
        &self,
        prescription_fill: NewPrescriptionFill,
    ) -> Result<PrescriptionFill, FillPrescriptionRepositoryError> {
        let result = sqlx::query!(
            r#"INSERT INTO prescription_fills (id, prescription_id, pharmacist_id) VALUES ($1, $2, $3) RETURNING id, prescription_id, pharmacist_id, created_at, updated_at"#,
            prescription_fill.id,
            prescription_fill.prescription_id,
            prescription_fill.pharmacist_id
        )
        .fetch_one(&self.pool)
        .await.map_err(|err| {
            match err {
                sqlx::Error::Database(err) if err.message().contains("insert or update on table \"prescription_fills\" violates foreign key constraint \"prescription_fills_pharmacist_id_fkey\"") => {
                    FillPrescriptionRepositoryError::PharmacistNotFound(prescription_fill.pharmacist_id)
                },
                err => FillPrescriptionRepositoryError::DatabaseError(err.to_string()),
            }
        })?;

        Ok(PrescriptionFill {
            id: result.id,
            prescription_id: result.prescription_id,
            pharmacist_id: result.pharmacist_id,
            created_at: result.created_at,
            updated_at: result.updated_at,
        })
    }
}

#[cfg(test)]
mod tests {
    use std::assert_matches::assert_matches;

    use uuid::Uuid;

    use super::PostgresPrescriptionsRepository;
    use crate::{
        domain::{
            doctors::{models::NewDoctor, repository::DoctorsRepository},
            drugs::{
                models::{DrugContentType, NewDrug},
                repository::DrugsRepository,
            },
            patients::{models::NewPatient, repository::PatientsRepository},
            pharmacists::{models::NewPharmacist, repository::PharmacistsRepository},
            prescriptions::{
                models::{NewPrescribedDrug, NewPrescription},
                repository::{
                    CreatePrescriptionRepositoryError, FillPrescriptionRepositoryError,
                    GetPrescriptionByIdRepositoryError, GetPrescriptionsRepositoryError,
                    PrescriptionsRepository,
                },
            },
        },
        infrastructure::postgres_repository_impl::{
            create_tables::create_tables, doctors::PostgresDoctorsRepository,
            drugs::PostgresDrugsRepository, patients::PostgresPatientsRepository,
            pharmacists::PostgresPharmacistsRepository,
        },
    };

    struct DatabaseSeedData {
        doctor: NewDoctor,
        pharmacist: NewPharmacist,
        patient: NewPatient,
        drugs: Vec<NewDrug>,
    }

    async fn seed_database(pool: sqlx::PgPool) -> DatabaseSeedData {
        let pharmacists_repo = PostgresPharmacistsRepository::new(pool.clone());
        let pharmacist = NewPharmacist::new(
            "John Pharmacist".into(), //
            "96021807250".into(),
        )
        .unwrap();
        pharmacists_repo
            .create_pharmacist(pharmacist.clone())
            .await
            .unwrap();

        let patients_repo = PostgresPatientsRepository::new(pool.clone());
        let patient = NewPatient::new(
            "John Patient".into(), //
            "96021807250".into(),
        )
        .unwrap();
        patients_repo.create_patient(patient.clone()).await.unwrap();
        let drugs_repo = PostgresDrugsRepository::new(pool.clone());
        let mut drugs = vec![];
        for _ in 0..4 {
            let drug = NewDrug::new(
                "Gripex".into(),
                DrugContentType::SolidPills,
                Some(20),
                Some(300),
                None,
                None,
            )
            .unwrap();
            drugs.push(drug.clone());
            drugs_repo.create_drug(drug).await.unwrap();
        }

        let doctors_repo = PostgresDoctorsRepository::new(pool);
        let doctor = NewDoctor::new(
            "John Doctor".into(), //
            "3123456".into(),
            "96021807250".into(),
        )
        .unwrap();
        doctors_repo.create_doctor(doctor.clone()).await.unwrap();

        DatabaseSeedData {
            doctor,
            pharmacist,
            patient,
            drugs,
        }
    }

    async fn setup_repository(
        pool: sqlx::PgPool,
    ) -> (PostgresPrescriptionsRepository, DatabaseSeedData) {
        create_tables(&pool, true).await.unwrap();
        let seeds = seed_database(pool.clone()).await;
        let repository = PostgresPrescriptionsRepository::new(pool);
        (repository, seeds)
    }

    #[sqlx::test]
    async fn creates_and_reads_prescription_by_id(pool: sqlx::PgPool) {
        let (repository, seeds) = setup_repository(pool).await;

        let new_prescription = NewPrescription::new(
            seeds.doctor.id,
            seeds.patient.id,
            None,
            None,
            vec![
                NewPrescribedDrug {
                    drug_id: seeds.drugs[0].id,
                    quantity: 1,
                },
                NewPrescribedDrug {
                    drug_id: seeds.drugs[1].id,
                    quantity: 1,
                },
            ],
        )
        .unwrap();

        repository
            .create_prescription(new_prescription.clone())
            .await
            .unwrap();

        let prescription_from_db = repository
            .get_prescription_by_id(new_prescription.id)
            .await
            .unwrap();

        assert_eq!(prescription_from_db, new_prescription);
    }

    #[sqlx::test]
    async fn doesnt_create_prescription_if_relations_dont_exist(pool: sqlx::PgPool) {
        let (repository, seeds) = setup_repository(pool).await;

        let nonexistent_doctor_id = Uuid::new_v4();
        let new_prescription_with_nonexisting_doctor_id = NewPrescription::new(
            nonexistent_doctor_id,
            seeds.patient.id,
            None,
            None,
            vec![NewPrescribedDrug {
                drug_id: seeds.drugs[0].id,
                quantity: 1,
            }],
        )
        .unwrap();

        assert_eq!(
            repository
                .create_prescription(new_prescription_with_nonexisting_doctor_id)
                .await,
            Err(CreatePrescriptionRepositoryError::DoctorNotFound(
                nonexistent_doctor_id
            ))
        );

        let nonexistent_patient_id = Uuid::new_v4();
        let new_prescription_with_nonexisting_patient_id = NewPrescription::new(
            seeds.patient.id,
            nonexistent_patient_id,
            None,
            None,
            vec![NewPrescribedDrug {
                drug_id: seeds.drugs[0].id,
                quantity: 1,
            }],
        )
        .unwrap();

        assert_eq!(
            repository
                .create_prescription(new_prescription_with_nonexisting_patient_id)
                .await,
            Err(CreatePrescriptionRepositoryError::PatientNotFound(
                nonexistent_patient_id
            ))
        );

        let nonexistent_drug_id = Uuid::new_v4();
        let new_prescription_with_nonexisting_drug_id = NewPrescription::new(
            seeds.doctor.id,
            seeds.patient.id,
            None,
            None,
            vec![NewPrescribedDrug {
                drug_id: nonexistent_drug_id,
                quantity: 1,
            }],
        )
        .unwrap();

        assert_eq!(
            repository
                .create_prescription(new_prescription_with_nonexisting_drug_id)
                .await,
            Err(CreatePrescriptionRepositoryError::DrugNotFound(
                nonexistent_drug_id
            ))
        );
    }

    #[sqlx::test]
    async fn get_prescription_by_id_returns_error_if_prescription_doesnt_exist(pool: sqlx::PgPool) {
        let (repository, _) = setup_repository(pool).await;
        let prescription_id = Uuid::new_v4();

        let prescription_from_db = repository.get_prescription_by_id(prescription_id).await;

        assert_eq!(
            prescription_from_db,
            Err(GetPrescriptionByIdRepositoryError::NotFound(
                prescription_id
            ))
        );
    }

    #[sqlx::test]
    async fn creates_and_reads_prescriptions_from_database(pool: sqlx::PgPool) {
        let (repository, seeds) = setup_repository(pool).await;

        let new_prescription = NewPrescription::new(
            seeds.doctor.id,
            seeds.patient.id,
            None,
            None,
            vec![
                NewPrescribedDrug {
                    drug_id: seeds.drugs[0].id,
                    quantity: 1,
                },
                NewPrescribedDrug {
                    drug_id: seeds.drugs[1].id,
                    quantity: 1,
                },
                NewPrescribedDrug {
                    drug_id: seeds.drugs[2].id,
                    quantity: 1,
                },
                NewPrescribedDrug {
                    drug_id: seeds.drugs[3].id,
                    quantity: 1,
                },
            ],
        )
        .unwrap();

        repository
            .create_prescription(new_prescription.clone())
            .await
            .unwrap();

        for _ in 0..10 {
            let another_prescription = NewPrescription::new(
                seeds.doctor.id,
                seeds.patient.id,
                None,
                None,
                vec![NewPrescribedDrug {
                    drug_id: seeds.drugs[0].id,
                    quantity: 1,
                }],
            )
            .unwrap();
            repository
                .create_prescription(another_prescription)
                .await
                .unwrap();
        }

        let prescriptions = repository.get_prescriptions(None, Some(7)).await.unwrap();

        assert_eq!(prescriptions.len(), 7);
        assert_eq!(prescriptions[0], new_prescription);

        let prescriptions = repository.get_prescriptions(None, Some(20)).await.unwrap();
        assert_eq!(prescriptions.len(), 11);

        let prescriptions = repository
            .get_prescriptions(Some(1), Some(10))
            .await
            .unwrap();
        assert_eq!(prescriptions.len(), 1);
    }

    #[sqlx::test]
    async fn get_prescriptions_returns_error_if_pagination_params_are_incorrect(
        pool: sqlx::PgPool,
    ) {
        let (repository, _) = setup_repository(pool).await;

        assert_matches!(
            repository.get_prescriptions(Some(-1), Some(10)).await,
            Err(GetPrescriptionsRepositoryError::InvalidPaginationParams(_))
        );

        assert_matches!(
            repository.get_prescriptions(Some(0), Some(0)).await,
            Err(GetPrescriptionsRepositoryError::InvalidPaginationParams(_))
        );
    }

    #[sqlx::test]
    async fn fills_prescription_and_saves_to_database(pool: sqlx::PgPool) {
        let (repository, seeds) = setup_repository(pool).await;

        let prescription = NewPrescription::new(
            seeds.doctor.id,
            seeds.patient.id,
            None,
            None,
            vec![NewPrescribedDrug {
                drug_id: seeds.drugs[0].id,
                quantity: 1,
            }],
        )
        .unwrap();

        repository
            .create_prescription(prescription.clone())
            .await
            .unwrap();

        let prescription_from_db = repository
            .get_prescription_by_id(prescription.id)
            .await
            .unwrap();

        assert!(prescription_from_db.fill.is_none());

        let new_prescription_fill = prescription_from_db.fill(seeds.pharmacist.id).unwrap();
        let created_prescription_fill = repository
            .fill_prescription(new_prescription_fill.clone())
            .await
            .unwrap();

        assert_eq!(created_prescription_fill, new_prescription_fill);

        let prescription_from_db = repository
            .get_prescription_by_id(prescription.id)
            .await
            .unwrap();

        assert_eq!(prescription_from_db.fill.unwrap(), new_prescription_fill);
    }

    #[sqlx::test]
    async fn doesnt_fill_if_pharmacist_relation_doesnt_exist(pool: sqlx::PgPool) {
        let (repository, seeds) = setup_repository(pool).await;

        let nonexistent_pharmacist_id = Uuid::new_v4();
        let new_prescription = NewPrescription::new(
            seeds.doctor.id,
            seeds.patient.id,
            None,
            None,
            vec![
                NewPrescribedDrug {
                    drug_id: seeds.drugs[0].id,
                    quantity: 1,
                },
                NewPrescribedDrug {
                    drug_id: seeds.drugs[1].id,
                    quantity: 1,
                },
            ],
        )
        .unwrap();

        let prescription_from_db = repository
            .create_prescription(new_prescription.clone())
            .await
            .unwrap();

        let new_prescription_fill_with_nonexistent_pharmacist_id = prescription_from_db
            .fill(nonexistent_pharmacist_id)
            .unwrap();

        assert_eq!(
            repository
                .fill_prescription(new_prescription_fill_with_nonexistent_pharmacist_id)
                .await,
            Err(FillPrescriptionRepositoryError::PharmacistNotFound(
                nonexistent_pharmacist_id
            ))
        );
    }
}
