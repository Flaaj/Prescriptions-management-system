use pwhash::bcrypt;

pub struct Hasher {}

impl Hasher {
    pub fn hash_password(pass: &str) -> String {
        bcrypt::hash(pass).unwrap()
    }

    pub fn verify_password(pass: &str, hash: &str) -> bool {
        bcrypt::verify(pass, hash)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hashes_password() {
        let pass = "password";
        let hash = Hasher::hash_password(pass);
        assert_ne!(pass, hash);
        assert!(Hasher::verify_password(pass, &hash));
    }
}
