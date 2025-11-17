use crate::validation::{Validate, ValidationResult, validate_length, validate_required};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Origin {
    pub id: Uuid,
    pub user_id: Uuid,
    pub name: String,
    pub country: String,
    pub region: Option<String>,
    pub description: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreateOrigin {
    pub name: String,
    pub country: String,
    pub region: Option<String>,
    pub description: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateOrigin {
    pub name: Option<String>,
    pub country: Option<String>,
    pub region: Option<String>,
    pub description: Option<String>,
}

impl Validate for CreateOrigin {
    fn validate(&self) -> ValidationResult<()> {
        validate_required(&self.name, "name")?;
        validate_length(&self.name, "name", 1, 100)?;

        validate_required(&self.country, "country")?;
        validate_length(&self.country, "country", 1, 100)?;

        if let Some(region) = &self.region
            && !region.is_empty()
        {
            validate_length(region, "region", 1, 100)?;
        }

        if let Some(desc) = &self.description
            && !desc.is_empty()
        {
            validate_length(desc, "description", 1, 500)?;
        }

        Ok(())
    }
}

impl Validate for UpdateOrigin {
    fn validate(&self) -> ValidationResult<()> {
        if let Some(name) = &self.name {
            validate_required(name, "name")?;
            validate_length(name, "name", 1, 100)?;
        }

        if let Some(country) = &self.country {
            validate_required(country, "country")?;
            validate_length(country, "country", 1, 100)?;
        }

        if let Some(region) = &self.region
            && !region.is_empty()
        {
            validate_length(region, "region", 1, 100)?;
        }

        if let Some(desc) = &self.description
            && !desc.is_empty()
        {
            validate_length(desc, "description", 1, 500)?;
        }

        Ok(())
    }
}
