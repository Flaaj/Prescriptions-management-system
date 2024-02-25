use chrono::{DateTime, FixedOffset};
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
struct NewPrescription {
    doctor_id: Uuid,
    patient_id: Uuid,
    drug_ids: Vec<Uuid>,
    start_date: DateTime<FixedOffset>,
    end_date: DateTime<FixedOffset>,
}

impl NewPrescription {}

#[cfg(test)]
mod test {

    use chrono::{Date, DateTime};
    use uuid::Uuid;

    use crate::domain::prescriptions::create_prescription::NewPrescription;

    // #[test]
    // fn creates_prescription() {
    //     let new_prescription = NewPrescription {
    //         doctor_id: Uuid::parse_str("11111111-1111-1111-1111-111111111111").unwrap(),
    //         patient_id: Uuid::parse_str("22222222-2222-2222-2222-222222222222").unwrap(),
    //         prescribed_drugs: vec![],
    //         start_date: DateTime::n
    //     };
    //     assert_eq!(prescription, new_prescription)
    // }
}
