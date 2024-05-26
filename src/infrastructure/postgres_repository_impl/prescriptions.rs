use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::Row;
use uuid::Uuid;

use crate::domain::{
    prescriptions::{
        models::{
            NewPrescription, NewPrescriptionFill, PrescribedDrug, Prescription, PrescriptionDoctor,
            PrescriptionFill, PrescriptionPatient, PrescriptionType,
        },
        repository::PrescriptionsRepository,
    },
    utils::pagination::get_pagination_params,
};

pub struct PostgresPrescriptionsRepository {
    pool: sqlx::PgPool,
}

impl PostgresPrescriptionsRepository {
    pub fn new(pool: sqlx::PgPool) -> Self {
        Self { pool }
    }
}

#[derive(thiserror::Error, Debug, PartialEq)]
pub enum GetPrescriptionError {
    #[error("Prescription with id {0} not found")]
    NotFound(Uuid),
}

#[async_trait]
impl PrescriptionsRepository for PostgresPrescriptionsRepository {
    async fn create_prescription(
        &self,
        prescription: NewPrescription,
    ) -> anyhow::Result<Prescription> {
        prescription.validate()?; // TODO: move this to domain (for instance: remove add_drug method from NewPrescription)

        let transaction = self.pool.begin().await?;

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
        .await?;

        for prescribed_drug in &prescription.prescribed_drugs {
            sqlx::query!(
                r#"INSERT INTO prescribed_drugs (id, prescription_id, drug_id, quantity) VALUES ($1, $2, $3, $4)"#,
                Uuid::new_v4(),
                prescription.id,
                prescribed_drug.drug_id,
                prescribed_drug.quantity as i32
            )
            .execute(&self.pool)
            .await?;
        }

        let prescription = self.get_prescription_by_id(prescription.id).await?;

        transaction.commit().await?;

        Ok(prescription)
    }

    async fn get_prescriptions(
        &self,
        page: Option<i64>,
        page_size: Option<i64>,
    ) -> anyhow::Result<Vec<Prescription>> {
        let (page_size, offset) = get_pagination_params(page, page_size)?;

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
        .await?;

        let mut prescriptions: Vec<Prescription> = vec![];

        for row in prescriptions_from_db {
            let prescription_id: Uuid = row.try_get(0)?;
            let prescription_code: String = row.try_get(1)?;
            let prescription_prescription_type: PrescriptionType = row.try_get(2)?;
            let prescription_start_date: DateTime<Utc> = row.try_get(3)?;
            let prescription_end_date: DateTime<Utc> = row.try_get(4)?;
            let prescription_created_at: DateTime<Utc> = row.try_get(5)?;
            let prescription_updated_at: DateTime<Utc> = row.try_get(6)?;
            let doctor_id: Uuid = row.try_get(7)?;
            let doctor_name: String = row.try_get(8)?;
            let doctor_pesel_number: String = row.try_get(9)?;
            let doctor_pwz_number: String = row.try_get(10)?;
            let patient_id: Uuid = row.try_get(11)?;
            let patient_name: String = row.try_get(12)?;
            let patient_pesel_number: String = row.try_get(13)?;
            let prescribed_drug_id: Uuid = row.try_get(14)?;
            let prescribed_drug_drug_id: Uuid = row.try_get(15)?;
            let prescribed_drug_quantity: i32 = row.try_get(16)?;
            let prescribed_drug_created_at: DateTime<Utc> = row.try_get(17)?;
            let prescribed_drug_updated_at: DateTime<Utc> = row.try_get(18)?;
            let prescription_fill_id: Option<Uuid> = row.try_get(19)?;
            let prescription_fill_pharmacist_id: Option<Uuid> = row.try_get(20)?;
            let prescription_fill_created_at: Option<DateTime<Utc>> = row.try_get(21)?;
            let prescription_fill_updated_at: Option<DateTime<Utc>> = row.try_get(22)?;

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

    async fn get_prescription_by_id(&self, id: Uuid) -> anyhow::Result<Prescription> {
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
        .await?;

        let mut prescriptions: Vec<Prescription> = vec![];

        for row in prescription_from_db {
            let prescription_id: Uuid = row.try_get(0)?;
            let prescription_code: String = row.try_get(1)?;
            let prescription_prescription_type: PrescriptionType = row.try_get(2)?;
            let prescription_start_date: DateTime<Utc> = row.try_get(3)?;
            let prescription_end_date: DateTime<Utc> = row.try_get(4)?;
            let prescription_created_at: DateTime<Utc> = row.try_get(5)?;
            let prescription_updated_at: DateTime<Utc> = row.try_get(6)?;
            let doctor_id: Uuid = row.try_get(7)?;
            let doctor_name: String = row.try_get(8)?;
            let doctor_pesel_number: String = row.try_get(9)?;
            let doctor_pwz_number: String = row.try_get(10)?;
            let patient_id: Uuid = row.try_get(11)?;
            let patient_name: String = row.try_get(12)?;
            let patient_pesel_number: String = row.try_get(13)?;
            let prescribed_drug_id: Uuid = row.try_get(14)?;
            let prescribed_drug_drug_id: Uuid = row.try_get(15)?;
            let prescribed_drug_quantity: i32 = row.try_get(16)?;
            let prescribed_drug_created_at: DateTime<Utc> = row.try_get(17)?;
            let prescribed_drug_updated_at: DateTime<Utc> = row.try_get(18)?;
            let prescription_fill_id: Option<Uuid> = row.try_get(19)?;
            let prescription_fill_pharmacist_id: Option<Uuid> = row.try_get(20)?;
            let prescription_fill_created_at: Option<DateTime<Utc>> = row.try_get(21)?;
            let prescription_fill_updated_at: Option<DateTime<Utc>> = row.try_get(22)?;

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
            .ok_or(GetPrescriptionError::NotFound(id))?;
        Ok(prescription.clone())
    }

    async fn fill_prescription(
        &self,
        prescription_fill: NewPrescriptionFill,
    ) -> anyhow::Result<PrescriptionFill> {
        let result = sqlx::query!(
            r#"INSERT INTO prescription_fills (id, prescription_id, pharmacist_id) VALUES ($1, $2, $3) RETURNING id, prescription_id, pharmacist_id, created_at, updated_at"#,
            prescription_fill.id,
            prescription_fill.prescription_id,
            prescription_fill.pharmacist_id
        )
        .fetch_one(&self.pool)
        .await?;

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
    use uuid::Uuid;

    use super::PostgresPrescriptionsRepository;
    use crate::{
        create_tables::create_tables,
        domain::{
            doctors::{models::NewDoctor, repository::DoctorsRepository},
            drugs::{
                models::{DrugContentType, NewDrug},
                repository::DrugsRepository,
            },
            patients::{models::NewPatient, repository::PatientsRepository},
            pharmacists::{models::NewPharmacist, repository::PharmacistsRepository},
            prescriptions::{models::NewPrescription, repository::PrescriptionsRepository},
        },
        infrastructure::postgres_repository_impl::{
            doctors::PostgresDoctorsRepository, drugs::PostgresDrugsRepository,
            patients::PostgresPatientsRepository, pharmacists::PostgresPharmacistsRepository,
        },
    };

    struct DatabaseSeedData {
        doctor: NewDoctor,
        pharmacist: NewPharmacist,
        patient: NewPatient,
        drugs: Vec<NewDrug>,
    }

    async fn seed_database(pool: sqlx::PgPool) -> anyhow::Result<DatabaseSeedData> {
        let pharmacists_repo = PostgresPharmacistsRepository::new(pool.clone());
        let pharmacist = NewPharmacist::new(
            "John Pharmacist".into(), //
            "96021807250".into(),
        )?;
        pharmacists_repo
            .create_pharmacist(pharmacist.clone())
            .await?;

        let patients_repo = PostgresPatientsRepository::new(pool.clone());
        let patient = NewPatient::new(
            "John Patient".into(), //
            "96021807250".into(),
        )?;
        patients_repo.create_patient(patient.clone()).await?;
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
            )?;
            drugs.push(drug.clone());
            drugs_repo.create_drug(drug).await?;
        }

        let doctors_repo = PostgresDoctorsRepository::new(pool);
        let doctor = NewDoctor::new(
            "John Doctor".into(), //
            "3123456".into(),
            "96021807250".into(),
        )?;
        doctors_repo.create_doctor(doctor.clone()).await?;

        Ok(DatabaseSeedData {
            doctor,
            pharmacist,
            patient,
            drugs,
        })
    }

    async fn setup_repository(
        pool: sqlx::PgPool,
    ) -> (PostgresPrescriptionsRepository, DatabaseSeedData) {
        create_tables(&pool, true).await.unwrap();
        let seeds = seed_database(pool.clone()).await.unwrap();
        let repository = PostgresPrescriptionsRepository::new(pool);
        (repository, seeds)
    }

    #[sqlx::test]
    async fn creates_and_reads_prescriptions_from_database(pool: sqlx::PgPool) {
        let (repository, seeds) = setup_repository(pool).await;

        let mut new_prescription =
            NewPrescription::new(seeds.doctor.id, seeds.patient.id, None, None);
        for i in 0..4 {
            new_prescription.add_drug(seeds.drugs[i].id, 1).unwrap();
        }

        repository
            .create_prescription(new_prescription.clone())
            .await
            .unwrap();

        for _ in 0..10 {
            let mut another_prescription =
                NewPrescription::new(seeds.doctor.id, seeds.patient.id, None, None);
            another_prescription
                .add_drug(seeds.drugs[0].id, 1)
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
        assert!(prescriptions.len() == 11);

        let prescriptions = repository
            .get_prescriptions(Some(1), Some(10))
            .await
            .unwrap();
        assert!(prescriptions.len() == 1);
    }

    #[sqlx::test]
    async fn creates_and_reads_prescription_by_id(pool: sqlx::PgPool) {
        let (repository, seeds) = setup_repository(pool).await;

        let mut new_prescription =
            NewPrescription::new(seeds.doctor.id, seeds.patient.id, None, None);
        for i in 0..2 {
            new_prescription.add_drug(seeds.drugs[i].id, 1).unwrap();
        }

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

        let mut new_prescription_with_nonexisting_doctor_id =
            NewPrescription::new(Uuid::new_v4(), seeds.doctor.id, None, None);
        new_prescription_with_nonexisting_doctor_id
            .add_drug(seeds.drugs[0].id, 1)
            .unwrap();

        assert!(repository
            .create_prescription(new_prescription_with_nonexisting_doctor_id)
            .await
            .is_err());

        let mut new_prescription_with_nonexisting_patient_id =
            NewPrescription::new(seeds.patient.id, Uuid::new_v4(), None, None);
        new_prescription_with_nonexisting_patient_id
            .add_drug(seeds.drugs[0].id, 1)
            .unwrap();

        assert!(repository
            .create_prescription(new_prescription_with_nonexisting_patient_id)
            .await
            .is_err());

        let mut new_prescription_with_nonexisting_drug_id =
            NewPrescription::new(seeds.doctor.id, seeds.patient.id, None, None);
        new_prescription_with_nonexisting_drug_id
            .add_drug(Uuid::new_v4(), 1)
            .unwrap();

        assert!(repository
            .create_prescription(new_prescription_with_nonexisting_drug_id)
            .await
            .is_err());
    }

    #[sqlx::test]
    async fn returns_error_if_prescription_doesnt_exist(pool: sqlx::PgPool) {
        let (repository, _) = setup_repository(pool).await;
        let prescription_id = Uuid::new_v4();

        let prescription_from_db = repository.get_prescription_by_id(prescription_id).await;

        assert!(prescription_from_db.is_err(),);
    }

    #[sqlx::test]
    async fn fills_prescription_and_saves_to_database(pool: sqlx::PgPool) {
        let (repository, seeds) = setup_repository(pool).await;

        let mut prescription =
            NewPrescription::new(seeds.doctor.id, seeds.patient.id, None, None);
        prescription.add_drug(seeds.drugs[0].id, 1).unwrap();

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

        let mut new_prescription =
            NewPrescription::new(seeds.doctor.id, seeds.patient.id, None, None);
        for i in 0..2 {
            new_prescription.add_drug(seeds.drugs[i].id, 1).unwrap();
        }
        let prescription_from_db = repository
            .create_prescription(new_prescription.clone())
            .await
            .unwrap();

        let new_prescription_fill_with_nonexistent_pharmacist_id =
            prescription_from_db.fill(Uuid::new_v4()).unwrap();

        assert!(repository
            .fill_prescription(new_prescription_fill_with_nonexistent_pharmacist_id)
            .await
            .is_err());
    }
}
