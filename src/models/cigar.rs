use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use crate::validation::{Validate, ValidationResult, validate_length, validate_positive};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Cigar {
    pub id: Uuid,
    pub humidor_id: Option<Uuid>,
    pub brand: String,
    pub name: String,
    pub size: String,
    pub strength: String,
    pub origin: String,
    pub wrapper: Option<String>,
    pub binder: Option<String>,
    pub filler: Option<String>,
    pub price: Option<f64>,
    pub purchase_date: Option<DateTime<Utc>>,
    pub notes: Option<String>,
    pub quantity: i32,
    pub ring_gauge: Option<i32>,
    pub length: Option<f64>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreateCigar {
    pub brand: String,
    pub name: String,
    pub size: String,
    pub strength: String,
    pub origin: String,
    pub wrapper: Option<String>,
    pub binder: Option<String>,
    pub filler: Option<String>,
    pub price: Option<f64>,
    pub purchase_date: Option<DateTime<Utc>>,
    pub notes: Option<String>,
    pub quantity: i32,
    pub ring_gauge: Option<i32>,
    pub length: Option<f64>,
    pub humidor_id: Option<Uuid>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateCigar {
    pub brand: Option<String>,
    pub name: Option<String>,
    pub size: Option<String>,
    pub strength: Option<String>,
    pub origin: Option<String>,
    pub wrapper: Option<String>,
    pub binder: Option<String>,
    pub filler: Option<String>,
    pub price: Option<f64>,
    pub purchase_date: Option<DateTime<Utc>>,
    pub notes: Option<String>,
    pub quantity: Option<i32>,
    pub ring_gauge: Option<i32>,
    pub length: Option<f64>,
    pub humidor_id: Option<Uuid>,
}

impl Validate for CreateCigar {
    fn validate(&self) -> ValidationResult<()> {
        validate_length(&self.brand, "Brand", 1, 100)?;
        validate_length(&self.name, "Name", 1, 100)?;
        validate_length(&self.size, "Size", 1, 50)?;
        validate_length(&self.strength, "Strength", 1, 50)?;
        validate_length(&self.origin, "Origin", 1, 100)?;
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
        // humidor_id is a UUID, no validation needed
        
        Ok(())
    }
}

impl Validate for UpdateCigar {
    fn validate(&self) -> ValidationResult<()> {
        if let Some(brand) = &self.brand {
            validate_length(brand, "Brand", 1, 100)?;
        }
        if let Some(name) = &self.name {
            validate_length(name, "Name", 1, 100)?;
        }
        if let Some(size) = &self.size {
            validate_length(size, "Size", 1, 50)?;
        }
        if let Some(strength) = &self.strength {
            validate_length(strength, "Strength", 1, 50)?;
        }
        if let Some(origin) = &self.origin {
            validate_length(origin, "Origin", 1, 100)?;
        }
        if let Some(quantity) = self.quantity {
            validate_positive(quantity, "Quantity")?;
        }
        
        Ok(())
    }
}