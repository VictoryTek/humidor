# Docker Configuration

This directory contains all Docker-related files for the Humidor project.

## Zero-Config Quick Start ⚡

The app is designed to **just work** with smart defaults:

```bash
docker compose up -d
```

That's it! The app will:
- ✅ Auto-generate and persist JWT secret
- ✅ Allow access from any IP (perfect for Tailscale, VPNs, dynamic IPs)
- ✅ Set up database with migrations
- ✅ Use sensible defaults for everything

**Access**: http://localhost:9898 (or your server's IP)

## Files

- **`Dockerfile`** - Multi-stage build for production image
- **`docker-compose.yml`** - Zero-config production setup
- **`docker-compose.dev.yml`** - Development overrides (builds locally, adds mailpit)

## Local Development

Build and run locally with development tools:

```bash
# From project root
docker compose -f docker/docker-compose.yml -f docker/docker-compose.dev.yml up --build
```

**Access:**
- App: http://localhost:9898
- Mailpit UI: http://localhost:8025 (email testing)

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

## Optional Configuration

**The app works without any configuration**, but you can customize if needed:

### Production with Fixed Domain (CORS Strict Mode)

```yaml
services:
  humidor:
    environment:
      CORS_MODE: strict
      ALLOWED_ORIGINS: "https://humidor.example.com"
```

### Custom Logging

```yaml
services:
  humidor:
    environment:
      RUST_LOG: debug  # trace, debug, info, warn, error
```

### Email Configuration (Password Reset)

```yaml
services:
  humidor:
    environment:
      SMTP_HOST: smtp.gmail.com
      SMTP_PORT: 587
      SMTP_USER: your-email@gmail.com
      SMTP_PASSWORD: your-app-password
      SMTP_FROM_EMAIL: noreply@yourapp.com
```

### All Defaults (What You Get Automatically)

```yaml
# These are the defaults - no need to set them:
PORT: 9898
RUST_LOG: info
CORS_MODE: permissive  # Works with any IP
JWT_TOKEN_LIFETIME_HOURS: 2
# JWT_SECRET: Auto-generated and persisted to /app/data/
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
## Troubleshooting

### App works! No CORS issues! 🎉

The app uses permissive CORS by default, so it works with:
- ✅ localhost
- ✅ LAN IPs (192.168.x.x)
- ✅ Tailscale IPs (100.x.x.x)
- ✅ Any VPN or dynamic IP

**Only configure CORS if** you're deploying to production with a fixed domain and want maximum security.

### Build fails with "no such file or directory"
**Solution**: The app now uses permissive CORS mode by default, which works with ANY IP address.

**If you're still getting this error:**
1. Check that `CORS_MODE=permissive` is set (it's the default)
2. Restart containers: `docker compose down && docker compose up -d`
3. Verify logs: `docker compose logs humidor | grep -i cors`
   - Should see: "CORS mode set to 'permissive'"

**For production with fixed domain:**
```env
CORS_MODE=strict
ALLOWED_ORIGINS=https://yourdomain.com
```

**See**: `../docs/CORS_CONFIGURATION.md` for details.

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
