use uuid::Uuid;

use crate::application::{authentication::models::{NewUser, UserRole}, helpers::hashing::Hasher};

#[derive(thiserror::Error, Debug, PartialEq)]
pub enum CreateNewUserError {
    #[error("Doctor id is required for doctor user")]
    DoctorIdRequired,
    #[error("Pharmacist id is required for pharmacist user")]
    PharmacistIdRequired,
}

impl NewUser {
    pub fn new(
        username: String,
        password: String,
        email: String,
        phone_number: String,
        role: UserRole,
        doctor_id: Option<Uuid>,
        pharmacist_id: Option<Uuid>,
    ) -> anyhow::Result<Self> {
        if role == UserRole::Doctor && doctor_id.is_none() {
            Err(CreateNewUserError::DoctorIdRequired)?;
        }
        if role == UserRole::Pharmacist && pharmacist_id.is_none() {
            Err(CreateNewUserError::PharmacistIdRequired)?;
        }

        Ok(Self {
            id: Uuid::new_v4(),
            username,
            password_hash: Hasher::hash_password(&password),
            email,
            phone_number,
            role,
            doctor_id,
            pharmacist_id,
        })
    }
}

#[cfg(test)]
mod tests {
    use uuid::Uuid;

    use crate::application::{
        authentication::models::{NewUser, UserRole},
        helpers::hashing::Hasher,
    };

    #[test]
    fn creates_new_user() {
        NewUser::new(
            "username".to_string(),
            "password".to_string(),
            "email@gmail.com".to_string(),
            "123456789".to_string(),
            UserRole::Doctor,
            Some(Uuid::default()),
            None,
        )
        .unwrap();

        NewUser::new(
            "username".to_string(),
            "password".to_string(),
            "email@gmail.com".to_string(),
            "123456789".to_string(),
            UserRole::Pharmacist,
            None,
            Some(Uuid::default()),
        )
        .unwrap();
    }

    #[test]
    fn requires_doctor_id_for_doctor_and_pharmacist_id_for_pharmacist() {
        NewUser::new(
            "username".to_string(),
            "password".to_string(),
            "email@gmail.com".to_string(),
            "123456789".to_string(),
            UserRole::Doctor,
            None,
            Some(Uuid::default()),
        )
        .unwrap_err();

        NewUser::new(
            "username".to_string(),
            "password".to_string(),
            "email@gmail.com".to_string(),
            "123456789".to_string(),
            UserRole::Doctor,
            None,
            Some(Uuid::default()),
        )
        .unwrap_err();
    }

    #[test]
    fn hashes_users_password() {
        let pass = "password".to_string();
        let user = NewUser::new(
            "username".to_string(),
            pass.clone(),
            "email@gmail.com".to_string(),
            "123456789".to_string(),
            UserRole::Doctor,
            Some(Uuid::default()),
            None,
        )
        .unwrap();

        assert!(Hasher::verify_password(&pass, &user.password_hash));
    }
}
