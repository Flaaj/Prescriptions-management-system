use chrono::{DateTime, Utc};
use rocket_okapi::okapi::schemars;
use rocket_okapi::JsonSchema;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Clone, Debug)]
pub struct NewDoctor {
    pub id: Uuid,
    pub name: String,
    pub pwz_number: String,
    pub pesel_number: String,
}

fn example_name() -> &'static str {
    "John Doe"
}
fn example_pesel_number() -> &'static str {
    "96021807250"
}
fn example_pwz_number() -> &'static str {
    "5425740"
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone, JsonSchema)]
pub struct Doctor {
    pub id: Uuid,
    #[schemars(example = "example_name")]
    pub name: String,
    #[schemars(example = "example_pwz_number")]
    pub pwz_number: String,
    #[schemars(example = "example_pesel_number")]
    pub pesel_number: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl PartialEq<NewDoctor> for Doctor {
    fn eq(&self, other: &NewDoctor) -> bool {
        self.id == other.id
            && self.name == other.name
            && self.pesel_number == other.pesel_number
            && self.pwz_number == other.pwz_number
    }
}

impl PartialEq<Doctor> for NewDoctor {
    fn eq(&self, other: &Doctor) -> bool {
        other.eq(self)
    }
}
