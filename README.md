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