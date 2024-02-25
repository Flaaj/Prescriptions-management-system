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

use chrono::{DateTime, Duration, Utc};
use uuid::Uuid;

#[derive(Debug, PartialEq)]
enum PrescriptionType {
    Regular,
    ForAntibiotics,
    ForImmunologicalDrugs,
    ForChronicDiseaseDrugs,
}

impl PrescriptionType {
    fn get_duration(&self) -> Duration {
        match self {
            PrescriptionType::Regular => Duration::days(30),
            PrescriptionType::ForAntibiotics => Duration::days(7),
            PrescriptionType::ForImmunologicalDrugs => Duration::days(120),
            PrescriptionType::ForChronicDiseaseDrugs => Duration::days(365),
        }
    }
}

#[derive(Debug, PartialEq)]
struct PrescribedDrug {
    drug_id: Uuid,
    amount: u16,
}

#[derive(Debug, PartialEq)]
struct NewPrescription {
    doctor_id: Uuid,
    patient_id: Uuid,
    prescribed_drugs: Vec<PrescribedDrug>,
    prescription_type: PrescriptionType,
    start_date: DateTime<Utc>,
    end_date: DateTime<Utc>,
}

impl NewPrescription {
    fn new(
        doctor_id: Uuid,
        patient_id: Uuid,
        start_date: Option<DateTime<Utc>>,
        prescription_type: Option<PrescriptionType>,
    ) -> Self {
        // defaults:
        let start_date = start_date.unwrap_or(Utc::now());
        let prescription_type = prescription_type.unwrap_or(PrescriptionType::Regular);

        let duration = prescription_type.get_duration();

        Self {
            doctor_id,
            patient_id,
            prescribed_drugs: vec![],
            prescription_type,
            start_date,
            end_date: start_date + duration,
        }
    }

    // Returns reference to prescribed_drugs for testability
    fn add_drug(&mut self, drug_id: Uuid, amount: u16) -> &Vec<PrescribedDrug> {
        let prescribed_drug = PrescribedDrug { drug_id, amount };
        self.prescribed_drugs.push(prescribed_drug);
        &self.prescribed_drugs
    }
}

#[cfg(test)]
mod test {
    use chrono::{Duration, Utc};
    use uuid::Uuid;

    use crate::domain::prescriptions::create_prescription::{
        NewPrescription, PrescribedDrug, PrescriptionType,
    };

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
        let doctor_id = Uuid::new_v4();
        let patient_id = Uuid::new_v4();
        let timestamp = Utc::now();

        let sut = NewPrescription::new(
            doctor_id,
            patient_id,
            Some(timestamp),
            Some(PrescriptionType::Regular),
        );

        assert_eq!(sut.prescription_type, PrescriptionType::Regular);
        assert_eq!(sut.start_date, timestamp);
        assert_eq!(sut.end_date, timestamp + Duration::days(30));
    }

    #[test]
    fn creates_prescription_with_7_days_duration_when_prescription_is_for_antibiotics() {
        let doctor_id = Uuid::new_v4();
        let patient_id = Uuid::new_v4();
        let timestamp = Utc::now();

        let sut = NewPrescription::new(
            doctor_id,
            patient_id,
            Some(timestamp),
            Some(PrescriptionType::ForAntibiotics),
        );

        assert_eq!(sut.prescription_type, PrescriptionType::ForAntibiotics);
        assert_eq!(sut.start_date, timestamp);
        assert_eq!(sut.end_date, timestamp + Duration::days(7));
    }

    #[test]
    fn creates_prescription_with_120_days_duration_when_prescription_is_for_immunological_drugs() {
        let doctor_id = Uuid::new_v4();
        let patient_id = Uuid::new_v4();
        let timestamp = Utc::now();

        let sut = NewPrescription::new(
            doctor_id,
            patient_id,
            Some(timestamp),
            Some(PrescriptionType::ForImmunologicalDrugs),
        );

        assert_eq!(
            sut.prescription_type,
            PrescriptionType::ForImmunologicalDrugs
        );
        assert_eq!(sut.start_date, timestamp);
        assert_eq!(sut.end_date, timestamp + Duration::days(120));
    }

    #[test]
    fn creates_prescription_with_365_days_duration_when_prescription_is_for_chronic_disease_drugs()
    {
        let doctor_id = Uuid::new_v4();
        let patient_id = Uuid::new_v4();
        let timestamp = Utc::now();

        let sut = NewPrescription::new(
            doctor_id,
            patient_id,
            Some(timestamp),
            Some(PrescriptionType::ForChronicDiseaseDrugs),
        );

        assert_eq!(
            sut.prescription_type,
            PrescriptionType::ForChronicDiseaseDrugs
        );
        assert_eq!(sut.start_date, timestamp);
        assert_eq!(sut.end_date, timestamp + Duration::days(365));
    }

    #[test]
    fn adds_prescribed_drug_to_prescription() {
        let doctor_id = Uuid::new_v4();
        let patient_id = Uuid::new_v4();
        let drug_id = Uuid::new_v4();
        let mut prescription = NewPrescription::new(doctor_id, patient_id, None, None);

        let sut = prescription.add_drug(drug_id, 2);

        let prescribed_drug = sut.get(0).unwrap();
        let expected = &PrescribedDrug { drug_id, amount: 2 };
        assert_eq!(prescribed_drug, expected);
    }

    #[test]
    fn adds_multiple_drugs_to_prescription() {
        let doctor_id = Uuid::new_v4();
        let patient_id = Uuid::new_v4();
        let mut prescription = NewPrescription::new(doctor_id, patient_id, None, None);

        prescription.add_drug(Uuid::new_v4(), 1);
        prescription.add_drug(Uuid::new_v4(), 2);
        let sut = prescription.add_drug(Uuid::new_v4(), 3);

        assert_eq!(sut.len(), 3);
    }
}
