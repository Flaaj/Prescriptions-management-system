pub mod api;
mod create_tables;
pub mod domain;
pub mod utils;
use std::sync::Arc;

use api::doctors_controller;
use create_tables::create_tables;
use domain::{
    doctors::{repository::doctors_repository_impl::DoctorsRepository, service::DoctorsService}, drugs::{repository::drugs_repository_impl::DrugsRepository, service::DrugsService}, patients::{repository::patients_repository_impl::PatientsRepository, service::PatientsService}, pharmacists::{
        repository::pharmacists_repository_impl::PharmacistsRepository, service::PharmacistsService,
    }
};
use rocket::{launch, Build, Rocket};
use rocket_okapi::swagger_ui::{make_swagger_ui, SwaggerUIConfig};
use sqlx::{postgres::PgPoolOptions, PgPool};

#[macro_use]
extern crate dotenv_codegen;

#[derive(Clone)]
pub struct Context {
    pub doctors_service: Arc<DoctorsService<DoctorsRepository>>,
    pub pharmacist_service: Arc<PharmacistsService<PharmacistsRepository>>,
    pub patients_service: Arc<PatientsService<PatientsRepository>>,
    pub drugs_service: Arc<DrugsService<DrugsRepository>>,
}
pub type Ctx = rocket::State<Context>;

pub fn setup_context(pool: PgPool) -> Context {
    let doctors_service = Arc::new(DoctorsService::new(DoctorsRepository::new(pool.clone())));
    let pharmacist_service = Arc::new(PharmacistsService::new(PharmacistsRepository::new(pool.clone())));
    let patients_service = Arc::new(PatientsService::new(PatientsRepository::new(pool.clone())));
    let drugs_service = Arc::new(DrugsService::new(DrugsRepository::new(pool)));

    Context {
        doctors_service,
        pharmacist_service,
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
