use chrono::NaiveDate;

pub struct NewDoctor {
    name: String,
    pwz_number: String,
    pesel_number: String,
}

impl NewDoctor {
    pub fn new(name: String, pwz_number: String, pesel_number: String) -> anyhow::Result<Self> {
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
        let mut sum = 0;
        for (i, c) in checksum_components.chars().enumerate() {
            let digit = c.to_digit(10).unwrap();
            sum += digit * (i + 1) as u32;
        }
        let control_digit = control_digit_str.parse::<u32>().unwrap() % 10;
        let checksum = sum % 11;
        if checksum != control_digit {
            anyhow::bail!("The checksum of provided PWZ number is incorrect")
        }

        Ok(())
    }
}

#[cfg(test)]
mod unit_tests {
    use crate::domain::doctors::create_doctor::NewDoctor;

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
        let sut = NewDoctor::new("John Doe".into(), "4123456".into(), "92223300009".into());
        assert!(sut.is_err());
    }

    #[test]
    fn doesnt_create_doctor_if_pwz_number_is_invalid() {
        let sut = NewDoctor::new("John Doe".into(), "1234567".into(), "96021817257".into());
        assert!(sut.is_err());
    }

    #[test]
    fn validates_pesel_number() {
        let valid_pesel_numbers: Vec<&str> = vec!["96021817257", "99031301347", "92022900002"];
        let invalid_pesel_numbers: Vec<&str> = vec![
            "",
            "30",
            "96221807250",
            "96021807251",
            "93022900005",
            "92223300009",
            "9222330000a",
            "aaaaaaaaaaa",
            "962218072500",
        ];

        for valid_pesel in valid_pesel_numbers {
            assert!(NewDoctor::validate_pesel_number(valid_pesel).is_ok())
        }

        for invalid_pesel in invalid_pesel_numbers {
            assert!(NewDoctor::validate_pesel_number(invalid_pesel).is_err())
        }
    }

    #[test]
    fn validates_pwz_number() {
        let valid_pwz_numbers: Vec<&str> = vec!["5425740", "8463856", "3123456"];
        let invalid_pwz_numbers: Vec<&str> = vec![
            "", "30", "1111111", "aaaaaaa", "111111a", "11111111", "4425740",
        ];

        for valid_pwz in valid_pwz_numbers {
            assert!(NewDoctor::validate_pwz_number(valid_pwz).is_ok())
        }

        for invalid_pwz in invalid_pwz_numbers {
            assert!(NewDoctor::validate_pwz_number(invalid_pwz).is_err())
        }
    }
}
