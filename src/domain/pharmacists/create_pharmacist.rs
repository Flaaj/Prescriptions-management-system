use uuid::Uuid;

use crate::utils::validators::{
    validate_name::validate_name, validate_pesel_number::validate_pesel_number,
};

pub struct NewPharmacist {
    pub id: Uuid,
    pub name: String,
    pub pesel: String,
}

impl NewPharmacist {
    pub fn new(name: String, pesel: String) -> anyhow::Result<Self> {
        validate_name(&name)?;
        validate_pesel_number(&pesel)?;

        Ok(NewPharmacist {
            id: Uuid::new_v4(),
            name,
            pesel,
        })
    }
}

#[cfg(test)]
mod unit_tests {
    use super::NewPharmacist;

    #[test]
    fn creates_pharmacist() {
        let sut = NewPharmacist::new("John Doe".into(), "96021817257".into()).unwrap();

        assert_eq!(sut.name, "John Doe");
        assert_eq!(sut.pesel, "96021817257");
    }

    #[test]
    fn doesnt_create_pharmacist_if_name_is_invalid() {
        assert!(NewPharmacist::new("John".into(), "96021817257".into()).is_err());
    }

    #[test]
    fn doesnt_create_pharmacist_if_pesel_number_is_invalid() {
        assert!(NewPharmacist::new("John Doe".into(), "92223300009".into()).is_err());
    }
}
