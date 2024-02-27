use chrono::Duration;

#[derive(Debug, PartialEq, sqlx::Type, Clone)]
#[sqlx(type_name = "prescriptiontype", rename_all = "lowercase")]
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
