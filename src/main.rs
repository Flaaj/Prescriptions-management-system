pub mod domain;

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

    create_tables(&pool).await?;

    prescriptions::controller::create_prescription(&pool).await?;
    prescriptions::controller::create_prescription(&pool).await?;
    prescriptions::controller::create_prescription(&pool).await?;

    let res = prescriptions::controller::get_prescriptions(&pool).await;

    match res {
        Err(e) => println!("{}", e),
        Ok(prescriptions) => {
            for prescription in prescriptions {
                println!("\n{:?}\n", prescription);
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
            patient_id UUID,
            doctor_id UUID,
            prescription_type PrescriptionType,
            start_date TIMESTAMPTZ,
            end_date TIMESTAMPTZ
        );"#
    )
    .execute(pool)
    .await?;

    sqlx::query!(
        r#"
        CREATE TABLE prescribed_drugs (
            id UUID PRIMARY KEY,
            prescription_id UUID,
            drug_id UUID,
            quantity INT
        );"#
    )
    .execute(pool)
    .await?;

    Ok(())
}
