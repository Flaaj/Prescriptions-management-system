use std::sync::RwLock;

use async_trait::async_trait;
use chrono::Utc;
use uuid::Uuid;

use crate::domain::{
    doctors::models::{Doctor, NewDoctor},
    utils::pagination::get_pagination_params,
};

use super::repository::DoctorsRepository;

pub struct FakeDoctorsRepository {
    doctors: RwLock<Vec<Doctor>>,
}

impl FakeDoctorsRepository {
    #[allow(dead_code)]
    pub fn new() -> Self {
        Self {
            doctors: RwLock::new(vec![]),
        }
    }
}

#[async_trait]
impl DoctorsRepository for FakeDoctorsRepository {
    async fn create_doctor(&self, new_doctor: NewDoctor) -> anyhow::Result<Doctor> {
        let does_pwz_or_pesel_number_exist = self.doctors.read().unwrap().iter().any(|doctor| {
            doctor.pwz_number == new_doctor.pwz_number
                || doctor.pesel_number == new_doctor.pesel_number
        });

        if does_pwz_or_pesel_number_exist {
            return Err(anyhow::anyhow!("PWZ or PESEL number already exists"));
        }

        let doctor = Doctor {
            id: new_doctor.id,
            name: new_doctor.name,
            pwz_number: new_doctor.pwz_number,
            pesel_number: new_doctor.pesel_number,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        self.doctors.write().unwrap().push(doctor.clone());

        Ok(doctor)
    }

    async fn get_doctors(
        &self,
        page: Option<i64>,
        page_size: Option<i64>,
    ) -> anyhow::Result<Vec<Doctor>> {
        let (page_size, offset) = get_pagination_params(page, page_size)?;
        let a = offset;
        let b = offset + page_size;

        let mut doctors: Vec<Doctor> = vec![];
        for i in a..b {
            match self.doctors.read().unwrap().get(i as usize) {
                Some(doctor) => doctors.push(doctor.clone()),
                None => {}
            }
        }

        Ok(doctors)
    }

    async fn get_doctor_by_id(&self, doctor_id: Uuid) -> anyhow::Result<Doctor> {
        match self
            .doctors
            .read()
            .unwrap()
            .iter()
            .find(|doctor| doctor.id == doctor_id)
        {
            Some(doctor) => Ok(doctor.clone()),
            None => Err(anyhow::anyhow!("Doctor not found")),
        }
    }
}

#[cfg(test)]
// the same tests as in postgres_repository_impl/doctors.rs to make sure mocks work the same way
mod mock_tests {
    use uuid::Uuid;

    use super::FakeDoctorsRepository;
    use crate::domain::doctors::{models::NewDoctor, repository::DoctorsRepository};

    #[tokio::test]
    async fn create_and_read_doctor_by_id() {
        let repository = FakeDoctorsRepository::new();

        let new_doctor =
            NewDoctor::new("John Does".into(), "5425740".into(), "96021817257".into()).unwrap();

        repository.create_doctor(new_doctor.clone()).await.unwrap();

        let doctor_from_repo = repository.get_doctor_by_id(new_doctor.id).await.unwrap();

        assert_eq!(doctor_from_repo, new_doctor);
    }

    #[tokio::test]
    async fn returns_error_if_doctor_with_given_id_doesnt_exist() {
        let repository = FakeDoctorsRepository::new();

        let doctor_from_repo = repository.get_doctor_by_id(Uuid::new_v4()).await;

        assert!(doctor_from_repo.is_err());
    }

    #[tokio::test]
    async fn create_and_read_doctors_from_database() {
        let repository = FakeDoctorsRepository::new();

        let new_doctor_0 =
            NewDoctor::new("John First".into(), "5425740".into(), "96021817257".into()).unwrap();
        let new_doctor_1 =
            NewDoctor::new("John Second".into(), "8463856".into(), "99031301347".into()).unwrap();
        let new_doctor_2 =
            NewDoctor::new("John Third".into(), "3123456".into(), "92022900002".into()).unwrap();
        let new_doctor_3 =
            NewDoctor::new("John Fourth".into(), "5425751".into(), "96021807250".into()).unwrap();

        repository
            .create_doctor(new_doctor_0.clone())
            .await
            .unwrap();
        repository
            .create_doctor(new_doctor_1.clone())
            .await
            .unwrap();
        repository
            .create_doctor(new_doctor_2.clone())
            .await
            .unwrap();
        repository
            .create_doctor(new_doctor_3.clone())
            .await
            .unwrap();

        let doctors = repository.get_doctors(None, Some(10)).await.unwrap();

        assert!(doctors.len() == 4);
        assert_eq!(doctors[0], new_doctor_0);
        assert_eq!(doctors[1], new_doctor_1);
        assert_eq!(doctors[2], new_doctor_2);
        assert_eq!(doctors[3], new_doctor_3);

        let doctors = repository.get_doctors(None, Some(2)).await.unwrap();

        assert_eq!(doctors.len(), 2);
        assert_eq!(doctors[0], new_doctor_0);
        assert_eq!(doctors[1], new_doctor_1);

        let doctors = repository.get_doctors(Some(1), Some(3)).await.unwrap();

        assert!(doctors.len() == 1);
        assert_eq!(doctors[0], new_doctor_3);

        let doctors = repository.get_doctors(Some(2), Some(3)).await.unwrap();

        assert!(doctors.len() == 0);
    }

    #[tokio::test]
    async fn doesnt_create_doctor_if_pwz_or_pesel_numbers_are_duplicated() {
        let repository = FakeDoctorsRepository::new();

        let doctor =
            NewDoctor::new("John Doe".into(), "5425740".into(), "96021817257".into()).unwrap();

        assert!(repository.create_doctor(doctor).await.is_ok());

        let doctor_with_duplicated_pwz_number =
            NewDoctor::new("John Doe".into(), "5425740".into(), "99031301347".into()).unwrap();

        assert!(repository
            .create_doctor(doctor_with_duplicated_pwz_number)
            .await
            .is_err());

        let doctor_with_duplicated_pesel_number =
            NewDoctor::new("John Doe".into(), "3123456".into(), "96021817257".into()).unwrap();

        assert!(repository
            .create_doctor(doctor_with_duplicated_pesel_number)
            .await
            .is_err());
    }
}
