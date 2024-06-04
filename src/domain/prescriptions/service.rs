use chrono::{DateTime, Utc};
use uuid::Uuid;

use super::{
    models::{NewPrescribedDrug, NewPrescription, Prescription, PrescriptionType},
    repository::{
        CreatePrescriptionRepositoryError, FillPrescriptionRepositoryError,
        GetPrescriptionByIdRepositoryError, GetPrescriptionsRepositoryError,
        PrescriptionsRepository,
    },
};

pub struct PrescriptionsService {
    repository: Box<dyn PrescriptionsRepository>,
}

#[derive(Debug)]
pub enum CreatePrescriptionError {
    DomainError(String),
    RepositoryError(CreatePrescriptionRepositoryError),
}

#[derive(Debug)]
pub enum GetPrescriptionByIdError {
    RepositoryError(GetPrescriptionByIdRepositoryError),
}

#[derive(Debug)]
pub enum GetPrescriptionsWithPaginationError {
    RepositoryError(GetPrescriptionsRepositoryError),
}

#[derive(Debug, PartialEq)]
pub enum FillPrescriptionError {
    DomainError(String),
    RepositoryError(FillPrescriptionRepositoryError),
}

impl PrescriptionsService {
    pub fn new(repository: Box<dyn PrescriptionsRepository>) -> Self {
        Self { repository }
    }

    pub async fn create_prescription(
        &self,
        doctor_id: Uuid,
        patient_id: Uuid,
        start_date: Option<DateTime<Utc>>,
        prescription_type: Option<PrescriptionType>,
        prescribed_drugs: Vec<(Uuid, u32)>,
    ) -> Result<Prescription, CreatePrescriptionError> {
        let new_prescription = NewPrescription::new(
            doctor_id,
            patient_id,
            start_date,
            prescription_type,
            prescribed_drugs
                .iter()
                .map(|&(drug_id, quantity)| NewPrescribedDrug { drug_id, quantity })
                .collect(),
        )
        .map_err(|err| CreatePrescriptionError::DomainError(err.to_string()))?;

        let created_prescription = self
            .repository
            .create_prescription(new_prescription)
            .await
            .map_err(|err| CreatePrescriptionError::RepositoryError(err))?;

        Ok(created_prescription)
    }

    pub async fn fill_prescription(
        &self,
        prescription_id: Uuid,
        pharmacist_id: Uuid,
    ) -> Result<Prescription, FillPrescriptionError> {
        let mut prescription = self
            .repository
            .get_prescription_by_id(prescription_id)
            .await
            .map_err(|err| match err {
                GetPrescriptionByIdRepositoryError::NotFound(id) => {
                    FillPrescriptionError::RepositoryError(
                        FillPrescriptionRepositoryError::PrescriptionNotFound(id),
                    )
                }
                _ => FillPrescriptionError::RepositoryError(
                    FillPrescriptionRepositoryError::DatabaseError(err.to_string()),
                ),
            })?;

        let new_prescription_fill = prescription
            .fill(pharmacist_id)
            .map_err(|err| FillPrescriptionError::DomainError(err.to_string()))?;

        let prescription_fill = self
            .repository
            .fill_prescription(new_prescription_fill)
            .await
            .map_err(|err| FillPrescriptionError::RepositoryError(err))?;
        prescription.fill = Some(prescription_fill);

        Ok(prescription)
    }

    pub async fn get_prescription_by_id(
        &self,
        prescription_id: Uuid,
    ) -> Result<Prescription, GetPrescriptionByIdError> {
        let prescription = self
            .repository
            .get_prescription_by_id(prescription_id)
            .await
            .map_err(|err| GetPrescriptionByIdError::RepositoryError(err))?;

        Ok(prescription)
    }

    pub async fn get_prescriptions_with_pagination(
        &self,
        page: Option<i64>,
        page_size: Option<i64>,
    ) -> Result<Vec<Prescription>, GetPrescriptionsWithPaginationError> {
        let result = self
            .repository
            .get_prescriptions(page, page_size)
            .await
            .map_err(|err| GetPrescriptionsWithPaginationError::RepositoryError(err))?;

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::{FillPrescriptionError, PrescriptionsService};
    use crate::domain::{
        doctors::{models::Doctor, repository::DoctorsRepositoryFake, service::DoctorsService},
        drugs::{
            models::{Drug, DrugContentType},
            repository::DrugsRepositoryFake,
            service::DrugsService,
        },
        patients::{models::Patient, repository::PatientsRepositoryFake, service::PatientsService},
        pharmacists::{
            models::Pharmacist, repository::PharmacistsRepositoryFake, service::PharmacistsService,
        },
        prescriptions::{models::PrescriptionType, repository::PrescriptionsRepositoryFake},
    };

    struct DatabaseSeeds {
        doctor: Doctor,
        pharmacist: Pharmacist,
        patient: Patient,
        drugs: Vec<Drug>,
    }

    async fn setup_services_and_seed_database() -> (PrescriptionsService, DatabaseSeeds) {
        let doctors_service = DoctorsService::new(Box::new(DoctorsRepositoryFake::new()));
        let created_doctor = doctors_service
            .create_doctor("John Doctor".into(), "92022900002".into(), "3123456".into())
            .await
            .unwrap();

        let pharmacist_service =
            PharmacistsService::new(Box::new(PharmacistsRepositoryFake::new()));
        let created_pharmacist = pharmacist_service
            .create_pharmacist("John Pharmacist".into(), "92022900002".into())
            .await
            .unwrap();

        let patients_service = PatientsService::new(Box::new(PatientsRepositoryFake::new()));
        let created_patient = patients_service
            .create_patient("John Patient".into(), "92022900002".into())
            .await
            .unwrap();

        let drugs_service = DrugsService::new(Box::new(DrugsRepositoryFake::new()));
        let created_drug_0 = drugs_service
            .create_drug(
                "Gripex".into(),
                DrugContentType::SolidPills,
                Some(20),
                Some(300),
                None,
                None,
            )
            .await
            .unwrap();
        let created_drug_1 = drugs_service
            .create_drug(
                "Gripex".into(),
                DrugContentType::SolidPills,
                Some(20),
                Some(300),
                None,
                None,
            )
            .await
            .unwrap();
        let created_drug_2 = drugs_service
            .create_drug(
                "Gripex".into(),
                DrugContentType::SolidPills,
                Some(20),
                Some(300),
                None,
                None,
            )
            .await
            .unwrap();
        let created_drug_3 = drugs_service
            .create_drug(
                "Gripex".into(),
                DrugContentType::SolidPills,
                Some(20),
                Some(300),
                None,
                None,
            )
            .await
            .unwrap();

        (
            PrescriptionsService::new(Box::new(PrescriptionsRepositoryFake::new(
                None,
                Some(vec![created_doctor.clone()]),
                Some(vec![created_patient.clone()]),
                Some(vec![created_pharmacist.clone()]),
                Some(vec![
                    created_drug_0.clone(),
                    created_drug_1.clone(),
                    created_drug_2.clone(),
                    created_drug_3.clone(),
                ]),
            ))),
            DatabaseSeeds {
                doctor: created_doctor,
                pharmacist: created_pharmacist,
                patient: created_patient,
                drugs: vec![
                    created_drug_0,
                    created_drug_1,
                    created_drug_2,
                    created_drug_3,
                ],
            },
        )
    }

    #[tokio::test]
    async fn creates_and_reads_prescription_by_id() {
        let (service, seeds) = setup_services_and_seed_database().await;

        let created_prescription = service
            .create_prescription(
                seeds.doctor.id,
                seeds.patient.id,
                None,
                Some(PrescriptionType::ForChronicDiseaseDrugs),
                vec![(seeds.drugs[0].id, 1), (seeds.drugs[1].id, 2)],
            )
            .await
            .unwrap();

        assert_eq!(
            created_prescription.prescription_type,
            PrescriptionType::ForChronicDiseaseDrugs
        );
        assert_eq!(created_prescription.prescribed_drugs.len(), 2);

        let prescription_from_repository = service
            .get_prescription_by_id(created_prescription.id)
            .await
            .unwrap();

        assert_eq!(created_prescription, prescription_from_repository);
    }

    #[tokio::test]
    async fn get_prescription_by_id_returns_error_if_prescription_doesnt_exist() {
        let (service, _) = setup_services_and_seed_database().await;
        let prescription_id = uuid::Uuid::new_v4();

        let prescription = service.get_prescription_by_id(prescription_id).await;

        assert!(prescription.is_err());
    }

    #[tokio::test]
    async fn fills_prescription() {
        let (service, seeds) = setup_services_and_seed_database().await;
        let seed_prescription = service
            .create_prescription(
                seeds.doctor.id,
                seeds.patient.id,
                None,
                Some(PrescriptionType::ForChronicDiseaseDrugs),
                vec![(seeds.drugs[0].id, 1), (seeds.drugs[1].id, 2)],
            )
            .await
            .unwrap();

        let filled_prescription = service
            .fill_prescription(seed_prescription.id, seeds.pharmacist.id)
            .await
            .unwrap();
        let fill = filled_prescription.fill.unwrap();

        assert!(fill.prescription_id == seed_prescription.id);
        assert!(fill.pharmacist_id == seeds.pharmacist.id);
    }

    #[tokio::test]
    async fn doesnt_fill_if_already_filled() {
        let (service, seeds) = setup_services_and_seed_database().await;
        let seed_prescription = service
            .create_prescription(
                seeds.doctor.id,
                seeds.patient.id,
                None,
                Some(PrescriptionType::ForChronicDiseaseDrugs),
                vec![(seeds.drugs[0].id, 1), (seeds.drugs[1].id, 2)],
            )
            .await
            .unwrap();

        let filled_prescription = service
            .fill_prescription(seed_prescription.id, seeds.pharmacist.id)
            .await
            .unwrap();
        let fill = filled_prescription.fill.unwrap();

        assert!(fill.prescription_id == seed_prescription.id);
        assert!(fill.pharmacist_id == seeds.pharmacist.id);

        let prescription_filled_again = service
            .fill_prescription(filled_prescription.id, seeds.pharmacist.id)
            .await;

        assert!(match prescription_filled_again {
            Err(FillPrescriptionError::DomainError(_)) => true,
            _ => false,
        });
    }

    #[tokio::test]
    async fn gets_pharmacists_with_pagination() {
        let (service, seeds) = setup_services_and_seed_database().await;

        service
            .create_prescription(
                seeds.doctor.id,
                seeds.patient.id,
                None,
                Some(PrescriptionType::ForChronicDiseaseDrugs),
                vec![(seeds.drugs[0].id, 1), (seeds.drugs[1].id, 2)],
            )
            .await
            .unwrap();
        service
            .create_prescription(
                seeds.doctor.id,
                seeds.patient.id,
                None,
                Some(PrescriptionType::ForChronicDiseaseDrugs),
                vec![(seeds.drugs[0].id, 1), (seeds.drugs[1].id, 2)],
            )
            .await
            .unwrap();
        service
            .create_prescription(
                seeds.doctor.id,
                seeds.patient.id,
                None,
                Some(PrescriptionType::ForChronicDiseaseDrugs),
                vec![(seeds.drugs[0].id, 1), (seeds.drugs[1].id, 2)],
            )
            .await
            .unwrap();
        service
            .create_prescription(
                seeds.doctor.id,
                seeds.patient.id,
                None,
                Some(PrescriptionType::ForChronicDiseaseDrugs),
                vec![(seeds.drugs[0].id, 1), (seeds.drugs[1].id, 2)],
            )
            .await
            .unwrap();

        let prescriptions = service
            .get_prescriptions_with_pagination(Some(1), Some(2))
            .await
            .unwrap();

        assert_eq!(prescriptions.len(), 2);

        let prescriptions = service
            .get_prescriptions_with_pagination(Some(1), Some(3))
            .await
            .unwrap();

        assert_eq!(prescriptions.len(), 1);

        let prescriptions = service
            .get_prescriptions_with_pagination(None, Some(10))
            .await
            .unwrap();

        assert_eq!(prescriptions.len(), 4);

        let prescriptions = service
            .get_prescriptions_with_pagination(Some(1), None)
            .await
            .unwrap();

        assert_eq!(prescriptions.len(), 0);

        let prescriptions = service
            .get_prescriptions_with_pagination(None, None)
            .await
            .unwrap();

        assert_eq!(prescriptions.len(), 4);

        let prescriptions = service
            .get_prescriptions_with_pagination(Some(2), Some(3))
            .await
            .unwrap();

        assert_eq!(prescriptions.len(), 0);
    }

    #[tokio::test]
    async fn get_pharmacists_with_pagination_returns_error_if_params_are_invalid() {
        let (service, _) = setup_services_and_seed_database().await;

        assert!(service
            .get_prescriptions_with_pagination(Some(-1), None)
            .await
            .is_err());

        assert!(service
            .get_prescriptions_with_pagination(None, Some(0))
            .await
            .is_err());
    }
}
