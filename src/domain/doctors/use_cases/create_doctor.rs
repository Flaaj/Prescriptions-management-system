use uuid::Uuid;

use crate::domain::{
    doctors::models::NewDoctor,
    utils::validators::{
        validate_name::validate_name, validate_pesel_number::validate_pesel_number,
        validate_pwz_number::validate_pwz_number,
    },
};

impl NewDoctor {
    pub fn new(name: String, pwz_number: String, pesel_number: String) -> anyhow::Result<Self> {
        validate_name(&name)?;
        validate_pesel_number(&pesel_number)?;
        validate_pwz_number(&pwz_number)?;

        Ok(NewDoctor {
            id: Uuid::new_v4(),
            name,
            pwz_number,
            pesel_number,
        })
    }
}

#[cfg(test)]
mod unit_tests {
    use crate::domain::doctors::models::NewDoctor;

    #[test]
    fn creates_doctor() {
        let sut =
            NewDoctor::new("John Doe".into(), "5425740".into(), "96021817257".into()).unwrap();

        assert_eq!(sut.name, "John Doe");
        assert_eq!(sut.pwz_number, "5425740");
        assert_eq!(sut.pesel_number, "96021817257");
    }

    #[test]
    fn doesnt_create_doctor_if_pesel_number_is_invalid() {
        assert!(NewDoctor::new("John Doe".into(), "4123456".into(), "92223300009".into()).is_err());
    }

    #[test]
    fn doesnt_create_doctor_if_pwz_number_is_invalid() {
        assert!(NewDoctor::new("John Doe".into(), "1234567".into(), "96021817257".into()).is_err());
    }

    #[test]
    fn doesnt_create_doctor_if_name_is_invalid() {
        assert!(NewDoctor::new("John".into(), "5425740".into(), "96021817257".into()).is_err());
    }
}
