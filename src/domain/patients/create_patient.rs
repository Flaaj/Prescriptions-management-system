use chrono::NaiveDate;

pub struct NewPatient {
    pub name: String,
    pub pesel: String,
}

#[derive(thiserror::Error, Debug)]
enum PeselNumberValidationError {
    #[error("PESEL number must be 11 characters long and contain only digits")]
    InvalidFormat,
    #[error("The date part of PESEL number is incorrect")]
    InvalidDate,
    #[error("The checksum of PESEL number is incorrect")]
    InvalidChecksum,
}

#[derive(thiserror::Error, Debug)]
enum NameValidationError {
    #[error("Name must be between {0} and {1} characters long")]
    InvalidLength(u16, u16),
    #[error("Name must be in format: Firstname Lastname")]
    InvalidFormat,
}

impl NewPatient {
    pub fn new(name: String, pesel: String) -> anyhow::Result<Self> {
        Self::validate_name(&name)?;
        Self::validate_pesel_number(&pesel)?;

        Ok(NewPatient { name, pesel })
    }

    fn validate_name(name: &str) -> anyhow::Result<()> {
        let min_len: u16 = 4;
        let max_len: u16 = 100;
        if name.len() < min_len.into() || name.len() > max_len.into() {
            Err(NameValidationError::InvalidLength(min_len, max_len))?;
        }

        let word_count = name.split(' ').count();
        if word_count < 2 {
            Err(NameValidationError::InvalidFormat)?;
        }

        let is_pascal_case = name.split(|c| c == ' ' || c == '-').all(|word| {
            let mut chars = word.chars();
            let is_first_uppercase = chars.next().unwrap().is_uppercase();
            let is_rest_lowercase = chars.all(|c| c.is_lowercase());
            is_first_uppercase && is_rest_lowercase
        });
        if !is_pascal_case {
            Err(NameValidationError::InvalidFormat)?;
        }

        Ok(())
    }

    fn validate_pesel_number(pesel_number: &str) -> anyhow::Result<()> {
        let pesel_length = 11;
        if pesel_number.len() != pesel_length || pesel_number.parse::<u64>().is_err() {
            Err(PeselNumberValidationError::InvalidFormat)?;
        }

        let (date_part, _) = pesel_number.split_at(6);
        let is_valid_date =
            NaiveDate::parse_from_str(&format!("19{}", date_part), "%Y%m%d").is_ok();
        if !is_valid_date {
            Err(PeselNumberValidationError::InvalidDate)?;
        }

        let (checksum_components, control_digit_str) = pesel_number.split_at(10);
        let digit_multipliters = [1, 3, 7, 9, 1, 3, 7, 9, 1, 3];
        let mut sum = 0;
        for (i, c) in checksum_components.chars().enumerate() {
            let digit = c.to_digit(10).unwrap();
            let multiplier = digit_multipliters[i];
            sum += digit * multiplier;
        }
        let control_digit = control_digit_str.parse::<u32>().unwrap();
        let checksum = sum % 10;
        if checksum != control_digit {
            Err(PeselNumberValidationError::InvalidChecksum)?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod unit_tests {
    use rstest::rstest;

    use super::NewPatient;

    #[test]
    fn creates_patient() {
        let sut = NewPatient::new("John Doe".into(), "96021817257".into());

        assert!(sut.is_ok());
    }

    #[rstest]
    #[case("John Doe", true)]
    #[case("Mark Zuckerberg", true)]
    #[case("Anne Pattison Clark", true)]
    #[case("Karl Heinz-Müller", true)]
    #[case("Ędward Żądło", true)]
    #[case("Hu Ho", true)]
    #[case("Aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa Aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa", true)]
    #[case("John", false)]
    #[case("John doe", false)]
    #[case("john Doe", false)]
    #[case("JOhn Doe", false)]
    #[case("john doe", false)]
    #[case("John-Doe", false)]
    #[case("J D", false)]
    #[case("", false)]
    #[case("AaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaAaaaaaaaaaaaaaaaaaa Aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa", false)]
    #[case("Aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa Aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa", false)]
    fn validates_name(#[case] name: &str, #[case] expected: bool) {
        assert_eq!(NewPatient::validate_name(name).is_ok(), expected)
    }

    #[test]
    fn doesnt_create_patient_if_name_is_invalid() {
        assert!(NewPatient::new("John".into(), "96021817257".into()).is_err());
    }

    #[rstest]
    #[case("96021817257", true)]
    #[case("99031301347", true)]
    #[case("92022900002", true)]
    #[case("96221807250", false)]
    #[case("96021807251", false)]
    #[case("93022900005", false)]
    #[case("92223300009", false)]
    #[case("9222330000a", false)]
    #[case("aaaaaaaaaaa", false)]
    #[case("960218072500", false)]
    #[case("30", false)]
    #[case("", false)]
    fn validates_pesel_number(#[case] pesel_number: &str, #[case] expected: bool) {
        assert_eq!(
            NewPatient::validate_pesel_number(pesel_number).is_ok(),
            expected
        );
    }

    #[test]
    fn doesnt_create_patient_if_pesel_number_is_invalid() {
        assert!(NewPatient::new("John Doe".into(), "92223300009".into()).is_err());
    }
}
