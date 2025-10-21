use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Cigar {
    pub id: Uuid,
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
    pub humidor_location: Option<String>,
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
    pub humidor_location: Option<String>,
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
    pub humidor_location: Option<String>,
}