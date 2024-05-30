use std::sync::RwLock;

use crate::domain::{
    doctors::models::Doctor,
    drugs::models::Drug,
    patients::models::Patient,
    pharmacists::models::Pharmacist,
    prescriptions::models::{NewPrescription, NewPrescriptionFill, Prescription, PrescriptionFill},
    utils::pagination::get_pagination_params,
};
use async_trait::async_trait;
use chrono::Utc;
use uuid::Uuid;

use super::models::{PrescribedDrug, PrescriptionDoctor, PrescriptionPatient};

#[async_trait]
pub trait PrescriptionsRepository: Send + Sync + 'static {
    async fn create_prescription(
        &self,
        prescription: NewPrescription,
    ) -> anyhow::Result<Prescription>;
    async fn get_prescriptions(
        &self,
        page: Option<i64>,
        page_size: Option<i64>,
    ) -> anyhow::Result<Vec<Prescription>>;
    async fn get_prescription_by_id(&self, prescription_id: Uuid) -> anyhow::Result<Prescription>;
    async fn fill_prescription(
        &self,
        prescription_fill: NewPrescriptionFill,
    ) -> anyhow::Result<PrescriptionFill>;
    // async fn get_prescriptions_by_prescription_id(&self, prescription_id: Uuid) -> anyhow::Result<Vec<Prescription>>;
    // async fn get_prescriptions_by_patient_id(&self, patient_id: Uuid) -> anyhow::Result<Vec<Prescription>>;
    // async fn update_prescription(&self, prescription: Prescription) -> anyhow::Result<()>;
    // async fn delete_prescription(&self, prescription_id: Uuid) -> anyhow::Result<()>;
}

pub struct PrescriptionsRepositoryFake {
    prescriptions: RwLock<Vec<Prescription>>,
    doctors: RwLock<Vec<Doctor>>,
    pharmacists: RwLock<Vec<Pharmacist>>,
    patients: RwLock<Vec<Patient>>,
    drugs: RwLock<Vec<Drug>>,
}

impl PrescriptionsRepositoryFake {
    #[allow(dead_code)]
    pub fn new(
        initial_prescriptions: Option<Vec<Prescription>>,
        initial_doctors: Option<Vec<Doctor>>,
        initial_patients: Option<Vec<Patient>>,
        initial_pharmacists: Option<Vec<Pharmacist>>,
        initial_drugs: Option<Vec<Drug>>,
    ) -> Self {
        Self {
            prescriptions: RwLock::new(initial_prescriptions.unwrap_or(Vec::new())),
            doctors: RwLock::new(initial_doctors.unwrap_or(Vec::new())),
            drugs: RwLock::new(initial_drugs.unwrap_or(Vec::new())),
            patients: RwLock::new(initial_patients.unwrap_or(Vec::new())),
            pharmacists: RwLock::new(initial_pharmacists.unwrap_or(Vec::new())),
        }
    }
}

#[async_trait]
impl PrescriptionsRepository for PrescriptionsRepositoryFake {
    async fn create_prescription(
        &self,
        new_prescription: NewPrescription,
    ) -> anyhow::Result<Prescription> {
        let patients = self.patients.read().unwrap();
        let found_patient = patients
            .iter()
            .find(|patient| patient.id == new_prescription.patient_id)
            .ok_or(anyhow::format_err!("Patient not found"))?;

        let doctors = self.doctors.read().unwrap();
        let found_doctor = doctors
            .iter()
            .find(|doctor: &&Doctor| doctor.id == new_prescription.doctor_id)
            .ok_or(anyhow::format_err!("Doctor not found"))?;

        let drugs = self.drugs.read().unwrap();
        for new_prescribed_drug in &new_prescription.prescribed_drugs {
            drugs
                .iter()
                .find(|drug| drug.id == new_prescribed_drug.drug_id)
                .ok_or(anyhow::format_err!("Drug not found"))?;
        }

        let prescription = Prescription {
            id: new_prescription.id,
            doctor: PrescriptionDoctor {
                id: found_doctor.id.clone(),
                name: found_doctor.name.clone(),
                pesel_number: found_doctor.pesel_number.clone(),
                pwz_number: found_doctor.pwz_number.clone(),
            },
            patient: PrescriptionPatient {
                id: found_patient.id.clone(),
                name: found_patient.name.clone(),
                pesel_number: found_patient.name.clone(),
            },
            prescribed_drugs: new_prescription
                .prescribed_drugs
                .iter()
                .map(|new_prescibed_drug| PrescribedDrug {
                    id: Uuid::new_v4(),
                    drug_id: new_prescibed_drug.drug_id,
                    prescription_id: new_prescription.id,
                    quantity: new_prescibed_drug.quantity as i32,
                    created_at: Utc::now(),
                    updated_at: Utc::now(),
                })
                .collect(),
            prescription_type: new_prescription.prescription_type,
            code: new_prescription.code,
            fill: None,
            start_date: new_prescription.start_date,
            end_date: new_prescription.end_date,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        self.prescriptions
            .write()
            .unwrap()
            .push(prescription.clone());

        Ok(prescription)
    }

    async fn get_prescriptions(
        &self,
        page: Option<i64>,
        page_size: Option<i64>,
    ) -> anyhow::Result<Vec<Prescription>> {
        let (page_size, offset) = get_pagination_params(page, page_size)?;
        let a = offset;
        let b = offset + page_size;

        let mut prescriptions: Vec<Prescription> = vec![];
        for i in a..b {
            match self.prescriptions.read().unwrap().get(i as usize) {
                Some(prescription) => prescriptions.push(prescription.clone()),
                None => {}
            }
        }

        Ok(prescriptions)
    }

    async fn get_prescription_by_id(&self, prescription_id: Uuid) -> anyhow::Result<Prescription> {
        match self
            .prescriptions
            .read()
            .unwrap()
            .iter()
            .find(|prescription| prescription.id == prescription_id)
        {
            Some(prescription) => Ok(prescription.clone()),
            None => Err(anyhow::anyhow!("Prescription not found")),
        }
    }

    async fn fill_prescription(
        &self,
        new_prescription_fill: NewPrescriptionFill,
    ) -> anyhow::Result<PrescriptionFill> {
        let pharmacists = self.pharmacists.read().unwrap();
        pharmacists
            .iter()
            .find(|pharmacist| pharmacist.id == new_prescription_fill.pharmacist_id)
            .ok_or(anyhow::format_err!("Pharmacist not found"))?;

        let prescription_fill = PrescriptionFill {
            id: new_prescription_fill.id,
            prescription_id: new_prescription_fill.prescription_id,
            pharmacist_id: new_prescription_fill.pharmacist_id,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        let prescriptions = self.prescriptions.read().unwrap().to_owned();
        let (index, prescription) = prescriptions
            .iter()
            .enumerate()
            .map(|(index, prescription)| {
                (index, {
                    let mut prescription = prescription.clone();
                    prescription.fill = Some(prescription_fill.clone());
                    prescription
                })
            })
            .find(|(_, prescription)| prescription.id == new_prescription_fill.prescription_id)
            .unwrap();

        self.prescriptions
            .write()
            .unwrap()
            .insert(index, prescription);

        Ok(prescription_fill)
    }
}

#[cfg(test)]
mod tests {
    use uuid::Uuid;

    use crate::domain::{
        doctors::{
            models::NewDoctor,
            repository::{DoctorsRepository, DoctorsRepositoryFake},
        },
        drugs::{
            models::{DrugContentType, NewDrug},
            repository::{DrugsRepository, DrugsRepositoryFake},
        },
        patients::{
            models::NewPatient,
            repository::{PatientsRepository, PatientsRepositoryFake},
        },
        pharmacists::{
            models::NewPharmacist,
            repository::{PharmacistsRepository, PharmacistsRepositoryFake},
        },
        prescriptions::{
            models::{NewPrescribedDrug, NewPrescription},
            repository::{PrescriptionsRepository, PrescriptionsRepositoryFake},
        },
    };

    struct DatabaseSeeds {
        doctor: NewDoctor,
        pharmacist: NewPharmacist,
        patient: NewPatient,
        drugs: Vec<NewDrug>,
    }

    async fn seed_in_memory_database(
        prescriptions_repo: &PrescriptionsRepositoryFake,
    ) -> anyhow::Result<DatabaseSeeds> {
        let pharmacists_repo = PharmacistsRepositoryFake::new();
        let pharmacist = NewPharmacist::new(
            "John Pharmacist".into(), //
            "96021807250".into(),
        )?;
        let created_pharmacist = pharmacists_repo
            .create_pharmacist(pharmacist.clone())
            .await?;
        prescriptions_repo
            .pharmacists
            .write()
            .unwrap()
            .push(created_pharmacist);

        let patients_repo = PatientsRepositoryFake::new();
        let patient = NewPatient::new(
            "John Patient".into(), //
            "96021807250".into(),
        )?;
        let created_patient = patients_repo.create_patient(patient.clone()).await?;
        prescriptions_repo
            .patients
            .write()
            .unwrap()
            .push(created_patient);

        let doctors_repo = DoctorsRepositoryFake::new();
        let doctor = NewDoctor::new(
            "John Doctor".into(), //
            "3123456".into(),
            "96021807250".into(),
        )?;
        let created_doctor = doctors_repo.create_doctor(doctor.clone()).await?;
        prescriptions_repo
            .doctors
            .write()
            .unwrap()
            .push(created_doctor);

        let drugs_repo = DrugsRepositoryFake::new();
        let mut drugs = vec![];
        for _ in 0..4 {
            let drug = NewDrug::new(
                "Gripex".into(),
                DrugContentType::SolidPills,
                Some(20),
                Some(300),
                None,
                None,
            )?;
            drugs.push(drug.clone());
            let created_drug = drugs_repo.create_drug(drug).await?;
            prescriptions_repo.drugs.write().unwrap().push(created_drug);
        }

        Ok(DatabaseSeeds {
            doctor,
            pharmacist,
            patient,
            drugs,
        })
    }

    async fn setup_repository() -> (PrescriptionsRepositoryFake, DatabaseSeeds) {
        let repository = PrescriptionsRepositoryFake::new(None, None, None, None, None);
        let seeds = seed_in_memory_database(&repository).await.unwrap();
        (repository, seeds)
    }

    #[tokio::test]
    async fn creates_and_reads_prescriptions_from_database() {
        let (repository, seeds) = setup_repository().await;

        let new_prescription = NewPrescription::new(
            seeds.doctor.id,
            seeds.patient.id,
            None,
            None,
            vec![
                NewPrescribedDrug {
                    drug_id: seeds.drugs[0].id,
                    quantity: 1,
                },
                NewPrescribedDrug {
                    drug_id: seeds.drugs[1].id,
                    quantity: 1,
                },
                NewPrescribedDrug {
                    drug_id: seeds.drugs[2].id,
                    quantity: 1,
                },
                NewPrescribedDrug {
                    drug_id: seeds.drugs[3].id,
                    quantity: 1,
                },
            ],
        )
        .unwrap();

        repository
            .create_prescription(new_prescription.clone())
            .await
            .unwrap();

        for _ in 0..10 {
            let another_prescription = NewPrescription::new(
                seeds.doctor.id,
                seeds.patient.id,
                None,
                None,
                vec![NewPrescribedDrug {
                    drug_id: seeds.drugs[0].id,
                    quantity: 1,
                }],
            )
            .unwrap();
            repository
                .create_prescription(another_prescription)
                .await
                .unwrap();
        }

        let prescriptions = repository.get_prescriptions(None, Some(7)).await.unwrap();

        assert_eq!(prescriptions.len(), 7);
        assert_eq!(prescriptions[0], new_prescription);

        let prescriptions = repository.get_prescriptions(None, Some(20)).await.unwrap();
        assert_eq!(prescriptions.len(), 11);

        let prescriptions = repository
            .get_prescriptions(Some(1), Some(10))
            .await
            .unwrap();
        assert_eq!(prescriptions.len(), 1);
    }

    #[tokio::test]
    async fn creates_and_reads_prescription_by_id() {
        let (repository, seeds) = setup_repository().await;

        let new_prescription = NewPrescription::new(
            seeds.doctor.id,
            seeds.patient.id,
            None,
            None,
            vec![
                NewPrescribedDrug {
                    drug_id: seeds.drugs[0].id,
                    quantity: 1,
                },
                NewPrescribedDrug {
                    drug_id: seeds.drugs[1].id,
                    quantity: 1,
                },
            ],
        )
        .unwrap();

        repository
            .create_prescription(new_prescription.clone())
            .await
            .unwrap();

        let prescription_from_db = repository
            .get_prescription_by_id(new_prescription.id)
            .await
            .unwrap();

        assert_eq!(prescription_from_db, new_prescription);
    }

    #[tokio::test]
    async fn doesnt_create_prescription_if_relations_dont_exist() {
        let (repository, seeds) = setup_repository().await;

        let new_prescription_with_nonexisting_doctor_id = NewPrescription::new(
            Uuid::new_v4(),
            seeds.doctor.id,
            None,
            None,
            vec![NewPrescribedDrug {
                drug_id: seeds.drugs[0].id,
                quantity: 1,
            }],
        )
        .unwrap();

        assert!(repository
            .create_prescription(new_prescription_with_nonexisting_doctor_id)
            .await
            .is_err());

        let new_prescription_with_nonexisting_patient_id = NewPrescription::new(
            seeds.patient.id,
            Uuid::new_v4(),
            None,
            None,
            vec![NewPrescribedDrug {
                drug_id: seeds.drugs[0].id,
                quantity: 1,
            }],
        )
        .unwrap();

        assert!(repository
            .create_prescription(new_prescription_with_nonexisting_patient_id)
            .await
            .is_err());

        let new_prescription_with_nonexisting_drug_id = NewPrescription::new(
            seeds.doctor.id,
            seeds.patient.id,
            None,
            None,
            vec![NewPrescribedDrug {
                drug_id: Uuid::new_v4(),
                quantity: 1,
            }],
        )
        .unwrap();

        assert!(repository
            .create_prescription(new_prescription_with_nonexisting_drug_id)
            .await
            .is_err());
    }

    #[tokio::test]
    async fn get_prescription_by_id_returns_error_if_prescription_doesnt_exist() {
        let (repository, _) = setup_repository().await;
        let prescription_id = Uuid::new_v4();

        let prescription_from_db = repository.get_prescription_by_id(prescription_id).await;

        assert!(prescription_from_db.is_err());
    }

    #[tokio::test]
    async fn fills_prescription_and_saves_to_database() {
        let (repository, seeds) = setup_repository().await;

        let prescription = NewPrescription::new(
            seeds.doctor.id,
            seeds.patient.id,
            None,
            None,
            vec![NewPrescribedDrug {
                drug_id: seeds.drugs[0].id,
                quantity: 1,
            }],
        )
        .unwrap();

        repository
            .create_prescription(prescription.clone())
            .await
            .unwrap();

        let prescription_from_db = repository
            .get_prescription_by_id(prescription.id)
            .await
            .unwrap();

        assert!(prescription_from_db.fill.is_none());

        let new_prescription_fill = prescription_from_db.fill(seeds.pharmacist.id).unwrap();
        let created_prescription_fill = repository
            .fill_prescription(new_prescription_fill.clone())
            .await
            .unwrap();

        assert_eq!(created_prescription_fill, new_prescription_fill);

        let prescription_from_db = repository
            .get_prescription_by_id(prescription.id)
            .await
            .unwrap();

        assert_eq!(prescription_from_db.fill.unwrap(), new_prescription_fill);
    }

    #[tokio::test]
    async fn doesnt_fill_if_pharmacist_relation_doesnt_exist() {
        let (repository, seeds) = setup_repository().await;

        let new_prescription = NewPrescription::new(
            seeds.doctor.id,
            seeds.patient.id,
            None,
            None,
            vec![
                NewPrescribedDrug {
                    drug_id: seeds.drugs[0].id,
                    quantity: 1,
                },
                NewPrescribedDrug {
                    drug_id: seeds.drugs[1].id,
                    quantity: 1,
                },
            ],
        )
        .unwrap();

        let prescription_from_db = repository
            .create_prescription(new_prescription.clone())
            .await
            .unwrap();

        let new_prescription_fill_with_nonexistent_pharmacist_id =
            prescription_from_db.fill(Uuid::new_v4()).unwrap();

        assert!(repository
            .fill_prescription(new_prescription_fill_with_nonexistent_pharmacist_id)
            .await
            .is_err());
    }
}
