use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RingGauge {
    pub id: Uuid,
    pub gauge: i32,
    pub description: Option<String>,
    pub common_names: Option<Vec<String>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreateRingGauge {
    pub gauge: i32,
    pub description: Option<String>,
    pub common_names: Option<Vec<String>>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateRingGauge {
    pub gauge: Option<i32>,
    pub description: Option<String>,
    pub common_names: Option<Vec<String>>,
}
