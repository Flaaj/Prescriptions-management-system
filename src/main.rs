pub mod api;
mod create_tables;
pub mod domain;
pub mod utils;
use api::doctors_controller;
use create_tables::create_tables;
use rocket::{launch, Build, Rocket};
use rocket_okapi::swagger_ui::{make_swagger_ui, SwaggerUIConfig};
use sqlx::{postgres::PgPoolOptions, Pool, Postgres};
use std::sync::Arc;

#[macro_use]
extern crate dotenv_codegen;

#[derive(Clone)]
pub struct Context {
    pub pool: Arc<Pool<Postgres>>,
}
pub type Ctx = rocket::State<Context>;

#[launch]
async fn rocket() -> Rocket<Build> {
    let pool = Arc::new(
        PgPoolOptions::new()
            .max_connections(5)
            .connect(dotenv!("DATABASE_URL"))
            .await
            .unwrap(),
    );

    create_tables(&pool, true).await.unwrap();

    rocket::build()
        .manage(Context { pool })
        .mount("/", doctors_controller::get_routes())
        .mount(
            "/swagger-ui/",
            make_swagger_ui(&SwaggerUIConfig {
                url: "../openapi.json".to_owned(),
                ..Default::default()
            }),
        )
}
