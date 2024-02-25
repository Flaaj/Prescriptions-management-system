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
        Self {
            doctor_id,
            patient_id,
            prescribed_drugs: vec![],
            start_date: Utc::now(),
            end_date: Utc::now() + Duration::days(30),
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
        let doctor_id = Uuid::parse_str("11111111-1111-1111-1111-111111111111").unwrap();
        let patient_id = Uuid::parse_str("22222222-2222-2222-2222-222222222222").unwrap();

        let sut = NewPrescription::new(doctor_id, patient_id, None, None);

        assert_eq!(sut.doctor_id, doctor_id);
        assert_eq!(sut.patient_id, patient_id);
        assert_eq!(sut.prescribed_drugs, vec![]);
    }
}
