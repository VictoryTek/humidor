# Security Model

This document describes Humidor's security architecture, data isolation, and protection mechanisms.

## Security Architecture

### Multi-Tenant Isolation

Humidor implements a **row-level security model** with user-scoped data:

```
┌─────────────────────────────────────────────┐
│                Application                   │
│  ┌───────────────────────────────────────┐  │
│  │     JWT Authentication Middleware     │  │
│  └───────────────────────────────────────┘  │
│                     ↓                        │
│  ┌───────────────────────────────────────┐  │
│  │    Authorization & Ownership Checks   │  │
│  └───────────────────────────────────────┘  │
│                     ↓                        │
│  ┌───────────────────────────────────────┐  │
│  │   Database Queries with user_id Filter│  │
│  └───────────────────────────────────────┘  │
└─────────────────────────────────────────────┘
                      ↓
┌─────────────────────────────────────────────┐
│            PostgreSQL Database               │
│  ┌─────────────┬──────────────┬──────────┐  │
│  │  User Data  │ Shared Data  │ Global   │  │
│  │  (Isolated) │ (Controlled) │ (Public) │  │
│  └─────────────┴──────────────┴──────────┘  │
└─────────────────────────────────────────────┘
```

### Authentication

#### JWT Tokens
- **Stateless authentication** using JSON Web Tokens
- **Claims**: user_id, username, is_admin, expiration
- **Signing**: HMAC-SHA256 with secret key
- **Lifetime**: 24 hours (configurable)
- **Storage**: Browser localStorage

#### Token Flow
1. User logs in with username/password
2. Server validates credentials against bcrypt hash
3. Server generates JWT with user claims
4. Client stores token in localStorage
5. Client sends token in Authorization header
6. Server validates and extracts user context
7. All requests include authenticated user_id

#### Password Security
- **Hashing**: bcrypt with cost factor 12
- **Salt**: Unique per password (automatic)
- **No plaintext storage**: Only hashes in database
- **Password reset**: Time-limited tokens (1 hour)

### Authorization

#### Two-Tier Permission System

**Standard Users**
- Access own data only
- Cannot manage other users
- Can share their humidors
- Can access shared humidors

**Admin Users**
- All standard user capabilities
- User management (CRUD)
- Password resets
- Account activation/deactivation
- No automatic access to user data (must be shared)

#### Permission Checks

Every API request goes through:
1. **Authentication check**: Valid JWT token?
2. **Permission check**: Admin required? (for admin endpoints)
3. **Ownership check**: Does user own this resource?
4. **Share check**: Is resource shared with user?

### Data Isolation

#### User-Scoped Data

**Humidors**
```sql
-- All queries filter by user_id or verify ownership
SELECT * FROM humidors WHERE user_id = $current_user_id;
```

**Cigars**
```sql
-- Access through humidor ownership via INNER JOIN
SELECT c.* FROM cigars c
INNER JOIN humidors h ON c.humidor_id = h.id
WHERE h.user_id = $current_user_id;
```

**Favorites**
```sql
-- Direct user_id filter
SELECT * FROM favorites WHERE user_id = $current_user_id;
```

**Wish List**
```sql
-- Direct user_id filter
SELECT * FROM wish_list WHERE user_id = $current_user_id;
```

#### Shared Data (Humidor Sharing)

**Shared Humidors**
```sql
-- User can access if they own it OR it's shared with them
SELECT h.* FROM humidors h
LEFT JOIN humidor_shares hs ON h.id = hs.humidor_id
WHERE h.user_id = $current_user_id  -- Owner
   OR hs.shared_with_user_id = $current_user_id;  -- Shared with
```

**Permission Levels**
- `view`: Read-only access to cigars
- `edit`: Can add and modify cigars
- `full`: Can add, modify, and delete cigars

**Verification Flow**
1. Check if user owns humidor (owner = full access)
2. Check if humidor is shared with user
3. Verify permission level meets requirement
4. Return 403 Forbidden if unauthorized

#### Global Reference Data

**Organizers** (Brands, Sizes, Origins, Strengths, Ring Gauges)
- **Design**: Globally shared, collaborative
- **Rationale**: Industry-standard data (e.g., "Cohiba", "Robusto")
- **Access**: All authenticated users can read and contribute
- **Benefit**: Prevents data duplication, maintains consistency

### Database Security

#### Foreign Key Constraints

All relationships enforce referential integrity:

```sql
-- Humidors belong to users
FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE

-- Cigars reference humidors
FOREIGN KEY (humidor_id) REFERENCES humidors(id) ON DELETE SET NULL

-- Shares cascade delete
FOREIGN KEY (humidor_id) REFERENCES humidors(id) ON DELETE CASCADE
FOREIGN KEY (shared_with_user_id) REFERENCES users(id) ON DELETE CASCADE
```

#### Cascade Behaviors

**User Deletion**
- Humidors → Deleted
- Cigars (owned) → Deleted (via humidor)
- Favorites → Deleted
- Wish list → Deleted
- Shares (as sharer) → Deleted
- Shares (as recipient) → Deleted

**Humidor Deletion**
- Shares → Deleted automatically
- Cigars → Set humidor_id to NULL (orphaned but preserved)

#### Unique Constraints

**Prevent Duplicates**
```sql
-- One user per username
UNIQUE (username)

-- One user per email
UNIQUE (email)

-- One share per humidor-user pair
UNIQUE (humidor_id, shared_with_user_id)
```

### API Security

#### CORS (Cross-Origin Resource Sharing)

**Configuration**
```
ALLOWED_ORIGINS=http://localhost:9898,https://humidor.example.com
```

**Default**: Same-origin only
**Production**: Explicit whitelist required
**Credentials**: Cookies and Authorization headers allowed

#### Rate Limiting

(Currently not implemented - future enhancement)

**Planned**:
- Login attempts: 5 per 15 minutes
- API requests: 100 per minute per user
- Password resets: 3 per hour per email

#### Input Validation

All inputs are validated:
- **Length checks**: Prevent buffer overflows
- **Type validation**: Ensure correct data types
- **Format validation**: Email, UUID, etc.
- **SQL injection**: Protected by parameterized queries
- **XSS protection**: Output encoding in frontend

### Network Security

#### HTTPS Recommendation

**Production Checklist**:
- ✅ Use reverse proxy (nginx, Traefik)
- ✅ Obtain SSL/TLS certificate (Let's Encrypt)
- ✅ Force HTTPS redirects
- ✅ Set HSTS headers
- ✅ Update `BASE_URL` to HTTPS

#### Docker Network Isolation

```yaml
# Docker Compose creates isolated network
services:
  db:
    # Not exposed to host network
  web:
    # Only port 9898 exposed
```

**Benefits**:
- Database not accessible from outside
- Service-to-service communication isolated
- Port exposure explicitly configured

### Audit Trail

#### Logging

All security-relevant actions are logged:

**Authenticated Events**
```rust
tracing::info!(
    user_id = %auth.user_id,
    action = "create_humidor",
    "User created humidor"
);
```

**Admin Actions**
```rust
tracing::warn!(
    admin_id = %auth.user_id,
    target_user = %user_id,
    action = "password_reset",
    "Admin reset user password"
);
```

**Security Events**
```rust
tracing::error!(
    ip = %remote_addr,
    username = %attempted_username,
    "Failed login attempt"
);
```

#### Log Information

Each entry includes:
- Timestamp
- Log level (info, warn, error)
- User ID (if authenticated)
- Action performed
- Resource affected
- Remote IP (if available)

### Threat Model

#### Protected Against

✅ **SQL Injection**
- Parameterized queries (tokio-postgres)
- No string concatenation

✅ **XSS (Cross-Site Scripting)**
- Input sanitization
- Output encoding
- Content Security Policy (recommended)

✅ **CSRF (Cross-Site Request Forgery)**
- JWT tokens (not cookies)
- CORS restrictions
- Origin validation

✅ **Authentication Bypass**
- Middleware on all protected routes
- Token validation on every request
- Ownership verification

✅ **Privilege Escalation**
- Role checks on admin endpoints
- Cannot self-promote to admin
- Last admin protection

✅ **Data Leakage**
- User-scoped queries
- Ownership verification
- Share permission checks

#### Not Protected Against (Recommendations)

⚠️ **Brute Force** (Planned)
- Implement rate limiting
- Account lockout after X failures
- CAPTCHA on repeated attempts

⚠️ **DoS (Denial of Service)** (Infrastructure)
- Use reverse proxy rate limiting
- Implement request throttling
- Monitor resource usage

⚠️ **Man-in-the-Middle** (Deployment)
- Require HTTPS in production
- Use HSTS headers
- Certificate pinning (advanced)

## Security Testing

### Automated Tests

**Security Isolation Tests** (`tests/security_isolation_tests.rs`)
- 17 comprehensive tests
- All passing ✅
- Covers:
  - User cannot access other users' humidors
  - User cannot access other users' cigars
  - User cannot access other users' favorites
  - User cannot access other users' wish lists
  - User cannot move cigars to other users' humidors
  - User cannot favorite other users' cigars

**Permission Tests** (`tests/permission_tests.rs`)
- Admin flag validation
- Permission enforcement
- Admin operations

**Humidor Sharing Tests** (`tests/humidor_sharing_tests.rs`)
- 12 comprehensive tests
- Permission level enforcement
- Cascade delete behavior
- Edge cases (self-sharing, duplicates)

### Manual Testing Checklist

Before production deployment:

- [ ] Verify HTTPS is enabled
- [ ] Test authentication with expired tokens
- [ ] Attempt to access other users' data
- [ ] Verify admin-only endpoints reject non-admins
- [ ] Test CORS with unauthorized origins
- [ ] Verify password reset token expiration
- [ ] Test account deactivation effects
- [ ] Verify last admin cannot be demoted
- [ ] Test humidor sharing permissions
- [ ] Verify CASCADE deletes work correctly

## Security Updates

### Dependency Management

**Rust Dependencies**
- Regular `cargo audit` for vulnerabilities
- Update dependencies quarterly
- Monitor security advisories

**Docker Images**
- Use official PostgreSQL images
- Update to latest stable versions
- Scan images for vulnerabilities

### Reporting Security Issues

If you discover a security vulnerability:

1. **Do not** open a public GitHub issue
2. Email maintainers directly (see README)
3. Include:
   - Description of the vulnerability
   - Steps to reproduce
   - Potential impact
   - Suggested fix (if any)
4. Allow reasonable time for response and fix
5. Coordinate disclosure timing

## Compliance Considerations

### GDPR (General Data Protection Regulation)

**User Rights**:
- ✅ Right to access data (export functionality needed)
- ✅ Right to deletion (account deactivation available)
- ⚠️ Right to portability (export feature recommended)
- ✅ Right to correction (profile editing available)

**Implementation**:
- Add data export feature
- Implement hard delete option for GDPR requests
- Document data retention policies

### Data Retention

**Current Policy**:
- User data retained indefinitely unless deactivated
- Deactivated user data preserved (soft delete)
- No automatic cleanup

**Recommendations**:
- Define retention periods
- Implement hard delete for GDPR
- Add data export/archive features
- Document policies in Terms of Service

## Best Practices Summary

### For Developers

1. ✅ Always use parameterized queries
2. ✅ Validate all inputs
3. ✅ Filter queries by user_id
4. ✅ Verify ownership before operations
5. ✅ Log security-relevant actions
6. ✅ Write security tests
7. ✅ Review dependencies regularly

### For Administrators

1. ✅ Use HTTPS in production
2. ✅ Set strong JWT_SECRET
3. ✅ Configure CORS properly
4. ✅ Enable comprehensive logging
5. ✅ Regular security audits
6. ✅ Keep software updated
7. ✅ Backup data regularly

### For Users

1. ✅ Use strong, unique passwords
2. ✅ Don't share account credentials
3. ✅ Use humidor sharing instead
4. ✅ Report suspicious activity
5. ✅ Review shared humidors regularly
6. ✅ Log out on shared computers

## Related Documentation

- [User Permissions & Roles](PERMISSIONS.md)
- [Admin Guide](ADMIN_GUIDE.md)
- [Security Audit Report](SECURITY_AUDIT_2025-01-11.md)
- [API Authentication](API.md#authentication)
