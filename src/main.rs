pub mod domain;

use chrono::Utc;
use domain::prescriptions::{self};
use sqlx::postgres::PgPoolOptions;
use uuid::timestamp;

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
    println!("nanoseconds passed: {}", (Utc::now() - timestamp).num_nanoseconds().unwrap());

    match res {
        Err(e) => println!("{}", e),
        Ok(prescriptions) => {
            for prescription in prescriptions {
                println!("\n{:#?}\n", prescription);
            }
        }
    }

    Ok(())
}

async fn create_tables(pool: &sqlx::PgPool) -> Result<(), sqlx::Error> {
    sqlx::query!("DROP TABLE IF EXISTS prescribed_drugs;")
        .execute(pool)
        .await?;
    sqlx::query!("DROP TABLE IF EXISTS prescriptions;")
        .execute(pool)
        .await?;
    sqlx::query!("DROP TYPE IF EXISTS prescriptiontype;")
        .execute(pool)
        .await?;

    sqlx::query!(r#"CREATE TYPE prescriptiontype AS ENUM ('regular', 'forantibiotics', 'forchronicdiseasedrugs', 'forimmunologicaldrugs');"#)//
        .execute(pool)
        .await?;

    sqlx::query!(
        r#"
        CREATE TABLE prescriptions (
            id UUID PRIMARY KEY,
            patient_id UUID NOT NULL,
            doctor_id UUID NOT NULL,
            prescription_type PrescriptionType NOT NULL,
            start_date TIMESTAMPTZ NOT NULL,
            end_date TIMESTAMPTZ NOT NULL
        );"#
    )
    .execute(pool)
    .await?;

    sqlx::query!(
        r#"
        CREATE TABLE prescribed_drugs (
            id UUID PRIMARY KEY,
            prescription_id UUID NOT NULL,
            drug_id UUID NOT NULL,
            quantity INT NOT NULL
        );"#
    )
    .execute(pool)
    .await?;

    Ok(())
}
