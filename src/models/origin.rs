use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Origin {
    pub id: Uuid,
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
