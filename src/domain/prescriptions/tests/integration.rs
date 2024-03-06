#[cfg(test)]
mod integration_tests {
    use chrono::{Duration, Utc};
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
    async fn create_and_read_prescriptions_from_database(pool: sqlx::PgPool) -> anyhow::Result<()> {
        create_tables(&pool, true).await?;

        let doctor_id = Uuid::new_v4();
        let patient_id = Uuid::new_v4();
        let prescription_type = PrescriptionType::Regular;
        let start_date = Utc::now() + Duration::days(21) + Duration::hours(37);
        let end_date = start_date + Duration::days(30);
        let drug_id = Uuid::new_v4();
        let drug_quantity = 2;

        let mut prescription = NewPrescription::new(
            doctor_id,
            patient_id,
            Some(start_date),
            Some(prescription_type),
        );
        prescription.add_drug(drug_id, drug_quantity)?;
        prescription.add_drug(Uuid::new_v4(), 1)?; // Add another drugs, this time we wont check their ids
        prescription.add_drug(Uuid::new_v4(), 1)?;
        prescription.add_drug(Uuid::new_v4(), 1)?;

        prescription.clone().commit_to_repository(&pool).await?;

        for _ in 0..10 {
            let mut another_prescription =
                NewPrescription::new(Uuid::new_v4(), Uuid::new_v4(), None, None); // Fields of this prescription also wont be checked
            another_prescription.add_drug(Uuid::new_v4(), 1)?;
            another_prescription.commit_to_repository(&pool).await?;
        }

        let prescriptions = PrescriptionRepository::get_prescriptions(&pool, None, Some(7)).await?;
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

        let prescriptions =
            PrescriptionRepository::get_prescriptions(&pool, None, Some(20)).await?;
        assert!(prescriptions.len() == 11);

        Ok(())
    }
}
