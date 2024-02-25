use chrono::{DateTime, Duration, Utc};
use uuid::Uuid;

/**
Prescription:
 - is prescribed by doctor
 - is prescribed to a patient
 - can have prescribed multiple different drugs, each with any quantity
 - has start date, which marks date from which it can be used
 - has end date, which marks date after which it can't be used anymore
 - each prescription can be used only once
 */

#[derive(Debug, PartialEq)]
enum PrescriptionType {
    Regular,
    ForAntibiotics,
    ForImmunologicalDrugs,
    ForChronicDiseasesDrugs,
}

#[derive(Debug, PartialEq)]
struct PrescribedDrug {
    id: Uuid,
    drug_id: Uuid,
    amount: i32,
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

        let duration = match prescription_type {
            PrescriptionType::Regular => Duration::days(30),
            PrescriptionType::ForAntibiotics => Duration::days(7),
            PrescriptionType::ForImmunologicalDrugs => Duration::days(120),
            PrescriptionType::ForChronicDiseasesDrugs => Duration::days(365),
        };

        Self {
            doctor_id,
            patient_id,
            prescribed_drugs: vec![],
            start_date,
            prescription_type,
            end_date: start_date + duration,
        }
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
    fn creates_prescription_with_30_days_duration_when_type_is_regular() {
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
            Some(PrescriptionType::ForChronicDiseasesDrugs),
        );

        assert_eq!(
            sut.prescription_type,
            PrescriptionType::ForChronicDiseasesDrugs
        );
        assert_eq!(sut.start_date, timestamp);
        assert_eq!(sut.end_date, timestamp + Duration::days(365));
    }
}
