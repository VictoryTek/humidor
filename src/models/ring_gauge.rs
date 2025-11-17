use crate::validation::{
    Validate, ValidationResult, validate_length, validate_positive, validate_range,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RingGauge {
    pub id: Uuid,
    pub user_id: Uuid,
    pub gauge: i32,
    pub description: Option<String>,
    pub common_names: Option<Vec<String>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreateRingGauge {
    pub gauge: i32,
    pub description: Option<String>,
    pub common_names: Option<Vec<String>>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateRingGauge {
    pub gauge: Option<i32>,
    pub description: Option<String>,
    pub common_names: Option<Vec<String>>,
}

impl Validate for CreateRingGauge {
    fn validate(&self) -> ValidationResult<()> {
        validate_positive(self.gauge, "gauge")?;
        validate_range(self.gauge, "gauge", 20, 100)?;

        if let Some(desc) = &self.description {
            if !desc.is_empty() {
                validate_length(desc, "description", 1, 500)?;
            }
        }

        Ok(())
    }
}

impl Validate for UpdateRingGauge {
    fn validate(&self) -> ValidationResult<()> {
        if let Some(gauge) = self.gauge {
            validate_positive(gauge, "gauge")?;
            validate_range(gauge, "gauge", 20, 100)?;
        }

        if let Some(desc) = &self.description {
            if !desc.is_empty() {
                validate_length(desc, "description", 1, 500)?;
            }
        }

        Ok(())
    }
}
