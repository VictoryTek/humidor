use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Request to create a public share link
#[derive(Debug, Deserialize)]
pub struct CreatePublicShareRequest {
    /// Optional expiration date. If None, defaults to 30 days from now.
    /// To make permanent, explicitly pass a very far future date or set a flag
    pub expires_at: Option<DateTime<Utc>>,
    /// If true, link never expires (overrides expires_at)
    #[serde(default)]
    pub never_expires: bool,
    /// Include favorites in the public share
    #[serde(default)]
    pub include_favorites: bool,
    /// Include wish list in the public share
    #[serde(default)]
    pub include_wish_list: bool,
    /// Optional label to identify this share
    pub label: Option<String>,
}

/// Response containing public share information
#[derive(Debug, Serialize)]
pub struct PublicShareResponse {
    pub token_id: Uuid,
    pub share_url: String,
    pub expires_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub include_favorites: bool,
    pub include_wish_list: bool,
    pub label: Option<String>,
}

/// Public humidor data returned for valid share tokens (no authentication required)
#[derive(Debug, Serialize)]
pub struct PublicHumidorResponse {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub image_url: Option<String>,
    pub created_at: DateTime<Utc>,
    pub owner: PublicUserInfo,
    pub cigars: Vec<PublicCigarResponse>,
    pub cigar_count: usize,
    pub favorites: Option<Vec<PublicCigarResponse>>,
    pub wish_list: Option<Vec<PublicCigarResponse>>,
}

/// User information included in public responses (limited data for privacy)
#[derive(Debug, Serialize)]
pub struct PublicUserInfo {
    pub username: String,
    pub full_name: Option<String>,
}

/// Cigar information for public sharing
#[derive(Debug, Serialize)]
pub struct PublicCigarResponse {
    pub id: Uuid,
    pub name: String,
    pub brand: Option<String>,
    pub origin: Option<String>,
    pub wrapper: Option<String>,
    pub strength: Option<String>,
    pub ring_gauge: Option<i32>,
    pub length_inches: Option<f64>,
    pub quantity: i32,
    pub notes: Option<String>,
    pub retail_link: Option<String>,
    pub image_url: Option<String>,
}
