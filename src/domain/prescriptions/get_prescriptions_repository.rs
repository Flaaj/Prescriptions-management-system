use super::get_prescriptions::{PrescribedDrug, Prescription};

pub struct PrescriptionRepository {}

#[derive(thiserror::Error, Debug)]
enum PaginationError {
    #[error("Invalid page or page_size: page must be at least 0 and page_size must be at least 1")]
    InvalidPageOrPageSize,
}

impl PrescriptionRepository {
    pub async fn get_prescriptions(
        pool: &sqlx::PgPool,
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
        .fetch_all(pool)
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
                    created_at: prescription_created_at,
                    updated_at: prescription_updated_at,
                });
            }
        }

        Ok(prescriptions)
    }
}
