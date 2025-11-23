<div align="center">
  <img src="static/logo.png" alt="Humidor Logo" width="150" height="150">
  <h1>Humidor</h1>
  <p><em>Cigar Inventory Management System</em></p>
  <p><strong>Version 1.0.0 Release Candidate 1</strong></p>
</div>

An application for managing your cigar collection. Built with Rust, PostgreSQL, and Docker. 
This project was started because I am a homelabber and couldn't find anything to help me track what I have and what I want, so this started out to fill a personal need. Hopefully if I can make it work, it will be useful to other cigar enjoyers, who want to keep track of their humidor inventory.

## Features

- **Inventory Management**: Add, edit, and delete cigars from your collection
- **Humidor Organization**: Organize cigars into multiple humidors
- **User Permissions**: Admin and standard user roles with proper access control
- **Humidor Sharing**: Share your humidors with other users (view/edit/full permissions)
- **Favorites & Wish Lists**: Mark favorite cigars and maintain a wish list
- **Search & Filter**: Find cigars by brand, strength, origin, or search terms
- **Mobile-Friendly**: Responsive design for phones and tablets
- **Multi-User Support**: Complete data isolation with secure sharing

## Tech Stack

- **Backend**: Rust with Warp web framework
- **Database**: PostgreSQL with tokio-postgres
- **Frontend**: HTML, CSS, JavaScript
- **Deployment**: Docker & Docker Compose

## Documentation

📚 **[Documentation Hub](docs/README.md)** - Complete documentation index

**User Documentation:**
- [User Guide](docs/USER_GUIDE.md) - Getting started and usage
- [Humidor Sharing](docs/SHARING.md) - Share your collection with others
- [User Permissions](docs/PERMISSIONS.md) - Roles and access control

**Administrator Documentation:**
- [Admin Guide](docs/ADMIN_GUIDE.md) - User management and administration
- [Security Model](docs/SECURITY_MODEL.md) - Security architecture

**Developer Documentation:**
- [API Documentation](docs/API.md) - REST API reference
- [Database Migrations](docs/MIGRATIONS.md) - Schema management
- [Security Audit Report](docs/SECURITY_AUDIT_2025-01-11.md) - Security review

## Quick Start

### Docker Compose (Recommended)

All Docker files are in the [`docker/`](docker/) directory. See [docker/README.md](docker/README.md) for detailed instructions.

**Local Development:**

```bash
# From project root
docker compose -f docker/docker-compose.yml -f docker/docker-compose.dev.yml up --build
```

**What you get:**
- Web app at http://localhost:9898
- Mailpit (email testing) at http://localhost:8025
- Debug logging and development tools

**Production Deployment:**

```bash
# From project root  
docker compose -f docker/docker-compose.yml up -d
```

**What you get:**
- Pre-built image from GitHub Container Registry
- Auto-generated JWT secret (persisted)
- Production-ready configuration

Access the application at `http://localhost:9898`

**First Run:**
- Complete the setup wizard to create your admin account
- JWT secret auto-generates and persists across restarts
- Configure SMTP for password reset emails (optional)

**Production Configuration:**

For production, set environment variables in Dockge or create a `.env` file:

```bash
# Required for production
JWT_SECRET=<generate with: openssl rand -base64 32>

# Database (optional, defaults provided)
POSTGRES_DB=humidor_db
POSTGRES_USER=your_db_user
POSTGRES_PASSWORD=your_secure_password

# Application (optional, defaults provided)
PORT=9898
RUST_LOG=info
ALLOWED_ORIGINS=https://your-domain.com

# Optional Email Configuration
BASE_URL=https://your-domain.com
SMTP_HOST=smtp.gmail.com
SMTP_PORT=587
SMTP_USER=your-email@gmail.com
SMTP_PASSWORD=your-app-password
SMTP_FROM_EMAIL=noreply@your-domain.com
```

Then run:
```bash
docker-compose up -d
```

### Docker Run (Manual)

```bash
docker run -d --name humidor -p 9898:9898 -e DATABASE_URL=postgresql://humidor_user:humidor_pass@db:5432/humidor_db -e RUST_LOG=info -e PORT=9898 humidor:latest
```

**Note**: This requires a separate PostgreSQL database. Use Docker Compose for a complete setup.

## Docker Compose File

The docker-compose.yml uses environment variables with sensible defaults for easy deployment:

```yaml
name: humidor

services:
  db:
    image: postgres:17
    environment:
      POSTGRES_DB: ${POSTGRES_DB:-humidor_db}
      POSTGRES_USER: ${POSTGRES_USER:-humidor_user}
      POSTGRES_PASSWORD: ${POSTGRES_PASSWORD:-humidor_pass}
    volumes:
      - postgres_data:/var/lib/postgresql/data
    ports:
      - "5432:5432"
    healthcheck:
      test: ["CMD-SHELL", "pg_isready -U ${POSTGRES_USER:-humidor_user} -d ${POSTGRES_DB:-humidor_db}"]
      interval: 5s
      timeout: 5s
      retries: 5

  web:
    build: .
    environment:
      DATABASE_URL: postgresql://${POSTGRES_USER:-humidor_user}:${POSTGRES_PASSWORD:-humidor_pass}@db:5432/${POSTGRES_DB:-humidor_db}
      RUST_LOG: ${RUST_LOG:-info}
      PORT: ${PORT:-9898}
      ALLOWED_ORIGINS: ${ALLOWED_ORIGINS:-http://localhost:9898,http://127.0.0.1:9898}
      JWT_SECRET: ${JWT_SECRET:-}
      JWT_TOKEN_LIFETIME_HOURS: ${JWT_TOKEN_LIFETIME_HOURS:-2}
    ports:
      - "9898:9898"
    depends_on:
      db:
        condition: service_healthy

volumes:
  postgres_data:
```

## Environment Variables

The following environment variables can be configured (all have sensible defaults):

| Variable | Required | Default | Description |
|----------|----------|---------|-------------|
| `POSTGRES_DB` | No | `humidor_db` | PostgreSQL database name |
| `POSTGRES_USER` | No | `humidor_user` | PostgreSQL username |
| `POSTGRES_PASSWORD` | No | `humidor_pass` | PostgreSQL password |
| `DATABASE_URL` | Auto | Auto-generated | Full PostgreSQL connection string |
| `PORT` | No | `9898` | Port for the web server to listen on |
| `RUST_LOG` | No | `info` | Logging level (`trace`, `debug`, `info`, `warn`, `error`) |
| `JWT_SECRET` | No | Auto-generated | Secret key for JWT token signing (auto-generated if not provided) |
| `JWT_TOKEN_LIFETIME_HOURS` | No | `2` | JWT token lifetime in hours |
| `BASE_URL` | No | `http://localhost:9898` | Base URL for the application (used in password reset emails) |
| `ALLOWED_ORIGINS` | No | `http://localhost:9898,http://127.0.0.1:9898` | Comma-separated list of allowed CORS origins |
| `SMTP_HOST` | No | - | SMTP server hostname for sending emails (optional) |
| `SMTP_PORT` | No | `587` | SMTP server port |
| `SMTP_USER` | No | - | SMTP authentication username |
| `SMTP_PASSWORD` | No | - | SMTP authentication password |
| `SMTP_FROM_EMAIL` | No | - | Email address to send from |

**Note**: For production deployments, always override the default database credentials and JWT secret!

### Password Reset Email Configuration

Password reset functionality works without SMTP configured - reset URLs will be logged to the console instead. To enable email delivery:

1. Configure the SMTP environment variables above
2. Restart the web service: `docker-compose restart web`

See [docs/PASSWORD_RESET_README.md](docs/PASSWORD_RESET_README.md) for detailed email provider setup instructions.

## Security Considerations
## Security Considerations

**Default Credentials:** The default database credentials (`humidor_user`/`humidor_pass`) are intended for development only.

**Production Recommendations:**
- Use strong, unique passwords for database access
- Set a secure `JWT_SECRET` (minimum 32 characters)
- Configure HTTPS/TLS with a reverse proxy (nginx, Traefik, Caddy)
- Use Docker Swarm secrets, Kubernetes secrets, or cloud provider secret managers (AWS Secrets Manager, Azure Key Vault, etc.)
- Restrict `ALLOWED_ORIGINS` to your actual domain(s)
- Enable firewall rules to restrict database port access
- Regular security updates and backups

## Credits

Humidor is heavily inspired by Mealie. Big thanks to the Mealie team for creating and maintaining such an amazing app!

Please support them at <https://github.com/mealie-recipes/mealie>

See [Security Model](docs/SECURITY_MODEL.md) for detailed security architecture.
