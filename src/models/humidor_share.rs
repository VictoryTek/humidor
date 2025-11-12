use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use uuid::Uuid;

/// Permission level for shared humidor access
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum PermissionLevel {
    /// Read-only access to view cigars in the humidor
    View,
    /// Can add and edit cigars (but not delete)
    Edit,
    /// Full access: add, edit, delete cigars and manage sharing
    Full,
}

impl PermissionLevel {
    pub fn as_str(&self) -> &str {
        match self {
            PermissionLevel::View => "view",
            PermissionLevel::Edit => "edit",
            PermissionLevel::Full => "full",
        }
    }

    /// Check if this permission level allows viewing
    pub fn can_view(&self) -> bool {
        matches!(
            self,
            PermissionLevel::View | PermissionLevel::Edit | PermissionLevel::Full
        )
    }

    /// Check if this permission level allows editing (add/update cigars)
    pub fn can_edit(&self) -> bool {
        matches!(self, PermissionLevel::Edit | PermissionLevel::Full)
    }

    /// Check if this permission level allows full management (including delete and sharing)
    pub fn can_manage(&self) -> bool {
        matches!(self, PermissionLevel::Full)
    }
}

impl FromStr for PermissionLevel {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "view" => Ok(PermissionLevel::View),
            "edit" => Ok(PermissionLevel::Edit),
            "full" => Ok(PermissionLevel::Full),
            _ => Err(format!("Invalid permission level: {}", s)),
        }
    }
}

/// Database model for a humidor share
#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HumidorShare {
    pub id: Uuid,
    pub humidor_id: Uuid,
    pub shared_with_user_id: Uuid,
    pub shared_by_user_id: Uuid,
    pub permission_level: PermissionLevel,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Request to share a humidor with a user
#[derive(Debug, Deserialize)]
pub struct ShareHumidorRequest {
    pub user_id: Uuid,
    pub permission_level: PermissionLevel,
}

/// Request to update share permissions
#[derive(Debug, Deserialize)]
pub struct UpdateSharePermissionRequest {
    pub permission_level: PermissionLevel,
}

/// Response containing share information
#[derive(Debug, Serialize)]
pub struct HumidorShareResponse {
    pub id: Uuid,
    pub humidor_id: Uuid,
    pub shared_with_user: UserInfo,
    pub shared_by_user: UserInfo,
    pub permission_level: PermissionLevel,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// User information included in share responses
#[derive(Debug, Serialize)]
pub struct UserInfo {
    pub id: Uuid,
    pub username: String,
    pub email: String,
    pub full_name: Option<String>,
}

/// Response containing a list of shares for a humidor
#[derive(Debug, Serialize)]
pub struct HumidorSharesListResponse {
    pub shares: Vec<HumidorShareResponse>,
    pub total: usize,
}

/// Response containing humidors shared with the current user
#[derive(Debug, Serialize)]
pub struct SharedHumidorsResponse {
    pub humidors: Vec<SharedHumidorInfo>,
    pub total: usize,
}

/// Information about a humidor shared with the user
#[derive(Debug, Serialize)]
pub struct SharedHumidorInfo {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub owner: UserInfo,
    pub permission_level: PermissionLevel,
    pub shared_at: DateTime<Utc>,
    pub cigar_count: i64,
}
