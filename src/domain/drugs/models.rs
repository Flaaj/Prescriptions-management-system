use chrono::{DateTime, Utc};
use uuid::Uuid;

#[derive(Debug, PartialEq, sqlx::Type, Clone, Copy)]
#[sqlx(type_name = "drug_content_type", rename_all = "snake_case")]
pub enum DrugContentType {
    BottleOfLiquid,
    SolidPills,
    LiquidPills,
}

#[derive(Debug, PartialEq)]
pub struct NewDrug {
    pub id: Uuid,
    pub name: String,
    pub content_type: DrugContentType,
    pub pills_count: Option<u32>,
    pub mg_per_pill: Option<u32>,
    pub ml_per_pill: Option<u32>,
    pub volume_ml: Option<u32>,
}

pub struct Drug {
    id: Uuid,
    name: String,
    content_type: DrugContentType,
    pills_count: Option<u32>,
    mg_per_pill: Option<u32>,
    volume_ml: Option<u32>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}
