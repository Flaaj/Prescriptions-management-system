use std::sync::Arc;

use crate::{
    application::{
        authentication::{
            repository::AuthenticationRepositoryFake, service::AuthenticationService,
        },
        sessions::{repository::SessionsRepositoryFake, service::SessionsService},
    },
    domain::{
        doctors::{repository::DoctorsRepositoryFake, service::DoctorsService},
        drugs::{repository::DrugsRepositoryFake, service::DrugsService},
        patients::{repository::PatientsRepositoryFake, service::PatientsService},
        pharmacists::{repository::PharmacistsRepositoryFake, service::PharmacistsService},
        prescriptions::{repository::PrescriptionsRepositoryFake, service::PrescriptionsService},
    },
    Context,
};

pub fn create_api_context() -> Context {
    let doctors_repository = Box::new(DoctorsRepositoryFake::new());
    let doctors_service = Arc::new(DoctorsService::new(doctors_repository));

    let pharmacists_repository = Box::new(PharmacistsRepositoryFake::new());
    let pharmacists_service = Arc::new(PharmacistsService::new(pharmacists_repository));

    let patients_repository = Box::new(PatientsRepositoryFake::new());
    let patients_service = Arc::new(PatientsService::new(patients_repository));

    let drugs_repository = Box::new(DrugsRepositoryFake::new());
    let drugs_service = Arc::new(DrugsService::new(drugs_repository));

    let prescriptions_repository = Box::new(PrescriptionsRepositoryFake::new(
        None, None, None, None, None,
    ));
    let prescriptions_service = Arc::new(PrescriptionsService::new(prescriptions_repository));

    let authentication_repository = Box::new(AuthenticationRepositoryFake::new());
    let authentication_service = Arc::new(AuthenticationService::new(authentication_repository));

    let sessions_repository = Box::new(SessionsRepositoryFake::new());
    let sessions_service = Arc::new(SessionsService::new(sessions_repository));

    Context {
        doctors_service,
        pharmacists_service,
        patients_service,
        drugs_service,
        prescriptions_service,
        authentication_service,
        sessions_service,
    }
}
