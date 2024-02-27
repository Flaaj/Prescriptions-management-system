use std::collections::HashMap;

use sqlx::{PgPool, Row};
use uuid::Uuid;

use super::get_prescriptions::{PrescribedDrug, Prescription};

pub struct PrescriptionRepository {}

impl PrescriptionRepository {
    pub async fn get_prescriptions(pool: &PgPool) -> anyhow::Result<Vec<Prescription>> {
        let rows = sqlx::query(
            r#"
        SELECT 
            prescriptions.*, 
            prescribed_drugs.id AS drug_id, 
            prescribed_drugs.prescription_id, 
            prescribed_drugs.drug_id, 
            prescribed_drugs.quantity
        FROM prescriptions
        LEFT JOIN prescribed_drugs ON prescriptions.id = prescribed_drugs.prescription_id
    "#,
        )
        .fetch_all(pool)
        .await?;

        let mut prescriptions: HashMap<Uuid, Prescription> = HashMap::new();

        for row in rows {
            let prescription_id: Uuid = row.try_get("id")?;
            let default = Prescription {
                id: prescription_id,
                patient_id: row.try_get("patient_id")?,
                doctor_id: row.try_get("doctor_id")?,
                prescription_type: row.try_get("prescription_type")?,
                start_date: row.try_get("start_date")?,
                end_date: row.try_get("end_date")?,
                prescribed_drugs: vec![],
            };
            let prescription = prescriptions
                .entry(prescription_id)
                .or_insert_with(|| default);

            let drug_id: Option<Uuid> = row.try_get("drug_id").ok();
            if let Some(drug_id) = drug_id {
                prescription.prescribed_drugs.push(PrescribedDrug {
                    id: drug_id,
                    prescription_id: row.try_get("prescription_id")?,
                    drug_id: row.try_get("drug_id")?,
                    quantity: row.try_get("quantity")?,
                });
            }
        }

        Ok(prescriptions.values().cloned().collect())
    }
}
