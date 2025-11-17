use crate::validation::{Validate, ValidationResult, validate_length, validate_required};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Strength {
    pub id: Uuid,
    pub user_id: Uuid,
    pub name: String,
    pub level: i32,
    pub description: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreateStrength {
    pub name: String,
    pub level: i32,
    pub description: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateStrength {
    pub name: Option<String>,
    pub level: Option<i32>,
    pub description: Option<String>,
}

impl Validate for CreateStrength {
    fn validate(&self) -> ValidationResult<()> {
        validate_required(&self.name, "name")?;
        validate_length(&self.name, "name", 1, 100)?;

        if self.level < 1 || self.level > 5 {
            return Err(crate::errors::AppError::ValidationError(
                "level must be between 1 and 5".to_string(),
            ));
        }

        if let Some(desc) = &self.description
            && !desc.is_empty()
        {
            validate_length(desc, "description", 1, 500)?;
        }

        Ok(())
    }
}

impl Validate for UpdateStrength {
    fn validate(&self) -> ValidationResult<()> {
        if let Some(name) = &self.name {
            validate_required(name, "name")?;
            validate_length(name, "name", 1, 100)?;
        }

        if let Some(level) = self.level
            && !(1..=5).contains(&level)
        {
            return Err(crate::errors::AppError::ValidationError(
                "level must be between 1 and 5".to_string(),
            ));
        }

        if let Some(desc) = &self.description
            && !desc.is_empty()
        {
            validate_length(desc, "description", 1, 500)?;
        }

        Ok(())
    }
}
