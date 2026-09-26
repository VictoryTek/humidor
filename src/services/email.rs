use anyhow::Result;
use lettre::{
    SmtpTransport, Transport,
    message::{Message, header::ContentType},
    transport::smtp::authentication::Credentials,
};
use std::env;

const DEFAULT_SMTP_HOST: &str = "smtp.gmail.com";
const DEFAULT_SMTP_PORT: u16 = 587;

pub struct EmailService {
    smtp_host: String,
    smtp_port: u16,
    smtp_user: String,
    smtp_password: String,
    from_email: String,
}

impl EmailService {
    pub fn from_env() -> Result<Self> {
        Ok(EmailService {
            smtp_host: env::var("SMTP_HOST").unwrap_or_else(|_| DEFAULT_SMTP_HOST.to_string()),
            smtp_port: env::var("SMTP_PORT")
                .ok()
                .and_then(|p| p.parse().ok())
                .unwrap_or(DEFAULT_SMTP_PORT),
            smtp_user: env::var("SMTP_USER")?,
            smtp_password: env::var("SMTP_PASSWORD")?,
            from_email: env::var("SMTP_FROM_EMAIL")?,
        })
    }

    /// Check the environment against exactly what `from_env`/the mailer use: `SMTP_USER`,
    /// `SMTP_PASSWORD` and `SMTP_FROM_EMAIL` are required (non-empty); `SMTP_HOST`/`SMTP_PORT`
    /// have defaults but must be valid if set. Returns every problem found.
    pub fn validate_env() -> std::result::Result<(), Vec<String>> {
        Self::validate_with(|name| env::var(name).ok())
    }

    fn validate_with(get: impl Fn(&str) -> Option<String>) -> std::result::Result<(), Vec<String>> {
        let mut problems = Vec::new();
        let non_empty = |name: &str| get(name).filter(|v| !v.trim().is_empty());

        for name in ["SMTP_USER", "SMTP_PASSWORD", "SMTP_FROM_EMAIL"] {
            if non_empty(name).is_none() {
                problems.push(format!("{name} is missing or empty"));
            }
        }
        if let Some(from) = non_empty("SMTP_FROM_EMAIL")
            && from.parse::<lettre::message::Mailbox>().is_err()
        {
            problems.push("SMTP_FROM_EMAIL is not a valid email address".to_string());
        }
        if let Some(port) = non_empty("SMTP_PORT")
            && port.parse::<u16>().is_err()
        {
            problems.push("SMTP_PORT is not a valid port number".to_string());
        }

        if problems.is_empty() {
            Ok(())
        } else {
            Err(problems)
        }
    }

    /// Effective sender/host/port for logging, applying the same defaults as `from_env`.
    pub fn describe_env() -> (String, String, String) {
        (
            env::var("SMTP_HOST").unwrap_or_else(|_| DEFAULT_SMTP_HOST.to_string()),
            env::var("SMTP_PORT").unwrap_or_else(|_| DEFAULT_SMTP_PORT.to_string()),
            env::var("SMTP_FROM_EMAIL").unwrap_or_default(),
        )
    }

    pub async fn send_password_reset_email(&self, to_email: &str, reset_url: &str) -> Result<()> {
        let html_body = format!(
            r#"
<!DOCTYPE html>
<html>
<head>
    <meta charset="utf-8">
    <style>
        body {{ font-family: Arial, sans-serif; line-height: 1.6; color: #333; }}
        .container {{ max-width: 600px; margin: 0 auto; padding: 20px; }}
        .header {{ background: linear-gradient(135deg, #8B6914 0%, #D4AF37 100%); color: white; padding: 30px; text-align: center; border-radius: 10px 10px 0 0; }}
        .content {{ background: #f9f9f9; padding: 30px; border-radius: 0 0 10px 10px; }}
        .button {{ display: inline-block; background: #D4AF37; color: white; padding: 12px 30px; text-decoration: none; border-radius: 5px; margin: 20px 0; font-weight: bold; }}
        .button:hover {{ background: #8B6914; }}
        .footer {{ text-align: center; color: #666; font-size: 0.9em; margin-top: 20px; }}
    </style>
</head>
<body>
    <div class="container">
        <div class="header">
            <h1>🔐 Password Reset Request</h1>
        </div>
        <div class="content">
            <p>Hello,</p>
            <p>You have requested to reset your password for your Humidor account. Click the button below to reset your password:</p>
            <p style="text-align: center;">
                <a href="{}" class="button">Reset Password</a>
            </p>
            <p>Or copy and paste this link into your browser:</p>
            <p style="word-break: break-all; background: white; padding: 10px; border-radius: 5px;">{}</p>
            <p><strong>This link will expire in 30 minutes for security reasons.</strong></p>
            <p>If you didn't request a password reset, please ignore this email. Your password will remain unchanged.</p>
            <div class="footer">
                <p>© 2025 Humidor - Cigar Inventory Management</p>
            </div>
        </div>
    </div>
</body>
</html>
            "#,
            reset_url, reset_url
        );

        let email = Message::builder()
            .from(self.from_email.parse()?)
            .to(to_email.parse()?)
            .subject("Reset Your Humidor Password")
            .header(ContentType::TEXT_HTML)
            .body(html_body)?;

        let creds = Credentials::new(self.smtp_user.clone(), self.smtp_password.clone());

        let mailer = SmtpTransport::relay(&self.smtp_host)?
            .port(self.smtp_port)
            .credentials(creds)
            .build();

        mailer.send(&email)?;

        tracing::info!(
            recipient = %to_email,
            subject = "Reset Your Humidor Password",
            "Password reset email sent successfully"
        );
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::EmailService;
    use std::collections::HashMap;

    fn check(vars: &[(&str, &str)]) -> Result<(), Vec<String>> {
        let map: HashMap<String, String> = vars
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect();
        EmailService::validate_with(|name| map.get(name).cloned())
    }

    #[test]
    fn documented_config_is_valid_without_host_or_port() {
        assert!(
            check(&[
                ("SMTP_USER", "me@example.com"),
                ("SMTP_PASSWORD", "app-password"),
                ("SMTP_FROM_EMAIL", "noreply@example.com"),
            ])
            .is_ok()
        );
    }

    #[test]
    fn names_the_variables_the_mailer_actually_reads() {
        let problems = check(&[]).unwrap_err().join(" | ");
        assert!(problems.contains("SMTP_USER"));
        assert!(problems.contains("SMTP_PASSWORD"));
        assert!(problems.contains("SMTP_FROM_EMAIL"));
        // The old validator's names must not be demanded.
        assert!(!problems.contains("SMTP_USERNAME"));
        assert!(!problems.contains("SMTP_FROM "));
    }

    #[test]
    fn old_variable_names_alone_do_not_count_as_configured() {
        assert!(
            check(&[
                ("SMTP_USERNAME", "me"),
                ("SMTP_PASSWORD", "pw"),
                ("SMTP_FROM", "noreply@example.com"),
            ])
            .is_err()
        );
    }

    #[test]
    fn rejects_empty_values_bad_from_address_and_bad_port() {
        let problems = check(&[
            ("SMTP_USER", "  "),
            ("SMTP_PASSWORD", "pw"),
            ("SMTP_FROM_EMAIL", "not-an-address"),
            ("SMTP_PORT", "smtp"),
        ])
        .unwrap_err()
        .join(" | ");
        assert!(problems.contains("SMTP_USER is missing or empty"));
        assert!(problems.contains("SMTP_FROM_EMAIL is not a valid email address"));
        assert!(problems.contains("SMTP_PORT is not a valid port number"));
    }

    #[test]
    fn empty_port_is_treated_as_unset() {
        assert!(
            check(&[
                ("SMTP_USER", "u"),
                ("SMTP_PASSWORD", "p"),
                ("SMTP_FROM_EMAIL", "a@b.co"),
                ("SMTP_PORT", ""),
            ])
            .is_ok()
        );
    }
}
