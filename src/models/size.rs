use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use crate::validation::{Validate, ValidationResult, validate_length, validate_required, validate_positive, validate_range, validate_range_f64};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Size {
    pub id: Uuid,
    pub name: String,
    pub length_inches: Option<f64>,
    pub ring_gauge: Option<i32>,
    pub description: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreateSize {
    pub name: String,
    pub length_inches: Option<f64>,
    pub ring_gauge: Option<i32>,
    pub description: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateSize {
    pub name: Option<String>,
    pub length_inches: Option<f64>,
    pub ring_gauge: Option<i32>,
    pub description: Option<String>,
}

impl Validate for CreateSize {
    fn validate(&self) -> ValidationResult<()> {
        validate_required(&self.name, "name")?;
        validate_length(&self.name, "name", 1, 100)?;
        
        if let Some(length) = self.length_inches {
            validate_range_f64(length, 3.0, 12.0, "length_inches")?;
        }
        
        if let Some(gauge) = self.ring_gauge {
            validate_positive(gauge, "ring_gauge")?;
            validate_range(gauge, "ring_gauge", 20, 100)?;
        }
        
        if let Some(desc) = &self.description {
            validate_length(desc, "description", 1, 500)?;
        }
        
        Ok(())
    }
}

impl Validate for UpdateSize {
    fn validate(&self) -> ValidationResult<()> {
        if let Some(name) = &self.name {
            validate_required(name, "name")?;
            validate_length(name, "name", 1, 100)?;
        }
        
        if let Some(length) = self.length_inches {
            validate_range_f64(length, 3.0, 12.0, "length_inches")?;
        }
        
        if let Some(gauge) = self.ring_gauge {
            validate_positive(gauge, "ring_gauge")?;
            validate_range(gauge, "ring_gauge", 20, 100)?;
        }
        
        if let Some(desc) = &self.description {
            validate_length(desc, "description", 1, 500)?;
        }
        
        Ok(())
    }
}
