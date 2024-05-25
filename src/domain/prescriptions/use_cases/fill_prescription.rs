use crate::domain::prescriptions::models::{NewPrescriptionFill, Prescription};
use chrono::Utc;
use uuid::Uuid;

#[derive(thiserror::Error, Debug, PartialEq)]
pub enum PrescriptionFillError {
    #[error("Current date is not between prescription's start and end date")]
    InvalidDate,
    #[error("Prescription is already filled")]
    AlreadyFilled,
}

impl Prescription {
    pub fn fill(&self, pharmacist_id: Uuid) -> anyhow::Result<NewPrescriptionFill> {
        let now = Utc::now();
        if now < self.start_date || now > self.end_date {
            Err(PrescriptionFillError::InvalidDate)?;
        }
        if self.fill.is_some() {
            Err(PrescriptionFillError::AlreadyFilled)?;
        }

        Ok(NewPrescriptionFill {
            id: Uuid::new_v4(),
            pharmacist_id,
            prescription_id: self.id,
        })
    }
}

#[cfg(test)]
mod tests {
    use chrono::{Duration, Utc};
    use uuid::Uuid;

    use crate::domain::prescriptions::{
        models::{
            PrescribedDrug, Prescription, PrescriptionDoctor, PrescriptionFill,
            PrescriptionPatient, PrescriptionType,
        },
        use_cases::fill_prescription::PrescriptionFillError,
    };

    fn create_mock_prescription() -> Prescription {
        let prescription_id = Uuid::new_v4();
        let prescription_type = PrescriptionType::Regular;
        let start_date = Utc::now() - Duration::hours(1);
        let end_date = start_date + prescription_type.get_duration();

        Prescription {
            id: prescription_id,
            doctor: PrescriptionDoctor {
                id: Uuid::new_v4(),
                name: "John Doctor".to_string(),
                pesel_number: "99031301347".to_string(),
                pwz_number: "8463856".to_string(),
            },
            patient: PrescriptionPatient {
                id: Uuid::new_v4(),
                name: "John Patient".to_string(),
                pesel_number: "92022900002".to_string(),
            },
            code: "12345678".to_string(),
            prescription_type,
            start_date,
            end_date,
            prescribed_drugs: vec![PrescribedDrug {
                id: Uuid::new_v4(),
                drug_id: Uuid::new_v4(),
                prescription_id,
                quantity: 1,
                created_at: start_date,
                updated_at: start_date,
            }],
            fill: None,
            created_at: start_date,
            updated_at: start_date,
        }
    }

    #[test]
    fn fills_prescription() {
        let prescription = create_mock_prescription();
        let pharmacist_id = Uuid::new_v4();

        let sut = prescription.fill(pharmacist_id);

        assert!(sut.is_ok())
    }

    #[test]
    fn doesnt_fill_if_prescription_the_date_is_before_start_date() {
        let mut prescription = create_mock_prescription();
        prescription.start_date = Utc::now() + Duration::minutes(1);
        let pharmacist_id = Uuid::new_v4();

        let sut = prescription.fill(pharmacist_id);

        let expected_err = PrescriptionFillError::InvalidDate;
        assert_eq!(sut.unwrap_err().downcast_ref(), Some(&expected_err));
    }

    #[test]
    fn doesnt_fill_if_prescription_the_date_is_after_end_date() {
        let mut prescription: Prescription = create_mock_prescription();
        prescription.end_date = Utc::now() - Duration::minutes(1);
        let pharmacist_id = Uuid::new_v4();

        let sut = prescription.fill(pharmacist_id);

        let expected_err = PrescriptionFillError::InvalidDate;
        assert_eq!(sut.unwrap_err().downcast_ref(), Some(&expected_err));
    }

    #[test]
    fn doesnt_fill_if_prescription_is_filled() {
        let mut prescription = create_mock_prescription();
        prescription.fill = Some(PrescriptionFill {
            id: Uuid::new_v4(),
            pharmacist_id: Uuid::new_v4(),
            prescription_id: prescription.id,
            created_at: Utc::now() - Duration::hours(1),
            updated_at: Utc::now() - Duration::hours(1),
        });

        let sut = prescription.fill(Uuid::new_v4());

        let expected_err = PrescriptionFillError::AlreadyFilled;
        assert_eq!(sut.unwrap_err().downcast_ref(), Some(&expected_err));
    }
}
