# Humidor v1.0.0-rc.1 Release Notes

**Release Date:** January 2025  
**Release Type:** Release Candidate 1

## 🎉 What's New

This is the first release candidate for Humidor 1.0! This release represents a fully-featured, production-ready cigar inventory management system with multi-user support, sharing capabilities, and comprehensive security.

## ✨ Major Features

### Core Functionality
- **Inventory Management**: Add, edit, and delete cigars from your collection
- **Humidor Organization**: Create and manage multiple humidors
- **Favorites & Wish Lists**: Mark favorite cigars and maintain wish lists for future purchases
- **Search & Filter**: Advanced search and filtering by brand, strength, origin, size, and more
- **Image Support**: Upload or link cigar images for visual identification

### Multi-User Support
- **User Roles**: Admin and standard user roles with proper access control
- **Data Isolation**: Complete user data isolation with secure sharing
- **User Management**: Full admin interface for creating and managing users
- **Account Management**: Self-service profile and password management

### Humidor Sharing
- **Three Permission Levels**:
  - **View**: Read-only access to cigars
  - **Edit**: Can add and modify cigars
  - **Full**: Complete management including sharing
- **Share Management**: Easy-to-use interface for sharing and revoking access
- **Shared Humidors View**: Dedicated section for humidors shared with you

### Security & Authentication
- **JWT Authentication**: Secure token-based authentication
- **Password Security**: bcrypt password hashing
- **Password Reset**: Email-based password reset (optional SMTP configuration)
- **Session Management**: Configurable token lifetime
- **CORS Protection**: Configurable allowed origins

## 🚀 Deployment

### Easy Docker Compose Deployment
- **One-Command Setup**: `docker-compose up --build` with sensible defaults
- **Environment Variables**: All configuration via environment variables
- **No Manual Setup Required**: Auto-generates JWT secrets, uses default credentials
- **Production Ready**: Override defaults with custom `.env` file

### System Requirements
- Docker & Docker Compose
- 512MB RAM minimum
- 1GB disk space for application and database

## 📚 Documentation

Complete documentation available in the `docs/` directory:
- **[User Guide](docs/USER_GUIDE.md)** - Getting started and daily usage
- **[Admin Guide](docs/ADMIN_GUIDE.md)** - User management and administration
- **[Sharing Guide](docs/SHARING.md)** - Humidor sharing features
- **[Permissions Guide](docs/PERMISSIONS.md)** - User roles and access control
- **[Security Model](docs/SECURITY_MODEL.md)** - Security architecture
- **[API Documentation](docs/API.md)** - REST API reference (60+ endpoints)

## 🔒 Security

### Security Improvements in This Release
- **Complete Data Isolation Audit**: All endpoints verified for proper user isolation
- **Security Vulnerability Fixes**: Fixed critical cigar access control issues
- **Comprehensive Testing**: 17 security isolation tests covering all data types
- **Permission Enforcement**: Proper authorization checks on all operations

### Security Best Practices
- Default credentials intended for development only
- JWT secrets auto-generated if not provided
- All passwords hashed with bcrypt
- Configurable token lifetime
- CORS protection enabled

## 🧪 Testing

- **134 Total Tests**: Comprehensive test coverage
- **17 Security Isolation Tests**: Verify complete user data isolation
- **12 Sharing Tests**: Cover all permission levels and edge cases
- **8 Permission Tests**: Verify role-based access control
- **100% Pass Rate**: All tests passing

## 🛠️ Technical Stack

- **Backend**: Rust 1.90 with Warp web framework
- **Database**: PostgreSQL 17 with tokio-postgres
- **Frontend**: HTML, CSS, JavaScript (vanilla, no framework dependencies)
- **Deployment**: Docker & Docker Compose
- **Authentication**: JWT with bcrypt password hashing

## 📱 User Interface

- **Mobile-Responsive**: Full mobile and tablet support
- **Modern Design**: Clean, intuitive interface
- **Dark Mode Ready**: CSS structure supports theming
- **Fast Performance**: Lightweight frontend, no heavy frameworks

## 🔧 Configuration

### Environment Variables (All Optional with Defaults)
```bash
# Database (defaults: humidor_db, humidor_user, humidor_pass)
POSTGRES_DB=humidor_db
POSTGRES_USER=humidor_user
POSTGRES_PASSWORD=humidor_pass

# Application
PORT=9898
RUST_LOG=info
BASE_URL=http://localhost:9898
ALLOWED_ORIGINS=http://localhost:9898

# Authentication
JWT_SECRET=auto-generated-if-not-set
JWT_TOKEN_LIFETIME_HOURS=2

# Email (Optional)
SMTP_HOST=smtp.example.com
SMTP_PORT=587
SMTP_USER=user@example.com
SMTP_PASSWORD=app-password
SMTP_FROM_EMAIL=noreply@example.com
```

## 📦 What's Included

### Complete Feature Set
- ✅ User authentication and authorization
- ✅ Multi-user support with data isolation
- ✅ Admin user management interface
- ✅ Humidor CRUD operations
- ✅ Cigar inventory management
- ✅ Favorites and wish lists
- ✅ Humidor sharing with 3 permission levels
- ✅ Search and filtering
- ✅ Image upload and linking
- ✅ Password reset via email
- ✅ Health check endpoints
- ✅ Comprehensive API

### Known Limitations
- Cigar scraping feature temporarily disabled (will be re-enabled in future release)
- Email notifications for sharing events not yet implemented
- No batch operations (bulk add/edit/delete)
- No export/import functionality yet

## 🚧 Temporarily Disabled Features

- **Cigar URL Scraper**: Disabled due to parsing issues with certain retailer websites. Will be improved and re-enabled in a future release.

## 🐛 Bug Fixes

- Fixed critical cigar access control vulnerability
- Fixed favorites allowing cross-user access
- Fixed humidor permission checks
- Fixed JWT token lifetime configuration
- Fixed icon loading on fresh page loads

## ⚠️ Breaking Changes from Development Versions

- Removed Docker secrets requirement (now uses environment variables)
- Changed docker-compose.yml structure (simplified configuration)
- Database connection now uses single `DATABASE_URL` variable

## 🔄 Migration Notes

If upgrading from a development version:
1. Update `docker-compose.yml` to new format (no secrets required)
2. Set environment variables instead of using secrets files
3. Database schema is automatically migrated on startup

## 📝 Release Checklist

- ✅ All 134 tests passing
- ✅ Security audit completed
- ✅ Documentation complete (6 comprehensive guides)
- ✅ Docker Compose simplified for easy deployment
- ✅ Version bumped to 1.0.0-rc.1
- ✅ README updated with RC status
- ✅ No compiler warnings
- ✅ No clippy warnings

## 🎯 What's Next

### Planned for 1.0.0 Final
- User feedback integration from RC testing
- Performance optimizations based on real-world usage
- Additional edge case testing
- Final documentation polish

### Future Enhancements (Post 1.0)
- Re-enable improved cigar scraper
- Email notifications for sharing events
- Batch operations support
- Data export/import functionality
- Mobile app (potential)
- API rate limiting enhancements
- Activity audit log

## 🙏 Acknowledgments

Heavily inspired by [Mealie](https://github.com/mealie-recipes/mealie), this project aims to bring the same level of quality and user experience to cigar inventory management.

## 📞 Support & Feedback

This is a Release Candidate - your feedback is valuable!

- **Issues**: Report bugs or request features via GitHub Issues
- **Documentation**: Check the `docs/` folder for comprehensive guides
- **Security**: Report security issues privately to the maintainer

## 📄 License

See LICENSE file for details.

---

**Enjoy managing your cigar collection with Humidor!** 🎯

**Note**: This is a Release Candidate. Please test thoroughly and report any issues before using in production.
