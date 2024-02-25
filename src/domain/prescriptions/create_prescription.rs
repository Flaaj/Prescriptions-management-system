use chrono::{Date, DateTime, Duration, Utc};
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
    fn new(doctor_id: Uuid, patient_id: Uuid, start_date: DateTime<Utc>) -> Self {
        Self {
            doctor_id,
            patient_id,
            prescribed_drugs: vec![],
            start_date,
            end_date: start_date + Duration::days(30),
        }
    }
}

#[cfg(test)]
mod test {

    use chrono::{Duration, Utc};
    use uuid::Uuid;

    use crate::domain::prescriptions::create_prescription::NewPrescription;

    #[test]
    fn creates_prescription() {
        let doctor_id = Uuid::parse_str("11111111-1111-1111-1111-111111111111").unwrap();
        let patient_id = Uuid::parse_str("22222222-2222-2222-2222-222222222222").unwrap();
        let timestamp = Utc::now();

        let sut = NewPrescription::new(doctor_id, patient_id, timestamp);

        let expected_prescription = NewPrescription {
            doctor_id,
            patient_id,
            prescribed_drugs: vec![],
            start_date: timestamp,
            end_date: timestamp + Duration::days(30),
        };
        assert_eq!(sut, expected_prescription)
    }
}
