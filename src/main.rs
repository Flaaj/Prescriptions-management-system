pub mod api;
pub mod domain;
pub mod infrastructure;
use api::{
    doctors_controller, drugs_controller, patients_controller, pharmacists_controller,
    prescriptions_controller,
};
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
use rocket::{get, launch, routes, Build, Rocket, Route};
use rocket_okapi::{
    openapi_get_routes,
    swagger_ui::{make_swagger_ui, SwaggerUIConfig},
};
use sqlx::{postgres::PgPoolOptions, PgPool};
use std::{env, sync::Arc};

async fn setup_database_connection() -> PgPool {
    let db_connection_string =
        &env::var("DATABASE_URL").unwrap_or("postgres://postgres:postgres@localhost:2137".into());

    PgPoolOptions::new()
        .max_connections(5)
        .connect(db_connection_string)
        .await
        .map_err(|err| {
            eprintln!(
                "Failed to connect to the database: {:?}, connection string: {}",
                err, db_connection_string
            );
            err
        })
        .unwrap()
}

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
        patients_controller::get_patients_with_pagination,
        pharmacists_controller::create_pharmacist,
        pharmacists_controller::get_pharmacist_by_id,
        pharmacists_controller::get_pharmacists_with_pagination,
        drugs_controller::create_drug,
        drugs_controller::get_drug_by_id,
        drugs_controller::get_drugs_with_pagination,
        prescriptions_controller::create_prescription,
        prescriptions_controller::get_prescription_by_id,
        prescriptions_controller::get_prescriptions_with_pagination,
        prescriptions_controller::fill_prescription
    ]
}

fn setup_swagger_ui() -> impl Into<Vec<Route>> {
    make_swagger_ui(&SwaggerUIConfig {
        url: "../openapi.json".to_owned(),
        ..Default::default()
    })
}

#[get("/")]
fn redirect_to_swagger_ui() -> rocket::response::Redirect {
    rocket::response::Redirect::to("/swagger-ui")
}

#[launch]
async fn rocket() -> Rocket<Build> {
    let pool = setup_database_connection().await;

    create_tables(&pool, false).await.unwrap();

    rocket::build()
        .manage(setup_context(pool))
        .mount("/", get_routes())
        .mount("/", routes![redirect_to_swagger_ui])
        .mount("/swagger-ui", setup_swagger_ui())
}

// define( 'DB_NAME', 'LendideaCms' );

// /** Database username */
// define( 'DB_USER', 'lendideaAdmin' );

// /** Database password */
// define( 'DB_PASSWORD', 'o3he8e3q' );