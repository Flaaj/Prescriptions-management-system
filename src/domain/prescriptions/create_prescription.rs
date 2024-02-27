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

use anyhow::bail;
use chrono::{DateTime, Utc};
use uuid::Uuid;

use super::prescription_type::PrescriptionType;

#[derive(Debug, PartialEq)]
pub struct PrescribedDrug {
    pub drug_id: Uuid,
    pub quantity: u32,
}

#[derive(Debug, PartialEq)]
pub struct NewPrescription {
    pub doctor_id: Uuid,
    pub patient_id: Uuid,
    pub prescribed_drugs: Vec<PrescribedDrug>,
    pub prescription_type: PrescriptionType,
    pub start_date: DateTime<Utc>,
    pub end_date: DateTime<Utc>,
}

impl NewPrescription {
    pub fn new(
        doctor_id: Uuid,
        patient_id: Uuid,
        start_date: Option<DateTime<Utc>>,
        prescription_type: Option<PrescriptionType>,
    ) -> Self {
        // defaults:
        let start_date = start_date.unwrap_or(Utc::now());
        let prescription_type = prescription_type.unwrap_or(PrescriptionType::Regular);

        let duration = prescription_type.get_duration();
        let end_date = start_date + duration;

        Self {
            doctor_id,
            patient_id,
            prescribed_drugs: vec![],
            prescription_type,
            start_date,
            end_date,
        }
    }

    pub fn add_drug(&mut self, drug_id: Uuid, quantity: u32) -> anyhow::Result<()> {
        if quantity == 0 {
            bail!(format!("Quantity of drug with id {} can't be 0", drug_id));
        }
        let prescribed_drug = PrescribedDrug { drug_id, quantity };
        self.prescribed_drugs.push(prescribed_drug);
        Ok(())
    }

    pub fn validate(&self) -> anyhow::Result<()> {
        if self.prescribed_drugs.is_empty() {
            bail!("Prescription must have at least one prescribed drug");
        }
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use chrono::{Duration, Utc};
    use uuid::Uuid;

    use crate::domain::prescriptions::create_prescription::{NewPrescription, PrescriptionType};

    #[test]
    fn creates_prescription() {
        let doctor_id = Uuid::new_v4();
        let patient_id = Uuid::new_v4();

        let sut = NewPrescription::new(doctor_id, patient_id, None, None);

        assert_eq!(sut.doctor_id, doctor_id);
        assert_eq!(sut.patient_id, patient_id);
        assert_eq!(sut.prescribed_drugs, vec![]);
        assert_eq!(sut.prescription_type, PrescriptionType::Regular);
    }

    #[test]
    fn creates_prescription_with_30_days_duration_for_regular_prescriptions() {
        let now = Utc::now();

        let sut = NewPrescription::new(
            Uuid::new_v4(),
            Uuid::new_v4(),
            Some(now),
            Some(PrescriptionType::Regular),
        );

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
        );

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
        );

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
        );

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
        let mut prescription = NewPrescription::new(Uuid::new_v4(), Uuid::new_v4(), None, None);

        prescription.add_drug(drug_id, 2).unwrap();
        let sut = prescription.prescribed_drugs;

        let prescribed_drug = sut.get(0).unwrap();
        assert_eq!(prescribed_drug.drug_id, drug_id);
        assert_eq!(prescribed_drug.quantity, 2);
    }

    #[test]
    fn adds_multiple_drugs_to_prescription() {
        let mut prescription = NewPrescription::new(Uuid::new_v4(), Uuid::new_v4(), None, None);

        prescription.add_drug(Uuid::new_v4(), 1).unwrap();
        prescription.add_drug(Uuid::new_v4(), 2).unwrap();
        prescription.add_drug(Uuid::new_v4(), 3).unwrap();
        let sut = prescription.prescribed_drugs;

        assert_eq!(sut.len(), 3);
    }

    #[test]
    fn cant_add_drug_with_zero_quantity() {
        let drug_id = Uuid::parse_str("00000000-0000-0000-0000-000000000000").unwrap();
        let mut prescription = NewPrescription::new(Uuid::new_v4(), Uuid::new_v4(), None, None);

        let sut = prescription.add_drug(drug_id, 0);

        let expected_error = String::from(
            "Quantity of drug with id 00000000-0000-0000-0000-000000000000 can't be 0",
        );
        assert!(sut.err().unwrap().to_string() == expected_error);
    }

    #[test]
    fn passes_validation_when_more_than_one_drug_is_prescribed() {
        let mut prescription = NewPrescription::new(Uuid::new_v4(), Uuid::new_v4(), None, None);
        prescription.add_drug(Uuid::new_v4(), 1).unwrap();

        let sut = prescription.validate();

        assert!(sut.is_ok());
    }

    #[test]
    fn doesnt_pass_validation_when_no_drugs_are_added_to_prescription() {
        let prescription = NewPrescription::new(Uuid::new_v4(), Uuid::new_v4(), None, None);

        let sut = prescription.validate();

        let expected_error = String::from("Prescription must have at least one prescribed drug");
        assert!(sut.err().unwrap().to_string() == expected_error);
    }
}
