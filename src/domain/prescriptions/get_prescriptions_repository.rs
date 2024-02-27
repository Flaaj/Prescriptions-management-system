use std::collections::HashMap;

use chrono::{DateTime, Utc};
use sqlx::PgPool;
use uuid::Uuid;

use super::{
    get_prescriptions::{PrescribedDrug, Prescription},
    prescription_type::PrescriptionType,
};

pub struct PrescriptionRepository {}

impl PrescriptionRepository {
    pub async fn get_prescriptions(
        pool: &PgPool,
        page: Option<i16>,
        page_size: Option<i16>,
    ) -> anyhow::Result<Vec<Prescription>> {
        let page = page.unwrap_or(0);
        let page_size = page_size.unwrap_or(10);
        let offset = page * page_size;

        let rows = sqlx::query_as::<
            _,
            (
                Uuid,
                Uuid,
                Uuid,
                PrescriptionType,
                DateTime<Utc>,
                DateTime<Utc>,
                Uuid,
                Uuid,
                i32,
            ),
        >(
            r#"
        SELECT 
            prescriptions.id, 
            prescriptions.patient_id, 
            prescriptions.doctor_id, 
            prescriptions.prescription_type, 
            prescriptions.start_date, 
            prescriptions.end_date, 
            prescribed_drugs.id, 
            prescribed_drugs.drug_id, 
            prescribed_drugs.quantity
        FROM prescriptions
        LEFT JOIN prescribed_drugs ON prescriptions.id = prescribed_drugs.prescription_id
        ORDER BY prescriptions.id
        LIMIT $1 OFFSET $2
    "#,
        )
        .bind(page_size)
        .bind(offset)
        .fetch_all(pool)
        .await?;

        let mut prescriptions: HashMap<Uuid, Prescription> = HashMap::new();

        for row in rows {
            let (
                prescription_id,
                patient_id,
                doctor_id,
                prescription_type,
                start_date,
                end_date,
                prescribed_drug_id,
                drug_id,
                quantity,
            ) = row;

            let prescription =
                prescriptions
                    .entry(prescription_id)
                    .or_insert_with(|| Prescription {
                        id: prescription_id,
                        patient_id,
                        doctor_id,
                        prescription_type,
                        start_date,
                        end_date,
                        prescribed_drugs: vec![],
                    });

            prescription.prescribed_drugs.push(PrescribedDrug {
                id: prescribed_drug_id,
                prescription_id,
                drug_id,
                quantity,
            });
        }
        Ok(prescriptions.into_values().collect())
    }
}
