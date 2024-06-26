pub mod application;
pub mod domain;
pub mod infrastructure;

use std::{env, sync::Arc};

use application::{
    api::controllers::{
        authentication_controller, doctors_controller, drugs_controller, patients_controller,
        pharmacists_controller, prescriptions_controller,
    },
    authentication::{repository::AuthenticationRepositoryFake, service::AuthenticationService},
    sessions::{repository::SessionsRepositoryFake, service::SessionsService},
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
    pub authentication_service: Arc<AuthenticationService>,
    pub sessions_service: Arc<SessionsService>,
}
pub type Ctx = rocket::State<Context>;

fn setup_context(pool: PgPool) -> Context {
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
        prescriptions_controller::fill_prescription,
        authentication_controller::login_doctor,
        authentication_controller::login_pharmacist,
        authentication_controller::register_doctor,
        authentication_controller::register_pharmacist,
        authentication_controller::logout,
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

// fn setup_scheduler(ctx: &Context) {
//     let mut scheduler = Scheduler::new();
//     scheduler.every(1.day()).at("3:00 AM").run(|| {
        // ctx.sessions_service.remove_sessions_older_than_one_week();
//     });

//     thread::spawn(move || loop {
//         scheduler.run_pending();
//         thread::sleep(Duration::from_secs(3600));
//     });
// }

#[launch]
async fn rocket() -> Rocket<Build> {
    let pool = setup_database_connection().await;

    create_tables(&pool, false).await.unwrap();

    let context = setup_context(pool);

    // setup_scheduler(&context);

    rocket::build()
        .manage(context)
        .mount("/", get_routes())
        .mount("/", routes![redirect_to_swagger_ui])
        .mount("/swagger-ui", setup_swagger_ui())
}
