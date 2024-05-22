use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::Row;
use uuid::Uuid;

use crate::{
    domain::prescriptions::models::{
        NewPrescription, NewPrescriptionFill, PrescribedDrug, Prescription, PrescriptionFill,
        PrescriptionType,
    },
    utils::pagination::get_pagination_params,
};

use super::prescriptions_repository_trait::PrescriptionsRepositoryTrait;

pub struct PrescriptionsRepository<'a> {
    pool: &'a sqlx::PgPool,
}

impl<'a> PrescriptionsRepository<'a> {
    pub fn new(pool: &'a sqlx::PgPool) -> Self {
        Self { pool }
    }
}

#[derive(thiserror::Error, Debug, PartialEq)]
pub enum GetPrescriptionError {
    #[error("Prescription with id {0} not found")]
    NotFound(Uuid),
}

#[async_trait]
impl<'a> PrescriptionsRepositoryTrait for PrescriptionsRepository<'a> {
    async fn create_prescription(&self, prescription: NewPrescription) -> anyhow::Result<()> {
        prescription.validate()?;

        let transaction = self.pool.begin().await?;

        sqlx::query!(
            r#"INSERT INTO prescriptions (id, patient_id, doctor_id, prescription_type, start_date, end_date) VALUES ($1, $2, $3, $4, $5, $6)"#,
            prescription.id,
            prescription.patient_id,
            prescription.doctor_id,
            prescription.prescription_type as _,
            prescription.start_date,
            prescription.end_date
        )
        .execute(self.pool)
        .await?;

        for prescribed_drug in &prescription.prescribed_drugs {
            sqlx::query!(
                r#"INSERT INTO prescribed_drugs (id, prescription_id, drug_id, quantity) VALUES ($1, $2, $3, $4)"#,
                Uuid::new_v4(),
                prescription.id,
                prescribed_drug.drug_id,
                prescribed_drug.quantity as i32
            )
            .execute(self.pool)
            .await?;
        }

        transaction.commit().await?;

        Ok(())
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
            prescriptions.patient_id, 
            prescriptions.doctor_id, 
            prescriptions.prescription_type, 
            prescriptions.start_date, 
            prescriptions.end_date, 
            prescriptions.created_at,
            prescriptions.updated_at,
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
        JOIN prescribed_drugs ON prescriptions.id = prescribed_drugs.prescription_id
    "#,
        )
        .bind(page_size)
        .bind(offset)
        .fetch_all(self.pool)
        .await?;

        let mut prescriptions: Vec<Prescription> = vec![];

        for row in prescriptions_from_db {
            let prescription_id: Uuid = row.get(0);
            let prescription_patient_id: Uuid = row.get(1);
            let prescription_doctor_id: Uuid = row.get(2);
            let prescription_prescription_type: PrescriptionType = row.get(3);
            let prescription_start_date: DateTime<Utc> = row.get(4);
            let prescription_end_date: DateTime<Utc> = row.get(5);
            let prescription_created_at: DateTime<Utc> = row.get(6);
            let prescription_updated_at: DateTime<Utc> = row.get(7);
            let prescribed_drug_id: Uuid = row.get(8);
            let prescribed_drug_drug_id: Uuid = row.get(9);
            let prescribed_drug_quantity: i32 = row.get(10);
            let prescribed_drug_created_at: DateTime<Utc> = row.get(11);
            let prescribed_drug_updated_at: DateTime<Utc> = row.get(12);
            let prescription_fill_id: Option<Uuid> = row.get(13);
            let prescription_fill_pharmacist_id: Option<Uuid> = row.get(14);
            let prescription_fill_created_at: Option<DateTime<Utc>> = row.get(15);
            let prescription_fill_updated_at: Option<DateTime<Utc>> = row.get(16);

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
                    patient_id: prescription_patient_id,
                    doctor_id: prescription_doctor_id,
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
            prescriptions.patient_id, 
            prescriptions.doctor_id, 
            prescriptions.prescription_type, 
            prescriptions.start_date, 
            prescriptions.end_date, 
            prescriptions.created_at,
            prescriptions.updated_at,
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
        JOIN prescribed_drugs ON prescriptions.id = prescribed_drugs.prescription_id
    "#,
        )
        .bind(id)
        .fetch_all(self.pool)
        .await?;

        let mut prescriptions: Vec<Prescription> = vec![];

        for row in prescription_from_db {
            let prescription_id: Uuid = row.get(0);
            let prescription_patient_id: Uuid = row.get(1);
            let prescription_doctor_id: Uuid = row.get(2);
            let prescription_prescription_type: PrescriptionType = row.get(3);
            let prescription_start_date: DateTime<Utc> = row.get(4);
            let prescription_end_date: DateTime<Utc> = row.get(5);
            let prescription_created_at: DateTime<Utc> = row.get(6);
            let prescription_updated_at: DateTime<Utc> = row.get(7);
            let prescribed_drug_id: Uuid = row.get(8);
            let prescribed_drug_drug_id: Uuid = row.get(9);
            let prescribed_drug_quantity: i32 = row.get(10);
            let prescribed_drug_created_at: DateTime<Utc> = row.get(11);
            let prescribed_drug_updated_at: DateTime<Utc> = row.get(12);
            let prescription_fill_id: Option<Uuid> = row.get(13);
            let prescription_fill_pharmacist_id: Option<Uuid> = row.get(14);
            let prescription_fill_created_at: Option<DateTime<Utc>> = row.get(15);
            let prescription_fill_updated_at: Option<DateTime<Utc>> = row.get(16);

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
                    patient_id: prescription_patient_id,
                    doctor_id: prescription_doctor_id,
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
    ) -> anyhow::Result<()> {
        sqlx::query!(
            r#"INSERT INTO prescription_fills (id, prescription_id, pharmacist_id) VALUES ($1, $2, $3)"#,
            prescription_fill.id,
            prescription_fill.prescription_id,
            prescription_fill.pharmacist_id
        )
        .execute(self.pool)
        .await?;

        Ok(())
    }
}

#[cfg(test)]
mod integration_tests {
    use uuid::Uuid;

    use super::{GetPrescriptionError, PrescriptionsRepository};
    use crate::{
        create_tables::create_tables,
        domain::{
            doctors::{
                models::NewDoctor,
                repository::{
                    doctors_repository_impl::DoctorsRepository,
                    doctors_repository_trait::DoctorsRepositoryTrait,
                },
            },
            drugs::{
                models::{DrugContentType, NewDrug},
                repository::{
                    drugs_repository_impl::DrugsRepository,
                    drugs_repository_trait::DrugsRepositoryTrait,
                },
            },
            patients::{
                models::NewPatient,
                repository::{
                    patients_repository_impl::PatientsRepository,
                    patients_repository_trait::PatientsRepositoryTrait,
                },
            },
            pharmacists::{
                models::NewPharmacist,
                repository::{
                    pharmacists_repository_impl::PharmacistsRepository,
                    pharmacists_repository_trait::PharmacistsRepositoryTrait,
                },
            },
            prescriptions::{
                models::NewPrescription,
                repository::prescriptions_repository_trait::PrescriptionsRepositoryTrait,
            },
        },
    };

    async fn seed_database(pool: &sqlx::PgPool) -> anyhow::Result<(Uuid, Uuid, Uuid, Vec<Uuid>)> {
        let pharmacists_repo = PharmacistsRepository::new(&pool);
        let pharmacist = NewPharmacist::new(
            "John Pharmacist".into(), //
            "96021807250".into(),
        )?;
        pharmacists_repo
            .create_pharmacist(pharmacist.clone())
            .await?;

        let patients_repo = PatientsRepository::new(&pool);
        let patient = NewPatient::new(
            "John Patient".into(), //
            "96021807250".into(),
        )?;
        patients_repo.create_patient(patient.clone()).await?;

        let doctors_repo = DoctorsRepository::new(&pool);
        let doctor = NewDoctor::new(
            "John Doctor".into(), //
            "3123456".into(),
            "96021807250".into(),
        )?;
        doctors_repo.create_doctor(doctor.clone()).await?;

        let drugs_repo = DrugsRepository::new(&pool);
        let mut drug_ids = vec![];
        for _ in 0..4 {
            let drug = NewDrug::new(
                "Gripex".into(),
                DrugContentType::SolidPills,
                Some(20),
                Some(300),
                None,
                None,
            )?;
            drug_ids.push(drug.id);
            drugs_repo.create_drug(drug).await?;
        }

        Ok((doctor.id, pharmacist.id, patient.id, drug_ids))
    }

    #[sqlx::test]
    async fn create_and_read_prescriptions_from_database(pool: sqlx::PgPool) {
        create_tables(&pool, true).await.unwrap();
        let (doctor_id, _, patient_id, drug_ids) = seed_database(&pool).await.unwrap();
        let repository = PrescriptionsRepository::new(&pool);

        let mut prescription = NewPrescription::new(doctor_id, patient_id, None, None);
        for i in 0..4 {
            prescription.add_drug(*drug_ids.get(i).unwrap(), 1).unwrap();
        }

        repository.create_prescription(prescription.clone())
            .await
            .unwrap();

        for _ in 0..10 {
            let mut another_prescription = NewPrescription::new(doctor_id, patient_id, None, None);
            another_prescription
                .add_drug(*drug_ids.get(0).unwrap(), 1)
                .unwrap();
            repository.create_prescription(another_prescription)
                .await
                .unwrap();
        }

        let prescriptions = repository.get_prescriptions(None, Some(7)).await.unwrap();
        assert_eq!(prescriptions.len(), 7);
        assert_eq!(prescriptions.first().unwrap().prescribed_drugs.len(), 4);

        let prescriptions = repository.get_prescriptions(None, Some(20)).await.unwrap();
        assert!(prescriptions.len() == 11);

        let prescriptions = repository.get_prescriptions(Some(1), Some(10)).await.unwrap();
        assert!(prescriptions.len() == 1);
    }

    #[sqlx::test]
    async fn create_and_read_prescription_by_id(pool: sqlx::PgPool) {
        create_tables(&pool, true).await.unwrap();
        let repository = PrescriptionsRepository::new(&pool);
        let (doctor_id, _, patient_id, drug_ids) = seed_database(&pool).await.unwrap();

        let mut prescription = NewPrescription::new(doctor_id, patient_id, None, None);
        for i in 0..2 {
            prescription.add_drug(*drug_ids.get(i).unwrap(), 1).unwrap();
        }

        repository.create_prescription(prescription.clone())
            .await
            .unwrap();

        let prescription_from_db = repository.get_prescription_by_id(prescription.id).await.unwrap();
        assert_eq!(prescription_from_db.id, prescription.id);
        assert_eq!(prescription_from_db.prescribed_drugs.len(), 2);
    }

    #[sqlx::test]
    async fn returns_error_if_prescription_doesnt_exist(pool: sqlx::PgPool) {
        create_tables(&pool, true).await.unwrap();
        let repository = PrescriptionsRepository::new(&pool);
        let prescription_id = Uuid::new_v4();

        let prescription_from_db = repository.get_prescription_by_id(prescription_id).await;

        assert_eq!(
            prescription_from_db.unwrap_err().downcast_ref(),
            Some(&GetPrescriptionError::NotFound(prescription_id)),
        );
    }

    #[sqlx::test]
    async fn fills_prescription_and_saves_to_database(pool: sqlx::PgPool) {
        create_tables(&pool, true).await.unwrap();
        let repository = PrescriptionsRepository::new(&pool);
        let (doctor_id, pharmacist_id, patient_id, drug_ids) = seed_database(&pool).await.unwrap();

        let mut prescription = NewPrescription::new(doctor_id, patient_id, None, None);
        prescription.add_drug(*drug_ids.get(0).unwrap(), 1).unwrap();

        repository.create_prescription(prescription.clone())
            .await
            .unwrap();

        let prescription_from_db = repository.get_prescription_by_id(prescription.id).await.unwrap();

        assert!(prescription_from_db.fill.is_none());

        let prescription_fill = prescription_from_db.fill(pharmacist_id).unwrap();
        repository.fill_prescription(prescription_fill).await.unwrap();

        let prescription_from_db = repository.get_prescription_by_id(prescription.id).await.unwrap();

        assert!(prescription_from_db.fill.is_some());
    }
}
