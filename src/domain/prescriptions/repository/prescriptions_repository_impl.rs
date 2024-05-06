use async_trait::async_trait;
use uuid::Uuid;

use crate::domain::prescriptions::{
    models::{NewPrescription, NewPrescriptionFill, PrescribedDrug, Prescription},
    use_cases::fill_prescription,
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

#[derive(thiserror::Error, Debug)]
enum PaginationError {
    #[error("Invalid page or page_size: page must be at least 0 and page_size must be at least 1")]
    InvalidPageOrPageSize,
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
        page: Option<i16>,
        page_size: Option<i16>,
    ) -> anyhow::Result<Vec<Prescription>> {
        let page = page.unwrap_or(0);
        let page_size = page_size.unwrap_or(10);
        if page_size < 1 || page < 0 {
            Err(PaginationError::InvalidPageOrPageSize)?;
        }
        let offset = page * page_size;

        let prescriptions_from_db = sqlx::query_as(
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
            prescribed_drugs.updated_at
        FROM (
            SELECT * FROM prescriptions
            ORDER BY created_at ASC
            LIMIT $1 OFFSET $2
        ) AS prescriptions
        JOIN prescribed_drugs ON prescriptions.id = prescribed_drugs.prescription_id
    "#,
        )
        .bind(page_size)
        .bind(offset)
        .fetch_all(self.pool)
        .await?;

        let mut prescriptions: Vec<Prescription> = vec![];

        for (
            prescription_id,
            patient_id,
            doctor_id,
            prescription_type,
            start_date,
            end_date,
            prescription_created_at,
            prescription_updated_at,
            prescribed_drug_id,
            drug_id,
            quantity,
            prescribed_drug_created_at,
            prescribed_drug_updated_at,
        ) in prescriptions_from_db
        {
            let prescription = prescriptions.iter_mut().find(|p| p.id == prescription_id);

            let prescribed_drug = PrescribedDrug {
                id: prescribed_drug_id,
                prescription_id,
                drug_id,
                quantity,
                created_at: prescribed_drug_created_at,
                updated_at: prescribed_drug_updated_at,
            };

            if let Some(prescription) = prescription {
                prescription.prescribed_drugs.push(prescribed_drug);
            } else {
                prescriptions.push(Prescription {
                    id: prescription_id,
                    patient_id,
                    doctor_id,
                    prescription_type,
                    start_date,
                    end_date,
                    prescribed_drugs: vec![prescribed_drug],
                    fill: None,
                    created_at: prescription_created_at,
                    updated_at: prescription_updated_at,
                });
            }
        }

        Ok(prescriptions)
    }

    async fn get_prescription_by_id(&self, id: Uuid) -> anyhow::Result<Prescription> {
        let prescription_from_db = sqlx::query_as(
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
            prescribed_drugs.updated_at
        FROM (
            SELECT * FROM prescriptions
            WHERE id = $1
        ) AS prescriptions
        JOIN prescribed_drugs ON prescriptions.id = prescribed_drugs.prescription_id
    "#,
        )
        .bind(id)
        .fetch_all(self.pool)
        .await?;

        let mut prescriptions: Vec<Prescription> = vec![];

        for (
            prescription_id,
            patient_id,
            doctor_id,
            prescription_type,
            start_date,
            end_date,
            prescription_created_at,
            prescription_updated_at,
            prescribed_drug_id,
            drug_id,
            quantity,
            prescribed_drug_created_at,
            prescribed_drug_updated_at,
        ) in prescription_from_db
        {
            let prescription = prescriptions.iter_mut().find(|p| p.id == prescription_id);

            let prescribed_drug = PrescribedDrug {
                id: prescribed_drug_id,
                prescription_id,
                drug_id,
                quantity,
                created_at: prescribed_drug_created_at,
                updated_at: prescribed_drug_updated_at,
            };

            if let Some(prescription) = prescription {
                prescription.prescribed_drugs.push(prescribed_drug);
            } else {
                prescriptions.push(Prescription {
                    id: prescription_id,
                    patient_id,
                    doctor_id,
                    prescription_type,
                    start_date,
                    end_date,
                    prescribed_drugs: vec![prescribed_drug],
                    fill: None,
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
    use chrono::{Duration, Utc};
    use uuid::Uuid;

    use super::{GetPrescriptionError, PrescriptionsRepository};
    use crate::{
        create_tables::create_tables,
        domain::prescriptions::{
            models::{NewPrescription, PrescriptionType},
            repository::prescriptions_repository_trait::PrescriptionsRepositoryTrait,
        },
    };

    #[sqlx::test]
    async fn create_and_read_prescriptions_from_database(pool: sqlx::PgPool) {
        create_tables(&pool, true).await.unwrap();
        let repo = PrescriptionsRepository::new(&pool);

        let doctor_id = Uuid::new_v4();
        let patient_id = Uuid::new_v4();
        let prescription_type = PrescriptionType::ForAntibiotics;
        let start_date = Utc::now() + Duration::days(21) + Duration::hours(37);
        let end_date = start_date + Duration::days(7);
        let drug_id = Uuid::new_v4();
        let drug_quantity = 2;

        let mut prescription = NewPrescription::new(
            doctor_id,
            patient_id,
            Some(start_date),
            Some(prescription_type),
        );
        prescription.add_drug(drug_id, drug_quantity).unwrap();
        prescription.add_drug(Uuid::new_v4(), 1).unwrap(); // Add another drugs, this time we wont check their ids
        prescription.add_drug(Uuid::new_v4(), 1).unwrap();
        prescription.add_drug(Uuid::new_v4(), 1).unwrap();

        repo.create_prescription(prescription.clone())
            .await
            .unwrap();

        for _ in 0..10 {
            let mut another_prescription =
                NewPrescription::new(Uuid::new_v4(), Uuid::new_v4(), None, None); // Fields of this prescription also wont be checked
            another_prescription.add_drug(Uuid::new_v4(), 1).unwrap();
            repo.create_prescription(another_prescription)
                .await
                .unwrap();
        }

        let prescriptions = repo.get_prescriptions(None, Some(7)).await.unwrap();

        assert!(prescriptions.len() == 7);

        let first_prescription = prescriptions.first().unwrap();
        assert_eq!(prescription.doctor_id, doctor_id);
        assert_eq!(prescription.patient_id, patient_id);
        assert_eq!(prescription.start_date, start_date);
        assert_eq!(prescription.end_date, end_date);
        assert_eq!(prescription.prescription_type, prescription_type);
        assert_eq!(prescription.prescribed_drugs.len(), 4);
        let first_prescribed_drug = first_prescription.prescribed_drugs.first().unwrap();
        assert_eq!(first_prescribed_drug.drug_id, drug_id);
        assert_eq!(first_prescribed_drug.quantity, drug_quantity as i32);

        let prescriptions = repo.get_prescriptions(None, Some(20)).await.unwrap();

        assert!(prescriptions.len() == 11);
    }

    #[sqlx::test]
    async fn create_and_read_prescription_by_id(pool: sqlx::PgPool) {
        create_tables(&pool, true).await.unwrap();
        let repo = PrescriptionsRepository::new(&pool);

        let doctor_id = Uuid::new_v4();
        let patient_id = Uuid::new_v4();
        let prescription_type = PrescriptionType::ForChronicDiseaseDrugs;
        let start_date = Utc::now() + Duration::days(21) + Duration::hours(37);
        let end_date = start_date + Duration::days(365);
        let drug_id = Uuid::new_v4();
        let drug_quantity = 2;

        let mut prescription = NewPrescription::new(
            doctor_id,
            patient_id,
            Some(start_date),
            Some(prescription_type),
        );
        prescription.add_drug(drug_id, drug_quantity).unwrap();

        repo.create_prescription(prescription.clone())
            .await
            .unwrap();

        let prescription_from_db = repo.get_prescription_by_id(prescription.id).await.unwrap();

        assert_eq!(prescription_from_db.doctor_id, doctor_id);
        assert_eq!(prescription_from_db.patient_id, patient_id);
        assert_eq!(prescription_from_db.start_date, start_date);
        assert_eq!(prescription_from_db.end_date, end_date);
        assert_eq!(prescription_from_db.prescription_type, prescription_type);
        assert_eq!(prescription_from_db.prescribed_drugs.len(), 1);
        let first_prescribed_drug = prescription_from_db.prescribed_drugs.first().unwrap();
        assert_eq!(first_prescribed_drug.drug_id, drug_id);
        assert_eq!(first_prescribed_drug.quantity, drug_quantity as i32);
    }

    #[sqlx::test]
    async fn returns_error_if_prescription_doesnt_exist(pool: sqlx::PgPool) {
        create_tables(&pool, true).await.unwrap();
        let repo = PrescriptionsRepository::new(&pool);
        let prescription_id = Uuid::new_v4();

        let prescription_from_db = repo.get_prescription_by_id(prescription_id).await;

        assert_eq!(
            prescription_from_db.unwrap_err().downcast_ref(),
            Some(&GetPrescriptionError::NotFound(prescription_id)),
        );
    }

    #[sqlx::test]
    async fn fills_prescription_and_saves_to_database(pool: sqlx::PgPool) {
        create_tables(&pool, true).await.unwrap();
        let repo = PrescriptionsRepository::new(&pool);

        let doctor_id = Uuid::new_v4();
        let patient_id = Uuid::new_v4();
        let prescription_type = PrescriptionType::ForChronicDiseaseDrugs;
        let start_date = Utc::now() - Duration::days(1);
        let drug_id = Uuid::new_v4();
        let drug_quantity = 2;

        let mut prescription = NewPrescription::new(
            doctor_id,
            patient_id,
            Some(start_date),
            Some(prescription_type),
        );
        prescription.add_drug(drug_id, drug_quantity).unwrap();

        repo.create_prescription(prescription.clone())
            .await
            .unwrap();

        let prescription_from_db = repo.get_prescription_by_id(prescription.id).await.unwrap();

        assert!(prescription_from_db.fill.is_none());

        let prescription_fill = prescription_from_db.fill(Uuid::new_v4()).unwrap();
        repo.fill_prescription(prescription_fill).await.unwrap();

        let prescription_from_db = repo.get_prescription_by_id(prescription.id).await.unwrap();

        assert!(prescription_from_db.fill.is_some());
    }
}
