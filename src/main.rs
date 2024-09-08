pub mod application;
pub mod context;
pub mod domain;
pub mod infrastructure;

use std::env;

use application::api::controllers::{
    authentication_controller, doctors_controller, drugs_controller, patients_controller,
    pharmacists_controller, prescriptions_controller,
};
use context::{setup_context, Context};
use infrastructure::postgres_repository_impl::create_tables::create_tables;
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

pub type Ctx = rocket::State<Context>;

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
