use crate::validation::{
    Validate, ValidationResult, validate_length, validate_positive, validate_required,
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
    pub image_url: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_owner: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub permission_level: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CreateHumidorRequest {
    pub name: String,
    pub description: Option<String>,
    pub capacity: Option<i32>,
    pub target_humidity: Option<i32>,
    pub location: Option<String>,
    pub image_url: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateHumidorRequest {
    pub name: Option<String>,
    pub description: Option<String>,
    pub capacity: Option<i32>,
    pub target_humidity: Option<i32>,
    pub location: Option<String>,
    pub image_url: Option<String>,
}

impl Validate for CreateHumidorRequest {
    fn validate(&self) -> ValidationResult<()> {
        validate_required(&self.name, "name")?;
        validate_length(&self.name, "name", 1, 100)?;

        if let Some(desc) = &self.description
            && !desc.is_empty()
        {
            validate_length(desc, "description", 1, 500)?;
        }

        if let Some(capacity) = self.capacity {
            validate_positive(capacity, "capacity")?;
            if capacity > 10000 {
                return Err(crate::errors::AppError::ValidationError(
                    "capacity must not exceed 10000".to_string(),
                ));
            }
        }

        if let Some(humidity) = self.target_humidity
            && !(50..=85).contains(&humidity)
        {
            return Err(crate::errors::AppError::ValidationError(
                "target_humidity must be between 50 and 85".to_string(),
            ));
        }

        if let Some(location) = &self.location
            && !location.is_empty()
        {
            validate_length(location, "location", 1, 200)?;
        }

        if let Some(image_url) = &self.image_url
            && !image_url.is_empty()
        {
            // Allow up to 20MB of base64 data (~26.7 million chars)
            // Actual limit enforced by Warp body size (10MB JSON payload)
            validate_length(image_url, "image_url", 1, 30_000_000)?;
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

        if let Some(desc) = &self.description
            && !desc.is_empty()
        {
            validate_length(desc, "description", 1, 500)?;
        }

        if let Some(capacity) = self.capacity {
            validate_positive(capacity, "capacity")?;
            if capacity > 10000 {
                return Err(crate::errors::AppError::ValidationError(
                    "capacity must not exceed 10000".to_string(),
                ));
            }
        }

        if let Some(humidity) = self.target_humidity
            && !(50..=85).contains(&humidity)
        {
            return Err(crate::errors::AppError::ValidationError(
                "target_humidity must be between 50 and 85".to_string(),
            ));
        }

        if let Some(location) = &self.location
            && !location.is_empty()
        {
            validate_length(location, "location", 1, 200)?;
        }

        if let Some(image_url) = &self.image_url
            && !image_url.is_empty()
        {
            // Allow up to 20MB of base64 data (~26.7 million chars)
            // Actual limit enforced by Warp body size (10MB JSON payload)
            validate_length(image_url, "image_url", 1, 30_000_000)?;
        }

        Ok(())
    }
}
