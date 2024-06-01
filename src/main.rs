pub mod api;
pub mod domain;
pub mod infrastructure;
use api::{doctors_controller, patients_controller};
use domain::{
    doctors::service::DoctorsService, drugs::service::DrugsService,
    patients::service::PatientsService, pharmacists::service::PharmacistsService,
    prescriptions::service::PrescriptionsService,
};
use infrastructure::postgres_repository_impl::{
    create_tables::create_tables, doctors::PostgresDoctorsRepository,
    drugs::PostgresDrugsRepository, patients::PostgresPatientsRepository,
    pharmacists::PostgresPharmacistsRepository, prescriptions::PostgresPrescriptionsRepository,
};
use rocket::{launch, Build, Rocket, Route};
use rocket_okapi::{
    openapi_get_routes,
    swagger_ui::{make_swagger_ui, SwaggerUIConfig},
};
use sqlx::{postgres::PgPoolOptions, PgPool};
use std::sync::Arc;

#[macro_use]
extern crate dotenv_codegen;

#[derive(Clone)]
pub struct Context {
    pub doctors_service: Arc<DoctorsService>,
    pub pharmacists_service: Arc<PharmacistsService>,
    pub patients_service: Arc<PatientsService>,
    pub drugs_service: Arc<DrugsService>,
    pub prescriptions_service: Arc<PrescriptionsService>,
}
pub type Ctx = rocket::State<Context>;

fn setup_context(pool: PgPool) -> Context {
    let doctors_repository = Box::new(PostgresDoctorsRepository::new(pool.clone()));
    let doctors_service = Arc::new(DoctorsService::new(doctors_repository));

    let pharmacists_rerpository = Box::new(PostgresPharmacistsRepository::new(pool.clone()));
    let pharmacists_service = Arc::new(PharmacistsService::new(pharmacists_rerpository));

    let patients_repository = Box::new(PostgresPatientsRepository::new(pool.clone()));
    let patients_service = Arc::new(PatientsService::new(patients_repository));

    let drugs_repository = Box::new(PostgresDrugsRepository::new(pool.clone()));
    let drugs_service = Arc::new(DrugsService::new(drugs_repository));

    let prescriptions_repository = Box::new(PostgresPrescriptionsRepository::new(pool.clone()));
    let prescriptions_service = Arc::new(PrescriptionsService::new(prescriptions_repository));

    Context {
        doctors_service,
        pharmacists_service,
        patients_service,
        drugs_service,
        prescriptions_service,
    }
}

fn get_routes() -> Vec<Route> {
    openapi_get_routes![
        doctors_controller::create_doctor,
        doctors_controller::get_doctor_by_id,
        doctors_controller::get_doctors_with_pagination,
        patients_controller::create_patient,
        patients_controller::get_patient_by_id,
    ]
}

fn setup_swagger_ui() -> impl Into<Vec<Route>> {
    make_swagger_ui(&SwaggerUIConfig {
        url: "../openapi.json".to_owned(),
        ..Default::default()
    })
}

#[launch]
async fn rocket() -> Rocket<Build> {
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(dotenv!("DATABASE_URL"))
        .await
        .unwrap();

    create_tables(&pool, false).await.unwrap();

    rocket::build()
        .manage(setup_context(pool))
        .mount("/", get_routes())
        .mount("/swagger-ui", setup_swagger_ui())
}
