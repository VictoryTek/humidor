# Docker Secrets Directory

This directory contains sensitive credentials for Docker Compose services.

## Files

- `db_user.txt` - PostgreSQL database username
- `db_password.txt` - PostgreSQL database password  
- `jwt_secret.txt` - JWT signing secret for authentication

## Setup

1. Copy the `.example` files and remove the `.example` extension:
   ```bash
   cp db_user.txt.example db_user.txt
   cp db_password.txt.example db_password.txt
   cp jwt_secret.txt.example jwt_secret.txt
   ```

2. Edit each file with your secure values:
   - `db_user.txt`: Your database username
   - `db_password.txt`: A strong database password
   - `jwt_secret.txt`: Generate with `openssl rand -base64 32`

## Security Notes

- **NEVER commit actual secret files to git** - only `.example` files should be committed
- The actual `.txt` files are in `.gitignore`
- In production, use proper secrets management (AWS Secrets Manager, Azure Key Vault, etc.)
- Rotate secrets regularly
- Use strong, randomly generated passwords

## Production Deployment

For production, consider using:
- Docker Swarm secrets
- Kubernetes secrets
- Cloud provider secret managers (AWS Secrets Manager, Azure Key Vault, Google Secret Manager)
- HashiCorp Vault

These provide better security than file-based secrets.
