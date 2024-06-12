use uuid::Uuid;

use crate::domain::drugs::models::{DrugContentType, NewDrug};

#[derive(thiserror::Error, Debug, PartialEq)]
pub enum CreateNewDrugDomainError {
    #[error("Pills count and mg per pill must be provided for solid pills")]
    InvalidSolidPillsDescription,
    #[error("Pills count and ml per pill must be provided for liquid pills")]
    InvalidLiquidPillsDescription,
    #[error("Volume in ml must be provided for bottle of liquid")]
    InvalidBottleOfLiquidDescription,
}

impl NewDrug {
    pub fn new(
        name: String,
        content_type: DrugContentType,
        pills_count: Option<i32>,
        mg_per_pill: Option<i32>,
        ml_per_pill: Option<i32>,
        volume_ml: Option<i32>,
    ) -> anyhow::Result<NewDrug> {
        match content_type {
            DrugContentType::SolidPills => {
                if pills_count.is_none()
                    || pills_count.unwrap() <= 0
                    || mg_per_pill.is_none()
                    || mg_per_pill.unwrap() <= 0
                {
                    Err(CreateNewDrugDomainError::InvalidSolidPillsDescription)?;
                }

                Ok(NewDrug {
                    id: Uuid::new_v4(),
                    name,
                    content_type,
                    pills_count,
                    mg_per_pill,
                    ml_per_pill: None,
                    volume_ml: None,
                })
            }
            DrugContentType::LiquidPills => {
                if pills_count.is_none()
                    || pills_count.unwrap() <= 0
                    || ml_per_pill.is_none()
                    || ml_per_pill.unwrap() <= 0
                {
                    Err(CreateNewDrugDomainError::InvalidLiquidPillsDescription)?;
                }

                Ok(NewDrug {
                    id: Uuid::new_v4(),
                    name,
                    content_type,
                    pills_count,
                    mg_per_pill: None,
                    ml_per_pill,
                    volume_ml: None,
                })
            }
            DrugContentType::BottleOfLiquid => {
                if volume_ml.is_none() || volume_ml.unwrap() <= 0 {
                    Err(CreateNewDrugDomainError::InvalidBottleOfLiquidDescription)?;
                }

                Ok(NewDrug {
                    id: Uuid::new_v4(),
                    name,
                    content_type,
                    pills_count: None,
                    mg_per_pill: None,
                    ml_per_pill: None,
                    volume_ml,
                })
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use uuid::Uuid;

    use crate::domain::drugs::models::{DrugContentType, NewDrug};

    #[test]
    fn creates_drug() {
        let new_drug = NewDrug::new(
            "Gripex".into(),
            DrugContentType::SolidPills,
            Some(20),
            Some(300),
            None,
            None,
        );
        assert!(new_drug.is_ok());
    }

    #[test]
    fn creates_solid_pills_drug() {
        let expected = NewDrug {
            id: Uuid::default(),
            name: "Gripex".into(),
            content_type: DrugContentType::SolidPills,
            pills_count: Some(20),
            mg_per_pill: Some(300),
            ml_per_pill: None,
            volume_ml: None,
        };

        let mut new_drug = NewDrug::new(
            "Gripex".into(),
            DrugContentType::SolidPills,
            Some(20),
            Some(300),
            Some(300),
            Some(1000),
        )
        .unwrap();

        new_drug.id = Uuid::default();
        assert_eq!(new_drug, expected);
    }

    #[test]
    fn doesnt_create_solid_pills_drug_if_didnt_provide_pills_count_or_mg_per_pill() {
        let new_drug = NewDrug::new(
            "Gripex".into(),
            DrugContentType::SolidPills,
            None,
            Some(300),
            None,
            None,
        );
        assert!(new_drug.is_err());

        let new_drug = NewDrug::new(
            "Gripex".into(),
            DrugContentType::SolidPills,
            Some(0),
            Some(300),
            None,
            None,
        );
        assert!(new_drug.is_err());

        let new_drug = NewDrug::new(
            "Gripex".into(),
            DrugContentType::SolidPills,
            Some(20),
            None,
            None,
            None,
        );
        assert!(new_drug.is_err());

        let new_drug = NewDrug::new(
            "Gripex".into(),
            DrugContentType::SolidPills,
            Some(20),
            Some(0),
            None,
            None,
        );
        assert!(new_drug.is_err());
    }

    #[test]
    fn creates_liquid_pills_drug() {
        let expected = NewDrug {
            id: Uuid::default(),
            name: "Gripex".into(),
            content_type: DrugContentType::LiquidPills,
            pills_count: Some(20),
            mg_per_pill: None,
            ml_per_pill: Some(300),
            volume_ml: None,
        };

        let mut new_drug = NewDrug::new(
            "Gripex".into(),
            DrugContentType::LiquidPills,
            Some(20),
            Some(300),
            Some(300),
            Some(1000),
        )
        .unwrap();

        new_drug.id = Uuid::default();
        assert_eq!(new_drug, expected);
    }

    #[test]
    fn doesnt_create_liquid_pills_drug_if_didnt_provide_pills_count_or_ml_per_pill() {
        let new_drug = NewDrug::new(
            "Gripex".into(),
            DrugContentType::LiquidPills,
            None,
            None,
            Some(300),
            None,
        );
        assert!(new_drug.is_err());

        let new_drug = NewDrug::new(
            "Gripex".into(),
            DrugContentType::LiquidPills,
            Some(0),
            None,
            Some(300),
            None,
        );
        assert!(new_drug.is_err());

        let new_drug = NewDrug::new(
            "Gripex".into(),
            DrugContentType::LiquidPills,
            Some(20),
            None,
            Some(0),
            None,
        );
        assert!(new_drug.is_err());

        let new_drug = NewDrug::new(
            "Gripex".into(),
            DrugContentType::LiquidPills,
            Some(20),
            None,
            None,
            None,
        );
        assert!(new_drug.is_err());
    }

    #[test]
    fn creates_bottle_of_liquid_drug() {
        let expected = NewDrug {
            id: Uuid::default(),
            name: "Gripex".into(),
            content_type: DrugContentType::BottleOfLiquid,
            pills_count: None,
            mg_per_pill: None,
            ml_per_pill: None,
            volume_ml: Some(1000),
        };

        let mut new_drug = NewDrug::new(
            "Gripex".into(),
            DrugContentType::BottleOfLiquid,
            Some(20),
            Some(300),
            Some(300),
            Some(1000),
        )
        .unwrap();

        new_drug.id = Uuid::default();
        assert_eq!(new_drug, expected);
    }

    #[test]
    fn doesnt_create_bottle_of_liquid_drug_if_didnt_provide_volume_ml() {
        let new_drug = NewDrug::new(
            "Gripex".into(),
            DrugContentType::BottleOfLiquid,
            None,
            None,
            None,
            None,
        );
        assert!(new_drug.is_err());

        let new_drug = NewDrug::new(
            "Gripex".into(),
            DrugContentType::BottleOfLiquid,
            None,
            None,
            None,
            Some(0),
        );
        assert!(new_drug.is_err());
    }
}
