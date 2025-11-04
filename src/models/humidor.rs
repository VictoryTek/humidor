use crate::validation::{
    validate_length, validate_positive, validate_required, Validate, ValidationResult,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Humidor {
    pub id: Uuid,
    pub user_id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub capacity: Option<i32>,
    pub target_humidity: Option<i32>,
    pub location: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreateHumidorRequest {
    pub name: String,
    pub description: Option<String>,
    pub capacity: Option<i32>,
    pub target_humidity: Option<i32>,
    pub location: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateHumidorRequest {
    pub name: Option<String>,
    pub description: Option<String>,
    pub capacity: Option<i32>,
    pub target_humidity: Option<i32>,
    pub location: Option<String>,
}

impl Validate for CreateHumidorRequest {
    fn validate(&self) -> ValidationResult<()> {
        validate_required(&self.name, "name")?;
        validate_length(&self.name, "name", 1, 100)?;

        if let Some(desc) = &self.description {
            if !desc.is_empty() {
                validate_length(desc, "description", 1, 500)?;
            }
        }

        if let Some(capacity) = self.capacity {
            validate_positive(capacity, "capacity")?;
            if capacity > 10000 {
                return Err(crate::errors::AppError::ValidationError(
                    "capacity must not exceed 10000".to_string(),
                ));
            }
        }

        if let Some(humidity) = self.target_humidity {
            if humidity < 50 || humidity > 85 {
                return Err(crate::errors::AppError::ValidationError(
                    "target_humidity must be between 50 and 85".to_string(),
                ));
            }
        }

        if let Some(location) = &self.location {
            if !location.is_empty() {
                validate_length(location, "location", 1, 200)?;
            }
        }

        Ok(())
    }
}

impl Validate for UpdateHumidorRequest {
    fn validate(&self) -> ValidationResult<()> {
        if let Some(name) = &self.name {
            validate_required(name, "name")?;
            validate_length(name, "name", 1, 100)?;
        }

        if let Some(desc) = &self.description {
            if !desc.is_empty() {
                validate_length(desc, "description", 1, 500)?;
            }
        }

        if let Some(capacity) = self.capacity {
            validate_positive(capacity, "capacity")?;
            if capacity > 10000 {
                return Err(crate::errors::AppError::ValidationError(
                    "capacity must not exceed 10000".to_string(),
                ));
            }
        }

        if let Some(humidity) = self.target_humidity {
            if humidity < 50 || humidity > 85 {
                return Err(crate::errors::AppError::ValidationError(
                    "target_humidity must be between 50 and 85".to_string(),
                ));
            }
        }

        if let Some(location) = &self.location {
            if !location.is_empty() {
                validate_length(location, "location", 1, 200)?;
            }
        }

        Ok(())
    }
}
