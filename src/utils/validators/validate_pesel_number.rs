use chrono::NaiveDate;

#[derive(thiserror::Error, Debug)]
pub enum PeselNumberValidationError {
    #[error("PESEL number must be 11 characters long and contain only digits")]
    InvalidFormat,
    #[error("The date part of PESEL number is incorrect")]
    InvalidDate,
    #[error("The checksum of PESEL number is incorrect")]
    InvalidChecksum,
}

pub fn validate_pesel_number(pesel_number: &str) -> anyhow::Result<()> {
    let pesel_length = 11;
    if pesel_number.len() != pesel_length || pesel_number.parse::<u64>().is_err() {
        Err(PeselNumberValidationError::InvalidFormat)?;
    }

    let (date_part, _) = pesel_number.split_at(6);
    let is_valid_date = NaiveDate::parse_from_str(&format!("19{}", date_part), "%Y%m%d").is_ok();
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

#[cfg(test)]
mod unit_tests {
    use super::validate_pesel_number;
    use rstest::rstest;

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
        assert_eq!(validate_pesel_number(pesel_number).is_ok(), expected);
    }
}
