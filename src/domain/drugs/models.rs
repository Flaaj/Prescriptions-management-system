use chrono::{DateTime, Utc};
use uuid::Uuid;

#[derive(Debug, PartialEq, sqlx::Type, Clone, Copy)]
#[sqlx(type_name = "drug_content_type", rename_all = "snake_case")]
pub enum DrugContentType {
    BottleOfLiquid,
    SolidPills,
    LiquidPills,
}

#[derive(Debug, PartialEq, Clone)]
pub struct NewDrug {
    pub id: Uuid,
    pub name: String,
    pub content_type: DrugContentType,
    pub pills_count: Option<i32>,
    pub mg_per_pill: Option<i32>,
    pub ml_per_pill: Option<i32>,
    pub volume_ml: Option<i32>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct Drug {
    pub id: Uuid,
    pub name: String,
    pub content_type: DrugContentType,
    pub pills_count: Option<i32>,
    pub mg_per_pill: Option<i32>,
    pub ml_per_pill: Option<i32>,
    pub volume_ml: Option<i32>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl PartialEq<NewDrug> for Drug {
    fn eq(&self, other: &NewDrug) -> bool {
        self.id == other.id
            && self.name == other.name
            && self.content_type == other.content_type
            && self.pills_count == other.pills_count
            && self.mg_per_pill == other.mg_per_pill
            && self.ml_per_pill == other.ml_per_pill
            && self.volume_ml == other.volume_ml
    }
}

impl PartialEq<Drug> for NewDrug {
    fn eq(&self, other: &Drug) -> bool {
        other.eq(self)
    }
}