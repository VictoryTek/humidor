# Docker Configuration

This directory contains all Docker-related files for the Humidor project, following best practices similar to Mealie.

## Files

- **`Dockerfile`** - Multi-stage build for production image
- **`docker-compose.yml`** - Production configuration (pulls pre-built image)
- **`docker-compose.dev.yml`** - Development overrides (builds locally, adds mailpit)
- **`.dockerignore`** - Files to exclude from Docker build context

## Quick Start

### Local Development

Build and run locally with development tools:

```bash
# From project root
docker compose -f docker/docker-compose.yml -f docker/docker-compose.dev.yml up --build

# Or from docker/ directory
docker compose -f docker-compose.yml -f docker-compose.dev.yml up --build
```

**What you get:**
- ✅ Local build from source
- ✅ Debug logging (`RUST_LOG=debug`)
- ✅ Mailpit for email testing at http://localhost:8025
- ✅ Hot reload capability (if volumes added)
- ✅ Development JWT secret

**Access:**
- App: http://localhost:9898
- Mailpit UI: http://localhost:8025 (see all captured emails)

### Production Deployment (Dockge, etc.)

Pull and run pre-built image from registry:

```bash
docker compose -f docker/docker-compose.yml up -d
```

**What you get:**
- ✅ Pre-built image from ghcr.io
- ✅ Auto-generated JWT secret (persisted to volume)
- ✅ Production logging
- ⚠️ No mailpit (configure real SMTP)

## Common Commands

### Build Image Locally

```bash
# Full rebuild without cache
docker compose -f docker/docker-compose.yml -f docker/docker-compose.dev.yml build --no-cache

# Quick rebuild
docker compose -f docker/docker-compose.yml -f docker/docker-compose.dev.yml build
```

### View Logs

```bash
docker compose -f docker/docker-compose.yml logs -f web
docker compose -f docker/docker-compose.yml logs -f mailpit  # dev only
```

### Stop Services

```bash
docker compose -f docker/docker-compose.yml down
# Or to remove volumes too:
docker compose -f docker/docker-compose.yml down -v
```

### Access Database

```bash
docker compose -f docker/docker-compose.yml exec db psql -U humidor_user -d humidor_db
```

## Environment Variables

### Required for Production

Set these in Dockge or via `.env` file:

```env
JWT_SECRET=<generate with: openssl rand -base64 32>
```

### Optional Configuration

```env
# Database (defaults provided)
POSTGRES_DB=humidor_db
POSTGRES_USER=humidor_user
POSTGRES_PASSWORD=humidor_pass

# Application
PORT=9898
RUST_LOG=info
JWT_TOKEN_LIFETIME_HOURS=2
BASE_URL=http://localhost:9898
ALLOWED_ORIGINS=http://localhost:9898,http://127.0.0.1:9898

# Email (for password reset)
SMTP_HOST=smtp.gmail.com
SMTP_PORT=587
SMTP_USER=your-email@gmail.com
SMTP_PASSWORD=your-app-password
SMTP_FROM_EMAIL=noreply@yourapp.com
```

## Email Testing (Development)

The dev configuration includes **Mailpit** for capturing emails without sending them.

1. Start dev environment:
   ```bash
   docker compose -f docker/docker-compose.yml -f docker-compose.dev.yml up
   ```

2. Trigger a password reset in your app

3. View email at: http://localhost:8025

All emails are captured and displayed in a web interface. No real emails are sent.

## Directory Structure

```
docker/
├── Dockerfile              # Multi-stage build
├── docker-compose.yml      # Production config
├── docker-compose.dev.yml  # Dev overrides
├── .dockerignore          # Build exclusions
└── README.md              # This file
```

## Tips

### Use Shell Aliases

Add to your shell profile (`.bashrc`, `.zshrc`, etc.):

```bash
alias dc-dev='docker compose -f docker/docker-compose.yml -f docker/docker-compose.dev.yml'
alias dc-prod='docker compose -f docker/docker-compose.yml'
```

Then use:
```bash
dc-dev up --build
dc-dev logs -f
dc-prod up -d
```

### PowerShell Functions

Add to your PowerShell profile:

```powershell
function dc-dev { docker compose -f docker/docker-compose.yml -f docker/docker-compose.dev.yml $args }
function dc-prod { docker compose -f docker/docker-compose.yml $args }
```

Usage:
```powershell
dc-dev up --build
dc-dev logs -f web
dc-prod up -d
```

## Troubleshooting

### Build fails with "no such file or directory"

Make sure you're running from the project root, or the context paths are correct.

### Cannot connect to database

Wait for database health check. Check with:
```bash
docker compose -f docker/docker-compose.yml ps
```

### Mailpit not showing emails

- Check web is configured to use `SMTP_HOST=mailpit` and `SMTP_PORT=1025`
- Verify in dev environment (mailpit not in production config)
- Check logs: `docker compose logs mailpit`

### JWT tokens not persisting

- Production: Set `JWT_SECRET` environment variable
- Or: Use a Docker volume for `/app/data` to persist auto-generated secret
