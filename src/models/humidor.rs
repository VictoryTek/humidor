use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};

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