use chrono::{DateTime, Utc};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, PartialEq, sqlx::Type, Clone, Copy, Deserialize, Serialize, JsonSchema)]
#[sqlx(type_name = "drug_content_type", rename_all = "snake_case")]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
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

fn example_drug_name() -> &'static str {
    "Apap"
}
fn example_drug_content_type() -> DrugContentType {
    DrugContentType::SolidPills
}
fn example_pills_count() -> Option<i32> {
    Some(30)
}
fn example_mg_per_pill() -> Option<i32> {
    Some(300)
}
fn example_ml_per_pill() -> Option<i32> {
    None
}
fn example_volume_ml() -> Option<i32> {
    None
}

#[derive(Debug, PartialEq, Clone, Deserialize, Serialize, JsonSchema)]
pub struct Drug {
    pub id: Uuid,
    #[schemars(example = "example_drug_name")]
    pub name: String,
    #[schemars(example = "example_drug_content_type")]
    pub content_type: DrugContentType,
    #[schemars(example = "example_pills_count")]
    pub pills_count: Option<i32>,
    #[schemars(example = "example_mg_per_pill")]
    pub mg_per_pill: Option<i32>,
    #[schemars(example = "example_ml_per_pill")]
    pub ml_per_pill: Option<i32>,
    #[schemars(example = "example_volume_ml")]
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
