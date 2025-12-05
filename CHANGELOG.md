# Changelog

All notable changes to Humidor will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [1.1.0] - 2025-12-05

### Added
- **Light/Dark Mode Theme Toggle**
  - Theme switcher button in top-right header (next to user account)
  - Light mode with soft warm parchment backgrounds
  - Dark mode (default) with rich mahogany and gold accents
  - Theme preference persisted to localStorage
  - Smooth CSS transitions between themes
  - Consistent dark styling for key UI elements across both themes
  - Material Design Icons for theme toggle (moon/sun)

### Changed
- Moved release notes from root directory to `release_notes/` folder
- Light mode uses same vibrant gold/copper accents as dark mode
- All button text uses deep chocolate brown for optimal contrast
- Navigation panel selection maintains dark theme styling in light mode

## [1.0.0] - 2025-01-12

### Added
- **Core Features**
  - Cigar inventory management (CRUD operations)
  - Multi-humidor organization with image support
  - Favorites and wish list functionality
  - Advanced search and filtering (brand, strength, origin, price)
  - Mobile-responsive design

- **User Management**
  - Multi-user support with data isolation
  - Admin and standard user roles
  - User creation, editing, and deactivation (admin only)
  - Profile management with password changes
  - Complete setup wizard for first-run experience

- **Humidor Sharing**
  - Share humidors with other users
  - Granular permissions (view, edit, full)
  - Real-time permission updates
  - Shared humidor indicators

- **Authentication & Security**
  - JWT-based authentication
  - Password reset with email tokens
  - Row-level security (RLS) for data isolation
  - Rate limiting on sensitive endpoints
  - Auto-generated JWT secrets with persistence

- **Backup & Restore**
  - Full database backup to ZIP format
  - Restore from backup with conflict handling
  - Admin-only backup operations

- **DevOps & Deployment**
  - Docker and Docker Compose deployment
  - Zero-config startup with smart defaults
  - Health checks and monitoring
  - Prometheus metrics endpoint
  - GitHub Actions CI/CD pipeline
  - Multi-stage Docker builds with caching

- **Developer Experience**
  - Mailpit integration for email testing
  - Hot-reload development setup
  - Comprehensive test suite (138 tests)
  - API documentation
  - Security model documentation

### Security
- Row-level security enforced at database query level
- JWT secret auto-generation and persistence
- Password hashing with bcrypt
- CORS configuration for API protection
- Input validation and sanitization
- SQL injection prevention via parameterized queries

### Documentation
- User Guide with screenshots
- Admin Guide for user management
- API documentation with examples
- Security architecture documentation
- Humidor sharing guide
- Migration guide
- CORS configuration guide

### Technical Details
- **Backend**: Rust 1.83+ with Warp 0.3.x
- **Database**: PostgreSQL 17 with Refinery migrations
- **Frontend**: Vanilla JavaScript (no framework dependencies)
- **Authentication**: JWT with RS256 signing
- **Email**: SMTP with optional Mailpit for development

## [1.0.0-rc.1] - 2025-01-11

Initial release candidate with all v1.0.0 features for testing.

[1.0.0]: https://github.com/VictoryTek/humidor/releases/tag/v1.0.0
[1.0.0-rc.1]: https://github.com/VictoryTek/humidor/releases/tag/v1.0.0-rc.1
