// This system tries to mimic polish electronic prescriptions system called "e-recepty".
// It's a bit simplified tho, but I will try to keep adding features to make it more similar
//
// Prescription:
//  - is prescribed by doctor
//  - is prescribed to a patient
//  - can have prescribed multiple different drugs, each with any quantity
//  - has start date, which marks date from which it can be used
//  - has end date, which marks date after which it can't be used anymore
//  - each prescription can be used only once

use std::collections::HashSet;

use chrono::{DateTime, Duration, Utc};
use uuid::Uuid;

use crate::domain::prescriptions::models::{NewPrescribedDrug, NewPrescription, PrescriptionType};

#[derive(thiserror::Error, Debug, PartialEq)]
pub enum CreateNewPrescriptionDomainError {
    #[error("Prescription must have at least one prescribed drug")]
    NoPrescribedDrugs,
    #[error("Quantity of drug with id {0} can't be 0")]
    InvalidDrugQuantity(Uuid),
    #[error("Can't prescribe two drugs with the same id {0}")]
    DuplicateDrugId(Uuid),
}

impl PrescriptionType {
    pub fn get_duration(&self) -> Duration {
        match self {
            PrescriptionType::Regular => Duration::days(30),
            PrescriptionType::ForAntibiotics => Duration::days(7),
            PrescriptionType::ForImmunologicalDrugs => Duration::days(120),
            PrescriptionType::ForChronicDiseaseDrugs => Duration::days(365),
        }
    }
}

impl NewPrescription {
    pub fn new(
        doctor_id: Uuid,
        patient_id: Uuid,
        start_date: Option<DateTime<Utc>>,
        prescription_type: Option<PrescriptionType>,
        prescribed_drugs: Vec<NewPrescribedDrug>,
    ) -> Result<Self, CreateNewPrescriptionDomainError> {
        if prescribed_drugs.is_empty() {
            Err(CreateNewPrescriptionDomainError::NoPrescribedDrugs)?;
        }

        let mut ids_hashset: HashSet<Uuid> = HashSet::new();
        for prescribed_drug in &prescribed_drugs {
            if prescribed_drug.quantity == 0 {
                Err(CreateNewPrescriptionDomainError::InvalidDrugQuantity(
                    prescribed_drug.drug_id,
                ))?;
            }
            if ids_hashset.contains(&prescribed_drug.drug_id) {
                Err(CreateNewPrescriptionDomainError::DuplicateDrugId(
                    prescribed_drug.drug_id,
                ))?;
            }

            ids_hashset.insert(prescribed_drug.drug_id);
        }

        let start_date = start_date.unwrap_or(Utc::now());
        let prescription_type = prescription_type.unwrap_or(PrescriptionType::Regular);
        let duration = prescription_type.get_duration();
        let end_date = start_date + duration;

        let code = rand::random::<u64>().to_string().chars().take(8).collect();

        Ok(Self {
            id: Uuid::new_v4(),
            doctor_id,
            patient_id,
            prescribed_drugs,
            prescription_type,
            code,
            start_date,
            end_date,
        })
    }
}

#[cfg(test)]
mod tests {
    use chrono::{Duration, Utc};
    use uuid::Uuid;

    use super::{CreateNewPrescriptionDomainError, NewPrescription, PrescriptionType};
    use crate::domain::prescriptions::models::NewPrescribedDrug;

    #[test]
    fn creates_prescription() {
        let doctor_id = Uuid::new_v4();
        let patient_id = Uuid::new_v4();
        let new_prescribed_drug = NewPrescribedDrug {
            drug_id: Uuid::new_v4(),
            quantity: 1,
        };

        let sut = NewPrescription::new(
            doctor_id,
            patient_id,
            None,
            None,
            vec![new_prescribed_drug.clone()],
        )
        .unwrap();

        assert_eq!(sut.doctor_id, doctor_id);
        assert_eq!(sut.patient_id, patient_id);
        assert_eq!(sut.prescribed_drugs, vec![new_prescribed_drug]);
        assert_eq!(sut.prescription_type, PrescriptionType::Regular);
    }

    #[test]
    fn generates_random_8_digit_code_on_creation() {
        let sut = NewPrescription::new(
            Uuid::new_v4(),
            Uuid::new_v4(),
            None,
            None,
            vec![NewPrescribedDrug {
                drug_id: Uuid::new_v4(),
                quantity: 1,
            }],
        )
        .unwrap();

        assert_eq!(sut.code.len(), 8);
        assert!(sut.code.chars().all(char::is_numeric));
    }

    #[test]
    fn creates_prescription_with_30_days_duration_for_regular_prescriptions() {
        let now = Utc::now();

        let sut = NewPrescription::new(
            Uuid::new_v4(),
            Uuid::new_v4(),
            Some(now),
            Some(PrescriptionType::Regular),
            vec![NewPrescribedDrug {
                drug_id: Uuid::new_v4(),
                quantity: 1,
            }],
        )
        .unwrap();

        assert_eq!(sut.prescription_type, PrescriptionType::Regular);
        assert_eq!(sut.start_date, now);
        assert_eq!(sut.end_date, now + Duration::days(30));
    }

    #[test]
    fn creates_prescription_with_7_days_duration_when_prescription_is_for_antibiotics() {
        let now = Utc::now();

        let sut = NewPrescription::new(
            Uuid::new_v4(),
            Uuid::new_v4(),
            Some(now),
            Some(PrescriptionType::ForAntibiotics),
            vec![NewPrescribedDrug {
                drug_id: Uuid::new_v4(),
                quantity: 1,
            }],
        )
        .unwrap();

        assert_eq!(sut.prescription_type, PrescriptionType::ForAntibiotics);
        assert_eq!(sut.start_date, now);
        assert_eq!(sut.end_date, now + Duration::days(7));
    }

    #[test]
    fn creates_prescription_with_120_days_duration_when_prescription_is_for_immunological_drugs() {
        let now = Utc::now();

        let sut = NewPrescription::new(
            Uuid::new_v4(),
            Uuid::new_v4(),
            Some(now),
            Some(PrescriptionType::ForImmunologicalDrugs),
            vec![NewPrescribedDrug {
                drug_id: Uuid::new_v4(),
                quantity: 1,
            }],
        )
        .unwrap();

        assert_eq!(
            sut.prescription_type,
            PrescriptionType::ForImmunologicalDrugs
        );
        assert_eq!(sut.start_date, now);
        assert_eq!(sut.end_date, now + Duration::days(120));
    }

    #[test]
    fn creates_prescription_with_365_days_duration_when_prescription_is_for_chronic_disease_drugs()
    {
        let now = Utc::now();

        let sut = NewPrescription::new(
            Uuid::new_v4(),
            Uuid::new_v4(),
            Some(now),
            Some(PrescriptionType::ForChronicDiseaseDrugs),
            vec![NewPrescribedDrug {
                drug_id: Uuid::new_v4(),
                quantity: 1,
            }],
        )
        .unwrap();

        assert_eq!(
            sut.prescription_type,
            PrescriptionType::ForChronicDiseaseDrugs
        );
        assert_eq!(sut.start_date, now);
        assert_eq!(sut.end_date, now + Duration::days(365));
    }

    #[test]
    fn adds_prescribed_drug_to_prescription() {
        let drug_id = Uuid::new_v4();
        let new_prescribed_drug = NewPrescribedDrug {
            drug_id,
            quantity: 2,
        };
        let prescription = NewPrescription::new(
            Uuid::new_v4(),
            Uuid::new_v4(),
            None,
            None,
            vec![new_prescribed_drug.clone()],
        )
        .unwrap();

        let sut = prescription.prescribed_drugs.get(0).unwrap();

        assert_eq!(sut, &new_prescribed_drug);
    }

    #[test]
    fn adds_multiple_drugs_to_prescription() {
        let prescription = NewPrescription::new(
            Uuid::new_v4(),
            Uuid::new_v4(),
            None,
            None,
            vec![
                NewPrescribedDrug {
                    drug_id: Uuid::new_v4(),
                    quantity: 1,
                },
                NewPrescribedDrug {
                    drug_id: Uuid::new_v4(),
                    quantity: 2,
                },
                NewPrescribedDrug {
                    drug_id: Uuid::new_v4(),
                    quantity: 3,
                },
            ],
        )
        .unwrap();

        let sut = prescription.prescribed_drugs;

        assert_eq!(sut.len(), 3);
    }

    #[test]
    fn cant_add_drug_with_zero_quantity() {
        let drug_id = Uuid::new_v4();

        let sut = NewPrescription::new(
            Uuid::new_v4(),
            Uuid::new_v4(),
            None,
            None,
            vec![NewPrescribedDrug {
                drug_id,
                quantity: 0,
            }],
        );

        assert_eq!(
            sut,
            Err(CreateNewPrescriptionDomainError::InvalidDrugQuantity(
                drug_id
            ))
        );
    }

    #[test]
    fn cant_add_two_drugs_with_the_same_id() {
        let drug_id = Uuid::new_v4();

        let sut = NewPrescription::new(
            Uuid::new_v4(),
            Uuid::new_v4(),
            None,
            None,
            vec![
                NewPrescribedDrug {
                    drug_id,
                    quantity: 1,
                },
                NewPrescribedDrug {
                    drug_id,
                    quantity: 2,
                },
            ],
        );

        assert_eq!(
            sut,
            Err(CreateNewPrescriptionDomainError::DuplicateDrugId(drug_id))
        );
    }

    #[test]
    fn doesnt_create_prescription_when_no_drugs_are_added_to_prescription() {
        let sut = NewPrescription::new(Uuid::new_v4(), Uuid::new_v4(), None, None, vec![]);

        assert_eq!(
            sut,
            Err(CreateNewPrescriptionDomainError::NoPrescribedDrugs)
        );
    }
}
