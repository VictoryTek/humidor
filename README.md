<div align="center">
  <img src="static/logo.png" alt="Humidor Logo" width="150" height="150">
  <h1>Humidor</h1>
  <p><em>Cigar Inventory Management System</em></p>
  <p><em>** WORK IN PROGRESS **</em></p>
</div>

Heavily inspired by Mealie. 
An application for managing your cigar collection. Built with Rust, PostgreSQL, and Docker. 
This project was started because I am a homelabber and couldn't find anything to help me track what I have and what I want, so this started out to fill a personal need. Hopefully if I can make it work, it will be useful to other cigar enjoyers, who want to keep track of their humidor inventory.

## Features

- **Inventory Management**: Add, edit, and delete cigars from your collection
- **Search & Filter**: Find cigars by brand, strength, origin, or search terms
- **Mobile-Friendly**: Responsive design for phones and tablets

## Quick Start

### Docker Compose (Recommended)

```bash
docker-compose up --build
```

This single command spins up the entire stack:
- PostgreSQL 15 database (port 5432)
- Humidor web application (port 9898)
- Persistent volume for database storage
- Automatic health checks and service dependencies

Access the application at `http://localhost:9898`

### Docker Run (Manual)

```bash
docker run -d --name humidor -p 9898:9898 -e DATABASE_URL=postgresql://humidor_user:humidor_pass@db:5432/humidor_db -e RUST_LOG=info -e PORT=9898 humidor:latest
```

**Note**: This requires a separate PostgreSQL database. Use Docker Compose for a complete setup.

## Docker Compose File

```yaml
name: humidor

services:
  db:
    image: postgres:17
    environment:
      POSTGRES_DB: humidor_db
      POSTGRES_USER: humidor_user
      POSTGRES_PASSWORD: humidor_pass
    volumes:
      - postgres_data:/var/lib/postgresql/data
    ports:
      - "5432:5432"
    healthcheck:
      test: ["CMD-SHELL", "pg_isready -U humidor_user -d humidor_db"]
      interval: 5s
      timeout: 5s
      retries: 5

  web:
    build: .
    environment:
      DATABASE_URL: postgresql://humidor_user:humidor_pass@db:5432/humidor_db
      RUST_LOG: info
      PORT: 9898
    ports:
      - "9898:9898"
    depends_on:
      db:
        condition: service_healthy

volumes:
  postgres_data:
```

## Environment Variables

The following environment variables can be configured:

| Variable | Required | Default | Description |
|----------|----------|---------|-------------|
| `DATABASE_URL` | Yes | - | PostgreSQL connection string (e.g., `postgresql://user:pass@host:5432/dbname`) |
| `PORT` | No | `9898` | Port for the web server to listen on |
| `RUST_LOG` | No | `info` | Logging level (`trace`, `debug`, `info`, `warn`, `error`) |
| `JWT_SECRET` | Yes* | - | Secret key for JWT token signing (auto-generated if not provided) |
| `BASE_URL` | No | `http://localhost:9898` | Base URL for the application (used in password reset emails) |
| `ALLOWED_ORIGINS` | No | `http://localhost:9898,http://127.0.0.1:9898` | Comma-separated list of allowed CORS origins |
| `SMTP_HOST` | No | - | SMTP server hostname for sending emails (optional) |
| `SMTP_PORT` | No | `587` | SMTP server port |
| `SMTP_USER` | No | - | SMTP authentication username |
| `SMTP_PASSWORD` | No | - | SMTP authentication password |
| `SMTP_FROM_EMAIL` | No | - | Email address to send from |

**Note**: Variables marked with * are required for production but can be auto-generated in development.

### Password Reset Email Configuration

Password reset functionality works without SMTP configured - reset URLs will be logged to the console instead. To enable email delivery:

1. Configure the SMTP environment variables above
2. Restart the web service: `docker-compose restart web`

See [docs/PASSWORD_RESET_README.md](docs/PASSWORD_RESET_README.md) for detailed email provider setup instructions.

## Tech Stack

- **Backend**: Rust with Warp web framework
- **Database**: PostgreSQL with tokio-postgres
- **Frontend**: HTML, CSS, JavaScript
- **Deployment**: Docker & Docker Compose

## Security

This application uses **Docker Compose secrets** for sensitive data like database credentials and JWT signing keys. See [DOCKER_SECRETS.md](DOCKER_SECRETS.md) for detailed setup instructions.

For production deployments, consider using:
- Docker Swarm secrets
- Kubernetes secrets
- Cloud provider secret managers (AWS Secrets Manager, Azure Key Vault, etc.)