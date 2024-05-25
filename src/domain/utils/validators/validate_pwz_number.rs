#[derive(thiserror::Error, Debug)]
pub enum PwzNumberValidationError {
    #[error("PWZ number must be 7 characters long and contain only digits")]
    InvalidFormat,
    #[error("The checksum of PWZ number is incorrect")]
    InvalidChecksum,
}

pub fn validate_pwz_number(pwz_number: &str) -> anyhow::Result<()> {
    let pwz_number_length = 7;
    if pwz_number.len() != pwz_number_length || pwz_number.parse::<u32>().is_err() {
        Err(PwzNumberValidationError::InvalidFormat)?;
    }

    let (control_digit_str, checksum_components) = pwz_number.split_at(1);

    let sum = checksum_components
        .chars()
        .enumerate()
        .fold(0, |acc, (i, c)| {
            let digit = c.to_digit(10).unwrap();
            acc + digit * (i + 1) as u32
        });

    let control_digit = control_digit_str.parse::<u32>().unwrap();
    let checksum = sum % 11;
    if checksum != control_digit {
        Err(PwzNumberValidationError::InvalidChecksum)?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::validate_pwz_number;
    use rstest::rstest;

    #[rstest]
    #[case("5425740", true)]
    #[case("8463856", true)]
    #[case("3123456", true)]
    #[case("4425740", false)]
    #[case("1234567", false)]
    #[case("aaaaaaa", false)]
    #[case("1111111", false)]
    #[case("111111a", false)]
    #[case("11111111", false)]
    #[case("30", false)]
    #[case("", false)]
    fn validates_pwz_number(#[case] pwz_number: &str, #[case] expected: bool) {
        assert_eq!(validate_pwz_number(pwz_number).is_ok(), expected)
    }
}
