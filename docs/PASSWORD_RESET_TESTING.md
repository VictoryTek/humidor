# Password Reset Testing Guide

The password reset feature has been successfully implemented following Mealie's architecture. This guide will help you test it.

## 🚀 Quick Test (Without Email)

Since SMTP is not configured, reset URLs will be logged to the Docker console.

### Step 1: Navigate to Login Page
```
http://localhost:9898/login.html
```

### Step 2: Click "Forgot your password?"
This will take you to the forgot password page.

### Step 3: Enter Your Email
Enter the email address you used during setup.

### Step 4: Check Docker Logs
```powershell
docker logs humidor-web-1 --tail 20 | Select-String "Reset URL"
```

You'll see output like:
```
Reset URL (for testing): http://localhost:9898/reset-password.html?token=abc123...
```

### Step 5: Copy and Open Reset URL
Copy the full URL from the logs and paste it into your browser.

### Step 6: Enter New Password
- Enter your new password (minimum 8 characters)
- Confirm the password
- Click "Reset Password"

### Step 7: Test Login
Return to the login page and verify you can log in with your new password.

## 📧 Testing with Email (Optional)

To test the full email flow, you'll need to configure SMTP. See `PASSWORD_RESET_README.md` for configuration details.

### Recommended Test Provider: Brevo (free tier)
1. Sign up at https://www.brevo.com/
2. Get your SMTP credentials
3. Configure environment variables in `.env`:
   ```env
   BASE_URL=http://localhost:9898
   SMTP_HOST=smtp-relay.brevo.com
   SMTP_PORT=587
   SMTP_USER=your-email@example.com
   SMTP_PASSWORD=your-api-key
   SMTP_FROM_EMAIL=noreply@example.com
   ```
4. Uncomment SMTP variables in `docker-compose.yml`
5. Restart: `docker-compose restart web`

## 🔒 Security Features

✅ **Token Security**
- 64-character cryptographically secure random tokens
- Single-use tokens (deleted after successful reset)
- 30-minute expiration window
- Case-insensitive email lookup

✅ **Privacy Protection**
- Generic success message regardless of whether email exists
- Prevents email enumeration attacks
- Token validation errors return generic messages

✅ **Password Requirements**
- Minimum 8 characters (enforced client and server-side)
- Bcrypt hashing with automatic salt generation
- Password confirmation required

## 📊 Expected Behavior

### Valid Email
```json
{
  "message": "If that email exists, a password reset link has been sent"
}
```

### Invalid/Non-existent Email
```json
{
  "message": "If that email exists, a password reset link has been sent"
}
```
*Same message - this is intentional for security*

### Successful Password Reset
```json
{
  "message": "Password has been reset successfully"
}
```

### Invalid Token
```json
{
  "error": "BAD_REQUEST",
  "message": "Invalid or expired reset token"
}
```

### Expired Token (>30 minutes)
```json
{
  "error": "BAD_REQUEST",
  "message": "Reset token has expired"
}
```

## 🐛 Troubleshooting

### No Reset URL in Logs
- Check that you're using the correct email address
- Verify the user exists in the database:
  ```powershell
  docker exec -it humidor-db-1 psql -U humidor -d humidor -c "SELECT username, email FROM users;"
  ```

### Email Not Sending (when SMTP configured)
- Check Docker logs for email service errors:
  ```powershell
  docker logs humidor-web-1 | Select-String "email"
  ```
- Verify SMTP credentials are correct
- Check that your email provider allows SMTP (some require 2FA app passwords)

### Token Already Used
- Tokens are single-use and deleted after successful reset
- Request a new password reset if needed

### Password Requirements Not Met
- Minimum 8 characters required
- Both password fields must match

## 📝 Database Schema

The password reset tokens are stored in:
```sql
CREATE TABLE password_reset_tokens (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    token VARCHAR(64) NOT NULL UNIQUE,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP
);
```

Tokens are automatically cleaned up when:
- Successfully used to reset password
- Token expires (30 minutes)
- User is deleted (CASCADE)

## 🎨 Frontend Pages

- **`/forgot-password.html`** - Email input form
- **`/reset-password.html?token=...`** - Password reset form
- Login page includes "Forgot your password?" link

All pages match the existing Humidor styling with gradients and responsive design.
