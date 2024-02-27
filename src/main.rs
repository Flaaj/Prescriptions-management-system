pub mod domain;

use domain::prescriptions::create_prescription::{NewPrescription, PrescriptionType};
use sqlx::postgres::PgPoolOptions;
use uuid::Uuid;

#[macro_use]
extern crate dotenv_codegen;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(dotenv!("DATABASE_URL"))
        .await?;

    create_tables(&pool).await?;

    let mut prescription = NewPrescription::new(
        Uuid::new_v4(),
        Uuid::new_v4(),
        None,
        Some(PrescriptionType::Regular),
    );
    prescription.add_drug(Uuid::new_v4(), 2)?;
    prescription.add_drug(Uuid::new_v4(), 3)?;

    match prescription.save_to_database(&pool).await {
        Ok(_) => println!("Prescription saved to database"),
        Err(e) => println!("Error saving prescription to database: {}", e),
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

    sqlx::query!(r#"CREATE TYPE prescriptiontype AS ENUM ('REGULAR', 'FORANTIBIOTICS', 'FORCHRONICDISEASEDRUGS', 'FORIMMUNOLOGICALDRUGS');"#)//
        .execute(pool)
        .await?;

    sqlx::query!(
        r#"
        CREATE TABLE prescriptions (
            id UUID PRIMARY KEY,
            patient_id UUID,
            doctor_id UUID,
            prescription_type PrescriptionType,
            start_date TIMESTAMP,
            end_date TIMESTAMP
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
