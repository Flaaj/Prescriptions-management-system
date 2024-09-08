use std::sync::Arc;

use crate::{
    application::{
        authentication::service::AuthenticationService, sessions::service::SessionsService,
    },
    domain::{
        doctors::service::DoctorsService, drugs::service::DrugsService,
        patients::service::PatientsService, pharmacists::service::PharmacistsService,
        prescriptions::service::PrescriptionsService,
    },
    infrastructure::postgres_repository_impl::{
        authentication::PostgresAuthenticationRepository, doctors::PostgresDoctorsRepository,
        drugs::PostgresDrugsRepository, patients::PostgresPatientsRepository,
        pharmacists::PostgresPharmacistsRepository, prescriptions::PostgresPrescriptionsRepository,
        sessions::PostgresSessionsRepository,
    },
};

#[derive(Clone)]
pub struct Context {
    pub doctors_service: Arc<DoctorsService>,
    pub pharmacists_service: Arc<PharmacistsService>,
    pub patients_service: Arc<PatientsService>,
    pub drugs_service: Arc<DrugsService>,
    pub prescriptions_service: Arc<PrescriptionsService>,
    pub authentication_service: Arc<AuthenticationService>,
    pub sessions_service: Arc<SessionsService>,
}

pub fn setup_context(pool: sqlx::PgPool) -> Context {
    let doctors_repository = Box::new(PostgresDoctorsRepository::new(pool.clone()));
    let doctors_service = Arc::new(DoctorsService::new(doctors_repository));

    let pharmacists_repository = Box::new(PostgresPharmacistsRepository::new(pool.clone()));
    let pharmacists_service = Arc::new(PharmacistsService::new(pharmacists_repository));

    let patients_repository = Box::new(PostgresPatientsRepository::new(pool.clone()));
    let patients_service = Arc::new(PatientsService::new(patients_repository));

    let drugs_repository = Box::new(PostgresDrugsRepository::new(pool.clone()));
    let drugs_service = Arc::new(DrugsService::new(drugs_repository));

    let prescriptions_repository = Box::new(PostgresPrescriptionsRepository::new(pool.clone()));
    let prescriptions_service = Arc::new(PrescriptionsService::new(prescriptions_repository));

    let authentication_repository = Box::new(PostgresAuthenticationRepository::new(pool.clone()));
    let authentication_service = Arc::new(AuthenticationService::new(authentication_repository));

    let sessions_repository = Box::new(PostgresSessionsRepository::new(pool));
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

pub type Ctx = rocket::State<Context>;
