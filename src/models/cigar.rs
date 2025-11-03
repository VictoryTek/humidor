use crate::validation::{
    validate_length, validate_non_negative, validate_positive, Validate, ValidationResult,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Cigar {
    pub id: Uuid,
    pub humidor_id: Option<Uuid>,
    pub brand_id: Option<Uuid>,
    pub name: String,
    pub size_id: Option<Uuid>,
    pub strength_id: Option<Uuid>,
    pub origin_id: Option<Uuid>,
    pub wrapper: Option<String>,
    pub binder: Option<String>,
    pub filler: Option<String>,
    pub price: Option<f64>,
    pub purchase_date: Option<DateTime<Utc>>,
    pub notes: Option<String>,
    pub quantity: i32,
    pub ring_gauge_id: Option<Uuid>,
    pub length: Option<f64>,
    pub image_url: Option<String>,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreateCigar {
    pub brand_id: Option<Uuid>,
    pub name: String,
    pub size_id: Option<Uuid>,
    pub strength_id: Option<Uuid>,
    pub origin_id: Option<Uuid>,
    pub wrapper: Option<String>,
    pub binder: Option<String>,
    pub filler: Option<String>,
    pub price: Option<f64>,
    pub purchase_date: Option<DateTime<Utc>>,
    pub notes: Option<String>,
    pub quantity: i32,
    pub ring_gauge_id: Option<Uuid>,
    pub length: Option<f64>,
    pub humidor_id: Option<Uuid>,
    pub image_url: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateCigar {
    pub brand_id: Option<Uuid>,
    pub name: Option<String>,
    pub size_id: Option<Uuid>,
    pub strength_id: Option<Uuid>,
    pub origin_id: Option<Uuid>,
    pub wrapper: Option<String>,
    pub binder: Option<String>,
    pub filler: Option<String>,
    pub price: Option<f64>,
    pub purchase_date: Option<DateTime<Utc>>,
    pub notes: Option<String>,
    pub quantity: Option<i32>,
    pub ring_gauge_id: Option<Uuid>,
    pub length: Option<f64>,
    pub humidor_id: Option<Uuid>,
    pub image_url: Option<String>,
}

impl Validate for CreateCigar {
    fn validate(&self) -> ValidationResult<()> {
        validate_length(&self.name, "Name", 1, 100)?;
        validate_positive(self.quantity, "Quantity")?;

        if let Some(wrapper) = &self.wrapper {
            validate_length(wrapper, "Wrapper", 1, 100)?;
        }
        if let Some(binder) = &self.binder {
            validate_length(binder, "Binder", 1, 100)?;
        }
        if let Some(filler) = &self.filler {
            validate_length(filler, "Filler", 1, 100)?;
        }
        if let Some(notes) = &self.notes {
            validate_length(notes, "Notes", 0, 1000)?;
        }
        // UUIDs don't need string length validation

        Ok(())
    }
}

impl Validate for UpdateCigar {
    fn validate(&self) -> ValidationResult<()> {
        if let Some(name) = &self.name {
            validate_length(name, "Name", 1, 100)?;
        }
        if let Some(quantity) = self.quantity {
            validate_non_negative(quantity, "Quantity")?;
        }
        // UUIDs don't need string length validation

        Ok(())
    }
}
