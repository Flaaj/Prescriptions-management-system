pub mod domain;
use std::borrow::Borrow;

use sqlx::postgres::PgPoolOptions;
use uuid::Uuid;

#[macro_use]
extern crate dotenv_codegen;

#[tokio::main]
async fn main() -> Result<(), sqlx::Error> {
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(dotenv!("DATABASE_URL"))
        .await?;

    let res1 = sqlx::query!(
        "
CREATE TABLE IF NOT EXISTS prescriptions (
    patient_id UUID,
    doctor_id UUID
)"
    )
    .execute(&pool)
    .await?;

    let patient_id = Uuid::new_v4();
    let doctor_id = Uuid::new_v4();
    let res2 = sqlx::query!(
        "INSERT INTO prescriptions (patient_id, doctor_id) VALUES ($1, $2)",
        patient_id,
        doctor_id
    )
    .execute(&pool)
    .await?;

    println!("rows affected: {}", res2.rows_affected());

    let res3 = sqlx::query!(
        "SELECT * FROM prescriptions WHERE patient_id = $1",
        Uuid::parse_str("cf402afa-fd60-4bf2-969c-00e4e0ed13b0").unwrap()
    )
    .fetch_all(&pool)
    .await?;

    for row in res3.iter() {
        let patient_id: Uuid = row.patient_id.unwrap();
        let doctor_id: Uuid = row.doctor_id.unwrap();
        println!("patient_id: {}, doctor_id: {}", patient_id, doctor_id);
    }

    Ok(())
}
