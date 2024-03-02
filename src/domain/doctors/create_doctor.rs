use chrono::NaiveDate;

pub struct NewDoctor {
    pub name: String,
    pub pwz_number: String,
    pub pesel_number: String,
}

impl NewDoctor {
    pub fn new(name: String, pwz_number: String, pesel_number: String) -> anyhow::Result<Self> {
        Self::validate_name(&name)?;
        Self::validate_pesel_number(&pesel_number)?;
        Self::validate_pwz_number(&pwz_number)?;

        Ok(NewDoctor {
            name,
            pwz_number,
            pesel_number,
        })
    }

    fn validate_pesel_number(pesel_number: &str) -> anyhow::Result<()> {
        let pesel_length = 11;
        if pesel_number.len() != pesel_length || pesel_number.parse::<u64>().is_err() {
            anyhow::bail!("Provided PESEL number is not a valid PESEL number.");
        }

        let (date_part, _) = pesel_number.split_at(6);
        let is_valid_date =
            NaiveDate::parse_from_str(&format!("19{}", date_part), "%Y%m%d").is_ok();
        if !is_valid_date {
            anyhow::bail!("The date part of provided PESEL number is incorrect");
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
            anyhow::bail!("The checksum of provided PESEL number is incorrect")
        }

        Ok(())
    }

    fn validate_pwz_number(pwz_number: &str) -> anyhow::Result<()> {
        let pwz_number_length = 7;
        if pwz_number.len() != pwz_number_length || pwz_number.parse::<u32>().is_err() {
            anyhow::bail!("Provided PWZ number is not a valid PWZ number.");
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
            anyhow::bail!("The checksum of provided PWZ number is incorrect")
        }

        Ok(())
    }

    fn validate_name(name: &str) -> anyhow::Result<()> {
        if name.len() < 4 || name.len() > 100 {
            anyhow::bail!("Name must be between 4 and 100 characters long");
        }

        let word_count = name.split(' ').count();
        if word_count < 2 {
            anyhow::bail!("Name must be in format: Firstname Lastname");
        }

        let is_pascal_case = name.split(|c| c == ' ' || c == '-').all(|word| {
            let is_first_uppercase = word.chars().next().unwrap().is_uppercase();
            let is_rest_lowercase = word.chars().skip(1).all(|c| c.is_lowercase());
            is_first_uppercase && is_rest_lowercase
        });
        if !is_pascal_case {
            anyhow::bail!("Name must be in format: Firstname Lastname");
        }
        Ok(())
    }
}

#[cfg(test)]
mod unit_tests {
    use rstest::rstest;

    use crate::domain::doctors::create_doctor::NewDoctor;

    #[test]
    fn creates_doctor() {
        let sut =
            NewDoctor::new("John Doe".into(), "5425740".into(), "96021817257".into()).unwrap();

        assert_eq!(sut.name, "John Doe");
        assert_eq!(sut.pwz_number, "5425740");
        assert_eq!(sut.pesel_number, "96021817257");
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
            NewDoctor::validate_pesel_number(pesel_number).is_ok(),
            expected
        );
    }

    #[test]
    fn doesnt_create_doctor_if_pesel_number_is_invalid() {
        assert!(NewDoctor::new("John Doe".into(), "4123456".into(), "92223300009".into()).is_err());
    }

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
        assert_eq!(NewDoctor::validate_pwz_number(pwz_number).is_ok(), expected)
    }

    #[test]
    fn doesnt_create_doctor_if_pwz_number_is_invalid() {
        assert!(NewDoctor::new("John Doe".into(), "1234567".into(), "96021817257".into()).is_err());
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
        assert_eq!(NewDoctor::validate_name(name).is_ok(), expected)
    }

    #[test]
    fn doesnt_create_doctor_if_name_is_invalid() {
        assert!(NewDoctor::new("John".into(), "5425740".into(), "96021817257".into()).is_err());
    }
}
