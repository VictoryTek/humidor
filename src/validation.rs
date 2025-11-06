use crate::errors::AppError;

/// Validation result type
pub type ValidationResult<T> = Result<T, AppError>;

/// Validate string length
pub fn validate_length(
    value: &str,
    field_name: &str,
    min: usize,
    max: usize,
) -> ValidationResult<()> {
    let len = value.len();
    if len < min {
        return Err(AppError::ValidationError(format!(
            "{} must be at least {} characters",
            field_name, min
        )));
    }
    if len > max {
        return Err(AppError::ValidationError(format!(
            "{} must be at most {} characters",
            field_name, max
        )));
    }
    Ok(())
}

/// Validate required string field
pub fn validate_required(value: &str, field_name: &str) -> ValidationResult<()> {
    if value.trim().is_empty() {
        Err(AppError::ValidationError(format!(
            "{} is required",
            field_name
        )))
    } else {
        Ok(())
    }
}

/// Validate range for f64
pub fn validate_range_f64(
    value: f64,
    min: f64,
    max: f64,
    field_name: &str,
) -> ValidationResult<()> {
    if value >= min && value <= max {
        Ok(())
    } else {
        Err(AppError::ValidationError(format!(
            "{} must be between {} and {}",
            field_name, min, max
        )))
    }
}

/// Validate email format
#[allow(dead_code)]
pub fn validate_email(email: &str) -> ValidationResult<()> {
    // This regex pattern is well-tested and should never fail to compile
    let email_regex = regex::Regex::new(
        r"^[a-zA-Z0-9.!#$%&'*+/=?^_`{|}~-]+@[a-zA-Z0-9](?:[a-zA-Z0-9-]{0,61}[a-zA-Z0-9])?(?:\.[a-zA-Z0-9](?:[a-zA-Z0-9-]{0,61}[a-zA-Z0-9])?)*$"
    ).map_err(|e| {
        // This should never happen with a valid regex pattern
        tracing::error!(error = %e, "Failed to compile email regex - this indicates a code error");
        AppError::InternalServerError("Email validation regex compilation failed".to_string())
    })?;

    if email_regex.is_match(email) {
        Ok(())
    } else {
        Err(AppError::ValidationError(
            "Invalid email format".to_string(),
        ))
    }
}

/// Validate positive integer
pub fn validate_positive(value: i32, field_name: &str) -> ValidationResult<()> {
    if value > 0 {
        Ok(())
    } else {
        Err(AppError::ValidationError(format!(
            "{} must be greater than 0",
            field_name
        )))
    }
}

/// Validate that a value is non-negative (allows 0)
pub fn validate_non_negative(value: i32, field_name: &str) -> ValidationResult<()> {
    if value >= 0 {
        Ok(())
    } else {
        Err(AppError::ValidationError(format!(
            "{} must be 0 or greater",
            field_name
        )))
    }
}

/// Validate range
pub fn validate_range(value: i32, field_name: &str, min: i32, max: i32) -> ValidationResult<()> {
    if value >= min && value <= max {
        Ok(())
    } else {
        Err(AppError::ValidationError(format!(
            "{} must be between {} and {}",
            field_name, min, max
        )))
    }
}

/// Validate URL format
pub fn validate_url(url: &str) -> ValidationResult<()> {
    if url.is_empty() {
        return Ok(());
    }

    if url.starts_with("http://") || url.starts_with("https://") {
        Ok(())
    } else {
        Err(AppError::ValidationError(
            "URL must start with http:// or https://".to_string(),
        ))
    }
}

/// Trait for validatable models
pub trait Validate {
    fn validate(&self) -> ValidationResult<()>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_length() {
        assert!(validate_length("test", "field", 1, 10).is_ok());
        assert!(validate_length("", "field", 1, 10).is_err());
        assert!(validate_length("toolongvalue", "field", 1, 5).is_err());
    }

    #[test]
    fn test_validate_email() {
        assert!(validate_email("test@example.com").is_ok());
        assert!(validate_email("invalid-email").is_err());
        assert!(validate_email("@example.com").is_err());
    }

    #[test]
    fn test_validate_positive() {
        assert!(validate_positive(1, "field").is_ok());
        assert!(validate_positive(0, "field").is_err());
        assert!(validate_positive(-1, "field").is_err());
    }

    #[test]
    fn test_validate_range() {
        assert!(validate_range(5, "field", 1, 10).is_ok());
        assert!(validate_range(0, "field", 1, 10).is_err());
        assert!(validate_range(11, "field", 1, 10).is_err());
    }
}
