use crate::validation::{validate_length, validate_url, Validate, ValidationResult};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Brand {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub country: Option<String>,
    pub website: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreateBrand {
    pub name: String,
    pub description: Option<String>,
    pub country: Option<String>,
    pub website: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateBrand {
    pub name: Option<String>,
    pub description: Option<String>,
    pub country: Option<String>,
    pub website: Option<String>,
}

impl Validate for CreateBrand {
    fn validate(&self) -> ValidationResult<()> {
        validate_length(&self.name, "Name", 1, 100)?;

        if let Some(desc) = &self.description {
            validate_length(desc, "Description", 0, 500)?;
        }
        if let Some(country) = &self.country {
            validate_length(country, "Country", 1, 100)?;
        }
        if let Some(website) = &self.website {
            validate_url(website)?;
        }

        Ok(())
    }
}

impl Validate for UpdateBrand {
    fn validate(&self) -> ValidationResult<()> {
        if let Some(name) = &self.name {
            validate_length(name, "Name", 1, 100)?;
        }
        if let Some(desc) = &self.description {
            validate_length(desc, "Description", 0, 500)?;
        }
        if let Some(country) = &self.country {
            validate_length(country, "Country", 1, 100)?;
        }
        if let Some(website) = &self.website {
            validate_url(website)?;
        }

        Ok(())
    }
}
