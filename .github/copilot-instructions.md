# Humidor - Cigar Inventory Management System

This is a Rust-based web application for managing cigar inventory in humidors.

## Project Structure
- **Backend**: Rust with Axum web framework
- **Database**: PostgreSQL with SQLx for database operations  
- **Frontend**: HTML/CSS/JavaScript with mobile-friendly responsive design
- **Containerization**: Docker and Docker Compose for easy deployment

## Development Guidelines
- Use `cargo` commands for Rust development
- Database migrations are managed with SQLx CLI
- Frontend assets are served from the `/static` directory
- API endpoints follow RESTful conventions under `/api/v1/`

## Key Features
- Add, edit, and delete cigars from inventory
- Track cigar details (brand, size, strength, origin, etc.)
- Mobile-responsive web interface
- Search and filter capabilities
- Humidor organization support

## Tech Stack
- **Language**: Rust
- **Web Framework**: Axum
- **Database**: PostgreSQL with SQLx
- **Frontend**: Vanilla HTML/CSS/JS with modern responsive design
- **Containerization**: Docker & Docker Compose