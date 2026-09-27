use crate::validation::{
    Validate, ValidationResult, validate_length, validate_positive, validate_range,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
pub struct SmokingSession {
    pub id: Uuid,
    pub user_id: Uuid,
    pub cigar_id: Uuid,
    pub smoked_at: DateTime<Utc>,
    pub rating: Option<i16>,
    pub duration_minutes: Option<i32>,
    pub pairing: Option<String>,
    pub notes: Option<String>,
    pub created_at: DateTime<Utc>,
}

/// A logged session joined with the cigar's name/brand, for the "my sessions" history view.
#[derive(Debug, Serialize)]
pub struct SmokingSessionWithCigar {
    #[serde(flatten)]
    pub session: SmokingSession,
    pub cigar_name: String,
    pub brand_name: Option<String>,
}

/// All fields optional: an empty `{}` logs "smoked this, no details".
#[derive(Debug, Deserialize, Default)]
pub struct CreateSmokingSession {
    pub smoked_at: Option<DateTime<Utc>>,
    pub rating: Option<i16>,
    pub duration_minutes: Option<i32>,
    pub pairing: Option<String>,
    pub notes: Option<String>,
}

impl Validate for CreateSmokingSession {
    fn validate(&self) -> ValidationResult<()> {
        if let Some(rating) = self.rating {
            validate_range(rating as i32, "Rating", 1, 5)?;
        }
        if let Some(duration) = self.duration_minutes {
            validate_positive(duration, "Duration (minutes)")?;
        }
        if let Some(pairing) = &self.pairing {
            validate_length(pairing, "Pairing", 0, 200)?;
        }
        if let Some(notes) = &self.notes {
            validate_length(notes, "Notes", 0, 1500)?;
        }
        Ok(())
    }
}

/// Response for `POST /cigars/:id/sessions`: the created session plus the cigar's new stock state.
#[derive(Debug, Serialize)]
pub struct SmokingSessionResponse {
    #[serde(flatten)]
    pub session: SmokingSession,
    pub cigar_quantity: i32,
    pub cigar_is_active: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn session(rating: Option<i16>, duration: Option<i32>) -> CreateSmokingSession {
        CreateSmokingSession {
            rating,
            duration_minutes: duration,
            ..Default::default()
        }
    }

    #[test]
    fn empty_session_is_valid() {
        assert!(CreateSmokingSession::default().validate().is_ok());
    }

    #[test]
    fn rating_must_be_between_1_and_5() {
        for r in [1, 3, 5] {
            assert!(
                session(Some(r), None).validate().is_ok(),
                "rating {r} should be valid"
            );
        }
        for r in [0, -1, 6, 100] {
            assert!(
                session(Some(r), None).validate().is_err(),
                "rating {r} should be rejected"
            );
        }
    }

    #[test]
    fn duration_must_be_positive() {
        assert!(session(None, Some(30)).validate().is_ok());
        assert!(session(None, Some(0)).validate().is_err());
        assert!(session(None, Some(-5)).validate().is_err());
    }

    #[test]
    fn overly_long_pairing_or_notes_are_rejected() {
        let mut s = CreateSmokingSession {
            pairing: Some("x".repeat(201)),
            ..Default::default()
        };
        assert!(s.validate().is_err());
        s.pairing = None;
        s.notes = Some("x".repeat(1501));
        assert!(s.validate().is_err());
    }
}
