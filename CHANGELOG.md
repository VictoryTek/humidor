# Changelog

All notable changes to Humidor will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [1.3.1] - 2025-12-10

### Added
- **Strength Indicators on Cigar Cards**
  - Visual 1-5 dot scale displays cigar strength at bottom-left of cards
  - Only appears when a strength organizer is selected during cigar creation
  - Hover tooltip shows "Strength: X/5" for clarity
  - Consistent across all views (main collection, favorites, wish list)
  - Uses gold accent color matching app theme

### Changed
- **Organizer Cards Visual Improvements**
  - Strength organizer cards now display level as visual bar indicators instead of text
  - 5-bar visualization shows strength level more intuitively
- **Cigar Card Layout Enhancements**
  - Improved flexbox layout for better content distribution
  - Card footer now properly contains strength indicator and quantity controls side-by-side
  - Fixed card height consistency in grid layouts
- **Report Card Modal**
  - Improved overflow handling for long retail links
  - Added horizontal scrollbar with custom styling for retail link section
  - Better responsive behavior for modal content

### Fixed
- Fixed cigar card content area flex layout to prevent overflow issues
- Improved retail link display in report card modal with proper scrolling

## [1.3.0] - 2025-12-09

### Added
- **Multiple Public Share Links**
  - Create unlimited public share links for the same humidor
  - Set different expiration dates for each link (30 days default, custom date, or never expires)
  - Add optional labels to identify each share (e.g., "For John - December 2025")
  - Customize what's included per link (favorites and wish list toggles)
  - Delete individual links without affecting other shares
  - Copy any link directly to clipboard with one click
- New API endpoints for managing multiple shares:
  - `GET /api/v1/humidors/:id/public-shares` - List all active shares
  - `DELETE /api/v1/humidors/:id/public-shares/:token_id` - Delete specific share
- Database migration V17 (automatic on startup)

### Changed
- Public share view now shows cleaner interface with only relevant sections
- Navigation panel hides Settings and Organizers menus in shared view
- Section title changes to "SHARED VIEW" for clarity
- Improved authentication flow redirects to dashboard instead of setup wizard

### Fixed
- Expired/revoked share links no longer show incorrect navigation items
- Removed "Create Account" button from expired share error page
- Fixed public share error page layout issues
- Fixed navigation visibility for expired public shares
- Fixed setup wizard redirect loop for authenticated users
- Fixed Organizers dropdown visibility in public share view

## [1.2.1] - 2025-12-07

### Fixed
- **Desktop Layout**: Search/filter bar now stays inline with title and buttons on wide screens (was appearing on separate row)
- **Hamburger Menu Alignment**: Fixed menu button jumping above logo at 769px screen width
- **Wish List Editing**: Users can now successfully edit cigars in their wish list (was returning "Failed to fetch" error)

### Changed
- Updated `verify_cigar_ownership()` to check wish list ownership
- Added explicit grid positioning to page header elements
- Improved flexbox properties for header-left section
- Service worker cache version updated to `humidor-v1.2.1`

## [1.2.0] - 2025-12-06

### Added
- **Progressive Web App (PWA) Support**
  - Installable application with custom "Install App" button
  - Native install prompts on supported browsers
  - Standalone app mode (no browser chrome)
  - Home screen icons on mobile devices
  - Desktop shortcuts on Windows/Mac/Linux
- **Offline Functionality**
  - Works completely offline after initial load
  - Custom offline page with auto-retry logic
  - Smart caching strategies for optimal performance
  - Precached essential assets (HTML, CSS, JS, images)
  - Cached API responses for offline viewing
- **Service Worker**
  - Versioned caching system (humidor-v1.2.0)
  - Network-first strategy for API calls
  - Cache-first strategy for static assets
  - Automatic update detection with user notifications
  - Hourly update checks for new versions
- **App Manifest**
  - App name: "Humidor - Premium Cigar Collection"
  - Theme color: Gold (#D4AF37)
  - 4 icon variants (192x192, 512x512, standard & maskable)
  - Standalone display mode
  - iOS and Android compatibility

### Changed
- **Mobile & Tablet Responsive Design**
  - Hamburger menu for mobile/tablet (≤1024px)
  - Slide-out navigation drawer with smooth animations
  - Single column grids on mobile for all content
  - Collapsible filter section to save screen space
  - Filter toggle badge showing active filter count
  - Near full-screen modals on mobile devices
  - 44px minimum touch targets (iOS guidelines)
  - Large, easy-to-tap buttons and controls
  - Optimized form inputs for mobile keyboards

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

[1.3.0]: https://github.com/VictoryTek/humidor/releases/tag/v1.3.0
[1.2.1]: https://github.com/VictoryTek/humidor/releases/tag/v1.2.1
[1.2.0]: https://github.com/VictoryTek/humidor/releases/tag/v1.2.0
[1.1.0]: https://github.com/VictoryTek/humidor/releases/tag/v1.1.0
[1.0.0]: https://github.com/VictoryTek/humidor/releases/tag/v1.0.0
[1.0.0-rc.1]: https://github.com/VictoryTek/humidor/releases/tag/v1.0.0-rc.1
