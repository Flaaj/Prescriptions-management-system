struct NewDoctor {
    name: String,
    pzw_number: String,
    pesel: String,
}

impl NewDoctor {
    pub fn new(name: &str, pzw_number: &str, pesel: &str) -> NewDoctor {
        NewDoctor {
            name: name.to_string(),
            pzw_number: pzw_number.to_string(),
            pesel: pesel.to_string(),
        }
    }
}

#[cfg(test)]
mod unit_tests {
    use crate::domain::doctors::create_doctor::NewDoctor;

    #[test]
    fn creates_doctor() {
        let sut = NewDoctor::new("John Doe", "1234567", "88112200000");

        assert_eq!(sut.name, "John Doe");
        assert_eq!(sut.pzw_number, "1234567");
        assert_eq!(sut.pesel, "88112200000");
    }
}
