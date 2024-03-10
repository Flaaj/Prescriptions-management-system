use chrono::Duration;

#[derive(Debug, PartialEq, sqlx::Type, Clone, Copy)]
#[sqlx(type_name = "prescription_type", rename_all = "snake_case")]
pub enum PrescriptionType {
    Regular,
    ForAntibiotics,
    ForImmunologicalDrugs,
    ForChronicDiseaseDrugs,
}

impl PrescriptionType {
    pub fn get_duration(&self) -> Duration {
        match self {
            PrescriptionType::Regular => Duration::days(30),
            PrescriptionType::ForAntibiotics => Duration::days(7),
            PrescriptionType::ForImmunologicalDrugs => Duration::days(120),
            PrescriptionType::ForChronicDiseaseDrugs => Duration::days(365),
        }
    }
}
