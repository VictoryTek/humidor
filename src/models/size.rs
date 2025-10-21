use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

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
