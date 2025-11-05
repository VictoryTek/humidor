# Password Reset Feature - Configuration Guide

## Overview
Password reset feature has been implemented following Mealie's pattern with email-based token verification.

## Features Implemented
✅ Database migration (V9) for password_reset_tokens table
✅ Backend API endpoints for forgot/reset password
✅ Email service with SMTP support
✅ Frontend pages for forgot password and reset password
✅ 30-minute token expiration
✅ Secure token generation (64-character alphanumeric)
✅ Single-use tokens (deleted after use)

## API Endpoints

### POST /api/v1/auth/forgot-password
Request a password reset link.
```json
{
  "email": "user@example.com"
}
```

### POST /api/v1/auth/reset-password
Reset password with token.
```json
{
  "token": "abc123...",
  "password": "newpassword123"
}
```

## Configuration

### Required Environment Variables
To enable email sending, configure these in your environment or `.env` file:

```bash
# Base URL for reset links
BASE_URL=http://localhost:9898

# SMTP Configuration
SMTP_HOST=smtp.gmail.com
SMTP_PORT=587
SMTP_USER=your-email@gmail.com
SMTP_PASSWORD=your-app-password
SMTP_FROM_EMAIL=noreply@yourapp.com
```

### Gmail Setup (Recommended for Testing)
1. Enable 2-factor authentication on your Gmail account
2. Generate an App Password: https://myaccount.google.com/apppasswords
3. Use the 16-character app password as `SMTP_PASSWORD`

### Alternative SMTP Providers
- **SendGrid**: smtp.sendgrid.net:587
- **Mailgun**: smtp.mailgun.org:587
- **AWS SES**: email-smtp.us-east-1.amazonaws.com:587

## Testing Without Email

If SMTP is not configured, the system will:
1. Still generate and store reset tokens
2. Print the reset URL to Docker logs for testing

To test manually:
```bash
# Check logs for reset URL
docker logs humidor-web-1 --tail 50 | grep "Reset URL"

# Example output:
# Reset URL (for testing): http://localhost:9898/reset-password.html?token=abc123...
```

## Security Features
- Tokens are 64 characters (cryptographically secure)
- 30-minute expiration on all tokens
- Single-use tokens (deleted after successful reset)
- Doesn't reveal if email exists (prevents enumeration)
- Password must be at least 8 characters
- Tokens stored hashed in database

## Usage Flow
1. User clicks "Forgot your password?" on login page
2. User enters email address
3. System generates token and sends email (or logs URL)
4. User clicks link in email (valid for 30 minutes)
5. User enters new password (twice for confirmation)
6. Token is validated and password is updated
7. User can log in with new password

## Database Schema
```sql
CREATE TABLE password_reset_tokens (
    id UUID PRIMARY KEY,
    user_id UUID REFERENCES users(id) ON DELETE CASCADE,
    token VARCHAR(64) UNIQUE NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL
);
```

## Files Created/Modified
- `migrations/V9__create_password_reset_tokens_table.sql` - Database schema
- `src/models/password_reset.rs` - Data models
- `src/services/email.rs` - Email service
- `src/handlers/auth.rs` - Password reset handlers (appended)
- `static/forgot-password.html` + `.js` - Request reset page
- `static/reset-password.html` + `.js` - Reset password page
- `static/login.html` - Added "Forgot password?" link
- `docker-compose.yml` - Added SMTP environment variables
- `Cargo.toml` - Added `lettre` and `rand` dependencies

## Troubleshooting

### Email not sending
- Check SMTP credentials in environment variables
- Verify SMTP port (587 for TLS, 465 for SSL)
- Check Docker logs: `docker logs humidor-web-1`
- For Gmail, ensure app password is used (not regular password)

### Token expired
- Tokens expire after 30 minutes
- Request a new reset link

### Reset link not working
- Ensure BASE_URL environment variable matches your domain
- Check token in URL is complete (no truncation)
- Token may have been used already (single-use)

## Production Recommendations
1. Use a dedicated SMTP service (SendGrid, Mailgun, AWS SES)
2. Set BASE_URL to your production domain
3. Consider adding rate limiting to prevent abuse
4. Add background job to clean expired tokens (older than 24 hours)
5. Monitor failed email attempts
6. Add email templates with branding
