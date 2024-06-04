#[derive(thiserror::Error, Debug)]
pub enum NameValidationError {
    #[error("Name must be between {0} and {1} characters long")]
    InvalidLength(usize, usize),
    #[error("Name must be in format: Firstname Lastname")]
    InvalidFormat,
}

pub fn validate_name(name: &str) -> anyhow::Result<()> {
    let min_len: usize = 4;
    let max_len: usize = 100;
    if name.len() < min_len || name.len() > max_len {
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

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use super::validate_name;

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
        assert_eq!(validate_name(name).is_ok(), expected)
    }
}
