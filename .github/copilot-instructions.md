# Humidor - Cigar Inventory Management System

This is a Rust-based web application for managing cigar inventory in humidors.

## Project Structure
- **Backend**: Rust with Warp web framework
- **Database**: PostgreSQL with tokio-postgres for database operations  
- **Frontend**: HTML/CSS/JavaScript with mobile-friendly responsive design
- **Containerization**: Docker and Docker Compose for easy deployment
- **Authentication**: JWT tokens with bcrypt password hashing
- **Validation**: Custom validation system with comprehensive error handling

## Development Guidelines
- Use `cargo` commands for Rust development
- Database migrations run inline at application startup (consider migrating to proper migration tool)
- Frontend assets are served from the `/static` directory
- API endpoints follow RESTful conventions under `/api/v1/`
- All input is validated before processing
- Errors are returned with consistent JSON structure and appropriate HTTP status codes
- JWT authentication required for most endpoints (except setup and login)

## Key Features
- Add, edit, and delete cigars from inventory
- Track cigar details (brand, size, strength, origin, etc.)
- Mobile-responsive web interface
- Search and filter capabilities
- Humidor organization support

## Tech Stack
- **Language**: Rust
- **Web Framework**: Warp 0.3
- **Database**: PostgreSQL with tokio-postgres
- **Frontend**: Vanilla HTML/CSS/JS with modern responsive design
- **Containerization**: Docker & Docker Compose
- **Authentication**: JWT with jsonwebtoken + bcrypt
- **Logging**: Tracing/tracing-subscriber
- **Environment**: dotenv for configuration