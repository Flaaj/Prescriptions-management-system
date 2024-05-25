pub mod api;
mod create_tables;
pub mod domain;
pub mod infrastructure;
use std::sync::Arc;

use api::doctors_controller;
use create_tables::create_tables;
use domain::{
    doctors::service::DoctorsService, drugs::service::DrugsService,
    patients::service::PatientsService, pharmacists::service::PharmacistsService,
};
use infrastructure::postgres_repository_impl::{
    doctors::DoctorsPostgresRepository, drugs::DrugsPostgresRepository,
    patients::PatientsPostgresRepository, pharmacists::PharmacistsPostgresRepository,
};
use rocket::{launch, Build, Rocket};
use rocket_okapi::swagger_ui::{make_swagger_ui, SwaggerUIConfig};
use sqlx::{postgres::PgPoolOptions, PgPool};

#[macro_use]
extern crate dotenv_codegen;

#[derive(Clone)]
pub struct Context {
    pub doctors_service: Arc<DoctorsService<DoctorsPostgresRepository>>,
    pub pharmacists_service: Arc<PharmacistsService<PharmacistsPostgresRepository>>,
    pub patients_service: Arc<PatientsService<PatientsPostgresRepository>>,
    pub drugs_service: Arc<DrugsService<DrugsPostgresRepository>>,
}
pub type Ctx = rocket::State<Context>;

pub fn setup_context(pool: PgPool) -> Context {
    let doctors_repository = DoctorsPostgresRepository::new(pool.clone());
    let doctors_service = Arc::new(DoctorsService::new(doctors_repository));
    let pharmacists_rerpository = PharmacistsPostgresRepository::new(pool.clone());
    let pharmacists_service = Arc::new(PharmacistsService::new(pharmacists_rerpository));
    let patients_repository = PatientsPostgresRepository::new(pool.clone());
    let patients_service = Arc::new(PatientsService::new(patients_repository));
    let drugs_repository = DrugsPostgresRepository::new(pool.clone());
    let drugs_service = Arc::new(DrugsService::new(drugs_repository));

    Context {
        doctors_service,
        pharmacists_service,
        patients_service,
        drugs_service,
    }
}

#[launch]
async fn rocket() -> Rocket<Build> {
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(dotenv!("DATABASE_URL"))
        .await
        .unwrap();

    create_tables(&pool, true).await.unwrap();

    let context = setup_context(pool);

    rocket::build()
        .manage(context)
        .mount("/", doctors_controller::get_routes())
        .mount(
            "/swagger-ui/",
            make_swagger_ui(&SwaggerUIConfig {
                url: "../openapi.json".to_owned(),
                ..Default::default()
            }),
        )
}
