use super::{
    authentication::PostgresAuthenticationRepository, create_tables::create_tables,
    doctors::PostgresDoctorsRepository, drugs::PostgresDrugsRepository,
    patients::PostgresPatientsRepository, pharmacists::PostgresPharmacistsRepository,
    prescriptions::PostgresPrescriptionsRepository, sessions::PostgresSessionsRepository,
};
use crate::{
    application::authentication::{
        entities::{NewUser, UserRole},
        repository::AuthenticationRepository,
    },
    domain::{
        doctors::{entities::NewDoctor, repository::DoctorsRepository},
        drugs::{
            entities::{DrugContentType, NewDrug},
            repository::DrugsRepository,
        },
        patients::{entities::NewPatient, repository::PatientsRepository},
        pharmacists::{entities::NewPharmacist, repository::PharmacistsRepository},
        prescriptions::{
            entities::{NewPrescribedDrug, NewPrescription},
            repository::PrescriptionsRepository,
        },
    },
};

pub struct Repositories {
    pub pharmacists: PostgresPharmacistsRepository,
    pub patients: PostgresPatientsRepository,
    pub doctors: PostgresDoctorsRepository,
    pub drugs: PostgresDrugsRepository,
    pub prescriptions: PostgresPrescriptionsRepository,
    pub authentication: PostgresAuthenticationRepository,
    pub sessions: PostgresSessionsRepository,
}

pub struct Seeds {
    pub doctors: Vec<NewDoctor>,
    pub pharmacists: Vec<NewPharmacist>,
    pub patients: Vec<NewPatient>,
    pub drugs: Vec<NewDrug>,
    pub prescriptions: Vec<NewPrescription>,
    pub users: Vec<NewUser>,
}

pub async fn setup_test_database(pool: &sqlx::PgPool) -> (Repositories, Seeds) {
    create_tables(&pool, true).await.unwrap();

    let repositories = Repositories {
        pharmacists: PostgresPharmacistsRepository::new(pool.clone()),
        patients: PostgresPatientsRepository::new(pool.clone()),
        doctors: PostgresDoctorsRepository::new(pool.clone()),
        drugs: PostgresDrugsRepository::new(pool.clone()),
        prescriptions: PostgresPrescriptionsRepository::new(pool.clone()),
        authentication: PostgresAuthenticationRepository::new(pool.clone()),
        sessions: PostgresSessionsRepository::new(pool.clone()),
    };

    let new_doctor_0 = NewDoctor::new(
        "John Doctor First".into(),
        "5425740".into(),
        "96021817257".into(),
    )
    .unwrap();
    let new_doctor_1 = NewDoctor::new(
        "John Doctor Second".into(),
        "8463856".into(),
        "99031301347".into(),
    )
    .unwrap();
    let new_doctor_2 = NewDoctor::new(
        "John Doctor Third".into(),
        "3123456".into(),
        "92022900002".into(),
    )
    .unwrap();
    let new_doctor_3 = NewDoctor::new(
        "John Doctor Fourth".into(),
        "5425751".into(),
        "96021807250".into(),
    )
    .unwrap();

    repositories
        .doctors
        .create_doctor(new_doctor_0.clone())
        .await
        .unwrap();
    repositories
        .doctors
        .create_doctor(new_doctor_1.clone())
        .await
        .unwrap();
    repositories
        .doctors
        .create_doctor(new_doctor_2.clone())
        .await
        .unwrap();
    repositories
        .doctors
        .create_doctor(new_doctor_3.clone())
        .await
        .unwrap();

    let new_pharmacist_0 =
        NewPharmacist::new("John Pharmacist First".into(), "96021817257".into()).unwrap();
    let new_pharmacist_1 =
        NewPharmacist::new("John Pharmacist Second".into(), "99031301347".into()).unwrap();
    let new_pharmacist_2 =
        NewPharmacist::new("John Pharmacist Third".into(), "92022900002".into()).unwrap();
    let new_pharmacist_3 =
        NewPharmacist::new("John Pharmacist Fourth".into(), "96021807250".into()).unwrap();

    repositories
        .pharmacists
        .create_pharmacist(new_pharmacist_0.clone())
        .await
        .unwrap();
    repositories
        .pharmacists
        .create_pharmacist(new_pharmacist_1.clone())
        .await
        .unwrap();
    repositories
        .pharmacists
        .create_pharmacist(new_pharmacist_2.clone())
        .await
        .unwrap();
    repositories
        .pharmacists
        .create_pharmacist(new_pharmacist_3.clone())
        .await
        .unwrap();

    let new_patient_0 = NewPatient::new("John Patient First".into(), "96021817257".into()).unwrap();
    let new_patient_1 =
        NewPatient::new("John Patient Second".into(), "99031301347".into()).unwrap();
    let new_patient_2 = NewPatient::new("John Patient Third".into(), "92022900002".into()).unwrap();
    let new_patient_3 =
        NewPatient::new("John Patient Fourth".into(), "96021807250".into()).unwrap();

    repositories
        .patients
        .create_patient(new_patient_0.clone())
        .await
        .unwrap();
    repositories
        .patients
        .create_patient(new_patient_1.clone())
        .await
        .unwrap();
    repositories
        .patients
        .create_patient(new_patient_2.clone())
        .await
        .unwrap();
    repositories
        .patients
        .create_patient(new_patient_3.clone())
        .await
        .unwrap();

    let new_drug_0 = NewDrug::new(
        "Gripex".into(),
        DrugContentType::SolidPills,
        Some(20),
        Some(300),
        None,
        None,
    )
    .unwrap();
    let new_drug_1 = NewDrug::new(
        "Apap".into(),
        DrugContentType::SolidPills,
        Some(10),
        Some(400),
        None,
        None,
    )
    .unwrap();
    let new_drug_2 = NewDrug::new(
        "Aspirin".into(),
        DrugContentType::SolidPills,
        Some(30),
        Some(200),
        None,
        None,
    )
    .unwrap();
    let new_drug_3 = NewDrug::new(
        "Flegamax".into(),
        DrugContentType::BottleOfLiquid,
        None,
        None,
        None,
        Some(400),
    )
    .unwrap();

    repositories
        .drugs
        .create_drug(new_drug_0.clone())
        .await
        .unwrap();
    repositories
        .drugs
        .create_drug(new_drug_1.clone())
        .await
        .unwrap();
    repositories
        .drugs
        .create_drug(new_drug_2.clone())
        .await
        .unwrap();
    repositories
        .drugs
        .create_drug(new_drug_3.clone())
        .await
        .unwrap();

    let new_prescription = NewPrescription::new(
        new_doctor_0.id,
        new_patient_0.id,
        None,
        None,
        vec![
            NewPrescribedDrug {
                drug_id: new_drug_0.id,
                quantity: 1,
            },
            NewPrescribedDrug {
                drug_id: new_drug_1.id,
                quantity: 2,
            },
            NewPrescribedDrug {
                drug_id: new_drug_2.id,
                quantity: 3,
            },
            NewPrescribedDrug {
                drug_id: new_drug_3.id,
                quantity: 4,
            },
        ],
    )
    .unwrap();
    repositories
        .prescriptions
        .create_prescription(new_prescription.clone())
        .await
        .unwrap();

    for _ in 0..10 {
        let new_prescription = NewPrescription::new(
            new_doctor_0.id,
            new_patient_0.id,
            None,
            None,
            vec![
                NewPrescribedDrug {
                    drug_id: new_drug_0.id,
                    quantity: 1,
                },
                NewPrescribedDrug {
                    drug_id: new_drug_1.id,
                    quantity: 2,
                },
                NewPrescribedDrug {
                    drug_id: new_drug_2.id,
                    quantity: 3,
                },
                NewPrescribedDrug {
                    drug_id: new_drug_3.id,
                    quantity: 4,
                },
            ],
        )
        .unwrap();
        repositories
            .prescriptions
            .create_prescription(new_prescription)
            .await
            .unwrap();
    }

    let doctor_user = NewUser::new(
        "doctor_login".into(),
        "doctor_password_123!".into(),
        "john.doctor@gmail.com".into(),
        "123456789".into(),
        UserRole::Doctor,
        Some(new_doctor_0.id),
        None,
    )
    .unwrap();
    let pharmacist_user = NewUser::new(
        "pharmacist_login".into(),
        "pharmacist_password_123!".into(),
        "john.pharmacist@gmail.com".into(),
        "123456789".into(),
        UserRole::Pharmacist,
        None,
        Some(new_pharmacist_0.id),
    )
    .unwrap();

    repositories
        .authentication
        .create_user(doctor_user.clone())
        .await
        .unwrap();
    repositories
        .authentication
        .create_user(pharmacist_user.clone())
        .await
        .unwrap();

    let seeds = Seeds {
        doctors: vec![new_doctor_0, new_doctor_1, new_doctor_2, new_doctor_3],
        pharmacists: vec![
            new_pharmacist_0,
            new_pharmacist_1,
            new_pharmacist_2,
            new_pharmacist_3,
        ],
        patients: vec![new_patient_0, new_patient_1, new_patient_2, new_patient_3],
        drugs: vec![new_drug_0, new_drug_1, new_drug_2, new_drug_3],
        prescriptions: vec![new_prescription],
        users: vec![doctor_user, pharmacist_user],
    };

    (repositories, seeds)
}
