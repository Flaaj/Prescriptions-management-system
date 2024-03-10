#[cfg(test)]
mod integration_tests {
    use chrono::{Duration, Utc};
    use uuid::Uuid;

    use crate::{
        create_tables::create_tables,
        domain::prescriptions::{
            create_prescription::NewPrescription,
            get_prescriptions_repository::{GetPrescriptionError, PrescriptionRepository},
            prescription_type::PrescriptionType,
        },
    };

    #[sqlx::test]
    async fn create_and_read_prescriptions_from_database(pool: sqlx::PgPool) {
        create_tables(&pool, true).await.unwrap();

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

        prescription
            .clone()
            .commit_to_repository(&pool)
            .await
            .unwrap();

        for _ in 0..10 {
            let mut another_prescription =
                NewPrescription::new(Uuid::new_v4(), Uuid::new_v4(), None, None); // Fields of this prescription also wont be checked
            another_prescription.add_drug(Uuid::new_v4(), 1).unwrap();
            another_prescription
                .commit_to_repository(&pool)
                .await
                .unwrap();
        }

        let prescriptions = PrescriptionRepository::get_prescriptions(&pool, None, Some(7))
            .await
            .unwrap();
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

        let prescriptions = PrescriptionRepository::get_prescriptions(&pool, None, Some(20))
            .await
            .unwrap();
        assert!(prescriptions.len() == 11);
    }

    #[sqlx::test]
    async fn create_and_read_prescription_by_id(pool: sqlx::PgPool) {
        create_tables(&pool, true).await.unwrap();

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

        prescription
            .clone()
            .commit_to_repository(&pool)
            .await
            .unwrap();

        let prescription_from_db =
            PrescriptionRepository::get_prescription_by_id(&pool, prescription.id)
                .await
                .unwrap();

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
        let prescription_id = Uuid::new_v4();

        let prescription_from_db =
            PrescriptionRepository::get_prescription_by_id(&pool, prescription_id).await;

        assert_eq!(
            prescription_from_db.unwrap_err().downcast_ref(),
            Some(&GetPrescriptionError::NotFound(prescription_id)),
        );
    }
}
