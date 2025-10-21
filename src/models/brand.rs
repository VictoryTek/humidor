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
