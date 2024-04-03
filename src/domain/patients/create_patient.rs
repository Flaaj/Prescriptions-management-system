use crate::utils::validators::{
    validate_name::validate_name, validate_pesel_number::validate_pesel_number,
};

pub struct NewPatient {
    pub name: String,
    pub pesel: String,
}

impl NewPatient {
    pub fn new(name: String, pesel: String) -> anyhow::Result<Self> {
        validate_name(&name)?;
        validate_pesel_number(&pesel)?;

        Ok(NewPatient { name, pesel })
    }
}

#[cfg(test)]
mod unit_tests {
    use super::NewPatient;

    #[test]
    fn creates_patient() {
        let sut = NewPatient::new("John Doe".into(), "96021817257".into()).unwrap();

        assert_eq!(sut.name, "John Doe");
        assert_eq!(sut.pesel, "96021817257");
    }

    #[test]
    fn doesnt_create_patient_if_name_is_invalid() {
        assert!(NewPatient::new("John".into(), "96021817257".into()).is_err());
    }

    #[test]
    fn doesnt_create_patient_if_pesel_number_is_invalid() {
        assert!(NewPatient::new("John Doe".into(), "92223300009".into()).is_err());
    }
}
