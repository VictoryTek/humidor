use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Strength {
    pub id: Uuid,
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
