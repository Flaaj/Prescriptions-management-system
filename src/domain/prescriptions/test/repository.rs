#[cfg(test)]
mod integration_tests {
    use chrono::{Duration, Utc};
    use sqlx::PgPool;
    use uuid::Uuid;

    use crate::{
        create_tables::create_tables,
        domain::prescriptions::{
            create_prescription::NewPrescription,
            get_prescriptions_repository::PrescriptionRepository,
            prescription_type::PrescriptionType,
        },
    };

    #[sqlx::test]
    async fn test_create_prescription(pool: PgPool) -> anyhow::Result<()> {
        create_tables(&pool).await?;

        let doctor_id = Uuid::new_v4();
        let patient_id = Uuid::new_v4();
        let start_date = Utc::now() + Duration::days(3);
        let prescription_type = PrescriptionType::ForChronicDiseaseDrugs;
        let drug_id = Uuid::new_v4();
        let drug_quantity = 2;

        let mut prescription = NewPrescription::new(
            doctor_id,
            patient_id,
            Some(start_date),
            Some(prescription_type),
        );

        prescription.add_drug(drug_id, drug_quantity)?;

        prescription.save_to_database(&pool).await?;

        let prescriptions = PrescriptionRepository::get_prescriptions(&pool, None, None).await?;

        let prescription = prescriptions.first().unwrap();
        assert_eq!(prescription.doctor_id, doctor_id);
        assert_eq!(prescription.patient_id, patient_id);
        assert_eq!(prescription.start_date, start_date);
        assert_eq!(prescription.prescription_type, prescription_type);
        let prescribed_drug = prescription.prescribed_drugs.first().unwrap();
        assert_eq!(prescribed_drug.drug_id, drug_id);
        assert_eq!(prescribed_drug.quantity, drug_quantity as i32);

        Ok(())
    }
}
