use chrono::NaiveDate;

struct NewDoctor {
    name: String,
    pzw_number: String,
    pesel: String,
}

impl NewDoctor {
    pub fn new(name: String, pzw_number: String, pesel: String) -> anyhow::Result<Self> {
        Self::validate_pesel_number(&pesel)?;

        Ok(NewDoctor {
            name,
            pzw_number,
            pesel,
        })
    }

    fn validate_pesel_number(pesel: &str) -> anyhow::Result<()> {
        let error_str = "Provided PESEL number is not a valid PESEL.";
        if pesel.len() != 11 || pesel.parse::<u64>().is_err() {
            anyhow::bail!(error_str);
        }

        let (date_part, _) = pesel.split_at(6);
        let is_valid_date: bool =
            NaiveDate::parse_from_str(&format!("19{}", date_part), "%Y%m%d").is_ok();
        if !is_valid_date {
            anyhow::bail!(error_str);
        }

        let digit_multipliters = [1, 3, 7, 9, 1, 3, 7, 9, 1, 3];
        let mut sum = 0;
        for (i, c) in pesel.chars().enumerate() {
            if i < 10 {
                let digit = c.to_digit(10).unwrap();
                let multiplier = digit_multipliters[i];
                sum += digit * multiplier;
            }
        }
        let last_digit = pesel.chars().last().unwrap().to_digit(10).unwrap();
        let checksum = sum % 10;
        if last_digit != checksum {
            anyhow::bail!(error_str)
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
            NewDoctor::new("John Doe".into(), "1234567".into(), "96021807250".into()).unwrap();

        assert_eq!(sut.name, "John Doe");
        assert_eq!(sut.pzw_number, "1234567");
        assert_eq!(sut.pesel, "96021807250");
    }

    #[test]
    fn doesnt_create_doctor_if_pesel_is_invalid() {
        let sut = NewDoctor::new("John Doe".into(), "1234567".into(), "92223300009".into());
        assert!(sut.is_err());
    }

    #[test]
    fn validates_pesel_number() {
        let valid_pesel_numbers: Vec<&str> = vec!["96021807250", "99031301347", "92022900002"];
        let invalid_pesel_numbers: Vec<&str> = vec![
            "",
            "30",
            "96221807250",
            "96021807251",
            "93022900005",
            "92223300009",
            "962218072500",
        ];

        for valid_pesel in valid_pesel_numbers {
            assert_eq!(
                NewDoctor::validate_pesel_number(valid_pesel.into()).is_ok(),
                true
            )
        }

        for invalid_pesel in invalid_pesel_numbers {
            assert_eq!(
                NewDoctor::validate_pesel_number(invalid_pesel.into()).is_err(),
                true
            )
        }
    }
}
