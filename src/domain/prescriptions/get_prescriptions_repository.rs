use std::borrow::Borrow;

use chrono::{DateTime, Utc};
use sqlx::{PgPool, Row};
use uuid::Uuid;

use super::{
    get_prescriptions::{PrescribedDrug, Prescription},
    prescription_type::PrescriptionType,
};

pub struct PrescriptionRepository {}

// required for `Option<Uuid>` to implement `Into<Uuid>`
trait Into<T> {
    fn into(self) -> T;
}

impl PrescriptionRepository {
    pub async fn get_prescriptions(pool: &PgPool) -> anyhow::Result<Vec<Prescription>> {
        let mut prescriptions = sqlx::query(r#" SELECT * FROM prescriptions"#)
            .fetch_all(pool)
            .await?
            .iter()
            .map(|row| {
                Ok(Prescription {
                    id: row.try_get::<Uuid, &str>("id")?,
                    patient_id: row.try_get::<Uuid, &str>("patient_id")?,
                    doctor_id: row.try_get::<Uuid, &str>("doctor_id")?,
                    prescription_type: row
                        .try_get::<PrescriptionType, &str>("prescription_type")?,
                    start_date: row.try_get::<DateTime<Utc>, &str>("start_date")?,
                    end_date: row.try_get::<DateTime<Utc>, &str>("end_date")?,
                    prescribed_drugs: vec![],
                })
            })
            .collect::<anyhow::Result<Vec<Prescription>>>()?;

        for prescription in &mut prescriptions {
            let prescribed_drugs = sqlx::query!(
                r#"SELECT * FROM prescribed_drugs WHERE prescription_id = $1"#,
                prescription.id,
            )
            .fetch_all(pool)
            .await?
            .iter()
            .map(|row| {
                Ok(PrescribedDrug {
                    id: row.id,
                    prescription_id: row.prescription_id.unwrap(),
                    drug_id: row.drug_id.unwrap(),
                    quantity: row.quantity.unwrap() as i16,
                })
            })
            .collect::<anyhow::Result<Vec<PrescribedDrug>>>()?;

            prescription.prescribed_drugs = prescribed_drugs;
        }

        Ok(prescriptions)
    }
}
