# Docker Compose Secrets Implementation

## Overview

Issue 6.2 has been addressed by implementing Docker Compose secrets for sensitive data instead of using plain environment variables.

## Changes Made

### 1. Docker Compose Configuration

**File**: `docker-compose.yml`

- Added `secrets` section defining three secret files:
  - `db_user` - Database username
  - `db_password` - Database password
  - `jwt_secret` - JWT signing secret

- **Database service (`db`)**:
  - Changed from `POSTGRES_USER` and `POSTGRES_PASSWORD` to `POSTGRES_USER_FILE` and `POSTGRES_PASSWORD_FILE`
  - Secrets mounted at `/run/secrets/` 
  - Health check updated to read username from secret file

- **Web service (`web`)**:
  - Removed plain `DATABASE_URL` and `JWT_SECRET` environment variables
  - Added `DATABASE_URL_TEMPLATE` with placeholders `{{DB_USER}}` and `{{DB_PASSWORD}}`
  - All three secrets mounted at `/run/secrets/`

### 2. Application Code

**File**: `src/main.rs`

- Added `read_secret()` helper function that:
  - Tries to read from Docker secrets at `/run/secrets/<secret_name>` first
  - Falls back to environment variable if secret file not found
  - Logs the source of the secret (docker_secret vs environment)

- Updated DATABASE_URL construction:
  - Detects `DATABASE_URL_TEMPLATE` environment variable
  - Reads `db_user` and `db_password` from secrets
  - Replaces placeholders in template with actual values
  - Falls back to `DATABASE_URL` env var for backwards compatibility

**File**: `src/handlers/auth.rs`

- Updated `jwt_secret()` function to:
  - Try reading from `/run/secrets/jwt_secret` first
  - Fall back to `JWT_SECRET` environment variable
  - Maintains startup panic if neither is available (security-first approach)

### 3. Secrets Directory Structure

```
secrets/
├── README.md                    # Documentation
├── db_user.txt                  # Actual username (gitignored)
├── db_password.txt              # Actual password (gitignored)
├── jwt_secret.txt               # Actual JWT secret (gitignored)
├── db_user.txt.example          # Template file (committed)
├── db_password.txt.example      # Template file (committed)
└── jwt_secret.txt.example       # Template file (committed)
```

### 4. Security Improvements

**File**: `.gitignore`

- Added entries to exclude actual secret files:
  ```
  .env
  secrets/*.txt
  !secrets/*.txt.example
  ```

- Only `.example` files are committed to version control
- Actual secrets must be created locally or in CI/CD pipelines

## Benefits

1. **Security**: Secrets are not exposed in environment variables or docker-compose.yml
2. **Docker Swarm Ready**: Secrets work with both Docker Compose and Docker Swarm
3. **Rotation**: Secrets can be updated by changing files and restarting containers
4. **Auditing**: Secret access is logged with structured logging
5. **Backwards Compatible**: Falls back to environment variables for local development

## Usage

### Development Setup

1. Create secret files from examples:
   ```bash
   cd secrets
   cp db_user.txt.example db_user.txt
   cp db_password.txt.example db_password.txt
   cp jwt_secret.txt.example jwt_secret.txt
   ```

2. Edit secret files with your values:
   ```bash
   echo "your_username" > db_user.txt
   echo "your_secure_password" > db_password.txt
   openssl rand -base64 32 > jwt_secret.txt  # Or use PowerShell to generate
   ```

3. Start services:
   ```bash
   docker-compose up -d
   ```

### Production Deployment

For production, use proper secrets management:

- **Docker Swarm**: Use `docker secret create`
- **Kubernetes**: Use Kubernetes Secrets or External Secrets Operator
- **Cloud Providers**:
  - AWS: AWS Secrets Manager
  - Azure: Azure Key Vault
  - GCP: Google Secret Manager
- **HashiCorp Vault**: For enterprise secret management

## Verification

### Check Secrets are Mounted

```bash
# Web container
docker-compose exec web ls -la /run/secrets/

# Database container
docker-compose exec db ls -la /run/secrets/
```

### View Logs for Secret Loading

```bash
docker-compose logs web | grep secret
```

Look for structured log entries showing secrets being read from Docker secrets.

## Security Best Practices

1. **Never commit actual secret files** - only `.example` templates
2. **Use strong, random passwords** - minimum 32 characters
3. **Rotate secrets regularly** - especially JWT_SECRET
4. **Restrict file permissions** - secrets should be readable only by necessary users
5. **Use different secrets per environment** - dev, staging, production
6. **Monitor secret access** - review logs for unauthorized access attempts

## Troubleshooting

### Container fails to start

- Check that secret files exist in `./secrets/` directory
- Verify file permissions (readable by Docker)
- Check logs: `docker-compose logs web`

### Database connection fails

- Verify `db_user.txt` and `db_password.txt` match PostgreSQL configuration
- Check that `DATABASE_URL_TEMPLATE` environment variable is set
- Ensure placeholders `{{DB_USER}}` and `{{DB_PASSWORD}}` are present

### JWT token generation fails

- Verify `jwt_secret.txt` exists and contains a value
- Check application logs for JWT_SECRET errors
- Ensure secret is at least 32 characters for security

## Migration from Environment Variables

If migrating from environment variables:

1. Create secret files from current values:
   ```bash
   echo "$POSTGRES_USER" > secrets/db_user.txt
   echo "$POSTGRES_PASSWORD" > secrets/db_password.txt
   echo "$JWT_SECRET" > secrets/jwt_secret.txt
   ```

2. Update docker-compose.yml (already done in this implementation)

3. Restart services:
   ```bash
   docker-compose down
   docker-compose up -d
   ```

4. Verify secrets are being used from logs

5. Remove old environment variables from `.env` file

## Backwards Compatibility

The implementation maintains backwards compatibility:

- If Docker secrets are not found, falls back to environment variables
- Existing `.env` files continue to work for local development
- Gradual migration path from environment variables to secrets
