# Release Notes - Humidor v1.0.0

**Release Date:** January 12, 2025

## 🎉 First Stable Release!

We're excited to announce the first stable release of Humidor - a modern cigar inventory management system built with Rust, PostgreSQL, and Docker.

## What's New in v1.0.0

### Core Features
- **Cigar Inventory Management** - Track your entire cigar collection with detailed information (brand, size, strength, origin, price, purchase date)
- **Multi-Humidor Support** - Organize cigars across multiple humidors with image support
- **Favorites & Wish Lists** - Mark favorite cigars and maintain a wish list of cigars you want to try
- **Advanced Search & Filtering** - Find cigars by brand, strength, origin, or search terms
- **Mobile-Friendly Interface** - Fully responsive design for phones and tablets

### User Management
- **Multi-User Support** - Complete data isolation between users
- **Role-Based Access** - Admin and standard user roles with proper permissions
- **User Administration** - Admins can create, edit, and manage users
- **Profile Management** - Users can update their profile and change passwords

### Humidor Sharing
- **Share Your Collection** - Share humidors with other users
- **Granular Permissions** - Choose between view, edit, or full access
- **Real-Time Updates** - Permission changes take effect immediately
- **Visual Indicators** - Clearly see which humidors are shared with you

### Security
- **JWT Authentication** - Secure token-based authentication
- **Password Reset** - Email-based password reset flow
- **Row-Level Security** - Data isolation enforced at the database level
- **Auto-Generated Secrets** - JWT secrets auto-generated and persisted securely
- **Rate Limiting** - Protection against brute-force attacks

### Deployment
- **Zero-Config Docker** - Start with `docker compose up -d`
- **Auto-Configuration** - Automatic JWT secret generation
- **Health Checks** - Built-in health monitoring
- **Database Migrations** - Automatic schema updates
- **Backup & Restore** - Full database backup/restore functionality

## System Requirements

- **Docker & Docker Compose** - For containerized deployment
- **PostgreSQL 17** - Database (included in Docker Compose)
- **Modern Web Browser** - Chrome, Firefox, Safari, or Edge

## Quick Start

```bash
# Clone the repository
git clone https://github.com/VictoryTek/humidor.git
cd humidor

# Start the application
docker compose up -d

# Access at http://localhost:9898
```

## Test Results

- **138 tests passed** - 100% success rate
- **Code quality** - Zero clippy warnings
- **Security audit** - 1 non-critical advisory (unmaintained dependency)
- **Docker build** - Successfully builds in ~8 minutes

## Documentation

- [User Guide](docs/USER_GUIDE.md) - Complete guide for end users
- [Admin Guide](docs/ADMIN_GUIDE.md) - Administrator documentation
- [API Documentation](docs/API.md) - REST API reference
- [Security Model](docs/SECURITY_MODEL.md) - Security architecture
- [Docker Deployment](README.md#quick-start) - Deployment guide

## Breaking Changes

This is the first stable release. Future updates will document breaking changes here.

## Known Issues

- JWT_SECRET warning in logs is informational only (secret is auto-generated)
- No breaking issues identified

## Upgrade Path

This is the first stable release. No upgrades are necessary.

## Contributors

- Jordan Howell (@VictoryTek)

## License

This project is licensed under the GNU General Public License v3.0 - see the [LICENSE](LICENSE) file for details.

## Support

- **Issues**: [GitHub Issues](https://github.com/VictoryTek/humidor/issues)
- **Discussions**: [GitHub Discussions](https://github.com/VictoryTek/humidor/discussions)

## What's Next?

See [FEATURES_TODO.md](docs/FEATURES_TODO.md) for planned features in future releases.

---

**Full Changelog**: https://github.com/VictoryTek/humidor/blob/main/CHANGELOG.md
