<div align="center">
  <img src="static/logo.png" alt="Humidor Logo" width="120" height="120">
  <h1>Humidor</h1>
  <p><em>Cigar Inventory Management System</em></p>
  <p><em>** WORK IN PROGRESS **</em></p>
</div>

A heavily inspired by Mealie application for managing your cigar collection to fill a personal need. Built with Rust, PostgreSQL, and Docker. 

## Features

- **Inventory Management**: Add, edit, and delete cigars from your collection
- **Search & Filter**: Find cigars by brand, strength, origin, or search terms
- **Mobile-Friendly**: Responsive design for phones and tablets

## Quick Start

1. **Prerequisites**: Docker and Docker Compose

2. **Run the application**:
   ```bash
   git clone https://github.com/VictoryTek/humidor.git
   cd humidor
   docker-compose up --build
   ```

3. **Access**: Open `http://localhost:9898` in your browser

## Tech Stack

- **Backend**: Rust with Warp web framework
- **Database**: PostgreSQL with tokio-postgres
- **Frontend**: HTML, CSS, JavaScript
- **Deployment**: Docker & Docker Compose