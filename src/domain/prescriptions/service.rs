use super::{
    models::{NewPrescription, Prescription, PrescriptionType},
    repository::PrescriptionsRepository,
};
use chrono::{DateTime, Utc};
use uuid::Uuid;

#[derive(Clone)]
pub struct PrescriptionsService<R: PrescriptionsRepository> {
    repository: R,
}

#[derive(Debug)]
pub enum CreatePrescriptionError {
    RepositoryError(String),
    DomainError(String),
}

impl<R: PrescriptionsRepository> PrescriptionsService<R> {
    pub fn new(repository: R) -> Self {
        Self { repository }
    }

    pub async fn create_prescription(
        &self,
        doctor_id: Uuid,
        patient_id: Uuid,
        start_date: Option<DateTime<Utc>>,
        prescription_type: Option<PrescriptionType>,
        prescribed_drug_ids: Vec<(Uuid, u32)>,
    ) -> Result<Prescription, CreatePrescriptionError> {
        let mut new_prescription =
            NewPrescription::new(doctor_id, patient_id, start_date, prescription_type);

        for (drug_id, quantity) in prescribed_drug_ids {
            new_prescription
                .add_drug(drug_id, quantity)
                .map_err(|err| CreatePrescriptionError::DomainError(err.to_string()))?;
        }

        let created_prescription = self
            .repository
            .create_prescription(new_prescription)
            .await
            .map_err(|err| CreatePrescriptionError::RepositoryError(err.to_string()))?;

        Ok(created_prescription)
    }

    pub async fn fill_prescription(
        &self,
        prescription_id: Uuid,
        pharmacist_id: Uuid,
    ) -> Result<Prescription, CreatePrescriptionError> {
        let mut prescription = self
            .repository
            .get_prescription_by_id(prescription_id)
            .await
            .map_err(|err| CreatePrescriptionError::RepositoryError(err.to_string()))?;

        let new_prescription_fill = prescription
            .fill(pharmacist_id)
            .map_err(|err| CreatePrescriptionError::DomainError(err.to_string()))?;

        let prescription_fill = self
            .repository
            .fill_prescription(new_prescription_fill)
            .await
            .map_err(|err| CreatePrescriptionError::RepositoryError(err.to_string()))?;
        prescription.fill = Some(prescription_fill);

        Ok(prescription)
    }
}

// #[cfg(test)]
// mod tests {
//     use super::PrescriptionsService;
//     use crate::domain::{
//         doctors::service::DoctorsService,
//         drugs::{models::DrugContentType, service::DrugsService},
//         patients::service::PatientsService,
//         pharmacists::service::PharmacistsService,
//         prescriptions::{models::PrescriptionType, repository::PrescriptionsRepository},
//     };
//     use sqlx::PgPool;
//     use uuid::Uuid;

//     struct DatabaseSeedRecordIds {
//         doctor_id: Uuid,
//         patient_id: Uuid,
//         #[allow(dead_code)]
//         pharmacist_id: Uuid,
//         drug_ids: Vec<Uuid>,
//     }

//     async fn setup_services_and_seed_database(
//         pool: PgPool,
//     ) -> (
//         PrescriptionsService<impl PrescriptionsRepository>,
//         DatabaseSeedRecordIds,
//     ) {
//         create_tables(&pool, true).await.unwrap();

//         let doctors_service = DoctorsService::new(PostgresDoctorsRepository::new(pool.clone()));
//         let pharmacist_service =
//             PharmacistsService::new(PostgresPharmacistsRepository::new(pool.clone()));
//         let patients_service = PatientsService::new(PostgresPatientsRepository::new(pool.clone()));
//         let drugs_service = DrugsService::new(PostgresDrugsRepository::new(pool.clone()));
//         let prescriptions_service =
//             PrescriptionsService::new(PostgresPrescriptionsRepository::new(pool));

//         (
//             prescriptions_service,
//             DatabaseSeedRecordIds {
//                 doctor_id: doctors_service
//                     .create_doctor("John Doctor".into(), "92022900002".into(), "3123456".into())
//                     .await
//                     .unwrap()
//                     .id,
//                 patient_id: patients_service
//                     .create_patient("John Patient".into(), "92022900002".into())
//                     .await
//                     .unwrap()
//                     .id,
//                 pharmacist_id: pharmacist_service
//                     .create_pharmacist("John Pharmacist".into(), "92022900002".into())
//                     .await
//                     .unwrap()
//                     .id,
//                 drug_ids: vec![
//                     drugs_service
//                         .create_drug(
//                             "Gripex".into(),
//                             DrugContentType::SolidPills,
//                             Some(20),
//                             Some(300),
//                             None,
//                             None,
//                         )
//                         .await
//                         .unwrap()
//                         .id,
//                     drugs_service
//                         .create_drug(
//                             "Gripex".into(),
//                             DrugContentType::SolidPills,
//                             Some(20),
//                             Some(300),
//                             None,
//                             None,
//                         )
//                         .await
//                         .unwrap()
//                         .id,
//                     drugs_service
//                         .create_drug(
//                             "Gripex".into(),
//                             DrugContentType::SolidPills,
//                             Some(20),
//                             Some(300),
//                             None,
//                             None,
//                         )
//                         .await
//                         .unwrap()
//                         .id,
//                     drugs_service
//                         .create_drug(
//                             "Gripex".into(),
//                             DrugContentType::SolidPills,
//                             Some(20),
//                             Some(300),
//                             None,
//                             None,
//                         )
//                         .await
//                         .unwrap()
//                         .id,
//                 ],
//             },
//         )
//     }

//     #[sqlx::test]
//     async fn creates_prescription(pool: sqlx::PgPool) {
//         let (service, seed_ids) = setup_services_and_seed_database(pool).await;

//         let created_prescription = service
//             .create_prescription(
//                 seed_ids.doctor_id,
//                 seed_ids.patient_id,
//                 None,
//                 Some(PrescriptionType::ForChronicDiseaseDrugs),
//                 vec![(seed_ids.drug_ids[0], 1), (seed_ids.drug_ids[1], 2)],
//             )
//             .await
//             .unwrap();

//         assert_eq!(
//             created_prescription.prescription_type,
//             PrescriptionType::ForChronicDiseaseDrugs
//         );
//         assert_eq!(created_prescription.prescribed_drugs.len(), 2)
//     }

//     #[sqlx::test]
//     async fn fills_prescription(pool: sqlx::PgPool) {
//         let (service, seed_ids) = setup_services_and_seed_database(pool).await;
//         let seed_prescription = service
//             .create_prescription(
//                 seed_ids.doctor_id,
//                 seed_ids.patient_id,
//                 None,
//                 Some(PrescriptionType::ForChronicDiseaseDrugs),
//                 vec![(seed_ids.drug_ids[0], 1), (seed_ids.drug_ids[1], 2)],
//             )
//             .await
//             .unwrap();

//         let filled_prescription = service
//             .fill_prescription(seed_prescription.id, seed_ids.pharmacist_id)
//             .await
//             .unwrap();
//         let fill = filled_prescription.fill.unwrap();

//         assert!(fill.prescription_id == seed_prescription.id);
//         assert!(fill.pharmacist_id == seed_ids.pharmacist_id);
//     }
// }
