use std::ops::Add;

use chrono::{DateTime, Duration, Utc};
use rand::{distributions, thread_rng, Rng};
use uuid::Uuid;

pub enum PrescriptionDurations {
    MinDays = 7,
    DefaultDays = 30,
    MaxDays = 365,
}

#[derive(Debug, PartialEq)]
pub struct Prescription {
    pub id: Uuid,
    pub patient_id: Uuid,
    pub doctor_id: Uuid,
    pub prescription_code: String,
    pub prescription_start_date: DateTime<Utc>,
    pub prescription_end_date: DateTime<Utc>,
    pub date_of_issue: DateTime<Utc>,
    // pub fills: Vec<PrescriptionFill>,
}

#[derive(Debug, PartialEq)]
pub enum PrescriptionCreationError {
    PrescriptionDurationTooShort,
    PrescriptionDurationTooLong,
}

impl Prescription {
    pub fn new(
        patient_id: Uuid,
        doctor_id: Uuid,
        prescription_start_date: Option<DateTime<Utc>>,
        prescription_end_date: Option<DateTime<Utc>>,
    ) -> Result<Self, PrescriptionCreationError> {
        let prescription_start_date = match prescription_start_date {
            Some(date) => date,
            None => Utc::now(),
        };

        let prescription_end_date = match prescription_end_date {
            Some(date) => date,
            None => prescription_start_date
                .add(Duration::days(PrescriptionDurations::DefaultDays as i64)),
        };

        let prescription_duration_days =
            (prescription_end_date - prescription_start_date).num_days();

        if prescription_duration_days < PrescriptionDurations::MinDays as i64 {
            Err(PrescriptionCreationError::PrescriptionDurationTooShort)?;
        }

        if prescription_duration_days > PrescriptionDurations::MaxDays as i64 {
            Err(PrescriptionCreationError::PrescriptionDurationTooLong)?;
        }

        Ok(Self {
            id: Uuid::new_v4(),

            patient_id: patient_id,
            doctor_id: doctor_id,
            prescription_start_date: prescription_start_date,
            prescription_end_date: prescription_end_date,

            prescription_code: Self::generate_random_prescription_code(),
            date_of_issue: Utc::now(),
            // fills: vec![],
        })
    }

    fn generate_random_prescription_code() -> String {
        thread_rng()
            .sample_iter(&distributions::Alphanumeric)
            .take(10)
            .map(char::from)
            .collect::<String>()
    }
}

#[derive(Debug, PartialEq)]
pub enum PrescriptionFillStatus {
    Pending,
    Filled,
    Cancelled,
}

#[derive(Debug, PartialEq)]
pub struct PrescriptionFill {
    pub id: Uuid,
    pub prescription_id: Uuid,
    pub farmacy_id: Uuid,
    pub farmacist_id: Uuid,
    pub fill_date: Option<DateTime<Utc>>,
    pub status: PrescriptionFillStatus,
}

#[cfg(test)]
mod tests {
    use std::ops::Add;

    use chrono::{Duration, Utc};
    use uuid::Uuid;

    use crate::domain::prescriptions::component::models::prescription_model::Prescription;

    #[test]
    fn creates_prescription() {
        let example_prescription_start_date = Utc::now().add(Duration::days(42));
        let example_prescription_end_date = Utc::now().add(Duration::days(69));

        let sut = Prescription::new(
            Uuid::new_v4(),
            Uuid::new_v4(),
            Some(example_prescription_start_date),
            Some(example_prescription_end_date),
        );

        assert!(sut.is_ok());
    }

    #[test]
    fn sets_now_as_prescription_start_date_if_not_specified() {
        let sut = Prescription::new(Uuid::new_v4(), Uuid::new_v4(), None, None).unwrap();

        let diff_from_now_seconds = (sut.prescription_start_date - Utc::now()).num_seconds();
        assert_eq!(diff_from_now_seconds, 0); // very safely assuming the test ran in less than 1 second
    }

    #[test]
    fn sets_default_prescription_duration_when_not_specified() {
        let example_prescription_start_date = Utc::now().add(Duration::days(42));

        let sut = Prescription::new(
            Uuid::new_v4(),
            Uuid::new_v4(),
            Some(example_prescription_start_date),
            None,
        )
        .unwrap();

        let prescription_duration_days =
            (sut.prescription_end_date - sut.prescription_start_date).num_days();
        assert_eq!(prescription_duration_days, 30);
    }

    #[test]
    fn errors_if_trying_to_create_prescription_with_duration_shorter_than_minimum_duration() {
        let prescription_start_date = Utc::now();
        let prescription_end_date = Utc::now().add(Duration::days(6));

        let sut = Prescription::new(
            Uuid::new_v4(),
            Uuid::new_v4(),
            Some(prescription_start_date),
            Some(prescription_end_date),
        );

        assert!(sut.is_err());
    }

    #[test]
    fn succeeds_if_trying_to_create_prescription_with_duration_equal_to_minimum_duration() {
        let prescription_start_date = Utc::now();
        let prescription_end_date = Utc::now().add(Duration::days(7));

        let sut = Prescription::new(
            Uuid::new_v4(),
            Uuid::new_v4(),
            Some(prescription_start_date),
            Some(prescription_end_date),
        );

        assert!(sut.is_ok());
    }

    #[test]
    fn errors_if_trying_to_create_prescription_with_duration_longer_than_maximum_duration() {
        let prescription_start_date = Utc::now();
        let prescription_end_date = Utc::now().add(Duration::days(366));

        let sut = Prescription::new(
            Uuid::new_v4(),
            Uuid::new_v4(),
            Some(prescription_start_date),
            Some(prescription_end_date),
        );

        assert!(sut.is_err());
    }

    #[test]
    fn succeeds_if_trying_to_create_prescription_with_duration_equal_to_maximum_duration() {
        let prescription_start_date = Utc::now();
        let prescription_end_date = Utc::now().add(Duration::days(365));

        let sut = Prescription::new(
            Uuid::new_v4(),
            Uuid::new_v4(),
            Some(prescription_start_date),
            Some(prescription_end_date),
        );

        assert!(sut.is_ok());
    }

    #[test]
    fn generates_prescription_code_with_length_10_when_created() {
        let sut = Prescription::new(Uuid::new_v4(), Uuid::new_v4(), None, None);

        assert_eq!(sut.unwrap().prescription_code.len(), 10);
    }
}
