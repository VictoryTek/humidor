use anyhow::Result;
use lettre::{
    SmtpTransport, Transport,
    message::{Message, header::ContentType},
    transport::smtp::authentication::Credentials,
};
use std::env;

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
            smtp_host: env::var("SMTP_HOST").unwrap_or_else(|_| "smtp.gmail.com".to_string()),
            smtp_port: env::var("SMTP_PORT")
                .ok()
                .and_then(|p| p.parse().ok())
                .unwrap_or(587),
            smtp_user: env::var("SMTP_USER")?,
            smtp_password: env::var("SMTP_PASSWORD")?,
            from_email: env::var("SMTP_FROM_EMAIL")?,
        })
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
