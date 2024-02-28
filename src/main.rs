pub mod domain;

use chrono::Utc;
use domain::prescriptions::{self};
use sqlx::postgres::PgPoolOptions;
mod create_tables;
use create_tables::create_tables;

#[macro_use]
extern crate dotenv_codegen;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(dotenv!("DATABASE_URL"))
        .await?;

    create_tables(&pool).await?;

    prescriptions::controller::create_prescription(&pool).await?;
    prescriptions::controller::create_prescription(&pool).await?;
    prescriptions::controller::create_prescription(&pool).await?;
    prescriptions::controller::create_prescription(&pool).await?;
    prescriptions::controller::create_prescription(&pool).await?;
    prescriptions::controller::create_prescription(&pool).await?;

    let timestamp = Utc::now();
    let res = prescriptions::controller::get_prescriptions(&pool).await;
    println!(
        "nanoseconds passed: {}",
        (Utc::now() - timestamp).num_nanoseconds().unwrap()
    );

    match res {
        Err(e) => println!("{}", e),
        Ok(prescriptions) => {
            println!("{}", prescriptions.len());
            for prescription in prescriptions {
                println!("\n{:#?}\n", prescription);
            }
        }
    }

    Ok(())
}
