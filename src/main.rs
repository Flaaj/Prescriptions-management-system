mod create_tables;
pub mod domain;

use chrono::Utc;
use create_tables::create_tables;
use domain::doctors::{create_doctor::NewDoctor, get_doctors_repository::DoctorsRepository};
use domain::prescriptions::{self};
use sqlx::postgres::PgPoolOptions;

#[macro_use]
extern crate dotenv_codegen;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(dotenv!("DATABASE_URL"))
        .await?;

    create_tables(&pool, true).await?;

    prescriptions::controller::create_prescription(&pool).await?;
    prescriptions::controller::create_prescription(&pool).await?;
    prescriptions::controller::create_prescription(&pool).await?;
    prescriptions::controller::create_prescription(&pool).await?;
    prescriptions::controller::create_prescription(&pool).await?;
    prescriptions::controller::create_prescription(&pool).await?;

    NewDoctor::new("John Doe".into(), "3123456".into(), "96021807250".into())
        .unwrap()
        .commit_to_repository(&pool)
        .await?;

    NewDoctor::new(
        "Mark Zuckerberk".into(),
        "3123456".into(),
        "96021807250".into(),
    )
    .unwrap()
    .commit_to_repository(&pool)
    .await?;

    let timestamp = Utc::now();
    let prescriptions = prescriptions::controller::get_prescriptions(&pool).await?;
    println!(
        "nanoseconds passed: {}",
        (Utc::now() - timestamp).num_nanoseconds().unwrap()
    );

    for prescription in prescriptions {
        println!("\n{:#?}\n", prescription);
    }

    let doctors = DoctorsRepository::get_doctors(&pool).await?;
    for doctor in doctors {
        println!("\n{:#?}\n", doctor);
    }

    Ok(())
}
