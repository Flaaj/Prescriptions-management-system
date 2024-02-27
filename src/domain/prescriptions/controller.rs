use chrono::{Duration, Utc};
use sqlx::PgPool;
use uuid::Uuid;

use super::{
    create_prescription::NewPrescription, get_prescriptions::Prescription,
    get_prescriptions_repository::PrescriptionRepository, prescription_type::PrescriptionType,
};

pub async fn create_prescription(pool: &PgPool) -> anyhow::Result<()> {
    let mut prescription = NewPrescription::new(
        Uuid::new_v4(),
        Uuid::new_v4(),
        Some(Utc::now() + Duration::days(3)),
        Some(PrescriptionType::ForChronicDiseaseDrugs),
    );
    prescription.add_drug(Uuid::new_v4(), 2)?;
    prescription.add_drug(Uuid::new_v4(), 3)?;

    match prescription.save_to_database(&pool).await {
        Ok(_) => println!("Prescription saved to database"),
        Err(e) => println!("{}", e),
    };

    Ok(())
}

pub async fn get_prescriptions(pool: &PgPool) -> anyhow::Result<Vec<Prescription>> {
    let result = PrescriptionRepository::get_prescriptions(pool, None, None).await?;
    Ok(result)
}
